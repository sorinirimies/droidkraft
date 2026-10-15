//! Background worker thread that owns an [`AdbManager`] and services requests
//! from the GUI without ever blocking the render thread.
//!
//! Communication is over `std::sync::mpsc` channels:
//! the UI sends [`WorkerRequest`]s and drains [`WorkerResponse`]s each tick.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use droidkraft_core::features::fastboot::{FastbootCommand, FastbootManager};
use droidkraft_core::features::flash::{RebootTarget, RootStatus};
use droidkraft_core::features::rom::{self, DeviceProfile, DownloadProgress, FlashStep, RomBuild};
use droidkraft_core::{
    collect_device_details, AdbDeviceEntry, AdbManager, CpuSample, DeviceDetails, DeviceHistory,
};

use crate::commands::CommandAction;

/// Convenience extension: map any `Display` error to `String`.
///
/// A trait (not a macro) is the right tool here because it chains onto a
/// `Result` — `foo().str_err()` reads better than a wrapping macro.
trait StringErr<T> {
    fn str_err(self) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> StringErr<T> for Result<T, E> {
    fn str_err(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

/// How often the worker refreshes the `adb devices` list.
const LIST_INTERVAL: Duration = Duration::from_secs(2);
/// A request from the UI to the worker. Every device-scoped request carries
/// the serial it targets.
pub enum WorkerRequest {
    /// Run a catalogue command, tagged with its button label.
    Run {
        serial: String,
        label: String,
        action: CommandAction,
    },
    /// Run a raw shell command, tagged with a label.
    Shell {
        serial: String,
        label: String,
        command: String,
    },
    DetectRoot {
        serial: String,
    },
    /// Remount `/` read-write (requires root).
    Remount {
        serial: String,
    },
    Fastboot {
        serial: String,
        label: String,
        command: FastbootCommand,
    },
    /// Force an immediate device-list refresh.
    RefreshStatus,
    /// Change how often device stats/graphs are polled.
    SetRate(droidkraft_core::RefreshRate),
    /// Which device's workspace is open (polled faster).
    Focus(Option<String>),
    DetectProfile {
        serial: String,
    },
    FetchRoms {
        serial: String,
        codename: String,
    },
    Download {
        serial: String,
        build: RomBuild,
        dest: PathBuf,
    },
    RunFlashStep {
        serial: String,
        idx: usize,
        step: FlashStep,
    },
    /// Save a PNG screenshot to `path`.
    Screenshot {
        serial: String,
        path: PathBuf,
    },
}

/// A response from the worker to the UI. Device-scoped responses carry the
/// serial so the UI can route them to the right device state.
pub enum WorkerResponse {
    /// Fresh `adb devices` snapshot.
    Devices(Vec<AdbDeviceEntry>),
    /// Rich details + rolling history for one device.
    Details {
        serial: String,
        details: Box<DeviceDetails>,
        history: Box<DeviceHistory>,
    },
    Output {
        serial: String,
        label: String,
        result: Result<String, String>,
    },
    Root {
        serial: String,
        result: Result<RootStatus, String>,
    },
    Profile {
        serial: String,
        profile: DeviceProfile,
    },
    Roms {
        serial: String,
        result: Result<Vec<RomBuild>, String>,
    },
    DownloadProgress {
        serial: String,
        progress: DownloadProgress,
    },
    Downloaded {
        serial: String,
        result: Result<PathBuf, String>,
    },
    FlashStepDone {
        serial: String,
        idx: usize,
        result: Result<String, String>,
    },
    Screenshot {
        serial: String,
        result: Result<PathBuf, String>,
    },
}

/// Handle to the background worker.
pub struct Worker {
    req_tx: Sender<WorkerRequest>,
    res_rx: Receiver<WorkerResponse>,
}

impl Worker {
    /// Spawn the worker thread.
    pub fn spawn() -> Self {
        let (req_tx, req_rx) = channel::<WorkerRequest>();
        let (res_tx, res_rx) = channel::<WorkerResponse>();
        thread::spawn(move || Poller::new(res_tx).run(req_rx));
        Self { req_tx, res_rx }
    }

    /// Send a request to the worker (ignored if the worker has stopped).
    pub fn send(&self, req: WorkerRequest) {
        let _ = self.req_tx.send(req);
    }

    /// Drain all currently available responses.
    pub fn drain(&self) -> Vec<WorkerResponse> {
        self.res_rx.try_iter().collect()
    }
}

/// Per-device polling state.
struct PollEntry {
    cpu: CpuSample,
    history: DeviceHistory,
    last: Option<Instant>,
}

struct Poller {
    adb: AdbManager,
    tx: Sender<WorkerResponse>,
    entries: HashMap<String, PollEntry>,
    online: Vec<String>,
    focus: Option<String>,
    rate: droidkraft_core::RefreshRate,
    last_list: Option<Instant>,
}

impl Poller {
    fn new(tx: Sender<WorkerResponse>) -> Self {
        Self {
            adb: AdbManager::new(),
            tx,
            entries: HashMap::new(),
            online: Vec::new(),
            focus: None,
            rate: Default::default(),
            last_list: None,
        }
    }

    fn run(mut self, req_rx: Receiver<WorkerRequest>) {
        loop {
            match req_rx.recv_timeout(Duration::from_millis(300)) {
                Ok(req) => self.handle(req),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if self.poll(&req_rx).is_err() {
                break;
            }
        }
    }

    fn refresh_list(&mut self) {
        let status = self.adb.fetch_device_status();
        self.online = status
            .devices
            .iter()
            .filter(|d| d.is_online())
            .map(|d| d.serial.clone())
            .collect();
        self.entries
            .retain(|s, _| status.devices.iter().any(|d| &d.serial == s));
        self.last_list = Some(Instant::now());
        let _ = self.tx.send(WorkerResponse::Devices(status.devices));
    }

    /// Refresh the list and any due device details. Pending requests are
    /// served between devices so commands never wait for a full poll round.
    /// `Err` means the UI hung up.
    fn poll(&mut self, req_rx: &Receiver<WorkerRequest>) -> Result<(), ()> {
        if self.last_list.is_none_or(|t| t.elapsed() >= LIST_INTERVAL) {
            self.refresh_list();
        }
        for serial in self.online.clone() {
            let every = if self.focus.as_deref() == Some(serial.as_str()) {
                self.rate.focused()
            } else {
                self.rate.background()
            };
            let entry = self
                .entries
                .entry(serial.clone())
                .or_insert_with(|| PollEntry {
                    cpu: CpuSample::default(),
                    history: DeviceHistory::default(),
                    last: None,
                });
            if entry.last.is_some_and(|t| t.elapsed() < every) {
                continue;
            }
            let prev = entry.last.map(|_| entry.cpu);
            let (details, sample) = collect_device_details(&serial, prev.as_ref());
            entry.last = Some(Instant::now());
            entry.cpu = sample;
            if details.online {
                entry.history.record(&details);
            }
            let _ = self.tx.send(WorkerResponse::Details {
                serial: serial.clone(),
                details: Box::new(details),
                history: Box::new(entry.history.clone()),
            });
            loop {
                match req_rx.try_recv() {
                    Ok(req) => self.handle(req),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return Err(()),
                }
            }
        }
        Ok(())
    }

    fn send(&self, r: WorkerResponse) {
        let _ = self.tx.send(r);
    }

    fn handle(&mut self, req: WorkerRequest) {
        match req {
            WorkerRequest::RefreshStatus => {
                self.last_list = None;
                for e in self.entries.values_mut() {
                    e.last = None;
                }
            }
            WorkerRequest::SetRate(r) => {
                self.rate = r;
                for e in self.entries.values_mut() {
                    e.last = None;
                }
            }
            WorkerRequest::Focus(s) => {
                // Make the newly focused workspace refresh right away.
                if let Some(e) = s.as_ref().and_then(|s| self.entries.get_mut(s)) {
                    e.last = None;
                }
                self.focus = s;
            }
            WorkerRequest::Run {
                serial,
                label,
                action,
            } => {
                self.adb.select_device(serial.clone());
                let result = run_action(&mut self.adb, action);
                self.send(WorkerResponse::Output {
                    serial,
                    label,
                    result,
                });
            }
            WorkerRequest::Shell {
                serial,
                label,
                command,
            } => {
                self.adb.select_device(serial.clone());
                let result = self.adb.shell_command(&command).str_err();
                self.send(WorkerResponse::Output {
                    serial,
                    label,
                    result,
                });
            }
            WorkerRequest::DetectRoot { serial } => {
                self.adb.select_device(serial.clone());
                let result = self.adb.detect_root().str_err();
                self.send(WorkerResponse::Root { serial, result });
            }
            WorkerRequest::Remount { serial } => {
                self.adb.select_device(serial.clone());
                let result = self.adb.remount().str_err();
                self.send(WorkerResponse::Output {
                    serial,
                    label: "Remount".into(),
                    result,
                });
            }
            WorkerRequest::Fastboot {
                serial,
                label,
                command,
            } => {
                let result = FastbootManager::new()
                    .with_serial(serial.clone())
                    .execute(command)
                    .str_err();
                self.send(WorkerResponse::Output {
                    serial,
                    label,
                    result,
                });
            }
            WorkerRequest::DetectProfile { serial } => {
                self.adb.select_device(serial.clone());
                if let Ok(profile) = self.adb.detect_device_profile() {
                    self.send(WorkerResponse::Profile { serial, profile });
                }
            }
            WorkerRequest::FetchRoms { serial, codename } => {
                let result = fetch_roms(&codename);
                self.send(WorkerResponse::Roms { serial, result });
            }
            WorkerRequest::Download {
                serial,
                build,
                dest,
            } => {
                // Dedicated thread so polling/commands keep flowing.
                let tx = self.tx.clone();
                thread::spawn(move || {
                    let result = rom::download_and_verify(
                        &build.download_url,
                        &dest,
                        build.sha256.as_deref(),
                        |p| {
                            let _ = tx.send(WorkerResponse::DownloadProgress {
                                serial: serial.clone(),
                                progress: p,
                            });
                        },
                    );
                    let _ = tx.send(WorkerResponse::Downloaded { serial, result });
                });
            }
            WorkerRequest::RunFlashStep { serial, idx, step } => {
                self.adb.select_device(serial.clone());
                let fastboot = FastbootManager::new().with_serial(serial.clone());
                let result = rom::run_flash_step(&step, &serial, &mut self.adb, &fastboot);
                self.send(WorkerResponse::FlashStepDone {
                    serial,
                    idx,
                    result,
                });
            }
            WorkerRequest::Screenshot { serial, path } => {
                self.adb.select_device(serial.clone());
                let result = screenshot(&mut self.adb, &path);
                self.send(WorkerResponse::Screenshot { serial, result });
            }
        }
    }
}

/// `screencap -p` → PNG file on the host.
fn screenshot(adb: &mut AdbManager, path: &std::path::Path) -> Result<PathBuf, String> {
    let bytes = adb.shell_command_raw("screencap -p").str_err()?;
    const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    // Some adb shells translate \n → \r\n; undo that when the signature is broken.
    let png = if bytes.starts_with(&SIG) {
        bytes
    } else {
        let fixed = unix_newlines(&bytes);
        if fixed.starts_with(&SIG) {
            fixed
        } else {
            return Err("screencap did not return a PNG (screen off or secure window?)".into());
        }
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    std::fs::write(path, &png).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(path.to_path_buf())
}

/// Replace every `\r\n` with `\n`.
fn unix_newlines(b: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\r' && b.get(i + 1) == Some(&b'\n') {
            i += 1;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

/// Resolve downloadable ROM builds for a device codename.
///
/// Only projects with a live download API (LineageOS) yield directly
/// downloadable builds; other catalog ROMs are informational.
fn fetch_roms(codename: &str) -> Result<Vec<RomBuild>, String> {
    Ok(droidkraft_core::features::rom::resolve_all(codename))
}

fn run_action(adb: &mut AdbManager, action: CommandAction) -> Result<String, String> {
    match action {
        CommandAction::Adb(cmd) => adb.execute(cmd).str_err(),
        CommandAction::Shell(cmd) => adb.shell_command(&cmd).str_err(),
        CommandAction::Reboot(target) => reboot(adb, target),
    }
}

fn reboot(adb: &mut AdbManager, target: RebootTarget) -> Result<String, String> {
    adb.reboot(target).str_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newline_fix() {
        assert_eq!(unix_newlines(b"a\r\nb\rc"), b"a\nb\rc");
    }
}
