//! DroidTUI GUI — a Zed GPUI desktop frontend for monitoring and controlling
//! Android devices, built on top of [`droidtui_core`].

pub mod app;
pub mod commands;
pub mod screen;
pub mod theme;
pub mod worker;

pub use app::{DroidGui, Panel};
