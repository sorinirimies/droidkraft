//! Bottom status bar: key hints (wrapped / trimmed to fit) + last message.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::layout::{footer_lines, footer_max_lines, split_hints, truncate_to};
use crate::model::Model;

/// What the footer will show for a terminal of `width`x`height`.
pub struct Plan {
    pub hints: Vec<String>,
    pub message: Option<String>,
}

impl Plan {
    /// Rows the footer needs.
    pub fn height(&self) -> u16 {
        (self.hints.len() + usize::from(self.message.is_some())).max(1) as u16
    }
}

fn items(m: &Model) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    if m.active.is_some() {
        v.extend(split_hints(super::hints(m.panel)));
        v.extend(
            [
                "Tab tabs",
                "alt-o/l/c/f/s",
                "[ ] device",
                "d home",
                "1-9 device",
                "t theme",
                "</> speed",
            ]
            .map(String::from),
        );
    } else {
        v.extend(
            [
                "←→↑↓ select",
                "Enter open",
                "o/l/c/m/R/f card actions",
                "1-9 device",
                "[ ]",
                "t theme",
                "</> speed",
                "r refresh",
            ]
            .map(String::from),
        );
    }
    v
}

/// Compute the footer layout. Rows are scarce on short terminals, so the
/// budget shrinks with `height` and low-priority hints are dropped first.
pub fn plan(m: &Model, width: u16, height: u16) -> Plan {
    let w = width.max(1) as usize;
    let max = footer_max_lines(height);
    let ess = vec!["? help".to_string(), "q quit".to_string()];
    let msg = m
        .message
        .as_ref()
        .map(|s| truncate_to(s, w.saturating_sub(2)));
    if let (Some(msg), 1) = (&msg, max) {
        // One row: the transient message wins over the hints.
        // ...but keep the essential "? q" escape hatch visible.
        let line = if w >= 12 {
            format!("{}  ? q", truncate_to(msg, w.saturating_sub(6)))
        } else {
            msg.clone()
        };
        return Plan {
            hints: vec![],
            message: Some(line),
        };
    }
    let budget = if msg.is_some() { max - 1 } else { max };
    Plan {
        hints: footer_lines(&items(m), &ess, w, budget),
        message: msg,
    }
}

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    let t = &m.themes;
    let p = plan(m, area.width, f.area().height);
    let mut lines: Vec<Line> = p
        .hints
        .iter()
        .map(|l| Line::from(Span::styled(l.clone(), t.muted())))
        .collect();
    if let Some(msg) = p.message {
        lines.push(Line::from(Span::styled(format!("▸ {msg}"), t.base())));
    }
    f.render_widget(Paragraph::new(lines).style(t.base()), area);
}
