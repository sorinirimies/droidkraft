//! Package operations on [`AdbManager`].

use crate::client::AdbManager;
use crate::error::{AdbError, AdbResult};
use crate::features::packages::PackageFilter;

impl AdbManager {
    /// List installed packages, optionally including the APK path (`-f`).
    pub fn list_packages(
        &mut self,
        include_path: bool,
        filter: PackageFilter,
    ) -> AdbResult<String> {
        let path_arg = if include_path { " -f" } else { "" };
        let command = format!("pm list packages{}{}", path_arg, filter.arg());
        self.shell_command(&command)
    }

    /// Detailed `dumpsys package` info for a package.
    pub fn get_package_info(&mut self, package_name: &str) -> AdbResult<String> {
        self.shell_command(&format!("dumpsys package {}", package_name))
    }

    /// Install an APK. Not yet supported by the pure-Rust backend.
    pub fn install_package(&mut self, _apk_path: &str) -> AdbResult<String> {
        Err(AdbError::CommandFailed(
            "Install via adb_client not yet implemented. Use a shell command.".to_string(),
        ))
    }
}

// Generate the simple single-argument package operations via macro.
impl AdbManager {
    crate::shell_arg_ops! {
        /// Uninstall a package by name.
        uninstall_package(package_name) => "pm uninstall {}";
        /// Clear a package's data.
        clear_package_data(package_name) => "pm clear {}";
        /// Force-stop a running application.
        force_stop(package_name) => "am force-stop {}";
    }
}
