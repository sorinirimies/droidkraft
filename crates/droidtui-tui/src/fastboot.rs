//! Fastboot operations — re-exported from [`droidtui_core`].
//!
//! The concrete fastboot logic now lives in the shared `droidtui-core` crate.
//! This module preserves the historical `crate::fastboot::…` import paths.

pub use droidtui_core::features::fastboot::*;
