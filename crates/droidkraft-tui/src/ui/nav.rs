//! Left navigation rail: Devices home, one entry per device, Theme/Help (mouse clickable).

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use tui_nuances::Status;

use crate::format::truncate;
use crate::model::{Model, NavTarget};

/// Status colour for a device state.
pub fn state_status(online: bool, state: &str) -> Status {
    if online {
        Status::Success
    } else if state == "disconnected" {
        Status::Error
    } else {
        Status::Warning
    }
}

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    let compact = area.width < 10;
    let block = super::panel_block(m, if compact { "" } else { "DroidKraft" });
    let inner = block.inner(area);
    f.render_widget(block, area);
    m.nav_rects.clear();
    let bottom = 2u16.min(inner.height);
    let mut entries: Vec<(NavTarget, Line<'static>, bool)> = Vec::new();
    let home_active = m.active.is_none();
    let t = &m.themes;
    entries.push((
        NavTarget::Home,
        if compact {
            Line::from(Span::raw(" ⌂"))
        } else {
            Line::from(vec![
                Span::raw(" ⌂ Devices "),
                Span::styled(format!("({})", m.workspaces.len()), t.muted()),
            ])
        },
        home_active,
    ));
    for (i, s) in m.ordered_serials().iter().enumerate() {
        let Some(ws) = m.ws(s) else { continue };
        let bat = ws.battery_pct().map_or(String::new(), |b| format!(" {b}%"));
        let rec = if m.is_recording(s) { " ●REC" } else { "" };
        let name_w =
            (inner.width as usize).saturating_sub(5 + bat.chars().count() + rec.chars().count());
        let mut dot_style = t.status(state_status(ws.online(), &ws.state));
        if !ws.online() {
            dot_style = dot_style.add_modifier(ratatui::style::Modifier::DIM);
        }
        let name_style = if ws.online() { t.base() } else { t.muted() };
        let line = if compact {
            Line::from(vec![
                Span::raw(format!("{} ", i + 1)),
                Span::styled("●", dot_style),
            ])
        } else {
            Line::from(vec![
                Span::raw(format!(" {} ", i + 1)),
                Span::styled("●", dot_style),
                Span::styled(format!(" {}", truncate(&ws.name(), name_w)), name_style),
                Span::styled(bat, t.muted()),
                Span::styled(rec, t.status(Status::Error)),
            ])
        };
        entries.push((
            NavTarget::Device(s.clone()),
            line,
            m.active.as_deref() == Some(s.as_str()),
        ));
    }
    let list_h = inner.height.saturating_sub(bottom);
    for (i, (target, line, active)) in entries.into_iter().enumerate() {
        if i as u16 >= list_h {
            break;
        }
        let r = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
        let style = if active {
            m.themes.selected()
        } else {
            m.themes.base()
        };
        f.render_widget(Paragraph::new(line).style(style), r);
        m.nav_rects.push((target, r));
    }
    let bottoms = if compact {
        [
            (NavTarget::Theme, " ◐".to_string()),
            (NavTarget::Help, " ?".to_string()),
        ]
    } else {
        [
            (NavTarget::Theme, format!(" ◐ Theme: {}", m.themes.name())),
            (NavTarget::Help, " ? Help".to_string()),
        ]
    };
    for (i, (target, label)) in bottoms.into_iter().enumerate() {
        if i as u16 >= bottom {
            break;
        }
        let y = inner.y + inner.height - bottom + i as u16;
        let r = Rect::new(inner.x, y, inner.width, 1);
        f.render_widget(
            Paragraph::new(truncate(&label, inner.width as usize)).style(m.themes.muted()),
            r,
        );
        m.nav_rects.push((target, r));
    }
}
