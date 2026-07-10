//! Colour palette for the GUI, expressed as gpui `Rgba` values.

use droidtui_core::LogLevel;
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

/// Message-text colour for a log level.
pub fn level_color(level: LogLevel) -> Rgba {
    match level {
        LogLevel::Verbose => rgb(0x6b7080),
        LogLevel::Debug => rgb(0x56b6c2),
        LogLevel::Info => rgb(0x98c379),
        LogLevel::Warn => rgb(0xe5c07b),
        LogLevel::Error => rgb(0xe06c75),
        LogLevel::Fatal => rgb(0xff5f87),
        LogLevel::Unknown => rgb(0x8b90a0),
    }
}

/// Stable, distinct colour for a tag string.
pub fn tag_color(tag: &str) -> Rgba {
    let mut hash: u32 = 5381;
    for b in tag.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    const PALETTE: &[u32] = &[
        0x569cd6, 0x4ec9b0, 0xdcdcaa, 0xce9178, 0xb5cea8, 0xc882c8, 0x64c8dc, 0xdcb464,
        0x82b4dc, 0xb48cb4, 0x96dc96, 0xdc9696,
    ];
    rgb(PALETTE[(hash as usize) % PALETTE.len()])
}
