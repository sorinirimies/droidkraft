//! ADB abstraction — re-exported from [`droidtui_core`].
//!
//! The concrete ADB logic now lives in the shared `droidtui-core` crate so it
//! can be reused by the GUI frontend.  This module preserves the historical
//! `crate::adb::…` import paths used throughout the TUI.

pub use droidtui_core::adb::*;
