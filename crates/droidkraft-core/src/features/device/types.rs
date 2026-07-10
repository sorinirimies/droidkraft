//! Device domain types.

use serde::{Deserialize, Serialize};

/// Detailed information about a single device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub serial: String,
    pub state: String,
    pub model: Option<String>,
    pub device: Option<String>,
}

/// A single device entry returned by `adb devices`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdbDeviceEntry {
    pub serial: String,
    /// ADB state string: "device", "offline", "unauthorized", etc.
    pub state: String,
}

impl AdbDeviceEntry {
    /// Whether this device is in the fully-authorized `device` state.
    pub fn is_online(&self) -> bool {
        self.state == "device"
    }
}

/// Comprehensive device status shown in a dashboard / header.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceStatus {
    /// All devices currently visible to ADB (empty → nothing connected).
    pub devices: Vec<AdbDeviceEntry>,
    /// Which entry in `devices` is active (index).
    pub selected_idx: usize,
    /// Product model (`ro.product.model`).
    pub model: String,
    /// Android version (`ro.build.version.release`).
    pub android_version: String,
    /// Battery percentage 0–100.
    pub battery_pct: u8,
    /// Total RAM in MiB.
    pub ram_total_mib: u64,
    /// Available RAM in MiB.
    pub ram_avail_mib: u64,
    /// 1-minute load average from `/proc/loadavg`.
    pub cpu_load_1min: f32,
}

impl DeviceStatus {
    /// Whether any device is connected.
    pub fn is_connected(&self) -> bool {
        !self.devices.is_empty()
    }

    /// The currently active device entry, if any.
    pub fn active(&self) -> Option<&AdbDeviceEntry> {
        self.devices.get(self.selected_idx)
    }

    /// Advance selection to the next device, wrapping around.
    pub fn cycle_next(&mut self) {
        if !self.devices.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.devices.len();
        }
    }

    /// RAM currently in use, in MiB.
    pub fn ram_used_mib(&self) -> u64 {
        self.ram_total_mib.saturating_sub(self.ram_avail_mib)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(serial: &str, state: &str) -> AdbDeviceEntry {
        AdbDeviceEntry {
            serial: serial.into(),
            state: state.into(),
        }
    }

    #[test]
    fn empty_status_not_connected() {
        assert!(!DeviceStatus::default().is_connected());
    }

    #[test]
    fn cycle_next_wraps() {
        let mut s = DeviceStatus {
            devices: vec![entry("a", "device"), entry("b", "device")],
            ..Default::default()
        };
        s.cycle_next();
        assert_eq!(s.selected_idx, 1);
        s.cycle_next();
        assert_eq!(s.selected_idx, 0);
    }

    #[test]
    fn is_online_only_for_device_state() {
        assert!(entry("a", "device").is_online());
        assert!(!entry("a", "unauthorized").is_online());
    }

    #[test]
    fn ram_used_is_total_minus_avail() {
        let s = DeviceStatus {
            ram_total_mib: 8000,
            ram_avail_mib: 3000,
            ..Default::default()
        };
        assert_eq!(s.ram_used_mib(), 5000);
    }
}
