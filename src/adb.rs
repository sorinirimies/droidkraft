//! ADB Client Abstraction Layer
//!
//! This module provides a high-level interface to ADB operations using the adb_client crate.
//! It abstracts away the complexity of working with ADB and provides typed command execution.

use adb_client::{ADBDeviceExt, ADBServer};
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};

/// Result type for ADB operations
pub type AdbResult<T> = Result<T, AdbError>;

/// Custom error type for ADB operations
#[derive(Debug)]
pub enum AdbError {
    ConnectionError(String),
    DeviceNotFound,
    CommandFailed(String),
    IoError(io::Error),
    ParseError(String),
    NoDeviceSelected,
}

impl std::fmt::Display for AdbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdbError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            AdbError::DeviceNotFound => write!(f, "Device not found"),
            AdbError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            AdbError::IoError(e) => write!(f, "IO error: {}", e),
            AdbError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AdbError::NoDeviceSelected => write!(f, "No device selected"),
        }
    }
}

impl std::error::Error for AdbError {}

impl From<io::Error> for AdbError {
    fn from(e: io::Error) -> Self {
        AdbError::IoError(e)
    }
}

impl From<adb_client::RustADBError> for AdbError {
    fn from(e: adb_client::RustADBError) -> Self {
        AdbError::CommandFailed(e.to_string())
    }
}

/// ADB command type
#[derive(Debug, Clone)]
pub enum AdbCommand {
    // Device commands
    ListDevices,
    GetDeviceState,
    GetSerialNumber,

    // Package commands
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

    // System commands
    GetBatteryInfo,
    GetMemoryInfo,
    GetCpuInfo,
    GetDeviceProperties,
    GetSystemLog {
        lines: usize,
    },

    // Network commands
    GetNetworkInfo,
    GetWifiStatus,

    // Screen commands
    TakeScreenshot,
    GetScreenResolution,

    // Process commands
    ListProcesses,
    ForceStop {
        package_name: String,
    },

    // Shell commands
    Shell {
        command: String,
    },

    // Version
    GetAdbVersion,
}

/// Package filter options
#[derive(Debug, Clone)]
pub enum PackageFilter {
    All,
    User,     // -3
    System,   // -s
    Enabled,  // -e
    Disabled, // -d
}

/// Device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub serial: String,
    pub state: String,
    pub model: Option<String>,
    pub device: Option<String>,
}

/// A single device entry returned by `adb devices`.
#[derive(Debug, Clone)]
pub struct AdbDeviceEntry {
    pub serial: String,
    /// ADB state string: "device", "offline", "unauthorized", etc.
    pub state: String,
}

/// Comprehensive device status shown in the right-hand dashboard.
#[derive(Debug, Clone, Default)]
pub struct DeviceStatus {
    /// All devices currently visible to ADB (empty → nothing connected).
    pub devices: Vec<AdbDeviceEntry>,
    /// Which entry in `devices` is active (index).
    pub selected_idx: usize,
    // ── Stats for the active device ──────────────────────────────────────────
    /// Product model (ro.product.model).
    pub model: String,
    /// Android version (ro.build.version.release).
    pub android_version: String,
    /// Battery percentage 0–100.
    pub battery_pct: u8,
    /// Total RAM in MiB.
    pub ram_total_mib: u64,
    /// Available RAM in MiB.
    pub ram_avail_mib: u64,
    /// 1-minute load average from /proc/loadavg.
    pub cpu_load_1min: f32,
}

impl DeviceStatus {
    pub fn is_connected(&self) -> bool {
        !self.devices.is_empty()
    }

    pub fn active(&self) -> Option<&AdbDeviceEntry> {
        self.devices.get(self.selected_idx)
    }

    /// Advance selection to the next device, wrapping around.
    pub fn cycle_next(&mut self) {
        if !self.devices.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.devices.len();
        }
    }
}

/// ADB Manager - handles connection and command execution
#[derive(Debug)]
pub struct AdbManager {
    server: Option<ADBServer>,
    selected_device: Option<String>,
}

impl AdbManager {
    /// Create a new ADB manager
    pub fn new() -> Self {
        Self {
            server: None,
            selected_device: None,
        }
    }

    /// Connect to ADB server
    pub fn connect(&mut self) -> AdbResult<()> {
        let socket_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);
        let server = ADBServer::new(socket_addr);
        self.server = Some(server);
        Ok(())
    }

    /// Ensure server is connected
    fn get_server(&mut self) -> AdbResult<&mut ADBServer> {
        if self.server.is_none() {
            self.connect()?;
        }
        self.server
            .as_mut()
            .ok_or(AdbError::ConnectionError("Failed to connect".to_string()))
    }

    /// Set the selected device
    pub fn select_device(&mut self, serial: String) {
        self.selected_device = Some(serial);
    }

    /// Fetch a comprehensive snapshot of all ADB devices and live stats for
    /// the active one.  Non-panicking — any failure returns a partial or empty
    /// `DeviceStatus`.
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

        // Build the device list
        let devices: Vec<AdbDeviceEntry> = raw
            .iter()
            .map(|d| AdbDeviceEntry {
                serial: d.identifier.clone(),
                state: format!("{:?}", d.state).to_lowercase(),
            })
            .collect();

        // Keep previously selected device if it is still present
        let selected_idx = if let Some(sel) = &self.selected_device {
            devices.iter().position(|d| &d.serial == sel).unwrap_or(0)
        } else {
            0
        };

        self.selected_device = Some(devices[selected_idx].serial.clone());

        // ── Per-device stats ─────────────────────────────────────────────────

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

    /// Select the device at `selected_idx` in `status` and re-fetch stats.
    pub fn select_device_from_status(&mut self, status: &mut DeviceStatus) {
        if let Some(dev) = status.devices.get(status.selected_idx) {
            self.selected_device = Some(dev.serial.clone());
        }
        *status = self.fetch_device_status();
    }

    /// Get the selected device serial
    fn get_selected_device(&self) -> AdbResult<&str> {
        self.selected_device
            .as_deref()
            .ok_or(AdbError::NoDeviceSelected)
    }

    /// Execute an ADB command
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

    /// List all connected devices
    fn list_devices(&mut self) -> AdbResult<String> {
        let server = self.get_server()?;
        let devices = server.devices()?;

        if devices.is_empty() {
            return Ok("No devices found.\n\nMake sure:\n- Device is connected via USB\n- USB debugging is enabled\n- Device is authorized".to_string());
        }

        let mut output = String::from("List of devices attached:\n\n");
        for device in devices {
            output.push_str(&format!("  {:<24} {:?}\n", device.identifier, device.state));

            // Auto-select first device if none selected
            if self.selected_device.is_none() {
                self.selected_device = Some(device.identifier.clone());
            }
        }

        Ok(output)
    }

    /// Get device state
    fn get_device_state(&mut self) -> AdbResult<String> {
        let serial = self.get_selected_device()?.to_string();
        let server = self.get_server()?;
        let devices = server.devices()?;

        for device in devices {
            if device.identifier == serial {
                return Ok(format!("Device state: {:?}", device.state));
            }
        }

        Err(AdbError::DeviceNotFound)
    }

    /// Get device serial number
    fn get_serial_number(&mut self) -> AdbResult<String> {
        let serial = self.get_selected_device()?;
        Ok(format!("Serial number: {}", serial))
    }

    /// List packages
    fn list_packages(&mut self, include_path: bool, filter: PackageFilter) -> AdbResult<String> {
        let filter_arg = match filter {
            PackageFilter::All => "",
            PackageFilter::User => " -3",
            PackageFilter::System => " -s",
            PackageFilter::Enabled => " -e",
            PackageFilter::Disabled => " -d",
        };

        let path_arg = if include_path { " -f" } else { "" };
        let command = format!("pm list packages{}{}", path_arg, filter_arg);

        self.shell_command(&command)
    }

    /// Get package information
    fn get_package_info(&mut self, package_name: &str) -> AdbResult<String> {
        let command = format!("dumpsys package {}", package_name);
        self.shell_command(&command)
    }

    /// Install package
    fn install_package(&mut self, _apk_path: &str) -> AdbResult<String> {
        // Note: adb_client doesn't have direct install support in the API we're using
        // We'll use shell command as fallback
        Err(AdbError::CommandFailed(
            "Install via adb_client not yet implemented. Use shell command.".to_string(),
        ))
    }

    /// Uninstall package
    fn uninstall_package(&mut self, package_name: &str) -> AdbResult<String> {
        let command = format!("pm uninstall {}", package_name);
        self.shell_command(&command)
    }

    /// Clear package data
    fn clear_package_data(&mut self, package_name: &str) -> AdbResult<String> {
        let command = format!("pm clear {}", package_name);
        self.shell_command(&command)
    }

    /// Get battery information
    fn get_battery_info(&mut self) -> AdbResult<String> {
        self.shell_command("dumpsys battery")
    }

    /// Get memory information
    fn get_memory_info(&mut self) -> AdbResult<String> {
        self.shell_command("dumpsys meminfo")
    }

    /// Get CPU information
    fn get_cpu_info(&mut self) -> AdbResult<String> {
        self.shell_command("cat /proc/cpuinfo")
    }

    /// Get device properties
    fn get_device_properties(&mut self) -> AdbResult<String> {
        self.shell_command("getprop")
    }

    /// Get system log
    fn get_system_log(&mut self, lines: usize) -> AdbResult<String> {
        let command = format!("logcat -d -t {}", lines);
        self.shell_command(&command)
    }

    /// Get network information
    fn get_network_info(&mut self) -> AdbResult<String> {
        self.shell_command("dumpsys connectivity")
    }

    /// Get WiFi status
    fn get_wifi_status(&mut self) -> AdbResult<String> {
        self.shell_command("ip addr show wlan0")
    }

    /// Take screenshot
    fn take_screenshot(&mut self) -> AdbResult<String> {
        self.shell_command("screencap -p /sdcard/screenshot.png")
    }

    /// Get screen resolution
    fn get_screen_resolution(&mut self) -> AdbResult<String> {
        let size = self.shell_command("wm size")?;
        let density = self.shell_command("wm density")?;
        Ok(format!("{}\n{}", size, density))
    }

    /// List processes
    fn list_processes(&mut self) -> AdbResult<String> {
        self.shell_command("ps")
    }

    /// Force stop application
    fn force_stop(&mut self, package_name: &str) -> AdbResult<String> {
        let command = format!("am force-stop {}", package_name);
        self.shell_command(&command)
    }

    /// Execute shell command
    fn shell_command(&mut self, command: &str) -> AdbResult<String> {
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

    /// Get ADB version
    fn get_adb_version(&mut self) -> AdbResult<String> {
        let server = self.get_server()?;
        let version = server.version()?;
        Ok(format!(
            "ADB server version: {}.{}.{}",
            version.major, version.minor, version.revision
        ))
    }
}

impl Default for AdbManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse `/proc/meminfo` output and return `(total_mib, available_mib)`.
fn parse_meminfo(s: &str) -> (u64, u64) {
    let mut total = 0u64;
    let mut avail = 0u64;
    for line in s.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("MemTotal:") => {
                total = parts
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
                    / 1024;
            }
            Some("MemAvailable:") => {
                avail = parts
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
                    / 1024;
            }
            _ => {}
        }
    }
    (total, avail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adb_manager_creation() {
        let manager = AdbManager::new();
        assert!(manager.server.is_none());
        assert!(manager.selected_device.is_none());
    }

    #[test]
    fn test_package_filter() {
        let filter = PackageFilter::User;
        assert!(matches!(filter, PackageFilter::User));
    }

    #[test]
    fn test_adb_command_creation() {
        let cmd = AdbCommand::ListDevices;
        assert!(matches!(cmd, AdbCommand::ListDevices));
    }

    #[test]
    fn test_error_display() {
        let error = AdbError::DeviceNotFound;
        assert_eq!(error.to_string(), "Device not found");
    }
}
