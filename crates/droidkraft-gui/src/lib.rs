//! DroidKraft GUI — a Zed GPUI desktop frontend for monitoring and controlling
//! Android devices, built on top of [`droidkraft_core`].

pub mod app;
pub mod cli;
pub mod commands;
pub mod fmt;
pub mod layout;
pub mod mirror_view;
pub mod screen;
pub mod state;
pub mod theme;
pub mod ui;
pub mod worker;

pub use app::DroidGui;
pub use state::{Route, Tab};
