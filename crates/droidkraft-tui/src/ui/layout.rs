//! Responsive breakpoints and pure layout helpers.

/// Width class of the terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bp {
    Compact,
    Medium,
    Wide,
}

impl Bp {
    pub fn of(width: u16) -> Bp {
        match width {
            0..=59 => Bp::Compact,
            60..=99 => Bp::Medium,
            _ => Bp::Wide,
        }
    }
}

/// Nav rail width for a terminal width.
pub fn nav_width(total: u16) -> u16 {
    match Bp::of(total) {
        Bp::Wide => 26,
        Bp::Medium => 16,
        Bp::Compact => 6,
    }
}

/// Whether side-by-side panes should stack vertically in `width`.
pub fn stack_panes(width: u16) -> bool {
    width < 70
}

/// Card grid columns for `width` (min card width 34), at least 1.
pub fn card_columns(width: u16) -> u16 {
    (width / 34).clamp(1, 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoints() {
        assert_eq!(Bp::of(40), Bp::Compact);
        assert_eq!(Bp::of(60), Bp::Medium);
        assert_eq!(Bp::of(99), Bp::Medium);
        assert_eq!(Bp::of(100), Bp::Wide);
    }

    #[test]
    fn widths_and_columns() {
        assert_eq!(nav_width(200), 26);
        assert_eq!(nav_width(80), 16);
        assert_eq!(nav_width(40), 6);
        assert_eq!(card_columns(10), 1);
        assert_eq!(card_columns(70), 2);
        assert_eq!(card_columns(400), 4);
        assert!(stack_panes(50) && !stack_panes(120));
    }
}
