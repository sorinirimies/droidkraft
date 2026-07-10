//! Fastboot operations — re-exported from [`droidkraft_core`].
//!
//! The concrete fastboot logic now lives in the shared `droidkraft-core` crate.
//! This module preserves the historical `crate::fastboot::…` import paths.

pub use droidkraft_core::features::fastboot::*;
