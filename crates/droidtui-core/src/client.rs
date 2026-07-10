//! ADB connection manager.
//!
//! [`AdbManager`] owns the connection to the local ADB server and the currently
//! selected device.  Operations are implemented as `impl AdbManager` blocks
//! spread across the feature modules (`features/device`, `features/packages`,
//! `features/system`, `features/screen`, …), keeping this file focused on
//! connection lifecycle and shell execution.

use adb_client::{ADBDeviceExt, ADBServer};
use std::net::{Ipv4Addr, SocketAddrV4};

use crate::error::{AdbError, AdbResult};
use crate::features::device::{AdbDeviceEntry, DeviceStatus};
use crate::features::shell::AdbCommand;
use crate::utils::parse_meminfo;

/// Local ADB server port.
pub const ADB_PORT: u16 = 5037;

/// Manages the ADB server connection and command execution.
#[derive(Debug)]
pub struct AdbManager {
    server: Option<ADBServer>,
    selected_device: Option<String>,
}

impl Default for AdbManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AdbManager {
    /// Create a new, unconnected ADB manager.
    pub fn new() -> Self {
        Self {
            server: None,
            selected_device: None,
        }
    }

    /// Address of the local ADB server (`127.0.0.1:5037`).
    fn server_addr() -> SocketAddrV4 {
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), ADB_PORT)
    }

    /// Connect to the local ADB server.
    pub fn connect(&mut self) -> AdbResult<()> {
        self.server = Some(ADBServer::new(Self::server_addr()));
        Ok(())
    }

    /// Ensure the server is connected, returning a mutable handle.
    pub(crate) fn get_server(&mut self) -> AdbResult<&mut ADBServer> {
        if self.server.is_none() {
            self.connect()?;
        }
        self.server
            .as_mut()
            .ok_or_else(|| AdbError::ConnectionError("Failed to connect".to_string()))
    }

    /// Set the selected device serial.
    pub fn select_device(&mut self, serial: String) {
        self.selected_device = Some(serial);
    }

    /// The currently selected device serial, if any.
    pub fn selected_device(&self) -> Option<&str> {
        self.selected_device.as_deref()
    }

    /// The selected device serial or [`AdbError::NoDeviceSelected`].
    pub(crate) fn require_selected_device(&self) -> AdbResult<&str> {
        self.selected_device
            .as_deref()
            .ok_or(AdbError::NoDeviceSelected)
    }

    /// Execute a shell command on the selected device and return its output.
    pub fn shell_command(&mut self, command: &str) -> AdbResult<String> {
        let server = self.get_server()?;
        let mut device = server
            .get_device()
            .map_err(|e| AdbError::ConnectionError(format!("Failed to get device: {}", e)))?;

        let mut output = Vec::new();
        device
            .shell_command(&[command], &mut output)
            .map_err(|e| AdbError::CommandFailed(format!("Shell command failed: {}", e)))?;

        let result = String::from_utf8_lossy(&output).to_string();
        if result.trim().is_empty() {
            Ok("Command executed successfully (no output)".to_string())
        } else {
            Ok(result)
        }
    }

    /// Execute a shell command and return its raw (binary-safe) stdout bytes.
    ///
    /// Use this for binary output such as `screencap -p` PNG frames, where a
    /// lossy UTF-8 conversion would corrupt the data.
    pub fn shell_command_raw(&mut self, command: &str) -> AdbResult<Vec<u8>> {
        let server = self.get_server()?;
        let mut device = server
            .get_device()
            .map_err(|e| AdbError::ConnectionError(format!("Failed to get device: {}", e)))?;
        let mut output = Vec::new();
        device
            .shell_command(&[command], &mut output)
            .map_err(|e| AdbError::CommandFailed(format!("Shell command failed: {}", e)))?;
        Ok(output)
    }

    /// Get the ADB server version string.
    pub fn get_adb_version(&mut self) -> AdbResult<String> {
        let server = self.get_server()?;
        let version = server.version()?;
        Ok(format!(
            "ADB server version: {}.{}.{}",
            version.major, version.minor, version.revision
        ))
    }

    /// Dispatch a typed [`AdbCommand`] to its concrete implementation.
    pub fn execute(&mut self, command: AdbCommand) -> AdbResult<String> {
        match command {
            AdbCommand::ListDevices => self.list_devices(),
            AdbCommand::GetDeviceState => self.get_device_state(),
            AdbCommand::GetSerialNumber => self.get_serial_number(),
            AdbCommand::ListPackages {
                include_path,
                filter,
            } => self.list_packages(include_path, filter),
            AdbCommand::GetPackageInfo { package_name } => self.get_package_info(&package_name),
            AdbCommand::InstallPackage { apk_path } => self.install_package(&apk_path),
            AdbCommand::UninstallPackage { package_name } => self.uninstall_package(&package_name),
            AdbCommand::ClearPackageData { package_name } => self.clear_package_data(&package_name),
            AdbCommand::GetBatteryInfo => self.get_battery_info(),
            AdbCommand::GetMemoryInfo => self.get_memory_info(),
            AdbCommand::GetCpuInfo => self.get_cpu_info(),
            AdbCommand::GetDeviceProperties => self.get_device_properties(),
            AdbCommand::GetSystemLog { lines } => self.get_system_log(lines),
            AdbCommand::GetNetworkInfo => self.get_network_info(),
            AdbCommand::GetWifiStatus => self.get_wifi_status(),
            AdbCommand::TakeScreenshot => self.take_screenshot(),
            AdbCommand::GetScreenResolution => self.get_screen_resolution(),
            AdbCommand::ListProcesses => self.list_processes(),
            AdbCommand::ForceStop { package_name } => self.force_stop(&package_name),
            AdbCommand::Shell { command } => self.shell_command(&command),
            AdbCommand::GetAdbVersion => self.get_adb_version(),
        }
    }

    /// Fetch a comprehensive snapshot of all ADB devices and live stats for the
    /// active one.  Non-panicking — any failure returns a partial or empty
    /// [`DeviceStatus`].
    pub fn fetch_device_status(&mut self) -> DeviceStatus {
        if self.connect().is_err() {
            return DeviceStatus::default();
        }

        let server = match self.server.as_mut() {
            Some(s) => s,
            None => return DeviceStatus::default(),
        };

        let raw = match server.devices() {
            Ok(d) => d,
            Err(_) => return DeviceStatus::default(),
        };

        if raw.is_empty() {
            return DeviceStatus::default();
        }

        let devices: Vec<AdbDeviceEntry> = raw
            .iter()
            .map(|d| AdbDeviceEntry {
                serial: d.identifier.clone(),
                state: format!("{:?}", d.state).to_lowercase(),
            })
            .collect();

        let selected_idx = if let Some(sel) = &self.selected_device {
            devices.iter().position(|d| &d.serial == sel).unwrap_or(0)
        } else {
            0
        };
        self.selected_device = Some(devices[selected_idx].serial.clone());

        let clean = |s: String| -> String {
            let t = s.trim().to_string();
            if t.is_empty() || t.contains("no output") {
                String::new()
            } else {
                t
            }
        };

        let model = self
            .shell_command("getprop ro.product.model")
            .ok()
            .map(clean)
            .unwrap_or_default();
        let android_version = self
            .shell_command("getprop ro.build.version.release")
            .ok()
            .map(clean)
            .unwrap_or_default();
        let battery_pct = self
            .shell_command("dumpsys battery | grep '  level:'")
            .ok()
            .and_then(|s| {
                s.lines()
                    .next()
                    .and_then(|l| l.split(':').nth(1))
                    .and_then(|v| v.trim().parse::<u8>().ok())
            })
            .unwrap_or(0);
        let (ram_total_mib, ram_avail_mib) = self
            .shell_command("grep -E 'MemTotal:|MemAvailable:' /proc/meminfo")
            .ok()
            .map(|s| parse_meminfo(&s))
            .unwrap_or((0, 0));
        let cpu_load_1min = self
            .shell_command("cat /proc/loadavg")
            .ok()
            .and_then(|s| {
                s.split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<f32>().ok())
            })
            .unwrap_or(0.0);

        DeviceStatus {
            devices,
            selected_idx,
            model,
            android_version,
            battery_pct,
            ram_total_mib,
            ram_avail_mib,
            cpu_load_1min,
        }
    }

    /// Select the device at `status.selected_idx` and re-fetch stats.
    pub fn select_device_from_status(&mut self, status: &mut DeviceStatus) {
        if let Some(dev) = status.devices.get(status.selected_idx) {
            self.selected_device = Some(dev.serial.clone());
        }
        *status = self.fetch_device_status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manager_starts_unconnected() {
        let m = AdbManager::new();
        assert!(m.server.is_none());
        assert!(m.selected_device.is_none());
    }

    #[test]
    fn select_device_sets_serial() {
        let mut m = AdbManager::new();
        m.select_device("emulator-5554".into());
        assert_eq!(m.selected_device(), Some("emulator-5554"));
    }
}
