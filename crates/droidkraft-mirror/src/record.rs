//! Screen recorder: `screenrecord` on the device, segments pulled to the host.
//! Independent from the live mirror (own remote files / own kill pattern).

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use adb_client::ADBDeviceExt;

use crate::{open_device, shell_bytes};

#[derive(Clone, Debug)]
pub struct RecordOptions {
    pub bit_rate_mbps: u32,
    pub max_height: Option<u32>,
    pub segment_secs: u32,
    pub show_touches: bool,
}

impl Default for RecordOptions {
    fn default() -> Self {
        Self {
            bit_rate_mbps: 8,
            max_height: None,
            segment_secs: 170,
            show_touches: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RecordStatus {
    Idle,
    Recording {
        since: Instant,
        segments_done: u32,
        bytes_pulled: u64,
    },
    Finishing,
    Done {
        files: Vec<PathBuf>,
    },
    Failed(String),
}

struct Inner {
    status: Mutex<RecordStatus>,
    stop: AtomicBool,
    /// Remote tag of the active recording (for targeted pkill).
    tag: Mutex<Option<(String, String)>>,
}

/// Cheap-to-share recorder handle.
#[derive(Clone)]
pub struct ScreenRecorder {
    inner: Arc<Inner>,
}

impl Default for ScreenRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenRecorder {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                status: Mutex::new(RecordStatus::Idle),
                stop: AtomicBool::new(false),
                tag: Mutex::new(None),
            }),
        }
    }

    pub fn status(&self) -> RecordStatus {
        self.inner.status.lock().unwrap().clone()
    }

    pub fn is_recording(&self) -> bool {
        matches!(
            self.status(),
            RecordStatus::Recording { .. } | RecordStatus::Finishing
        )
    }

    pub fn elapsed(&self) -> Option<Duration> {
        match self.status() {
            RecordStatus::Recording { since, .. } => Some(since.elapsed()),
            _ => None,
        }
    }

    pub fn start(
        &self,
        serial: &str,
        out_dir: &Path,
        base_name: &str,
        opts: RecordOptions,
    ) -> Result<(), String> {
        {
            let mut st = self.inner.status.lock().unwrap();
            if matches!(
                *st,
                RecordStatus::Recording { .. } | RecordStatus::Finishing
            ) {
                return Err("already recording".into());
            }
            std::fs::create_dir_all(out_dir)
                .map_err(|e| format!("create {}: {e}", out_dir.display()))?;
            *st = RecordStatus::Recording {
                since: Instant::now(),
                segments_done: 0,
                bytes_pulled: 0,
            };
        }
        self.inner.stop.store(false, Ordering::SeqCst);
        let inner = self.inner.clone();
        let (serial, out_dir, base) = (
            serial.to_string(),
            out_dir.to_path_buf(),
            base_name.to_string(),
        );
        std::thread::spawn(move || {
            let res = run(&inner, &serial, &out_dir, &base, &opts);
            *inner.tag.lock().unwrap() = None;
            *inner.status.lock().unwrap() = match res {
                Ok(files) => RecordStatus::Done { files },
                Err(e) => RecordStatus::Failed(e),
            };
        });
        Ok(())
    }

    /// Request stop; the worker finalizes the current segment, pulls it, and
    /// moves to `Done`.
    pub fn stop(&self) {
        self.inner.stop.store(true, Ordering::SeqCst);
        let mut st = self.inner.status.lock().unwrap();
        if matches!(*st, RecordStatus::Recording { .. }) {
            *st = RecordStatus::Finishing;
        }
        drop(st);
        // Interrupt in the background; the tag may not be set yet (worker
        // checks the stop flag right after launching, see `run`).
        if let Some((serial, tag)) = self.inner.tag.lock().unwrap().clone() {
            std::thread::spawn(move || {
                let _ = shell_bytes(&serial, &kill_cmd(&tag));
            });
        }
    }
}

fn kill_cmd(tag: &str) -> String {
    format!("pkill -2 -f '[d]k_rec_{tag}'")
}

fn run(
    inner: &Arc<Inner>,
    serial: &str,
    out_dir: &Path,
    base: &str,
    opts: &RecordOptions,
) -> Result<Vec<PathBuf>, String> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let tag = format!("{ts}");
    let mut size_arg = String::new();
    if let Some(h) = opts.max_height {
        if let Some((w0, h0)) = device_size(serial) {
            if h0 > h {
                let w = (u64::from(w0) * u64::from(h) / u64::from(h0)) as u32 & !1;
                size_arg = format!(" --size {w}x{}", h & !1);
            }
        }
    }
    let prev_touches = if opts.show_touches {
        let prev = shell_bytes(serial, "settings get system show_touches")
            .ok()
            .map(|b| String::from_utf8_lossy(&b).trim().to_string())
            .filter(|s| s == "0" || s == "1")
            .unwrap_or_else(|| "0".into());
        let _ = shell_bytes(serial, "settings put system show_touches 1");
        Some(prev)
    } else {
        None
    };

    let result = segments(inner, serial, out_dir, base, opts, &tag, &size_arg);

    if let Some(prev) = prev_touches {
        let _ = shell_bytes(serial, &format!("settings put system show_touches {prev}"));
    }
    result
}

fn segments(
    inner: &Arc<Inner>,
    serial: &str,
    out_dir: &Path,
    base: &str,
    opts: &RecordOptions,
    tag: &str,
    size_arg: &str,
) -> Result<Vec<PathBuf>, String> {
    let seg_secs = opts.segment_secs.clamp(1, 180);
    let mut files: Vec<PathBuf> = Vec::new();
    let mut bytes_total = 0u64;
    let mut n = 0u32;
    loop {
        if inner.stop.load(Ordering::SeqCst) {
            break;
        }
        n += 1;
        let seg_tag = format!("{tag}_{n}");
        let remote = format!("/sdcard/.dk_rec_{seg_tag}.mp4");
        *inner.tag.lock().unwrap() = Some((serial.to_string(), seg_tag.clone()));
        let cmd = format!(
            "screenrecord --bit-rate {}{size_arg} --time-limit {seg_secs} {remote}",
            u64::from(opts.bit_rate_mbps.max(1)) * 1_000_000
        );
        let mut dev = open_device(serial)?;
        // Watchdog: stop() may have raced before the tag was visible.
        let done = Arc::new(AtomicBool::new(false));
        {
            let (inner, done, serial, kc) = (
                inner.clone(),
                done.clone(),
                serial.to_string(),
                kill_cmd(&seg_tag),
            );
            std::thread::spawn(move || {
                while !done.load(Ordering::SeqCst) {
                    if inner.stop.load(Ordering::SeqCst) {
                        std::thread::sleep(Duration::from_millis(800));
                        if !done.load(Ordering::SeqCst) {
                            let _ = shell_bytes(&serial, &kc);
                        }
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            });
        }
        let mut sink = Vec::new();
        let r = dev.shell_command(&[cmd.as_str()], &mut sink);
        done.store(true, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(300)); // let moov flush
        let out = out_dir.join(format!("{base}_{n:03}.mp4"));
        let pulled = pull_to(serial, &remote, &out);
        let _ = shell_bytes(serial, &format!("rm -f {remote}"));
        match pulled {
            Ok(b) if b > 0 => {
                bytes_total += b;
                files.push(out);
            }
            Ok(_) => {
                let _ = std::fs::remove_file(&out);
            }
            Err(e) => {
                let _ = std::fs::remove_file(&out);
                if files.is_empty() {
                    return Err(match r {
                        Err(re) => format!("screenrecord: {re}; pull: {e}"),
                        Ok(()) => e,
                    });
                }
                return Err(e);
            }
        }
        {
            let mut st = inner.status.lock().unwrap();
            if let RecordStatus::Recording {
                segments_done,
                bytes_pulled,
                ..
            } = &mut *st
            {
                *segments_done = files.len() as u32;
                *bytes_pulled = bytes_total;
            }
        }
        if let Err(e) = r {
            if !inner.stop.load(Ordering::SeqCst) {
                return Err(format!("screenrecord failed: {e}"));
            }
        }
    }
    if files.len() == 1 {
        let single = out_dir.join(format!("{base}.mp4"));
        if std::fs::rename(&files[0], &single).is_ok() {
            files[0] = single;
        }
    }
    Ok(files)
}

fn pull_to(serial: &str, remote: &str, local: &Path) -> Result<u64, String> {
    let mut dev = open_device(serial)?;
    let mut f = File::create(local).map_err(|e| format!("create {}: {e}", local.display()))?;
    dev.pull(&remote, &mut f)
        .map_err(|e| format!("pull {remote}: {e}"))?;
    Ok(f.metadata().map(|m| m.len()).unwrap_or(0))
}

fn device_size(serial: &str) -> Option<(u32, u32)> {
    let out = shell_bytes(serial, "wm size").ok()?;
    parse_wm_size(&String::from_utf8_lossy(&out))
}

fn parse_wm_size(s: &str) -> Option<(u32, u32)> {
    // Prefer "Override size" (last line) over "Physical size".
    let line = s.lines().rev().find(|l| l.contains("size:"))?;
    let (w, h) = line.split(':').nth(1)?.trim().split_once('x')?;
    Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
}

/// `~/Movies` on macOS, `~/Videos` elsewhere; falls back to home, then cwd.
pub fn default_output_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    match home {
        Some(h) => {
            let sub = if cfg!(target_os = "macos") {
                "Movies"
            } else {
                "Videos"
            };
            let d = h.join(sub);
            if d.is_dir() {
                d
            } else {
                h
            }
        }
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

/// `droidkraft-<serial>-YYYYmmdd-HHMMSS` (UTC).
pub fn timestamped_name(serial: &str) -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_name(serial, secs)
}

fn format_name(serial: &str, secs: u64) -> String {
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    let r = secs % 86_400;
    let safe: String = serial
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!(
        "droidkraft-{safe}-{y:04}{m:02}{d:02}-{:02}{:02}{:02}",
        r / 3600,
        r % 3600 / 60,
        r % 60
    )
}

/// Howard Hinnant's civil-from-days.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_783), (2024, 3, 1));
        assert_eq!(
            format_name("ab:1", 1_709_294_706),
            "droidkraft-ab_1-20240301-120506"
        );
    }

    #[test]
    fn wm() {
        assert_eq!(
            parse_wm_size("Physical size: 1080x2400\nOverride size: 720x1600\n"),
            Some((720, 1600))
        );
        assert_eq!(
            parse_wm_size("Physical size: 1080x2400"),
            Some((1080, 2400))
        );
        assert_eq!(parse_wm_size("x"), None);
    }

    #[test]
    fn defaults() {
        let o = RecordOptions::default();
        assert_eq!((o.bit_rate_mbps, o.segment_secs), (8, 170));
        assert!(!ScreenRecorder::new().is_recording());
    }
}
