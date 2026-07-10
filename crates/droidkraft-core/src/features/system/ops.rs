//! System information operations on [`AdbManager`].
//!
//! Most of these are fixed shell commands, generated via the
//! [`shell_getters!`](crate::shell_getters) macro.

use crate::client::AdbManager;
use crate::error::AdbResult;

impl AdbManager {
    crate::shell_getters! {
        /// Full battery dump (`dumpsys battery`).
        get_battery_info => "dumpsys battery";
        /// Memory info (`dumpsys meminfo`).
        get_memory_info => "dumpsys meminfo";
        /// Raw CPU info (`/proc/cpuinfo`).
        get_cpu_info => "cat /proc/cpuinfo";
        /// All device system properties (`getprop`).
        get_device_properties => "getprop";
        /// Connectivity dump (`dumpsys connectivity`).
        get_network_info => "dumpsys connectivity";
        /// Wi-Fi interface addresses (`ip addr show wlan0`).
        get_wifi_status => "ip addr show wlan0";
        /// Running processes (`ps`).
        list_processes => "ps";
    }

    /// Retrieve the last `lines` lines of the system log (`logcat -d -t N`).
    pub fn get_system_log(&mut self, lines: usize) -> AdbResult<String> {
        self.shell_command(&format!("logcat -d -t {}", lines))
    }
}
