//! Device feature — types and operations for enumerating and inspecting
//! connected Android devices.

pub mod ops;
pub mod types;

pub use types::{AdbDeviceEntry, DeviceInfo, DeviceStatus};
