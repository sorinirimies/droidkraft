//! Fastboot domain types shared by every backend.

use std::fmt;

pub type FastbootResult<T> = Result<T, FastbootError>;

/// Errors from fastboot operations.
#[derive(Debug)]
pub enum FastbootError {
    /// `fastboot` binary not found in PATH.
    NotInstalled,
    /// No device detected in fastboot mode.
    NoDevice,
    /// USB-level failure in the native backend (permissions, driver, …).
    Usb(String),
    /// Command failed.
    CommandFailed(String),
    /// I/O error spawning the process.
    Io(std::io::Error),
}

impl fmt::Display for FastbootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FastbootError::NotInstalled => write!(
                f,
                "fastboot not found in PATH.\n\
                 Install Android platform-tools:\n\
                 https://developer.android.com/tools/releases/platform-tools"
            ),
            FastbootError::NoDevice => write!(
                f,
                "No device detected in fastboot mode.\n\n\
                 Steps:\n\
                 1. Connect the device via USB\n\
                 2. Reboot it into the bootloader (`adb reboot bootloader`)\n\
                 3. Retry this command"
            ),
            FastbootError::Usb(msg) => write!(f, "USB error: {msg}"),
            FastbootError::CommandFailed(msg) => write!(f, "{}", msg),
            FastbootError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for FastbootError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FastbootError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for FastbootError {
    fn from(e: std::io::Error) -> Self {
        FastbootError::Io(e)
    }
}

/// High-level fastboot operations.
///
/// Run them with [`FastbootManager::execute`](crate::FastbootManager::execute).
#[derive(Debug, Clone)]
pub enum FastbootCommand {
    /// Unlock the bootloader — enables custom ROM flashing. ⚠ Wipes device data.
    OemUnlock,
    /// Re-lock the bootloader after flashing.
    OemLock,
    /// Factory reset: wipe userdata + cache partitions. ⚠ All data lost.
    WipeData,
    /// Flash an image file onto a named partition.
    FlashPartition {
        partition: String,
        image_path: String,
    },
    /// Retrieve all fastboot variables (device information).
    GetVarAll,
    /// Reboot the device normally from fastboot mode.
    Reboot,
    /// Reboot directly into recovery from fastboot mode.
    RebootRecovery,
    /// Erase a partition (`fastboot erase <partition>`). ⚠ Data loss.
    Erase { partition: String },
    /// Switch the active A/B slot (`a` or `b`).
    SetActiveSlot { slot: String },
    /// Read a single fastboot variable.
    GetVar { name: String },
    /// Arbitrary fastboot command line (e.g. `oem device-info`). ⚠ Treated as destructive.
    Raw(String),
    /// Reboot back into the bootloader.
    RebootBootloader,
    /// Reboot into userspace fastboot (fastbootd).
    RebootFastbootd,
}

impl FastbootCommand {
    /// Flash `image` onto `partition`.
    pub fn flash(partition: impl Into<String>, image: impl AsRef<std::path::Path>) -> Self {
        Self::FlashPartition {
            partition: partition.into(),
            image_path: image.as_ref().to_string_lossy().into_owned(),
        }
    }

    /// The argument vector passed to the `fastboot` binary.
    pub fn args(&self) -> Vec<String> {
        match self {
            Self::OemUnlock => vec!["flashing".into(), "unlock".into()],
            Self::OemLock => vec!["flashing".into(), "lock".into()],
            Self::WipeData => vec!["-w".into()],
            Self::GetVarAll => vec!["getvar".into(), "all".into()],
            Self::Reboot => vec!["reboot".into()],
            Self::RebootRecovery => vec!["reboot".into(), "recovery".into()],
            Self::FlashPartition {
                partition,
                image_path,
            } => vec!["flash".into(), partition.clone(), image_path.clone()],
            Self::Erase { partition } => vec!["erase".into(), partition.clone()],
            Self::SetActiveSlot { slot } => vec![format!("--set-active={slot}")],
            Self::GetVar { name } => vec!["getvar".into(), name.clone()],
            Self::Raw(line) => line.split_whitespace().map(String::from).collect(),
            Self::RebootBootloader => vec!["reboot".into(), "bootloader".into()],
            Self::RebootFastbootd => vec!["reboot".into(), "fastboot".into()],
        }
    }

    /// A short human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::OemUnlock => "Unlock Bootloader",
            Self::OemLock => "Lock Bootloader",
            Self::WipeData => "Wipe Data (Factory Reset)",
            Self::FlashPartition { .. } => "Flash Partition",
            Self::GetVarAll => "Get Device Variables",
            Self::Reboot => "Reboot",
            Self::RebootRecovery => "Reboot to Recovery",
            Self::Erase { .. } => "Erase Partition",
            Self::SetActiveSlot { .. } => "Set Active Slot",
            Self::GetVar { .. } => "Get Variable",
            Self::Raw(_) => "Raw Command",
            Self::RebootBootloader => "Reboot to Bootloader",
            Self::RebootFastbootd => "Reboot to Fastbootd",
        }
    }

    /// Whether this command is destructive (wipes data / changes lock state).
    pub fn is_destructive(&self) -> bool {
        matches!(
            self,
            Self::OemUnlock | Self::OemLock | Self::WipeData | Self::Erase { .. } | Self::Raw(_)
        )
    }
}

/// Parse `getvar all` output (or a single `getvar`) into a map.
///
/// Accepts `key: value` lines with an optional `(bootloader) ` prefix, as
/// printed by both the binary and the native backend. Lines without a colon
/// (e.g. `Finished. Total time: ...`) are skipped.
pub fn parse_variables(output: &str) -> std::collections::BTreeMap<String, String> {
    let mut map = std::collections::BTreeMap::new();
    for line in output.lines() {
        let line = line.trim().trim_start_matches("(bootloader)").trim();
        // Keys may contain ':' (`partition-size:boot_a`): split at ": ".
        let Some((k, v)) = line.split_once(": ") else {
            continue;
        };
        if k.is_empty() || k.contains(' ') || k == "Finished" {
            continue;
        }
        map.insert(k.to_string(), v.trim().to_string());
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oem_unlock_args() {
        assert_eq!(
            FastbootCommand::OemUnlock.args(),
            vec!["flashing", "unlock"]
        );
    }

    #[test]
    fn oem_lock_args() {
        assert_eq!(FastbootCommand::OemLock.args(), vec!["flashing", "lock"]);
    }

    #[test]
    fn wipe_data_args() {
        assert_eq!(FastbootCommand::WipeData.args(), vec!["-w"]);
    }

    #[test]
    fn get_var_all_args() {
        assert_eq!(FastbootCommand::GetVarAll.args(), vec!["getvar", "all"]);
    }

    #[test]
    fn reboot_args() {
        assert_eq!(FastbootCommand::Reboot.args(), vec!["reboot"]);
    }

    #[test]
    fn flash_partition_args() {
        let cmd = FastbootCommand::FlashPartition {
            partition: "boot".into(),
            image_path: "/tmp/boot.img".into(),
        };
        assert_eq!(cmd.args(), vec!["flash", "boot", "/tmp/boot.img"]);
    }

    #[test]
    fn destructive_classification() {
        assert!(FastbootCommand::WipeData.is_destructive());
        assert!(!FastbootCommand::Reboot.is_destructive());
    }

    #[test]
    fn new_command_args() {
        assert_eq!(
            FastbootCommand::Erase {
                partition: "userdata".into()
            }
            .args(),
            vec!["erase", "userdata"]
        );
        assert_eq!(
            FastbootCommand::SetActiveSlot { slot: "b".into() }.args(),
            vec!["--set-active=b"]
        );
        assert_eq!(
            FastbootCommand::GetVar {
                name: "product".into()
            }
            .args(),
            vec!["getvar", "product"]
        );
        assert_eq!(
            FastbootCommand::Raw("oem device-info".into()).args(),
            vec!["oem", "device-info"]
        );
        assert_eq!(
            FastbootCommand::RebootBootloader.args(),
            vec!["reboot", "bootloader"]
        );
        assert_eq!(
            FastbootCommand::RebootFastbootd.args(),
            vec!["reboot", "fastboot"]
        );
        assert!(FastbootCommand::Erase {
            partition: "x".into()
        }
        .is_destructive());
        assert!(!FastbootCommand::GetVar { name: "x".into() }.is_destructive());
    }

    #[test]
    fn parses_variables() {
        let m = parse_variables(
            "(bootloader) product: husky\n(bootloader) partition-size:boot_a: 0x4000000\nmax-download-size: 0x10000000\nFinished. Total time: 0.001s\n",
        );
        assert_eq!(m["product"], "husky");
        assert_eq!(m["partition-size:boot_a"], "0x4000000");
        assert_eq!(m["max-download-size"], "0x10000000");
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn flash_constructor() {
        assert_eq!(
            FastbootCommand::flash("boot", "/tmp/b.img").args(),
            vec!["flash", "boot", "/tmp/b.img"]
        );
    }
}
