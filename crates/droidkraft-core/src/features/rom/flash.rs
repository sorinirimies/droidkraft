//! Flash orchestration — an ordered [`FlashPlan`] and a step-by-step
//! [`FlashSession`] that drives a full custom-ROM install through fastboot and
//! ADB.
//!
//! ## Safety
//!
//! Flashing a custom ROM **erases all data** and can render a device
//! unbootable. Every destructive step is marked
//! [`requires_confirmation`](FlashStep::requires_confirmation) so the frontend
//! must gate it behind explicit user consent. The session only ever runs the
//! *next* step when the caller asks it to.

use std::path::PathBuf;
use std::process::Command;

use crate::client::AdbManager;
use crate::features::fastboot::{FastbootCommand, FastbootManager};
use crate::features::flash::RebootTarget;
use crate::features::rom::types::{DeviceProfile, InstallMethod};

/// A single step in a custom-ROM install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashStep {
    /// `fastboot flashing unlock` — unlocks the bootloader (wipes data).
    UnlockBootloader,
    /// `adb reboot bootloader` — enter fastboot mode.
    RebootToBootloader,
    /// `fastboot -w` — wipe userdata + cache (factory reset).
    WipeData,
    /// `fastboot flash <partition> <image>` — flash the custom recovery.
    FlashRecovery { image: PathBuf, partition: String },
    /// `fastboot reboot recovery` — boot into the freshly flashed recovery.
    RebootToRecovery,
    /// `adb sideload <zip>` — install the ROM from recovery sideload mode.
    Sideload { zip: PathBuf },
    /// Extract a signed factory-image zip and run its `flash-all` script
    /// (GrapheneOS / Pixel factory install).
    FactoryFlash { zip: PathBuf },
    /// `fastboot flashing lock` — re-lock the bootloader (verified boot).
    LockBootloader,
    /// `adb reboot` — boot into the newly installed system.
    RebootToSystem,
}

impl FlashStep {
    pub fn label(&self) -> &'static str {
        match self {
            FlashStep::UnlockBootloader => "Unlock bootloader",
            FlashStep::RebootToBootloader => "Reboot to bootloader",
            FlashStep::WipeData => "Factory reset (wipe data)",
            FlashStep::FlashRecovery { .. } => "Flash recovery",
            FlashStep::RebootToRecovery => "Reboot to recovery",
            FlashStep::Sideload { .. } => "Sideload ROM",
            FlashStep::FactoryFlash { .. } => "Flash factory image",
            FlashStep::LockBootloader => "Re-lock bootloader",
            FlashStep::RebootToSystem => "Reboot to system",
        }
    }

    /// A longer, human-facing description (including any manual action needed).
    pub fn description(&self) -> String {
        match self {
            FlashStep::UnlockBootloader => {
                "Unlocks the bootloader. Confirm on the device screen if prompted. ⚠ Erases all data.".into()
            }
            FlashStep::RebootToBootloader => "Reboots the device into fastboot/bootloader mode.".into(),
            FlashStep::WipeData => "Wipes user data and cache (factory reset). ⚠ All data lost.".into(),
            FlashStep::FlashRecovery { partition, .. } => {
                format!("Flashes the custom recovery image to the '{partition}' partition.")
            }
            FlashStep::RebootToRecovery => "Boots into the custom recovery.".into(),
            FlashStep::Sideload { .. } => {
                "In recovery, choose 'Apply update' → 'Apply from ADB', then this sideloads the ROM zip.".into()
            }
            FlashStep::FactoryFlash { .. } => {
                "Extracts the signed factory image and runs its flash-all script (flashes bootloader, radio, and system, then reboots).".into()
            }
            FlashStep::LockBootloader => {
                "Re-locks the bootloader to re-enable verified boot (recommended for GrapheneOS). ⚠ Only do this with a signed OS installed.".into()
            }
            FlashStep::RebootToSystem => "Reboots into the newly installed ROM.".into(),
        }
    }

    /// Whether this step is destructive (irreversibly changes device state/data).
    pub fn is_destructive(&self) -> bool {
        matches!(
            self,
            FlashStep::UnlockBootloader
                | FlashStep::WipeData
                | FlashStep::Sideload { .. }
                | FlashStep::FactoryFlash { .. }
                | FlashStep::LockBootloader
        )
    }

    /// Whether the frontend must obtain explicit confirmation before running it.
    pub fn requires_confirmation(&self) -> bool {
        self.is_destructive()
    }

    /// Whether the step needs the user to interact with the device physically
    /// (e.g. select an option in recovery) before/while it runs.
    pub fn needs_manual_action(&self) -> bool {
        matches!(self, FlashStep::Sideload { .. })
    }
}

/// Options describing a full install, used to build a [`FlashPlan`].
#[derive(Debug, Clone)]
pub struct FlashOptions {
    /// The installation strategy (sideload vs fastboot factory).
    pub install_method: InstallMethod,
    /// Whether to unlock the bootloader first (needed if still locked).
    pub unlock_bootloader: bool,
    /// Whether to factory-reset before installing (sideload path only).
    pub wipe_data: bool,
    /// Whether to re-lock the bootloader after a factory install.
    pub relock_after: bool,
    /// Recovery image to flash, if the ROM needs a custom recovery.
    pub recovery_image: Option<PathBuf>,
    /// Partition to flash the recovery to (`"boot"` for most A/B devices,
    /// `"recovery"` for legacy A-only devices).
    pub recovery_partition: String,
    /// The ROM zip to sideload, or the factory-image zip to flash.
    pub rom_zip: PathBuf,
}

impl FlashOptions {
    /// Sensible defaults for a modern A/B device installing a sideload ROM.
    pub fn new(rom_zip: PathBuf) -> Self {
        Self {
            install_method: InstallMethod::RecoverySideload,
            unlock_bootloader: false,
            wipe_data: true,
            relock_after: false,
            recovery_image: None,
            recovery_partition: "boot".to_string(),
            rom_zip,
        }
    }

    /// Defaults for a GrapheneOS/Pixel fastboot **factory** install.
    pub fn factory(factory_zip: PathBuf) -> Self {
        Self {
            install_method: InstallMethod::FastbootFactory,
            unlock_bootloader: true,
            wipe_data: false, // the factory flash wipes as part of flashing
            relock_after: false,
            recovery_image: None,
            recovery_partition: "boot".to_string(),
            rom_zip: factory_zip,
        }
    }
}

/// An ordered plan of [`FlashStep`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlashPlan {
    pub steps: Vec<FlashStep>,
}

/// Build a flash plan from the given options.
pub fn build_plan(opts: &FlashOptions) -> FlashPlan {
    match opts.install_method {
        InstallMethod::FastbootFactory => build_factory_plan(opts),
        InstallMethod::RecoverySideload => build_sideload_plan(opts),
    }
}

fn build_factory_plan(opts: &FlashOptions) -> FlashPlan {
    let mut steps = vec![FlashStep::RebootToBootloader];
    if opts.unlock_bootloader {
        steps.push(FlashStep::UnlockBootloader);
    }
    steps.push(FlashStep::FactoryFlash {
        zip: opts.rom_zip.clone(),
    });
    if opts.relock_after {
        steps.push(FlashStep::LockBootloader);
    }
    // flash-all reboots the device itself; no explicit reboot step needed.
    FlashPlan { steps }
}

fn build_sideload_plan(opts: &FlashOptions) -> FlashPlan {
    let mut steps = Vec::new();
    if opts.unlock_bootloader {
        steps.push(FlashStep::RebootToBootloader);
        steps.push(FlashStep::UnlockBootloader);
    }
    if (opts.recovery_image.is_some() || opts.wipe_data)
        && !steps.contains(&FlashStep::RebootToBootloader)
    {
        steps.push(FlashStep::RebootToBootloader);
    }
    if opts.wipe_data {
        steps.push(FlashStep::WipeData);
    }
    if let Some(image) = &opts.recovery_image {
        steps.push(FlashStep::FlashRecovery {
            image: image.clone(),
            partition: opts.recovery_partition.clone(),
        });
        steps.push(FlashStep::RebootToRecovery);
    }
    steps.push(FlashStep::Sideload {
        zip: opts.rom_zip.clone(),
    });
    steps.push(FlashStep::RebootToSystem);
    FlashPlan { steps }
}

/// Pre-flight warnings that should be surfaced before starting a flash.
pub fn preflight(profile: &DeviceProfile) -> Vec<String> {
    let mut warnings = Vec::new();
    if profile.codename.is_empty() {
        warnings.push("Device codename could not be detected.".to_string());
    }
    if profile.bootloader_unlocked == Some(false) {
        warnings.push("Bootloader is locked — it must be unlocked (this wipes data).".to_string());
    }
    if !FastbootManager::is_available() {
        warnings.push("`fastboot` not found in PATH — install Android platform-tools.".to_string());
    }
    if !adb_binary_available() {
        warnings
            .push("`adb` binary not found in PATH — required for the sideload step.".to_string());
    }
    warnings
}

/// Status of a single step within a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Done,
    Failed(String),
}

/// A running flash session: the plan plus per-step status. The caller advances
/// it one step at a time, gating destructive steps behind confirmation.
#[derive(Debug, Clone)]
pub struct FlashSession {
    pub plan: FlashPlan,
    pub statuses: Vec<StepStatus>,
    pub serial: String,
}

impl FlashSession {
    pub fn new(plan: FlashPlan, serial: String) -> Self {
        let statuses = vec![StepStatus::Pending; plan.steps.len()];
        Self {
            plan,
            statuses,
            serial,
        }
    }

    /// Index of the next step that has not completed yet.
    pub fn next_pending(&self) -> Option<usize> {
        self.statuses.iter().position(|s| *s == StepStatus::Pending)
    }

    /// The next step to run, if any.
    pub fn next_step(&self) -> Option<(usize, &FlashStep)> {
        self.next_pending().map(|i| (i, &self.plan.steps[i]))
    }

    /// `(completed, total)` step counts.
    pub fn progress(&self) -> (usize, usize) {
        let done = self
            .statuses
            .iter()
            .filter(|s| **s == StepStatus::Done)
            .count();
        (done, self.plan.steps.len())
    }

    pub fn is_complete(&self) -> bool {
        self.statuses.iter().all(|s| *s == StepStatus::Done)
    }

    pub fn has_failed(&self) -> bool {
        self.statuses
            .iter()
            .any(|s| matches!(s, StepStatus::Failed(_)))
    }

    /// Execute the step at `idx`, updating its status. Blocking — run this on a
    /// background thread. Returns the command output or an error string.
    pub fn run_step(
        &mut self,
        idx: usize,
        adb: &mut AdbManager,
        fastboot: &FastbootManager,
    ) -> Result<String, String> {
        let step = self.plan.steps[idx].clone();
        self.statuses[idx] = StepStatus::Running;
        let result = execute_step(&step, &self.serial, adb, fastboot);
        self.statuses[idx] = match &result {
            Ok(_) => StepStatus::Done,
            Err(e) => StepStatus::Failed(e.clone()),
        };
        result
    }
}

/// Execute a single flash step against the device (public entry point used by
/// frontends that track their own [`FlashSession`] status).
pub fn run_flash_step(
    step: &FlashStep,
    serial: &str,
    adb: &mut AdbManager,
    fastboot: &FastbootManager,
) -> Result<String, String> {
    execute_step(step, serial, adb, fastboot)
}

/// Execute a single flash step against the device.
fn execute_step(
    step: &FlashStep,
    serial: &str,
    adb: &mut AdbManager,
    fastboot: &FastbootManager,
) -> Result<String, String> {
    match step {
        FlashStep::UnlockBootloader => fastboot
            .execute(FastbootCommand::OemUnlock)
            .map_err(|e| e.to_string()),
        FlashStep::WipeData => fastboot
            .execute(FastbootCommand::WipeData)
            .map_err(|e| e.to_string()),
        FlashStep::FlashRecovery { image, partition } => fastboot
            .execute(FastbootCommand::FlashPartition {
                partition: partition.clone(),
                image_path: image.to_string_lossy().into_owned(),
            })
            .map_err(|e| e.to_string()),
        FlashStep::RebootToRecovery => fastboot
            .execute(FastbootCommand::RebootRecovery)
            .map_err(|e| e.to_string()),
        FlashStep::RebootToBootloader => adb
            .reboot(RebootTarget::Bootloader)
            .map_err(|e| e.to_string()),
        FlashStep::RebootToSystem => adb.reboot(RebootTarget::System).map_err(|e| e.to_string()),
        FlashStep::Sideload { zip } => adb_sideload(serial, &zip.to_string_lossy()),
        FlashStep::LockBootloader => fastboot
            .execute(FastbootCommand::OemLock)
            .map_err(|e| e.to_string()),
        FlashStep::FactoryFlash { zip } => factory_flash(zip),
    }
}

/// Whether the `adb` binary is available in `PATH` (needed for sideload, which
/// the pure-Rust ADB backend does not implement).
pub fn adb_binary_available() -> bool {
    Command::new("adb").arg("version").output().is_ok()
}

/// Sideload a zip via the `adb` binary (`adb -s <serial> sideload <zip>`).
fn adb_sideload(serial: &str, zip: &str) -> Result<String, String> {
    if !adb_binary_available() {
        return Err("`adb` binary not found in PATH (required for sideload)".into());
    }
    let mut cmd = Command::new("adb");
    if !serial.is_empty() {
        cmd.arg("-s").arg(serial);
    }
    let output = cmd
        .arg("sideload")
        .arg(zip)
        .output()
        .map_err(|e| format!("failed to run adb sideload: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.success() {
        Ok(format!("{stdout}{stderr}").trim().to_string())
    } else {
        Err(format!("sideload failed: {}", stderr.trim()))
    }
}

/// Extract a factory-image zip and run its bundled `flash-all` script.
///
/// GrapheneOS/Pixel factory zips ship the officially-maintained flashing
/// sequence as `flash-all.sh` (Unix) / `flash-all.bat` (Windows); running it is
/// the recommended CLI install method. Requires `fastboot` in `PATH`.
fn factory_flash(zip: &std::path::Path) -> Result<String, String> {
    if !FastbootManager::is_available() {
        return Err("`fastboot` not found in PATH (required for factory flashing)".into());
    }
    let dir = zip.with_file_name(format!(
        "{}-extracted",
        zip.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("factory")
    ));
    extract_zip(zip, &dir)?;

    let script = find_flash_all(&dir)
        .ok_or_else(|| "flash-all script not found in factory image".to_string())?;
    let script_dir = script.parent().unwrap_or(&dir).to_path_buf();

    #[cfg(windows)]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(&script);
        c
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = Command::new("bash");
        c.arg(&script);
        c
    };
    cmd.current_dir(&script_dir);

    let output = cmd
        .output()
        .map_err(|e| format!("failed to run flash-all script: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.success() {
        Ok(format!("{stdout}\n{stderr}").trim().to_string())
    } else {
        Err(format!("factory flash failed:\n{}", stderr.trim()))
    }
}

/// Extract a zip archive into `dest` (creating it).
fn extract_zip(zip_path: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| format!("open zip: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("read zip: {e}"))?;
    std::fs::create_dir_all(dest).map_err(|e| format!("create dir: {e}"))?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("zip entry: {e}"))?;
        let out_path = match entry.enclosed_name() {
            Some(p) => dest.join(p),
            None => continue,
        };
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| format!("mkdir: {e}"))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
            }
            let mut out = std::fs::File::create(&out_path).map_err(|e| format!("create: {e}"))?;
            std::io::copy(&mut entry, &mut out).map_err(|e| format!("extract: {e}"))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = entry.unix_mode() {
                    let _ =
                        std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(mode));
                }
            }
        }
    }
    Ok(())
}

/// Recursively locate the `flash-all` script within an extracted factory image.
fn find_flash_all(dir: &std::path::Path) -> Option<PathBuf> {
    let script_name = if cfg!(windows) {
        "flash-all.bat"
    } else {
        "flash-all.sh"
    };
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_flash_all(&path) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(script_name) {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> FlashOptions {
        FlashOptions::new(PathBuf::from("/tmp/rom.zip"))
    }

    #[test]
    fn plan_minimal_is_sideload_then_reboot() {
        let mut o = opts();
        o.wipe_data = false;
        let plan = build_plan(&o);
        assert_eq!(
            plan.steps,
            vec![
                FlashStep::Sideload {
                    zip: PathBuf::from("/tmp/rom.zip")
                },
                FlashStep::RebootToSystem,
            ]
        );
    }

    #[test]
    fn plan_with_unlock_wipe_and_recovery() {
        let mut o = opts();
        o.unlock_bootloader = true;
        o.recovery_image = Some(PathBuf::from("/tmp/recovery.img"));
        let plan = build_plan(&o);
        // First reboots to bootloader, unlocks, wipes, flashes recovery, etc.
        assert_eq!(plan.steps[0], FlashStep::RebootToBootloader);
        assert_eq!(plan.steps[1], FlashStep::UnlockBootloader);
        assert!(plan.steps.contains(&FlashStep::WipeData));
        assert!(plan
            .steps
            .iter()
            .any(|s| matches!(s, FlashStep::FlashRecovery { .. })));
        assert_eq!(*plan.steps.last().unwrap(), FlashStep::RebootToSystem);
    }

    #[test]
    fn destructive_steps_require_confirmation() {
        assert!(FlashStep::UnlockBootloader.requires_confirmation());
        assert!(FlashStep::WipeData.requires_confirmation());
        assert!(FlashStep::Sideload {
            zip: PathBuf::from("x")
        }
        .requires_confirmation());
        assert!(!FlashStep::RebootToSystem.requires_confirmation());
    }

    #[test]
    fn session_tracks_progress() {
        let plan = build_plan(&opts());
        let mut session = FlashSession::new(plan, "serial123".into());
        let total = session.plan.steps.len();
        assert_eq!(session.progress(), (0, total));
        assert!(!session.is_complete());
        let (idx, step) = session.next_step().unwrap();
        assert_eq!(idx, 0);
        // Default plan (wipe, no unlock/recovery) starts by entering the bootloader.
        assert_eq!(*step, FlashStep::RebootToBootloader);
        session.statuses[0] = StepStatus::Done;
        assert_eq!(session.progress().0, 1);
        assert_eq!(session.next_pending(), Some(1));
    }

    #[test]
    fn factory_plan_uses_fastboot_factory_flow() {
        let mut o = FlashOptions::factory(PathBuf::from("/tmp/grapheneos-factory.zip"));
        o.relock_after = true;
        let plan = build_plan(&o);
        assert_eq!(plan.steps[0], FlashStep::RebootToBootloader);
        assert_eq!(plan.steps[1], FlashStep::UnlockBootloader);
        assert!(plan
            .steps
            .iter()
            .any(|s| matches!(s, FlashStep::FactoryFlash { .. })));
        assert_eq!(*plan.steps.last().unwrap(), FlashStep::LockBootloader);
        // No sideload/recovery steps in the factory flow.
        assert!(!plan
            .steps
            .iter()
            .any(|s| matches!(s, FlashStep::Sideload { .. })));
    }

    #[test]
    fn preflight_flags_locked_bootloader() {
        let profile = DeviceProfile {
            codename: "sunfish".into(),
            bootloader_unlocked: Some(false),
            ..Default::default()
        };
        let warnings = preflight(&profile);
        assert!(warnings.iter().any(|w| w.contains("Bootloader is locked")));
    }
}
