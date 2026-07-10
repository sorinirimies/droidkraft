//! Application-wide colour theme system.
//!
//! Every colour used in the UI is sourced from the active [`Theme`].
//! A theme selector overlay (Shift+T) lets the user cycle through
//! named presets at runtime.

use ratatui::style::Color;

// ── Theme struct ──────────────────────────────────────────────────────────────

/// A complete colour theme for DroidKraft.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Brand colour — titles, app name.
    pub brand: Color,
    /// Accent colour — borders, active elements.
    pub accent: Color,
    /// Success colour — positive results, connected status.
    pub success: Color,
    /// Dimmed text — footer hints, inactive labels.
    pub dim: Color,
    /// Default foreground.
    pub fg: Color,
    /// Background of selected / highlighted rows.
    pub sel_bg: Color,
    /// Warning colour — warn-level logs, caution.
    pub warn: Color,
    /// Error colour — error-level logs, failures.
    pub error: Color,
    /// Surface colour — panel/card backgrounds.
    pub surface: Color,
    /// Border colour — default borders.
    pub border: Color,
    /// Key hint highlight — the letter portion of shortcuts.
    pub key_hint: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            brand: Color::Rgb(80, 200, 120),     // Green
            accent: Color::Rgb(80, 200, 255),    // Cyan
            success: Color::Rgb(80, 220, 120),   // Green
            dim: Color::Rgb(100, 100, 120),      // Muted grey
            fg: Color::Rgb(220, 220, 220),       // Light grey
            sel_bg: Color::Rgb(30, 50, 70),      // Dark blue-grey
            warn: Color::Rgb(220, 180, 50),      // Yellow
            error: Color::Rgb(220, 60, 60),      // Red
            surface: Color::Rgb(25, 25, 35),     // Near-black
            border: Color::Rgb(50, 50, 70),      // Dark grey
            key_hint: Color::Rgb(130, 180, 255), // Light blue (for key hints)
        }
    }
}

// ── Builder-style setters ─────────────────────────────────────────────────────

impl Theme {
    pub fn brand(mut self, c: Color) -> Self {
        self.brand = c;
        self
    }
    pub fn accent(mut self, c: Color) -> Self {
        self.accent = c;
        self
    }
    pub fn success(mut self, c: Color) -> Self {
        self.success = c;
        self
    }
    pub fn dim(mut self, c: Color) -> Self {
        self.dim = c;
        self
    }
    pub fn fg(mut self, c: Color) -> Self {
        self.fg = c;
        self
    }
    pub fn sel_bg(mut self, c: Color) -> Self {
        self.sel_bg = c;
        self
    }
    pub fn warn(mut self, c: Color) -> Self {
        self.warn = c;
        self
    }
    pub fn error(mut self, c: Color) -> Self {
        self.error = c;
        self
    }
    pub fn surface(mut self, c: Color) -> Self {
        self.surface = c;
        self
    }
    pub fn border(mut self, c: Color) -> Self {
        self.border = c;
        self
    }
    pub fn key_hint(mut self, c: Color) -> Self {
        self.key_hint = c;
        self
    }
}

// ── Named presets ─────────────────────────────────────────────────────────────

impl Theme {
    pub fn dracula() -> Self {
        Self {
            brand: Color::Rgb(255, 121, 198),
            accent: Color::Rgb(139, 233, 253),
            success: Color::Rgb(80, 250, 123),
            dim: Color::Rgb(98, 114, 164),
            fg: Color::Rgb(248, 248, 242),
            sel_bg: Color::Rgb(68, 71, 90),
            warn: Color::Rgb(241, 250, 140),
            error: Color::Rgb(255, 85, 85),
            surface: Color::Rgb(40, 42, 54),
            border: Color::Rgb(68, 71, 90),
            key_hint: Color::Rgb(139, 233, 253),
        }
    }

    pub fn nord() -> Self {
        Self {
            brand: Color::Rgb(136, 192, 208),
            accent: Color::Rgb(129, 161, 193),
            success: Color::Rgb(163, 190, 140),
            dim: Color::Rgb(76, 86, 106),
            fg: Color::Rgb(216, 222, 233),
            sel_bg: Color::Rgb(59, 66, 82),
            warn: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            surface: Color::Rgb(46, 52, 64),
            border: Color::Rgb(59, 66, 82),
            key_hint: Color::Rgb(136, 192, 208),
        }
    }

    pub fn gruvbox_dark() -> Self {
        Self {
            brand: Color::Rgb(254, 128, 25),
            accent: Color::Rgb(250, 189, 47),
            success: Color::Rgb(184, 187, 38),
            dim: Color::Rgb(146, 131, 116),
            fg: Color::Rgb(235, 219, 178),
            sel_bg: Color::Rgb(60, 56, 54),
            warn: Color::Rgb(250, 189, 47),
            error: Color::Rgb(251, 73, 52),
            surface: Color::Rgb(40, 40, 40),
            border: Color::Rgb(60, 56, 54),
            key_hint: Color::Rgb(142, 192, 124),
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            brand: Color::Rgb(203, 166, 247),
            accent: Color::Rgb(137, 180, 250),
            success: Color::Rgb(166, 227, 161),
            dim: Color::Rgb(108, 112, 134),
            fg: Color::Rgb(205, 214, 244),
            sel_bg: Color::Rgb(49, 50, 68),
            warn: Color::Rgb(249, 226, 175),
            error: Color::Rgb(243, 139, 168),
            surface: Color::Rgb(30, 30, 46),
            border: Color::Rgb(49, 50, 68),
            key_hint: Color::Rgb(148, 226, 213),
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            brand: Color::Rgb(187, 154, 247),
            accent: Color::Rgb(122, 162, 247),
            success: Color::Rgb(158, 206, 106),
            dim: Color::Rgb(86, 95, 137),
            fg: Color::Rgb(192, 202, 245),
            sel_bg: Color::Rgb(41, 46, 66),
            warn: Color::Rgb(224, 175, 104),
            error: Color::Rgb(247, 118, 142),
            surface: Color::Rgb(26, 27, 38),
            border: Color::Rgb(41, 46, 66),
            key_hint: Color::Rgb(115, 218, 202),
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            brand: Color::Rgb(38, 139, 210),
            accent: Color::Rgb(42, 161, 152),
            success: Color::Rgb(133, 153, 0),
            dim: Color::Rgb(88, 110, 117),
            fg: Color::Rgb(131, 148, 150),
            sel_bg: Color::Rgb(7, 54, 66),
            warn: Color::Rgb(181, 137, 0),
            error: Color::Rgb(220, 50, 47),
            surface: Color::Rgb(0, 43, 54),
            border: Color::Rgb(7, 54, 66),
            key_hint: Color::Rgb(42, 161, 152),
        }
    }

    pub fn moonfly() -> Self {
        Self {
            brand: Color::Rgb(174, 129, 255),
            accent: Color::Rgb(128, 160, 255),
            success: Color::Rgb(140, 200, 95),
            dim: Color::Rgb(78, 78, 78),
            fg: Color::Rgb(178, 178, 178),
            sel_bg: Color::Rgb(28, 28, 28),
            warn: Color::Rgb(227, 199, 138),
            error: Color::Rgb(255, 92, 92),
            surface: Color::Rgb(8, 8, 8),
            border: Color::Rgb(38, 38, 38),
            key_hint: Color::Rgb(121, 219, 195),
        }
    }

    pub fn oxocarbon() -> Self {
        Self {
            brand: Color::Rgb(255, 126, 182),
            accent: Color::Rgb(120, 169, 255),
            success: Color::Rgb(66, 190, 101),
            dim: Color::Rgb(82, 82, 82),
            fg: Color::Rgb(242, 244, 248),
            sel_bg: Color::Rgb(38, 38, 38),
            warn: Color::Rgb(255, 213, 0),
            error: Color::Rgb(238, 83, 80),
            surface: Color::Rgb(22, 22, 22),
            border: Color::Rgb(50, 50, 50),
            key_hint: Color::Rgb(51, 177, 255),
        }
    }

    pub fn grape() -> Self {
        Self::default()
            .brand(Color::Rgb(200, 120, 255))
            .accent(Color::Rgb(130, 180, 255))
            .success(Color::Rgb(160, 110, 255))
            .dim(Color::Rgb(110, 100, 130))
            .fg(Color::Rgb(220, 210, 240))
            .sel_bg(Color::Rgb(50, 35, 80))
            .warn(Color::Rgb(200, 180, 120))
            .error(Color::Rgb(220, 80, 80))
            .surface(Color::Rgb(25, 18, 40))
            .border(Color::Rgb(60, 45, 90))
            .key_hint(Color::Rgb(180, 130, 255))
    }

    pub fn ocean() -> Self {
        Self::default()
            .brand(Color::Rgb(0, 200, 180))
            .accent(Color::Rgb(0, 175, 210))
            .success(Color::Rgb(80, 230, 200))
            .dim(Color::Rgb(80, 120, 130))
            .fg(Color::Rgb(200, 240, 245))
            .sel_bg(Color::Rgb(0, 50, 70))
            .warn(Color::Rgb(200, 200, 100))
            .error(Color::Rgb(220, 80, 80))
            .surface(Color::Rgb(5, 25, 35))
            .border(Color::Rgb(20, 60, 80))
            .key_hint(Color::Rgb(80, 220, 210))
    }

    pub fn sunset() -> Self {
        Self::default()
            .brand(Color::Rgb(255, 80, 80))
            .accent(Color::Rgb(255, 150, 50))
            .success(Color::Rgb(255, 180, 80))
            .dim(Color::Rgb(140, 100, 80))
            .fg(Color::Rgb(255, 235, 210))
            .sel_bg(Color::Rgb(80, 30, 20))
            .warn(Color::Rgb(255, 200, 60))
            .error(Color::Rgb(220, 50, 50))
            .surface(Color::Rgb(35, 15, 10))
            .border(Color::Rgb(80, 35, 25))
            .key_hint(Color::Rgb(255, 160, 80))
    }

    pub fn rose() -> Self {
        Self::default()
            .brand(Color::Rgb(255, 100, 150))
            .accent(Color::Rgb(255, 140, 180))
            .success(Color::Rgb(255, 160, 190))
            .dim(Color::Rgb(140, 90, 110))
            .fg(Color::Rgb(255, 230, 235))
            .sel_bg(Color::Rgb(80, 20, 40))
            .warn(Color::Rgb(255, 200, 150))
            .error(Color::Rgb(220, 60, 60))
            .surface(Color::Rgb(35, 10, 20))
            .border(Color::Rgb(80, 30, 50))
            .key_hint(Color::Rgb(255, 150, 180))
    }

    pub fn solarized_light() -> Self {
        Self {
            brand: Color::Rgb(38, 139, 210),
            accent: Color::Rgb(42, 161, 152),
            success: Color::Rgb(133, 153, 0),
            dim: Color::Rgb(147, 161, 161),
            fg: Color::Rgb(101, 123, 131),
            sel_bg: Color::Rgb(238, 232, 213),
            warn: Color::Rgb(181, 137, 0),
            error: Color::Rgb(220, 50, 47),
            surface: Color::Rgb(253, 246, 227),
            border: Color::Rgb(238, 232, 213),
            key_hint: Color::Rgb(42, 161, 152),
        }
    }

    pub fn gruvbox_light() -> Self {
        Self {
            brand: Color::Rgb(214, 93, 14),
            accent: Color::Rgb(215, 153, 33),
            success: Color::Rgb(121, 116, 14),
            dim: Color::Rgb(146, 131, 116),
            fg: Color::Rgb(60, 56, 54),
            sel_bg: Color::Rgb(213, 196, 161),
            warn: Color::Rgb(181, 118, 20),
            error: Color::Rgb(204, 36, 29),
            surface: Color::Rgb(251, 241, 199),
            border: Color::Rgb(213, 196, 161),
            key_hint: Color::Rgb(104, 157, 106),
        }
    }

    pub fn catppuccin_latte() -> Self {
        Self {
            brand: Color::Rgb(136, 57, 239),
            accent: Color::Rgb(30, 102, 245),
            success: Color::Rgb(64, 160, 43),
            dim: Color::Rgb(156, 160, 176),
            fg: Color::Rgb(76, 79, 105),
            sel_bg: Color::Rgb(204, 208, 218),
            warn: Color::Rgb(223, 142, 29),
            error: Color::Rgb(210, 15, 57),
            surface: Color::Rgb(239, 241, 245),
            border: Color::Rgb(204, 208, 218),
            key_hint: Color::Rgb(23, 146, 153),
        }
    }

    pub fn catppuccin_frappe() -> Self {
        Self {
            brand: Color::Rgb(202, 158, 230),
            accent: Color::Rgb(140, 170, 238),
            success: Color::Rgb(166, 209, 137),
            dim: Color::Rgb(115, 121, 148),
            fg: Color::Rgb(198, 208, 245),
            sel_bg: Color::Rgb(65, 69, 89),
            warn: Color::Rgb(229, 200, 144),
            error: Color::Rgb(231, 130, 132),
            surface: Color::Rgb(48, 52, 70),
            border: Color::Rgb(65, 69, 89),
            key_hint: Color::Rgb(129, 200, 190),
        }
    }

    pub fn catppuccin_macchiato() -> Self {
        Self {
            brand: Color::Rgb(198, 160, 246),
            accent: Color::Rgb(138, 173, 244),
            success: Color::Rgb(166, 218, 149),
            dim: Color::Rgb(110, 115, 141),
            fg: Color::Rgb(202, 211, 245),
            sel_bg: Color::Rgb(54, 58, 79),
            warn: Color::Rgb(238, 212, 159),
            error: Color::Rgb(237, 135, 150),
            surface: Color::Rgb(36, 39, 58),
            border: Color::Rgb(54, 58, 79),
            key_hint: Color::Rgb(139, 213, 202),
        }
    }

    pub fn tokyo_night_storm() -> Self {
        Self {
            brand: Color::Rgb(187, 154, 247),
            accent: Color::Rgb(122, 162, 247),
            success: Color::Rgb(158, 206, 106),
            dim: Color::Rgb(86, 95, 137),
            fg: Color::Rgb(192, 202, 245),
            sel_bg: Color::Rgb(45, 49, 75),
            warn: Color::Rgb(224, 175, 104),
            error: Color::Rgb(247, 118, 142),
            surface: Color::Rgb(36, 40, 59),
            border: Color::Rgb(45, 49, 75),
            key_hint: Color::Rgb(115, 218, 202),
        }
    }

    pub fn tokyo_night_light() -> Self {
        Self {
            brand: Color::Rgb(90, 74, 120),
            accent: Color::Rgb(46, 126, 233),
            success: Color::Rgb(72, 94, 48),
            dim: Color::Rgb(132, 140, 176),
            fg: Color::Rgb(52, 59, 88),
            sel_bg: Color::Rgb(208, 213, 227),
            warn: Color::Rgb(140, 108, 62),
            error: Color::Rgb(200, 55, 75),
            surface: Color::Rgb(213, 214, 219),
            border: Color::Rgb(208, 213, 227),
            key_hint: Color::Rgb(15, 75, 110),
        }
    }

    pub fn kanagawa_wave() -> Self {
        Self {
            brand: Color::Rgb(210, 126, 153),
            accent: Color::Rgb(126, 156, 216),
            success: Color::Rgb(118, 148, 106),
            dim: Color::Rgb(114, 113, 105),
            fg: Color::Rgb(220, 215, 186),
            sel_bg: Color::Rgb(42, 42, 55),
            warn: Color::Rgb(220, 165, 97),
            error: Color::Rgb(195, 64, 67),
            surface: Color::Rgb(31, 31, 40),
            border: Color::Rgb(42, 42, 55),
            key_hint: Color::Rgb(106, 149, 137),
        }
    }

    pub fn kanagawa_dragon() -> Self {
        Self {
            brand: Color::Rgb(210, 126, 153),
            accent: Color::Rgb(139, 164, 176),
            success: Color::Rgb(135, 169, 135),
            dim: Color::Rgb(166, 166, 156),
            fg: Color::Rgb(197, 201, 197),
            sel_bg: Color::Rgb(40, 39, 39),
            warn: Color::Rgb(200, 170, 109),
            error: Color::Rgb(195, 64, 67),
            surface: Color::Rgb(24, 24, 24),
            border: Color::Rgb(40, 39, 39),
            key_hint: Color::Rgb(142, 164, 162),
        }
    }

    pub fn kanagawa_lotus() -> Self {
        Self {
            brand: Color::Rgb(160, 154, 190),
            accent: Color::Rgb(77, 105, 155),
            success: Color::Rgb(111, 137, 78),
            dim: Color::Rgb(196, 178, 138),
            fg: Color::Rgb(84, 84, 100),
            sel_bg: Color::Rgb(231, 219, 160),
            warn: Color::Rgb(119, 113, 63),
            error: Color::Rgb(195, 64, 67),
            surface: Color::Rgb(242, 236, 188),
            border: Color::Rgb(231, 219, 160),
            key_hint: Color::Rgb(78, 140, 162),
        }
    }

    pub fn nightfly() -> Self {
        Self {
            brand: Color::Rgb(199, 146, 234),
            accent: Color::Rgb(130, 170, 255),
            success: Color::Rgb(161, 205, 94),
            dim: Color::Rgb(75, 100, 121),
            fg: Color::Rgb(172, 187, 203),
            sel_bg: Color::Rgb(11, 41, 66),
            warn: Color::Rgb(255, 202, 40),
            error: Color::Rgb(255, 85, 85),
            surface: Color::Rgb(1, 22, 39),
            border: Color::Rgb(11, 41, 66),
            key_hint: Color::Rgb(33, 199, 168),
        }
    }

    pub fn forest() -> Self {
        Self::default()
            .brand(Color::Rgb(100, 200, 80))
            .accent(Color::Rgb(80, 160, 80))
            .success(Color::Rgb(120, 210, 90))
            .dim(Color::Rgb(90, 120, 80))
            .fg(Color::Rgb(210, 235, 200))
            .sel_bg(Color::Rgb(20, 50, 20))
            .warn(Color::Rgb(200, 180, 80))
            .error(Color::Rgb(200, 80, 60))
            .surface(Color::Rgb(15, 30, 15))
            .border(Color::Rgb(30, 60, 30))
            .key_hint(Color::Rgb(160, 230, 130))
    }

    pub fn neon() -> Self {
        Self::default()
            .brand(Color::Rgb(255, 0, 200))
            .accent(Color::Rgb(0, 255, 200))
            .success(Color::Rgb(0, 255, 130))
            .dim(Color::Rgb(100, 80, 120))
            .fg(Color::Rgb(230, 230, 255))
            .sel_bg(Color::Rgb(30, 0, 50))
            .warn(Color::Rgb(255, 220, 0))
            .error(Color::Rgb(255, 50, 50))
            .surface(Color::Rgb(10, 0, 20))
            .border(Color::Rgb(40, 0, 60))
            .key_hint(Color::Rgb(0, 255, 200))
    }

    pub fn mono() -> Self {
        Self::default()
            .brand(Color::Rgb(220, 220, 220))
            .accent(Color::Rgb(180, 180, 180))
            .success(Color::Rgb(200, 200, 200))
            .dim(Color::Rgb(110, 110, 115))
            .fg(Color::Rgb(210, 210, 210))
            .sel_bg(Color::Rgb(50, 50, 55))
            .warn(Color::Rgb(180, 180, 150))
            .error(Color::Rgb(180, 130, 130))
            .surface(Color::Rgb(20, 20, 20))
            .border(Color::Rgb(60, 60, 60))
            .key_hint(Color::Rgb(220, 220, 220))
    }

    /// Return every named preset as `(name, description, theme)`.
    pub fn all_presets() -> Vec<(&'static str, &'static str, Theme)> {
        vec![
            (
                "Default",
                "Green & cyan — the built-in DroidKraft palette",
                Theme::default(),
            ),
            (
                "Grape",
                "Deep violet & soft blue — easy on the eyes",
                Theme::grape(),
            ),
            (
                "Ocean",
                "Teal & aquamarine — calm nautical feel",
                Theme::ocean(),
            ),
            (
                "Sunset",
                "Warm amber & rose — vibrant high-energy",
                Theme::sunset(),
            ),
            (
                "Forest",
                "Earthy greens & bark browns — natural",
                Theme::forest(),
            ),
            (
                "Rose",
                "Pinks & corals — playful pastel-inspired",
                Theme::rose(),
            ),
            ("Mono", "Greyscale only — distraction-free", Theme::mono()),
            ("Neon", "Electric brights — synthwave retro", Theme::neon()),
            (
                "Dracula",
                "Pink, cyan & purple on dark grey",
                Theme::dracula(),
            ),
            ("Nord", "Arctic bluish tones", Theme::nord()),
            (
                "Solarized Dark",
                "Precision colours — dark",
                Theme::solarized_dark(),
            ),
            (
                "Solarized Light",
                "Precision colours — light",
                Theme::solarized_light(),
            ),
            (
                "Gruvbox Dark",
                "Retro groove — warm dark background",
                Theme::gruvbox_dark(),
            ),
            (
                "Gruvbox Light",
                "Retro groove — warm light background",
                Theme::gruvbox_light(),
            ),
            (
                "Catppuccin Latte",
                "Soothing pastel — light",
                Theme::catppuccin_latte(),
            ),
            (
                "Catppuccin Frappé",
                "Soothing pastel — medium-dark",
                Theme::catppuccin_frappe(),
            ),
            (
                "Catppuccin Macchiato",
                "Soothing pastel — dark",
                Theme::catppuccin_macchiato(),
            ),
            (
                "Catppuccin Mocha",
                "Soothing pastel — darkest",
                Theme::catppuccin_mocha(),
            ),
            (
                "Tokyo Night",
                "Clean dark blue / purple night",
                Theme::tokyo_night(),
            ),
            (
                "Tokyo Night Storm",
                "Slightly lighter dark variant",
                Theme::tokyo_night_storm(),
            ),
            (
                "Tokyo Night Light",
                "Light variant",
                Theme::tokyo_night_light(),
            ),
            (
                "Kanagawa Wave",
                "Deep blue ink on parchment",
                Theme::kanagawa_wave(),
            ),
            (
                "Kanagawa Dragon",
                "Darker earth tones — charcoal & moss",
                Theme::kanagawa_dragon(),
            ),
            (
                "Kanagawa Lotus",
                "Light parchment variant",
                Theme::kanagawa_lotus(),
            ),
            (
                "Moonfly",
                "Deep dark with vibrant accents",
                Theme::moonfly(),
            ),
            ("Nightfly", "Deep ocean blues", Theme::nightfly()),
            (
                "Oxocarbon",
                "IBM Carbon Design System inspired",
                Theme::oxocarbon(),
            ),
        ]
    }
}

// ── Theme selector state ──────────────────────────────────────────────────────

/// State for the theme selector overlay panel.
#[derive(Debug, Clone, Default)]
pub struct ThemeSelector {
    /// Whether the selector panel is visible.
    pub open: bool,
    /// Index of the currently highlighted preset.
    pub cursor: usize,
    /// Index of the currently active (applied) theme.
    pub active: usize,
}

impl ThemeSelector {
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn next(&mut self) {
        let total = Theme::all_presets().len();
        self.cursor = (self.cursor + 1) % total;
    }

    pub fn prev(&mut self) {
        let total = Theme::all_presets().len();
        self.cursor = if self.cursor == 0 {
            total - 1
        } else {
            self.cursor - 1
        };
    }

    /// Apply the theme at the current cursor position. Returns the new Theme.
    pub fn apply(&mut self) -> Theme {
        self.active = self.cursor;
        let presets = Theme::all_presets();
        presets[self.active].2.clone()
    }

    /// Get the currently active theme.
    pub fn current_theme(&self) -> Theme {
        let presets = Theme::all_presets();
        presets
            .get(self.active)
            .map(|(_, _, t)| t.clone())
            .unwrap_or_default()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_has_expected_fields() {
        let t = Theme::default();
        // Just check it doesn't panic and has reasonable values
        assert_ne!(t.brand, Color::Reset);
        assert_ne!(t.accent, Color::Reset);
    }

    #[test]
    fn all_presets_non_empty() {
        assert!(Theme::all_presets().len() >= 27);
    }

    #[test]
    fn all_preset_names_unique() {
        let presets = Theme::all_presets();
        let mut names: Vec<&str> = presets.iter().map(|(n, _, _)| *n).collect();
        let len_before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate preset names found");
    }

    #[test]
    fn first_preset_is_default() {
        let presets = Theme::all_presets();
        assert_eq!(presets[0].0, "Default");
        assert_eq!(presets[0].2, Theme::default());
    }

    #[test]
    fn selector_toggle() {
        let mut sel = ThemeSelector::default();
        assert!(!sel.open);
        sel.toggle();
        assert!(sel.open);
        sel.toggle();
        assert!(!sel.open);
    }

    #[test]
    fn selector_next_prev_wrap() {
        let mut sel = ThemeSelector::default();
        let total = Theme::all_presets().len();
        sel.cursor = total - 1;
        sel.next();
        assert_eq!(sel.cursor, 0); // wraps
        sel.prev();
        assert_eq!(sel.cursor, total - 1); // wraps back
    }

    #[test]
    fn selector_apply_changes_active() {
        let mut sel = ThemeSelector::default();
        sel.cursor = 1;
        let theme = sel.apply();
        assert_eq!(sel.active, 1);
        assert_eq!(theme, Theme::all_presets()[1].2);
    }

    #[test]
    fn builder_chain_works() {
        let t = Theme::default().brand(Color::Red).accent(Color::Blue);
        assert_eq!(t.brand, Color::Red);
        assert_eq!(t.accent, Color::Blue);
    }

    #[test]
    fn clone_equals_original() {
        let t = Theme::dracula();
        assert_eq!(t.clone(), t);
    }
}
