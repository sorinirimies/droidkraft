//! Framework-neutral colour helpers shared by the TUI and GUI frontends.
//!
//! The frontends convert [`Rgb`] into their own colour types (Ratatui `Color`,
//! gpui `Rgba`).  This keeps the tag-hashing algorithm and the palette in one
//! place instead of being copy-pasted per frontend.

use crate::features::logcat::LogLevel;

/// A simple 8-bit-per-channel RGB colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Build from a `0xRRGGBB` hex value.
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xff) as u8,
            g: ((hex >> 8) & 0xff) as u8,
            b: (hex & 0xff) as u8,
        }
    }

    /// The `(r, g, b)` tuple.
    pub const fn tuple(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// The packed `0xRRGGBB` value.
    pub const fn hex(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// Stable palette of visually distinct colours used to colourise log tags.
pub const TAG_PALETTE: &[Rgb] = &[
    Rgb::new(86, 156, 214),  // blue
    Rgb::new(78, 201, 176),  // teal
    Rgb::new(220, 220, 170), // light yellow
    Rgb::new(206, 145, 120), // salmon
    Rgb::new(181, 206, 168), // light green
    Rgb::new(200, 130, 200), // purple
    Rgb::new(100, 200, 220), // cyan
    Rgb::new(220, 180, 100), // gold
    Rgb::new(130, 180, 220), // periwinkle
    Rgb::new(180, 140, 180), // mauve
    Rgb::new(150, 220, 150), // mint
    Rgb::new(220, 150, 150), // rose
    Rgb::new(170, 200, 130), // olive
    Rgb::new(140, 180, 200), // steel blue
    Rgb::new(200, 170, 140), // tan
    Rgb::new(160, 200, 200), // powder blue
];

/// Hash a tag string into a stable index seed (djb2).
pub fn hash_tag(tag: &str) -> u32 {
    let mut hash: u32 = 5381;
    for b in tag.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    hash
}

/// Map a tag string to a stable, visually distinct colour from [`TAG_PALETTE`].
pub fn tag_color(tag: &str) -> Rgb {
    TAG_PALETTE[(hash_tag(tag) as usize) % TAG_PALETTE.len()]
}

/// Framework-neutral message-text colour for a log level.
///
/// The TUI intentionally uses named ANSI colours (so they follow the terminal
/// theme); the GUI uses these concrete RGB values.
pub fn level_text_color(level: LogLevel) -> Rgb {
    match level {
        LogLevel::Verbose => Rgb::from_hex(0x6b7080),
        LogLevel::Debug => Rgb::from_hex(0x56b6c2),
        LogLevel::Info => Rgb::from_hex(0x98c379),
        LogLevel::Warn => Rgb::from_hex(0xe5c07b),
        LogLevel::Error => Rgb::from_hex(0xe06c75),
        LogLevel::Fatal => Rgb::from_hex(0xff5f87),
        LogLevel::Unknown => Rgb::from_hex(0x8b90a0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_hex_roundtrip() {
        let c = Rgb::from_hex(0x56b6c2);
        assert_eq!(c.tuple(), (0x56, 0xb6, 0xc2));
        assert_eq!(c.hex(), 0x56b6c2);
    }

    #[test]
    fn hash_tag_is_stable() {
        assert_eq!(hash_tag("ActivityManager"), hash_tag("ActivityManager"));
    }

    #[test]
    fn hash_tag_differs_by_input() {
        assert_ne!(hash_tag("Foo"), hash_tag("Bar"));
    }

    #[test]
    fn tag_color_is_stable_and_in_palette() {
        let c = tag_color("OkHttp");
        assert_eq!(c, tag_color("OkHttp"));
        assert!(TAG_PALETTE.contains(&c));
    }

    #[test]
    fn level_colors_distinct_by_severity() {
        assert_ne!(
            level_text_color(LogLevel::Error),
            level_text_color(LogLevel::Info)
        );
    }
}
