//! Typed ADB command descriptions.

use crate::features::packages::PackageFilter;

/// A declarative description of an ADB operation.
///
/// Dispatched to a concrete `AdbManager` method by [`AdbManager::execute`].
#[derive(Debug, Clone)]
pub enum AdbCommand {
    // Device
    ListDevices,
    GetDeviceState,
    GetSerialNumber,

    // Packages
    ListPackages {
        include_path: bool,
        filter: PackageFilter,
    },
    GetPackageInfo {
        package_name: String,
    },
    InstallPackage {
        apk_path: String,
    },
    UninstallPackage {
        package_name: String,
    },
    ClearPackageData {
        package_name: String,
    },

    // System
    GetBatteryInfo,
    GetMemoryInfo,
    GetCpuInfo,
    GetDeviceProperties,
    GetSystemLog {
        lines: usize,
    },

    // Network
    GetNetworkInfo,
    GetWifiStatus,

    // Screen
    TakeScreenshot,
    GetScreenResolution,

    // Processes
    ListProcesses,
    ForceStop {
        package_name: String,
    },

    // Raw shell
    Shell {
        command: String,
    },

    // Meta
    GetAdbVersion,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_is_cloneable() {
        let cmd = AdbCommand::Shell {
            command: "ls".into(),
        };
        let _ = cmd.clone();
        assert!(matches!(AdbCommand::ListDevices, AdbCommand::ListDevices));
    }
}
