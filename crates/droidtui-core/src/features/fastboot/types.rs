//! Fastboot domain types.

use std::fmt;

pub type FastbootResult<T> = Result<T, FastbootError>;

/// Errors from fastboot operations.
#[derive(Debug)]
pub enum FastbootError {
    /// `fastboot` binary not found in PATH.
    NotInstalled,
    /// No device detected in fastboot mode.
    NoDevice,
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
                 2. Run 'Reboot to Bootloader' from the ROOT & FLASH menu\n\
                 3. Retry this command"
            ),
            FastbootError::CommandFailed(msg) => write!(f, "{}", msg),
            FastbootError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for FastbootError {}

impl From<std::io::Error> for FastbootError {
    fn from(e: std::io::Error) -> Self {
        FastbootError::Io(e)
    }
}

/// Fastboot commands that require the `fastboot` binary.
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
}

impl FastbootCommand {
    /// The argument vector passed to the `fastboot` binary.
    pub fn args(&self) -> Vec<String> {
        match self {
            Self::OemUnlock => vec!["flashing".into(), "unlock".into()],
            Self::OemLock => vec!["flashing".into(), "lock".into()],
            Self::WipeData => vec!["-w".into()],
            Self::GetVarAll => vec!["getvar".into(), "all".into()],
            Self::Reboot => vec!["reboot".into()],
            Self::FlashPartition {
                partition,
                image_path,
            } => vec!["flash".into(), partition.clone(), image_path.clone()],
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
        }
    }

    /// Whether this command is destructive (wipes data / changes lock state).
    pub fn is_destructive(&self) -> bool {
        matches!(self, Self::OemUnlock | Self::OemLock | Self::WipeData)
    }
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
}
