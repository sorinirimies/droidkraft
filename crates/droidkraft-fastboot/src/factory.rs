//! Native Pixel/Google factory-image flashing (what `flash-all.sh` does).
//!
//! Sequence: bootloader → `reboot-bootloader` → radio → `reboot-bootloader` →
//! physical partitions from `image-*.zip` → `reboot-fastboot` → logical
//! partitions (`create`/`resize-logical-partition` + flash) → optional wipe →
//! `reboot`. Partitions with A/B slots are flashed to **both** slots.
//! `android-info.txt` requirements are not checked.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::native::{self, run_reboot};
use crate::transport::Transport;
use crate::types::{FastbootError, FastbootResult};

/// One progress notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlashProgress {
    /// 1-based index of the step being started.
    pub step_index: usize,
    pub total_steps: usize,
    pub label: String,
}

/// Options for [`flash_all`].
#[derive(Debug, Clone)]
pub struct FactoryOptions {
    /// Erase userdata/metadata at the end (`false` keeps user data).
    pub wipe: bool,
    /// Leave the device in fastboot instead of the final `reboot`.
    pub skip_reboot: bool,
    /// How long to wait for the device after each reboot (default 30 s).
    pub reconnect_timeout: Duration,
    /// Pause after a reboot command before polling (default 3 s).
    pub settle_delay: Duration,
}

impl Default for FactoryOptions {
    fn default() -> Self {
        Self {
            wipe: false,
            skip_reboot: false,
            reconnect_timeout: Duration::from_secs(30),
            settle_delay: Duration::from_secs(3),
        }
    }
}

const LOGICAL: &[&str] = &[
    "system",
    "system_ext",
    "product",
    "vendor",
    "vendor_dlkm",
    "odm",
    "odm_dlkm",
    "system_dlkm",
];

type Opener<'a> = dyn FnMut() -> Result<Box<dyn Transport>, FastbootError> + 'a;

fn cmd_err(m: impl Into<String>) -> FastbootError {
    FastbootError::CommandFailed(m.into())
}

/// Flash a factory image given as a `.zip` (extracted next to it) or an
/// already extracted directory, using the native backend.
pub fn flash_all(
    source: &Path,
    serial: Option<&str>,
    opts: &FactoryOptions,
    progress: &mut dyn FnMut(FlashProgress),
) -> FastbootResult<String> {
    let root = if source.is_dir() {
        source.to_path_buf()
    } else {
        let dir = source.with_file_name(format!(
            "{}-fb-extracted",
            source
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("factory")
        ));
        extract_zip(source, &dir)?;
        dir
    };
    let mut open = || native::open(serial);
    flash_dir(&root, opts, &mut open, progress)
}

/// Alias of [`flash_all`] for zip inputs.
pub fn flash_factory_zip(
    zip: &Path,
    serial: Option<&str>,
    opts: &FactoryOptions,
    progress: &mut dyn FnMut(FlashProgress),
) -> FastbootResult<String> {
    flash_all(zip, serial, opts, progress)
}

fn extract_zip(zip_path: &Path, dest: &Path) -> FastbootResult<()> {
    let mut ar = zip::ZipArchive::new(File::open(zip_path)?)
        .map_err(|e| cmd_err(format!("read zip: {e}")))?;
    std::fs::create_dir_all(dest)?;
    for i in 0..ar.len() {
        let mut e = ar
            .by_index(i)
            .map_err(|e| cmd_err(format!("zip entry: {e}")))?;
        let Some(rel) = e.enclosed_name() else {
            continue;
        };
        let out = dest.join(rel);
        if e.is_dir() {
            std::fs::create_dir_all(&out)?;
        } else {
            if let Some(p) = out.parent() {
                std::fs::create_dir_all(p)?;
            }
            std::io::copy(&mut e, &mut File::create(&out)?)?;
        }
    }
    Ok(())
}

/// Find the directory holding `image-*.zip` (searches two levels).
fn find_root(dir: &Path, depth: u8) -> Option<PathBuf> {
    let entries: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .collect();
    if entries.iter().any(|p| prefixed(p, "image-", ".zip")) {
        return Some(dir.to_path_buf());
    }
    if depth == 0 {
        return None;
    }
    entries
        .iter()
        .filter(|p| p.is_dir())
        .find_map(|p| find_root(p, depth - 1))
}

fn prefixed(p: &Path, pre: &str, suf: &str) -> bool {
    p.is_file()
        && p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(pre) && n.ends_with(suf))
}

fn first(root: &Path, pre: &str, suf: &str) -> Option<PathBuf> {
    let mut v: Vec<_> = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| prefixed(p, pre, suf))
        .collect();
    v.sort();
    v.into_iter().next()
}

fn getvar(t: &mut dyn Transport, name: &str) -> Option<String> {
    match native::execute_on(t, &crate::FastbootCommand::GetVar { name: name.into() }) {
        native::Outcome::Done(Ok(v)) => Some(v.trim().to_string()),
        _ => None,
    }
}

/// Slot-suffixed partition names for flashing.
fn targets(t: &mut dyn Transport, name: &str) -> Vec<String> {
    if getvar(t, &format!("has-slot:{name}")).as_deref() == Some("yes") {
        vec![format!("{name}_a"), format!("{name}_b")]
    } else {
        vec![name.to_string()]
    }
}

fn reconnect(open: &mut Opener, opts: &FactoryOptions) -> FastbootResult<Box<dyn Transport>> {
    std::thread::sleep(opts.settle_delay);
    let start = Instant::now();
    loop {
        match open() {
            Ok(t) => return Ok(t),
            Err(_) if start.elapsed() < opts.reconnect_timeout => {
                std::thread::sleep(Duration::from_millis(500))
            }
            Err(e) => return Err(e),
        }
    }
}

struct Steps<'a> {
    i: usize,
    total: usize,
    cb: &'a mut dyn FnMut(FlashProgress),
}
impl Steps<'_> {
    fn next(&mut self, label: impl Into<String>) {
        self.i += 1;
        (self.cb)(FlashProgress {
            step_index: self.i,
            total_steps: self.total,
            label: label.into(),
        });
    }
}

fn flash_named(t: &mut dyn Transport, name: &str, path: &Path) -> FastbootResult<()> {
    for target in targets(t, name) {
        native::flash(t, &target, &path.to_string_lossy())?;
    }
    Ok(())
}

pub(crate) fn flash_dir(
    dir: &Path,
    opts: &FactoryOptions,
    open: &mut Opener,
    progress: &mut dyn FnMut(FlashProgress),
) -> FastbootResult<String> {
    let root = find_root(dir, 2).ok_or_else(|| cmd_err("no image-*.zip found in factory image"))?;
    let bootloader = first(&root, "bootloader-", ".img");
    let radio = first(&root, "radio-", ".img");
    let inner = first(&root, "image-", ".zip").ok_or_else(|| cmd_err("no image-*.zip found"))?;
    let mut zip = zip::ZipArchive::new(File::open(&inner)?)
        .map_err(|e| cmd_err(format!("read {inner:?}: {e}")))?;
    let mut physical = Vec::new();
    let mut logical = Vec::new();
    for n in zip.file_names().map(String::from).collect::<Vec<_>>() {
        let Some(part) = n
            .strip_suffix(".img")
            .filter(|p| !p.contains('/') && *p != "super_empty")
        else {
            continue;
        };
        if LOGICAL.contains(&part) {
            &mut logical
        } else {
            &mut physical
        }
        .push((n.clone(), part.to_string()));
    }
    physical.sort();
    logical.sort();

    let total = 2 * usize::from(bootloader.is_some())
        + 2 * usize::from(radio.is_some())
        + physical.len()
        + if logical.is_empty() {
            0
        } else {
            1 + logical.len()
        }
        + usize::from(opts.wipe)
        + usize::from(!opts.skip_reboot);
    let mut steps = Steps {
        i: 0,
        total,
        cb: progress,
    };
    let mut t = open()?;

    for (img, part) in [(&bootloader, "bootloader"), (&radio, "radio")] {
        if let Some(path) = img {
            steps.next(format!("Flashing {part}"));
            flash_named(&mut *t, part, path)?;
            steps.next("Rebooting to bootloader");
            run_reboot(&mut *t, "reboot-bootloader")?;
            t = reconnect(open, opts)?;
        }
    }

    let extract = |zip: &mut zip::ZipArchive<File>, entry: &str| -> FastbootResult<PathBuf> {
        let out = root.join(format!(".dk-extract-{entry}"));
        let mut e = zip
            .by_name(entry)
            .map_err(|e| cmd_err(format!("{entry}: {e}")))?;
        std::io::copy(&mut e, &mut File::create(&out)?)?;
        Ok(out)
    };

    for (entry, part) in &physical {
        steps.next(format!("Flashing {part}"));
        let img = extract(&mut zip, entry)?;
        let r = flash_named(&mut *t, part, &img);
        let _ = std::fs::remove_file(&img);
        r?;
    }

    if !logical.is_empty() {
        steps.next("Rebooting to fastbootd");
        run_reboot(&mut *t, "reboot-fastboot")?;
        t = reconnect(open, opts)?;
        for (entry, part) in &logical {
            steps.next(format!("Flashing {part}"));
            let img = extract(&mut zip, entry)?;
            let r = flash_logical(&mut *t, part, &img);
            let _ = std::fs::remove_file(&img);
            r?;
        }
    }

    if opts.wipe {
        steps.next("Wiping userdata");
        match native::execute_on(&mut *t, &crate::FastbootCommand::WipeData) {
            native::Outcome::Done(r) => r.map(|_| ())?,
            native::Outcome::Fallback(e) => return Err(e),
        }
    }
    if !opts.skip_reboot {
        steps.next("Rebooting");
        run_reboot(&mut *t, "reboot")?;
    }
    Ok("Factory image flashed".into())
}

fn flash_logical(t: &mut dyn Transport, name: &str, img: &Path) -> FastbootResult<()> {
    let mut f = File::open(img)?;
    let size = crate::sparse::expanded_size(&mut f).map_err(|e| cmd_err(e.to_string()))?;
    for target in targets(t, name) {
        if getvar(t, &format!("is-logical:{target}")).as_deref() != Some("yes") {
            run_cmd(t, &format!("create-logical-partition:{target}:0"))?;
        }
        run_cmd(t, &format!("resize-logical-partition:{target}:{size}"))?;
        native::flash(t, &target, &img.to_string_lossy())?;
    }
    Ok(())
}

fn run_cmd(t: &mut dyn Transport, cmd: &str) -> FastbootResult<()> {
    match native::execute_on(t, &crate::FastbootCommand::Raw(cmd.into())) {
        native::Outcome::Done(r) => r.map(|_| ()),
        native::Outcome::Fallback(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::io::Write;
    use std::rc::Rc;

    #[derive(Default)]
    struct State {
        log: Vec<String>,
        pending: Vec<Vec<u8>>,
        expect: usize,
        userspace: bool,
    }
    struct Dev(Rc<RefCell<State>>);
    impl Transport for Dev {
        fn send(&mut self, d: &[u8]) -> Result<(), String> {
            let mut s = self.0.borrow_mut();
            if s.expect > 0 {
                s.expect = s.expect.saturating_sub(d.len());
                if s.expect == 0 {
                    s.pending.push(b"OKAY".to_vec());
                }
                return Ok(());
            }
            let c = String::from_utf8_lossy(d).into_owned();
            let reply = if let Some(n) = c.strip_prefix("download:") {
                s.expect = usize::from_str_radix(n, 16).unwrap();
                format!("DATA{n}")
            } else if c == "getvar:max-download-size" {
                "OKAY0x10000000".into()
            } else if c.starts_with("getvar:has-slot:") {
                "OKAYyes".into()
            } else if c.starts_with("getvar:is-logical:") {
                "OKAYno".into()
            } else {
                if c == "reboot-fastboot" {
                    s.userspace = true;
                }
                "OKAY".into()
            };
            s.log.push(c);
            s.pending.push(reply.into_bytes());
            Ok(())
        }
        fn recv(&mut self) -> Result<Vec<u8>, String> {
            let mut s = self.0.borrow_mut();
            if s.pending.is_empty() {
                return Err("no reply".into());
            }
            Ok(s.pending.remove(0))
        }
        fn chunk_size(&self) -> usize {
            1 << 20
        }
    }

    fn build(dir: &Path) {
        std::fs::write(dir.join("bootloader-x-1.img"), b"bl").unwrap();
        std::fs::write(dir.join("radio-x-1.img"), b"rd").unwrap();
        let mut z = zip::ZipWriter::new(File::create(dir.join("image-x-1.zip")).unwrap());
        let o = zip::write::SimpleFileOptions::default();
        for n in [
            "boot.img",
            "vbmeta.img",
            "system.img",
            "vendor.img",
            "super_empty.img",
            "android-info.txt",
        ] {
            z.start_file(n, o).unwrap();
            z.write_all(b"data").unwrap();
        }
        z.finish().unwrap();
    }

    fn run_flash(opts: FactoryOptions) -> (Vec<String>, Vec<FlashProgress>) {
        let dir =
            std::env::temp_dir().join(format!("dk_factory_{}_{}", std::process::id(), opts.wipe));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        build(&dir);
        let st = Rc::new(RefCell::new(State::default()));
        let mut open = || Ok(Box::new(Dev(st.clone())) as Box<dyn Transport>);
        let mut prog = Vec::new();
        flash_dir(&dir, &opts, &mut open, &mut |p| prog.push(p)).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        let log = st.borrow().log.clone();
        (log, prog)
    }

    fn fast(wipe: bool) -> FactoryOptions {
        FactoryOptions {
            wipe,
            settle_delay: Duration::ZERO,
            ..Default::default()
        }
    }

    #[test]
    fn full_sequence_without_wipe() {
        let (log, prog) = run_flash(fast(false));
        let flashes: Vec<&str> = log
            .iter()
            .filter(|c| c.starts_with("flash:"))
            .map(String::as_str)
            .collect();
        assert_eq!(
            flashes,
            [
                "flash:bootloader_a",
                "flash:bootloader_b",
                "flash:radio_a",
                "flash:radio_b",
                "flash:boot_a",
                "flash:boot_b",
                "flash:vbmeta_a",
                "flash:vbmeta_b",
                "flash:system_a",
                "flash:system_b",
                "flash:vendor_a",
                "flash:vendor_b",
            ]
        );
        let pos = |s: &str| log.iter().position(|c| c == s).unwrap();
        assert!(pos("reboot-bootloader") < pos("flash:radio_a"));
        assert!(pos("flash:vbmeta_b") < pos("reboot-fastboot"));
        assert!(pos("reboot-fastboot") < pos("create-logical-partition:system_a:0"));
        assert!(log
            .iter()
            .any(|c| c.starts_with("resize-logical-partition:vendor_b:")));
        assert!(!log.iter().any(|c| c.starts_with("erase:")));
        assert_eq!(log.last().unwrap(), "reboot");
        let last = prog.last().unwrap();
        assert_eq!(last.step_index, last.total_steps);
        assert_eq!(prog[0].step_index, 1);
    }

    #[test]
    fn wipe_adds_erase_before_reboot() {
        let (log, _) = run_flash(fast(true));
        let n = log.len();
        assert_eq!(log[n - 3], "erase:userdata");
        assert_eq!(log[n - 2], "erase:metadata");
        assert_eq!(log[n - 1], "reboot");
    }

    #[test]
    fn missing_image_zip_errors() {
        let dir = std::env::temp_dir().join("dk_factory_empty");
        std::fs::create_dir_all(&dir).unwrap();
        let mut open = || Err(FastbootError::NoDevice);
        let r = flash_dir(&dir, &fast(false), &mut open, &mut |_| {});
        assert!(matches!(r, Err(FastbootError::CommandFailed(_))));
    }
}
