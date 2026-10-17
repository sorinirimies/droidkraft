//! Device feature — types and operations for enumerating and inspecting
//! connected Android devices.

pub mod details;
pub mod ops;
pub mod types;

pub use details::{
    collect_device_details, collect_device_details_with, list_all_details, BatteryInfo,
    BatteryStatus, CpuInfo, CpuSample, DetailsCollector, DeviceDetails, DeviceHistory, DisplayInfo,
    MemoryInfo, NetworkInfo, PlugType, RefreshRate, Ring, StorageInfo, SystemInfo, ThermalInfo,
};
pub use types::{AdbDeviceEntry, DeviceInfo, DeviceStatus};
