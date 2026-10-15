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

    /// Side panes (recording panel etc.) stack below the main pane.
    pub fn stack_panes(&self) -> bool {
        self.content_width() < 760.0
    }
}

/// How tab labels are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabStyle {
    Full,
    Short,
    Initial,
}

/// How the refresh-rate control is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateStyle {
    /// `Refresh − 1.5s +`
    Full,
    /// `− 1.5s +`
    Compact,
    Hidden,
}

/// Tab bar decision for a given width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabBar {
    pub tabs: TabStyle,
    pub rate: RateStyle,
}

/// Rough text width (px) at ~14px UI font.
fn text_px(chars: usize) -> f32 {
    chars as f32 * 7.6
}

impl TabBar {
    /// Width (px) the tab labels need in `style`.
    pub fn tabs_width(labels: &[&str], style: TabStyle) -> f32 {
        let pad = if style == TabStyle::Full { 24.0 } else { 16.0 };
        labels
            .iter()
            .map(|l| text_px(l.chars().count()) + pad)
            .sum::<f32>()
            + 4.0 * labels.len().saturating_sub(1) as f32
    }

    /// Width (px) of the rate control in `style`.
    pub fn rate_width(style: RateStyle) -> f32 {
        match style {
            RateStyle::Full => 170.0,
            RateStyle::Compact => 112.0,
            RateStyle::Hidden => 0.0,
        }
    }
}

impl Layout {
    /// Pick the richest tab-bar variant that fits the content width.
    /// `full`, `short` and `initial` are the label sets per style.
    pub fn tab_bar(&self, full: &[&str], short: &[&str], initial: &[&str]) -> TabBar {
        let avail = (self.content_width() - 32.0).max(0.0);
        let sets = |s: TabStyle| match s {
            TabStyle::Full => full,
            TabStyle::Short => short,
            TabStyle::Initial => initial,
        };
        const ORDER: [(TabStyle, RateStyle); 8] = [
            (TabStyle::Full, RateStyle::Full),
            (TabStyle::Full, RateStyle::Compact),
            (TabStyle::Short, RateStyle::Full),
            (TabStyle::Short, RateStyle::Compact),
            (TabStyle::Short, RateStyle::Hidden),
            (TabStyle::Initial, RateStyle::Compact),
            (TabStyle::Initial, RateStyle::Hidden),
            (TabStyle::Initial, RateStyle::Hidden),
        ];
        for (tabs, rate) in ORDER {
            let need = TabBar::tabs_width(sets(tabs), tabs)
                + TabBar::rate_width(rate)
                + if rate == RateStyle::Hidden { 0.0 } else { 12.0 };
            if need <= avail {
                return TabBar { tabs, rate };
            }
        }
        TabBar {
            tabs: TabStyle::Initial,
            rate: RateStyle::Hidden,
        }
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

    const FULL: [&str; 5] = ["Overview", "Logs", "Commands", "Flash & Root", "Screen"];
    const SHORT: [&str; 5] = ["Stats", "Logs", "Cmds", "Flash", "Screen"];
    const INIT: [&str; 5] = ["O", "L", "C", "F", "S"];

    #[test]
    fn tab_bar_leanest_when_tiny() {
        let b = Layout::new(300., 600.).tab_bar(&FULL, &SHORT, &INIT);
        assert_eq!((b.tabs, b.rate), (TabStyle::Initial, RateStyle::Hidden));
    }

    #[test]
    fn tab_bar_fits_at_every_width() {
        for w in (300..=2400).step_by(10) {
            let lay = Layout::new(w as f32, 700.);
            let b = lay.tab_bar(&FULL, &SHORT, &INIT);
            let set: &[&str] = match b.tabs {
                TabStyle::Full => &FULL,
                TabStyle::Short => &SHORT,
                TabStyle::Initial => &INIT,
            };
            let need = TabBar::tabs_width(set, b.tabs) + TabBar::rate_width(b.rate);
            // Only the leanest variant may overflow, and only on tiny windows.
            if !(b.tabs == TabStyle::Initial && b.rate == RateStyle::Hidden) {
                assert!(
                    need <= lay.content_width() - 32.0,
                    "w={w} {b:?} need={need}"
                );
            }
        }
    }

    #[test]
    fn tab_bar_choices() {
        let wide = Layout::new(1600., 900.).tab_bar(&FULL, &SHORT, &INIT);
        assert_eq!((wide.tabs, wide.rate), (TabStyle::Full, RateStyle::Full));
        let min = Layout::new(640., 420.).tab_bar(&FULL, &SHORT, &INIT);
        assert_ne!(min.rate, RateStyle::Full, "640px window must save space");
    }

    #[test]
    fn layout_never_degenerate() {
        for w in (0..=3000).step_by(40) {
            for h in (0..=2000).step_by(100) {
                let l = Layout::new(w as f32, h as f32);
                assert!(l.card_columns() >= 1);
                assert!(l.content_width() >= 0.0);
                assert!((1..=3).contains(&l.overview_columns()));
                assert!(l.card_detail() <= 2);
            }
        }
    }
}
