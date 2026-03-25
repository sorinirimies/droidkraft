//! Application-wide colour theme system.
//!
//! Every colour used in the UI is sourced from the active [`Theme`].
//! A theme selector overlay (Shift+T) lets the user cycle through
//! named presets at runtime.

use ratatui::style::Color;

// ── Theme struct ──────────────────────────────────────────────────────────────

/// A complete colour theme for DroidTUI.
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
                "Green & cyan — the built-in DroidTUI palette",
                Theme::default(),
            ),
            (
                "Dracula",
                "Pink, cyan & purple on dark grey",
                Theme::dracula(),
            ),
            ("Nord", "Arctic bluish tones", Theme::nord()),
            (
                "Gruvbox Dark",
                "Retro groove — warm dark background",
                Theme::gruvbox_dark(),
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
                "Solarized Dark",
                "Precision colours — dark",
                Theme::solarized_dark(),
            ),
            (
                "Moonfly",
                "Deep dark with vibrant accents",
                Theme::moonfly(),
            ),
            (
                "Oxocarbon",
                "IBM Carbon Design System inspired",
                Theme::oxocarbon(),
            ),
            ("Forest", "Earthy greens & bark browns", Theme::forest()),
            ("Neon", "Electric brights — synthwave retro", Theme::neon()),
            ("Mono", "Greyscale only — distraction-free", Theme::mono()),
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
        assert!(Theme::all_presets().len() >= 10);
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
