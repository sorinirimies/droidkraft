//! Fastboot operations via the `fastboot` CLI tool.
//!
//! The fastboot protocol is completely separate from ADB and requires the `fastboot`
//! binary (part of Android platform-tools). There is currently no pure-Rust fastboot
//! library, so this module shells out to the `fastboot` binary.

use std::fmt;
use std::process::Command;

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
    fn args(&self) -> Vec<String> {
        match self {
            Self::OemUnlock => vec!["flashing".into(), "unlock".into()],
            Self::OemLock => vec!["flashing".into(), "lock".into()],
            Self::WipeData => vec!["-w".into()],
            Self::GetVarAll => vec!["getvar".into(), "all".into()],
            Self::Reboot => vec!["reboot".into()],
            Self::FlashPartition {
                partition,
                image_path,
            } => {
                vec!["flash".into(), partition.clone(), image_path.clone()]
            }
        }
    }
}

/// Executes fastboot operations by shelling out to the `fastboot` binary.
#[derive(Debug, Default)]
pub struct FastbootManager;

impl FastbootManager {
    pub fn new() -> Self {
        Self
    }

    /// Returns `true` when `fastboot` is available in `PATH`.
    pub fn is_available() -> bool {
        Command::new("fastboot").arg("--version").output().is_ok()
    }

    pub fn execute(&self, command: FastbootCommand) -> FastbootResult<String> {
        if !Self::is_available() {
            return Err(FastbootError::NotInstalled);
        }

        let output = Command::new("fastboot").args(command.args()).output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        // fastboot writes most output to stderr even on success
        let body = if stdout.trim().is_empty() {
            stderr.clone()
        } else {
            stdout
        };

        if stderr.contains("no devices/emulators found") || stderr.contains("no fastboot devices") {
            return Err(FastbootError::NoDevice);
        }

        if !output.status.success() && body.contains("FAILED") {
            return Err(FastbootError::CommandFailed(body));
        }

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oem_unlock_args() {
        assert_eq!(
            FastbootCommand::OemUnlock.args(),
            vec!["flashing", "unlock"]
        );
    }

    #[test]
    fn test_oem_lock_args() {
        assert_eq!(FastbootCommand::OemLock.args(), vec!["flashing", "lock"]);
    }

    #[test]
    fn test_wipe_data_args() {
        assert_eq!(FastbootCommand::WipeData.args(), vec!["-w"]);
    }

    #[test]
    fn test_get_var_all_args() {
        assert_eq!(FastbootCommand::GetVarAll.args(), vec!["getvar", "all"]);
    }

    #[test]
    fn test_reboot_args() {
        assert_eq!(FastbootCommand::Reboot.args(), vec!["reboot"]);
    }

    #[test]
    fn test_flash_partition_args() {
        let cmd = FastbootCommand::FlashPartition {
            partition: "boot".into(),
            image_path: "/tmp/boot.img".into(),
        };
        assert_eq!(cmd.args(), vec!["flash", "boot", "/tmp/boot.img"]);
    }
}
