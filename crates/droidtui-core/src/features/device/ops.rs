//! Device operations on [`AdbManager`].

use crate::client::AdbManager;
use crate::error::{AdbError, AdbResult};

impl AdbManager {
    /// List all connected devices as a human-readable string.
    pub fn list_devices(&mut self) -> AdbResult<String> {
        let server = self.get_server()?;
        let devices = server.devices()?;

        if devices.is_empty() {
            return Ok("No devices found.\n\nMake sure:\n- Device is connected via USB\n- USB debugging is enabled\n- Device is authorized".to_string());
        }

        let mut selected_to_set = None;
        let mut output = String::from("List of devices attached:\n\n");
        for device in devices {
            output.push_str(&format!("  {:<24} {:?}\n", device.identifier, device.state));
            if self.selected_device().is_none() && selected_to_set.is_none() {
                selected_to_set = Some(device.identifier.clone());
            }
        }
        if let Some(serial) = selected_to_set {
            self.select_device(serial);
        }
        Ok(output)
    }

    /// Get the state of the currently selected device.
    pub fn get_device_state(&mut self) -> AdbResult<String> {
        let serial = self.require_selected_device()?.to_string();
        let server = self.get_server()?;
        let devices = server.devices()?;
        for device in devices {
            if device.identifier == serial {
                return Ok(format!("Device state: {:?}", device.state));
            }
        }
        Err(AdbError::DeviceNotFound)
    }

    /// Get the serial number of the currently selected device.
    pub fn get_serial_number(&mut self) -> AdbResult<String> {
        let serial = self.require_selected_device()?;
        Ok(format!("Serial number: {}", serial))
    }
}
