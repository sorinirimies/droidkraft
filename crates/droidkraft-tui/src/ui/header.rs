//! Header strip: Devices summary on home, device identity inside a workspace.

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use tui_nuances::Status;

use crate::model::Model;

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    let title = if m.active.is_some() {
        "Device"
    } else {
        "Devices"
    };
    let block = super::panel_block(m, title);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let t = &m.themes;
    let theme = Span::styled(format!("  theme: {}", t.name()), t.muted());
    let rate = Span::styled(format!("  ⟳ {}", m.refresh.label()), t.muted());
    let line = if m.loading {
        Line::from(Span::styled("Connecting…", t.muted()))
    } else if let Some(ws) = m.active_ws() {
        let st = super::nav::state_status(ws.online(), &ws.state);
        let mut v = vec![
            Span::styled("● ", t.status(st)),
            Span::styled(ws.name(), t.title()),
            Span::styled(
                format!(
                    "  {}  {}  {}",
                    crate::format::truncate(&ws.serial, 16),
                    droidkraft_core::connection_kind(&ws.serial),
                    ws.state
                ),
                t.muted(),
            ),
        ];
        // Ordered by priority: `fit_line` drops what does not fit.
        v.push(rate.clone());
        if let Some(d) = &ws.details {
            let s = &d.system;
            v.push(Span::styled(
                format!("  Android {}", s.android_release),
                t.muted(),
            ));
            v.push(Span::styled(format!("  🔋{}%", d.battery.level), t.muted()));
            {
                v.push(Span::styled(
                    format!("  Wi-Fi {}", d.network.wifi_ssid.as_deref().unwrap_or("-")),
                    t.muted(),
                ));
                v.push(Span::styled(
                    format!(
                        "  root {}",
                        match s.is_rooted {
                            Some(true) => "yes",
                            Some(false) => "no",
                            None => "?",
                        }
                    ),
                    t.muted(),
                ));
                v.push(Span::styled(
                    format!(
                        "  BL {}",
                        match s.flash_locked {
                            Some(true) => "locked",
                            Some(false) => "unlocked",
                            None => "?",
                        }
                    ),
                    t.muted(),
                ));
            }
        }
        v.push(theme);
        Line::from(v)
    } else {
        let f = m.fleet();
        if f.total == 0 {
            Line::from(vec![
                Span::styled("● ", t.status(Status::Error)),
                Span::raw("No device connected"),
                rate,
                theme,
            ])
        } else {
            Line::from(vec![
                Span::styled("● ", t.status(Status::Success)),
                Span::raw(format!("{} online of {}", f.online, f.total)),
                rate,
                theme,
            ])
        }
    };
    f.render_widget(Paragraph::new(fit_line(line, inner.width as usize)), inner);
}

/// Keep leading spans while they fit in `width`; later spans that do not fit
/// are skipped (so low-priority info drops first). The first two spans (status
/// dot + name) are always kept but truncated.
fn fit_line(line: Line<'static>, width: usize) -> Line<'static> {
    let mut out: Vec<Span<'static>> = Vec::new();
    let mut used = 0usize;
    for (i, sp) in line.spans.into_iter().enumerate() {
        let w = super::layout::cells(&sp.content);
        if used + w <= width {
            used += w;
            out.push(sp);
        } else if i < 2 {
            let room = width.saturating_sub(used);
            let content = super::layout::truncate_to(&sp.content, room);
            used += super::layout::cells(&content);
            out.push(Span::styled(content, sp.style));
        }
    }
    Line::from(out)
}
