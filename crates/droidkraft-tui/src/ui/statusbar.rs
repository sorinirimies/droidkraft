//! Bottom status bar: key hints + last message.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::model::Model;

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    let t = &m.themes;
    let text = if m.active.is_some() {
        format!(
            " {} Tab tabs · alt-o/l/c/f/s · [ ] device · d home · 1-9 device · t theme · </> speed · ? help · q quit",
            super::hints(m.panel)
        )
    } else {
        " ←→↑↓ select · Enter open · o/l/c/m/R/f card actions · 1-9 device · [ ] · t theme · </> speed · r refresh · ? help · q quit"
            .to_string()
    };
    let mut spans = vec![Span::styled(text, t.muted())];
    if let Some(msg) = &m.message {
        spans.push(Span::styled(format!("  │ {msg}"), t.base()));
    }
    f.render_widget(Paragraph::new(Line::from(spans)).style(t.base()), area);
}
