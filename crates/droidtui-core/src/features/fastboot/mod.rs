//! Fastboot feature — types and operations for the `fastboot` protocol.
//!
//! The fastboot protocol is separate from ADB and requires the `fastboot`
//! binary (part of Android platform-tools). There is no pure-Rust fastboot
//! library, so this module shells out to the `fastboot` binary.

pub mod ops;
pub mod types;

pub use ops::FastbootManager;
pub use types::{FastbootCommand, FastbootError, FastbootResult};
