//! ADB feature modules.
//!
//! Each sub-module owns its domain types (`types.rs`) and, where applicable,
//! its `AdbManager` operation methods (`ops.rs`), re-exporting both at the
//! module level for ergonomic access.  This mirrors the layout of
//! `gitkraft-core`.

pub mod device;
pub mod fastboot;
pub mod flash;
pub mod logcat;
pub mod packages;
pub mod screen;
pub mod shell;
pub mod system;
