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
use crate::features::rom::types::DeviceProfile;

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
            FlashStep::RebootToSystem => "Reboots into the newly installed ROM.".into(),
        }
    }

    /// Whether this step is destructive (irreversibly changes device state/data).
    pub fn is_destructive(&self) -> bool {
        matches!(
            self,
            FlashStep::UnlockBootloader | FlashStep::WipeData | FlashStep::Sideload { .. }
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
    /// Whether to unlock the bootloader first (needed if still locked).
    pub unlock_bootloader: bool,
    /// Whether to factory-reset before installing.
    pub wipe_data: bool,
    /// Recovery image to flash, if the ROM needs a custom recovery.
    pub recovery_image: Option<PathBuf>,
    /// Partition to flash the recovery to (`"boot"` for most A/B devices,
    /// `"recovery"` for legacy A-only devices).
    pub recovery_partition: String,
    /// The ROM zip to sideload.
    pub rom_zip: PathBuf,
}

impl FlashOptions {
    /// Sensible defaults for a modern A/B device that is already unlocked.
    pub fn new(rom_zip: PathBuf) -> Self {
        Self {
            unlock_bootloader: false,
            wipe_data: true,
            recovery_image: None,
            recovery_partition: "boot".to_string(),
            rom_zip,
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
    let mut steps = Vec::new();
    if opts.unlock_bootloader {
        steps.push(FlashStep::RebootToBootloader);
        steps.push(FlashStep::UnlockBootloader);
    }
    if opts.recovery_image.is_some() || opts.wipe_data {
        // Ensure we are in the bootloader for fastboot operations.
        if !steps.contains(&FlashStep::RebootToBootloader) {
            steps.push(FlashStep::RebootToBootloader);
        }
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
