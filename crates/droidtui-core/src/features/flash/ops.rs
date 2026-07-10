//! Flash & root operations on [`AdbManager`].

use crate::client::AdbManager;
use crate::error::AdbResult;
use crate::features::flash::types::{RebootTarget, RootMethod, RootStatus};

impl AdbManager {
    /// Reboot the selected device into the given [`RebootTarget`].
    pub fn reboot(&mut self, target: RebootTarget) -> AdbResult<String> {
        let arg = target.arg();
        let cmd = if arg.is_empty() {
            "reboot".to_string()
        } else {
            format!("reboot {}", arg)
        };
        self.shell_command(&cmd)?;
        Ok(format!("{} requested.", target.label()))
    }

    /// Detect the root status of the selected device.
    ///
    /// Checks, in order: an already-root `adbd` (`id`), a Magisk install
    /// (`magisk -V`), and a generic `su` binary (`which su`).
    pub fn detect_root(&mut self) -> AdbResult<RootStatus> {
        // 1. Is adbd already root?
        if let Ok(id) = self.shell_command("id") {
            if id.contains("uid=0(root)") {
                return Ok(RootStatus {
                    is_rooted: true,
                    method: RootMethod::AdbdRoot,
                    magisk_version: None,
                });
            }
        }

        // 2. Magisk?
        if let Ok(ver) = self.shell_command("su -c 'magisk -V' 2>/dev/null") {
            let ver = ver.trim();
            if !ver.is_empty()
                && ver.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
            {
                return Ok(RootStatus {
                    is_rooted: true,
                    method: RootMethod::Magisk,
                    magisk_version: Some(ver.to_string()),
                });
            }
        }

        // 3. Generic su binary?
        if let Ok(which) = self.shell_command("which su") {
            if which.trim().contains("/su") {
                return Ok(RootStatus {
                    is_rooted: true,
                    method: RootMethod::SuBinary,
                    magisk_version: None,
                });
            }
        }

        Ok(RootStatus::not_rooted())
    }

    /// Remount `/system` (and other partitions) read-write. Requires root.
    pub fn remount(&mut self) -> AdbResult<String> {
        self.shell_command("su -c 'mount -o rw,remount /' || mount -o rw,remount /")
    }

    /// Run an arbitrary command as root via `su -c`.
    pub fn run_as_root(&mut self, command: &str) -> AdbResult<String> {
        self.shell_command(&format!("su -c '{}'", command.replace('\'', "'\\''")))
    }
}
