//! Colour palette for the GUI, expressed as gpui `Rgba` values.

use droidkraft_core::LogLevel;
use gpui::{rgb, Rgba};

pub const BG: u32 = 0x0f1117;
pub const BG_PANEL: u32 = 0x161923;
pub const BG_ELEV: u32 = 0x1e222e;
pub const BG_HOVER: u32 = 0x272c3a;
pub const BORDER: u32 = 0x2a2f3d;
pub const ACCENT: u32 = 0x4f9cf5;
pub const ACCENT_DIM: u32 = 0x2d5b91;
pub const TEXT: u32 = 0xd7dae0;
pub const TEXT_DIM: u32 = 0x8b90a0;
pub const TEXT_FAINT: u32 = 0x5b6070;
pub const OK: u32 = 0x6ec06e;
pub const WARN: u32 = 0xe5c07b;
pub const ERR: u32 = 0xe06c75;
pub const DANGER: u32 = 0xc0392b;

/// Message-text colour for a log level (shared with the TUI via core).
pub fn level_color(level: LogLevel) -> Rgba {
    rgb(droidkraft_core::level_text_color(level).hex())
}

/// Stable, distinct colour for a tag string (shared palette via core).
pub fn tag_color(tag: &str) -> Rgba {
    rgb(droidkraft_core::tag_color(tag).hex())
}
