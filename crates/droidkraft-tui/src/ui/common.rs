//! Shared UI helpers: hit-testing, modals, file-explorer overlay, scrolling.

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;
use tui_file_explorer::{FileExplorer, SelectionMode};
use tui_spinner::FluxSpinner;

use ratatui::style::Style;
use ratatui::text::Span;

use crate::format::truncate;
use crate::model::Model;

/// Whether the cell `(col,row)` lies inside `r`.
pub fn hit(r: Rect, col: u16, row: u16) -> bool {
    col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
}

/// A rect of at most `w`x`h` centred in `area`.
pub fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

/// Approximate number of rows `text` occupies when wrapped to `width`.
pub fn wrapped_lines(text: &str, width: u16) -> usize {
    super::layout::wrapped_height(text, width.max(1) as usize).max(1)
}

/// Adjust a scroll offset by `delta`, clamped to `0..=max`.
pub fn scroll_by(v: &mut u16, delta: i32, max: usize) {
    let max = max.min(u16::MAX as usize) as i32;
    *v = (*v as i32 + delta).clamp(0, max) as u16;
}

/// Centered modal box with a title and body lines.
pub fn draw_modal(
    f: &mut Frame,
    area: Rect,
    m: &Model,
    title: &str,
    body: Vec<Line<'static>>,
    w: u16,
    h: u16,
) {
    // Grow to fit the wrapped body (so the key hint never falls off the
    // bottom), bounded by the available area.
    let w = w.min(area.width);
    let text: Vec<String> = body.iter().map(|l| l.to_string()).collect();
    let need =
        super::layout::wrapped_height(&text.join("\n"), w.saturating_sub(2) as usize) as u16 + 2;
    let h = h.max(need).min(area.height);
    let r = centered(area, w, h);
    f.render_widget(Clear, r);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(m.themes.status(tui_nuances::Status::Warning))
        .title(format!(" {title} "))
        .style(m.themes.base());
    let inner = block.inner(r);
    f.render_widget(block, r);
    f.render_widget(Paragraph::new(body).wrap(Wrap { trim: false }), inner);
}

/// Build a file explorer. `exts` filters files; `mode` picks what can be selected.
pub fn new_explorer(exts: &[&str], mode: SelectionMode) -> FileExplorer {
    let dir = std::env::current_dir().unwrap_or_else(|_| ".".into());
    FileExplorer::builder(dir)
        .extension_filter(exts.iter().map(|s| s.to_string()).collect())
        .selection_mode(mode)
        .build()
}

/// Draw a file explorer as a large overlay over `area`.
pub fn draw_explorer(f: &mut Frame, area: Rect, ex: &mut FileExplorer) {
    let r = centered(
        area,
        area.width.saturating_sub(4).max(1),
        area.height.saturating_sub(2).max(1),
    );
    f.render_widget(Clear, r);
    tui_file_explorer::render(ex, f, r);
}

/// Render a 1-cell spinner at `(x,y)`.
pub fn spinner(f: &mut Frame, m: &Model, x: u16, y: u16) {
    let sp = FluxSpinner::new(m.tick).color(m.themes.accent());
    f.render_widget(&sp, Rect::new(x, y, 1, 1));
}

/// Shared card/panel: bordered block with truncated title. `selected` = double
/// accent border. Renders the frame and returns the inner rect.
pub fn card_block(f: &mut Frame, area: Rect, m: &Model, title: &str, selected: bool) -> Rect {
    let t = &m.themes;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if selected {
            BorderType::Double
        } else {
            BorderType::Rounded
        })
        .border_style(t.border(selected))
        .title(Span::styled(
            format!(
                " {} ",
                truncate(title, area.width.saturating_sub(4) as usize)
            ),
            t.title(),
        ))
        .style(t.base());
    let inner = block.inner(area);
    f.render_widget(block, area);
    inner
}

/// Partial-block bar string of `width` cells for `pct` (0..=100), 1/8-cell resolution.
pub fn bar_string(pct: f64, width: usize) -> String {
    const PARTS: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];
    let eighths = ((pct.clamp(0.0, 100.0) / 100.0) * width as f64 * 8.0).round() as usize;
    let (full, rem) = (eighths / 8, eighths % 8);
    let mut s = "█".repeat(full.min(width));
    if full < width {
        s.push(PARTS[rem]);
        s.push_str(&" ".repeat(width - full - 1));
    }
    s
}

/// Smooth gauge: partial-block bar + right-aligned bright value.
pub fn smooth_gauge(f: &mut Frame, area: Rect, m: &Model, pct: f64, label: &str, invert: bool) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let t = &m.themes;
    let lw = (label.chars().count() as u16).min(area.width / 2);
    let bw = area.width.saturating_sub(lw + 1) as usize;
    let color = super::overview::level_bands(m, invert)
        .iter()
        .find(|(lim, _)| pct <= *lim)
        .map_or(t.accent(), |(_, c)| *c);
    let line = Line::from(vec![
        Span::styled(bar_string(pct, bw), Style::default().fg(color)),
        Span::raw(" "),
        Span::styled(truncate(label, lw as usize), t.title()),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

/// Friendly empty/loading/error body inside `inner`.
pub fn notice(f: &mut Frame, inner: Rect, m: &Model, head: &str, hint: &str, warn: bool) {
    let t = &m.themes;
    let hs = if warn {
        t.status(tui_nuances::Status::Warning)
    } else {
        t.muted()
    };
    f.render_widget(
        Paragraph::new(vec![
            Line::styled(head.to_string(), hs),
            Line::styled(hint.to_string(), t.muted()),
        ])
        .wrap(Wrap { trim: true }),
        inner,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_partial() {
        assert_eq!(bar_string(0.0, 4).chars().count(), 4);
        assert_eq!(bar_string(100.0, 4), "████");
        assert_eq!(bar_string(50.0, 4), "██  ");
        assert_eq!(bar_string(12.5, 4), "▌   ");
        assert_eq!(bar_string(150.0, 3), "███");
    }
}
