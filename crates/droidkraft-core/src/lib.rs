//! # DroidKraft Core
//!
//! Shared, framework-free logic for talking to Android devices over the
//! embedded Android Debug Bridge (ADB) and fastboot.  Reused by both the
//! Ratatui TUI and the GPUI GUI frontends — this crate has **no** GUI or TUI
//! dependencies.
//!
//! ## Overview
//!
//! | Module | What lives here |
//! |--------|-----------------|
//! | [`client`] | [`AdbManager`] — connection lifecycle + shell execution |
//! | [`features::device`] | Device enumeration and live status |
//! | [`features::packages`] | Package listing / install / uninstall |
//! | [`features::system`] | Battery, memory, CPU, properties, network |
//! | [`features::screen`] | Screenshots, resolution, frame capture |
//! | [`features::logcat`] | Log parsing, filtering, stats, and streaming |
//! | [`features::fastboot`] | Bootloader / fastboot operations |
//! | [`features::flash`] | Reboot targets and root detection toolkit |
//! | [`features::shell`] | The typed [`AdbCommand`] enum |
//! | [`color`] | Framework-neutral [`Rgb`] + tag/level colour helpers |
//! | [`utils`] | Framework-free helpers (text wrap, shell quoting, clipboard) |
//!
//! ## Example
//!
//! ```no_run
//! use droidkraft_core::AdbManager;
//!
//! let mut adb = AdbManager::new();
//! let status = adb.fetch_device_status();
//! if status.is_connected() {
//!     println!("Model: {}  Android {}", status.model, status.android_version);
//! }
//! ```

#[macro_use]
pub mod macros;

pub mod client;
pub mod color;
pub mod error;
pub mod features;
pub mod utils;

// ── Convenience re-exports ─────────────────────────────────────────────────
pub use client::{AdbManager, ADB_PORT};
pub use color::{hash_tag, level_text_color, tag_color, Rgb, TAG_PALETTE};
pub use error::{AdbError, AdbResult};

pub use features::device::{AdbDeviceEntry, DeviceInfo, DeviceStatus};
pub use features::fastboot::{FastbootCommand, FastbootError, FastbootManager, FastbootResult};
pub use features::flash::{RebootTarget, RootMethod, RootStatus};
pub use features::logcat::{
    ChannelWriter, DrainStatus, FilterField, LogEntry, LogLevel, LogStats, LogcatFilter,
    LogcatStream, SaveFormat,
};
pub use features::packages::PackageFilter;
pub use features::screen::ScreenResolution;
pub use features::shell::AdbCommand;

/// Compatibility façade mirroring the pre-workspace `droidkraft_tui::adb` module.
///
/// Lets the TUI keep `use crate::adb::{…}` paths working while the concrete
/// logic lives here in the core crate.
pub mod adb {
    pub use crate::client::AdbManager;
    pub use crate::error::{AdbError, AdbResult};
    pub use crate::features::device::{AdbDeviceEntry, DeviceInfo, DeviceStatus};
    pub use crate::features::packages::PackageFilter;
    pub use crate::features::shell::AdbCommand;
}
