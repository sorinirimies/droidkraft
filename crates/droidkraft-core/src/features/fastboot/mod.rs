//! Fastboot feature — re-exported from the `droidkraft-fastboot` crate.
//!
//! The default backend talks to the device over USB in pure Rust and falls
//! back to the `fastboot` binary for operations it doesn't cover yet.

pub use droidkraft_fastboot::{
    Backend, FastbootCommand, FastbootError, FastbootManager, FastbootResult,
};
