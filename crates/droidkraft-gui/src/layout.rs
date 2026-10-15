//! Pure responsive-layout logic: window size → breakpoint → layout choices.

/// Width breakpoints (logical px).
pub const COMPACT_MAX: f32 = 760.0;
pub const MEDIUM_MAX: f32 = 1100.0;

/// Window size class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Compact,
    Medium,
    Wide,
}

/// Layout decisions derived from the window size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub width: f32,
    pub height: f32,
    pub mode: Mode,
}

impl Layout {
    pub fn new(width: f32, height: f32) -> Self {
        let mode = if width < COMPACT_MAX {
            Mode::Compact
        } else if width < MEDIUM_MAX {
            Mode::Medium
        } else {
            Mode::Wide
        };
        Self {
            width,
            height,
            mode,
        }
    }

    /// Nav rail width in px: full / shorter / icon-only.
    pub fn nav_width(&self) -> f32 {
        match self.mode {
            Mode::Wide => 210.0,
            Mode::Medium => 150.0,
            Mode::Compact => 52.0,
        }
    }

    pub fn nav_labels(&self) -> bool {
        self.mode != Mode::Compact
    }

    pub fn nav_name_chars(&self) -> usize {
        match self.mode {
            Mode::Wide => 22,
            _ => 12,
        }
    }

    /// Width available to the main content.
    pub fn content_width(&self) -> f32 {
        (self.width - self.nav_width()).max(0.0)
    }

    /// Device-card grid columns and the per-card width.
    pub fn card_columns(&self) -> usize {
        const GAP: f32 = 12.0;
        const MIN_CARD: f32 = 300.0;
        let avail = (self.content_width() - 32.0).max(0.0);
        (((avail + GAP) / (MIN_CARD + GAP)).floor() as usize).max(1)
    }

    /// Level of detail for a device card: 2 = all, 1 = no sparkline, 0 = minimal.
    pub fn card_detail(&self) -> u8 {
        let cols = self.card_columns() as f32;
        let w = (self.content_width() - 32.0 - 12.0 * (cols - 1.0)) / cols;
        if self.height < 480.0 || w < 260.0 {
            0
        } else if self.height < 640.0 || w < 330.0 {
            1
        } else {
            2
        }
    }

    /// Overview panel columns (1..=3).
    pub fn overview_columns(&self) -> usize {
        let w = self.content_width();
        if w >= 900.0 {
            3
        } else if w >= 640.0 {
            2
        } else {
            1
        }
    }

    pub fn show_charts(&self) -> bool {
        self.height >= 520.0 && self.content_width() >= 300.0
    }

    /// Show secondary header chips in the workspace.
    pub fn header_chips(&self) -> usize {
        match self.mode {
            Mode::Wide => usize::MAX,
            Mode::Medium => 3,
            Mode::Compact => 1,
        }
    }

    pub fn short_tabs(&self) -> bool {
        self.mode != Mode::Wide
    }

    /// Side panes (recording panel etc.) stack below the main pane.
    pub fn stack_panes(&self) -> bool {
        self.content_width() < 760.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes() {
        assert_eq!(Layout::new(640., 420.).mode, Mode::Compact);
        assert_eq!(Layout::new(900., 600.).mode, Mode::Medium);
        assert_eq!(Layout::new(1400., 900.).mode, Mode::Wide);
    }

    #[test]
    fn columns_reflow() {
        assert_eq!(Layout::new(640., 420.).card_columns(), 1);
        assert!(Layout::new(1100., 700.).card_columns() >= 2);
        assert!(Layout::new(2000., 900.).card_columns() >= 5);
        assert_eq!(Layout::new(0., 0.).card_columns(), 1);
    }

    #[test]
    fn overview_and_detail() {
        assert_eq!(Layout::new(640., 420.).overview_columns(), 1);
        assert_eq!(Layout::new(900., 700.).overview_columns(), 2);
        assert_eq!(Layout::new(1600., 900.).overview_columns(), 3);
        assert!(!Layout::new(640., 420.).show_charts());
        assert!(Layout::new(1400., 900.).show_charts());
        assert_eq!(Layout::new(640., 420.).card_detail(), 0);
        assert_eq!(Layout::new(1400., 900.).card_detail(), 2);
    }

    #[test]
    fn nav_and_panes() {
        assert!(!Layout::new(640., 420.).nav_labels());
        assert!(Layout::new(1400., 900.).nav_labels());
        assert!(Layout::new(800., 600.).stack_panes());
        assert!(!Layout::new(1400., 900.).stack_panes());
        assert_eq!(Layout::new(1400., 900.).header_chips(), usize::MAX);
    }
}
