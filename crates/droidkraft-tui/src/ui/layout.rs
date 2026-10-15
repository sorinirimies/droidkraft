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

/// Display width of `s` in terminal cells (chars; the UI uses no wide glyphs
/// in hint/label text).
pub fn cells(s: &str) -> usize {
    s.chars().count()
}

/// Truncate to at most `w` cells, ending with `…` when cut.
pub fn truncate_to(s: &str, w: usize) -> String {
    if cells(s) <= w {
        s.to_string()
    } else if w == 0 {
        String::new()
    } else {
        let mut t: String = s.chars().take(w - 1).collect();
        t.push('…');
        t
    }
}

/// Split a hints string (`"a b  c d · e"`) into individual hint items.
pub fn split_hints(s: &str) -> Vec<String> {
    s.split(" · ")
        .flat_map(|p| p.split("  "))
        .map(|p| p.trim().trim_matches('·').trim())
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect()
}

/// Greedily pack `items` into lines of at most `width` cells joined by `sep`.
/// Items are never split; an item wider than a line is truncated.
pub fn wrap_items(items: &[String], sep: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for it in items {
        let it = truncate_to(it, width);
        if cur.is_empty() {
            cur = it;
        } else if cells(&cur) + cells(sep) + cells(&it) <= width {
            cur.push_str(sep);
            cur.push_str(&it);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur = it;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// Lay out footer hints in at most `max_lines` lines. `essentials` (help/quit)
/// are always kept; optional `items` are dropped from the end until it fits.
pub fn footer_lines(
    items: &[String],
    essentials: &[String],
    width: usize,
    max_lines: usize,
) -> Vec<String> {
    let max_lines = max_lines.max(1);
    let mut keep = items.len();
    loop {
        let all: Vec<String> = items[..keep]
            .iter()
            .chain(essentials.iter())
            .cloned()
            .collect();
        let lines = wrap_items(&all, " · ", width);
        if lines.len() <= max_lines {
            return lines;
        }
        if keep == 0 {
            // Even the essentials do not fit: show the tail of what we have.
            let mut l = lines;
            l.truncate(max_lines);
            return l;
        }
        keep -= 1;
    }
}

/// Footer line budget for a terminal `height`.
pub fn footer_max_lines(height: u16) -> usize {
    match height {
        0..=13 => 1,
        14..=23 => 2,
        _ => 3,
    }
}

/// Number of rows `text` needs when word-wrapped to `width` cells.
pub fn wrapped_height(text: &str, width: usize) -> usize {
    let width = width.max(1);
    let mut rows = 0;
    for line in text.split('\n') {
        let mut cur = 0usize;
        let mut r = 1;
        for word in line.split(' ') {
            let w = cells(word);
            if cur == 0 {
                cur = w;
            } else if cur + 1 + w <= width {
                cur += 1 + w;
            } else {
                r += 1;
                cur = w;
            }
            while cur > width {
                r += 1;
                cur -= width;
            }
        }
        rows += r;
    }
    rows
}

/// Choose tab labels for `width`: full names, then 3-letter, then 1-letter.
/// Each label is padded with a space either side; tabs are 1 cell apart.
/// Returns `None` when even single letters do not fit.
pub fn tab_labels(full: &[&str], width: usize) -> Option<Vec<String>> {
    let tiers: [fn(&str) -> String; 3] = [
        |s| format!(" {s} "),
        |s| format!(" {} ", s.chars().take(3).collect::<String>()),
        |s| format!(" {} ", s.chars().take(1).collect::<String>()),
    ];
    for tier in tiers {
        let labels: Vec<String> = full.iter().map(|s| tier(s)).collect();
        let total: usize =
            labels.iter().map(|l| cells(l)).sum::<usize>() + labels.len().saturating_sub(1);
        if total <= width {
            return Some(labels);
        }
    }
    None
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

    #[test]
    fn splits_hints() {
        assert_eq!(
            split_hints("j/k select  Enter run · : shell ·"),
            vec!["j/k select", "Enter run", ": shell"]
        );
    }

    #[test]
    fn wrap_never_exceeds_width() {
        let items: Vec<String> = (0..12).map(|i| format!("key{i} does thing")).collect();
        for w in [8usize, 20, 33, 60, 120] {
            for l in wrap_items(&items, " · ", w) {
                assert!(cells(&l) <= w, "{l:?} > {w}");
            }
        }
    }

    #[test]
    fn footer_keeps_essentials() {
        let items: Vec<String> = (0..20).map(|i| format!("item{i}")).collect();
        let ess = vec!["? help".to_string(), "q quit".to_string()];
        for w in [20usize, 40, 80, 200] {
            for max in 1..=3 {
                let l = footer_lines(&items, &ess, w, max);
                assert!(l.len() <= max);
                assert!(l.iter().all(|x| cells(x) <= w));
                if w >= 20 {
                    assert!(
                        l.last().unwrap().contains("q quit"),
                        "w={w} max={max} {l:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn wrapped_height_counts_rows() {
        assert_eq!(wrapped_height("aaa bbb", 10), 1);
        assert_eq!(wrapped_height("aaa bbb", 5), 2);
        assert_eq!(wrapped_height("a\nb", 10), 2);
        assert_eq!(wrapped_height("abcdefghij", 4), 3);
    }

    #[test]
    fn tab_labels_degrade() {
        let names = ["Overview", "Logs", "Commands", "Flash & Root", "Screen"];
        assert!(tab_labels(&names, 80).unwrap()[0].contains("Overview"));
        assert_eq!(tab_labels(&names, 29).unwrap()[0], " Ove ");
        assert_eq!(tab_labels(&names, 19).unwrap()[0], " O ");
        assert!(tab_labels(&names, 5).is_none());
    }
}
