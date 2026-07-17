//! Device-profile detection used to filter compatible ROMs and gate flashing.

use crate::client::AdbManager;
use crate::error::AdbResult;
use crate::features::rom::types::DeviceProfile;

impl AdbManager {
    /// Detect the connected device's profile (codename, model, bootloader
    /// state) used to select compatible custom ROMs.
    pub fn detect_device_profile(&mut self) -> AdbResult<DeviceProfile> {
        let serial = self.selected_device().unwrap_or_default().to_string();

        let get = |mgr: &mut AdbManager, prop: &str| -> String {
            mgr.shell_command(&format!("getprop {prop}"))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && !s.contains("no output"))
                .unwrap_or_default()
        };

        let codename = {
            let c = get(self, "ro.product.device");
            if c.is_empty() {
                get(self, "ro.build.product")
            } else {
                c
            }
        };
        let manufacturer = get(self, "ro.product.manufacturer");
        let model = get(self, "ro.product.model");
        let android_version = get(self, "ro.build.version.release");

        // ro.boot.flash.locked: "0" = unlocked, "1" = locked.
        let bootloader_unlocked = match get(self, "ro.boot.flash.locked").as_str() {
            "0" => Some(true),
            "1" => Some(false),
            _ => None,
        };

        Ok(DeviceProfile {
            serial,
            codename,
            manufacturer,
            model,
            android_version,
            bootloader_unlocked,
        })
    }
}
