//! Flash & root toolkit — reboot targets, root detection, remount, and
//! bootloader flashing.  Combines ADB (reboots, root state) with fastboot
//! (partition flashing) behind a single feature surface used by both the TUI
//! and GUI toolkits.

pub mod ops;
pub mod types;

pub use types::{RebootTarget, RootMethod, RootStatus};
