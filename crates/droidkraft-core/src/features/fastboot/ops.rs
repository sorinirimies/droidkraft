//! Fastboot execution — shells out to the `fastboot` binary.

use std::process::Command;

use super::types::{FastbootCommand, FastbootError, FastbootResult};

/// Executes fastboot operations by shelling out to the `fastboot` binary.
#[derive(Debug, Default, Clone, Copy)]
pub struct FastbootManager;

impl FastbootManager {
    pub fn new() -> Self {
        Self
    }

    /// Returns `true` when `fastboot` is available in `PATH`.
    pub fn is_available() -> bool {
        Command::new("fastboot").arg("--version").output().is_ok()
    }

    /// Execute a fastboot command, returning its combined output.
    pub fn execute(&self, command: FastbootCommand) -> FastbootResult<String> {
        if !Self::is_available() {
            return Err(FastbootError::NotInstalled);
        }

        let output = Command::new("fastboot").args(command.args()).output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        // fastboot writes most output to stderr even on success.
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
