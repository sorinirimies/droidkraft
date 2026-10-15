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
    let theme = Span::styled(
        format!("  theme: {}  ⟳ {}", t.name(), m.refresh.label()),
        t.muted(),
    );
    let line = if m.loading {
        Line::from(Span::styled("Connecting…", t.muted()))
    } else if let Some(ws) = m.active_ws() {
        let st = super::nav::state_status(ws.online(), &ws.state);
        let mut v = vec![
            Span::styled("● ", t.status(st)),
            Span::styled(ws.name(), t.title()),
            Span::styled(
                format!(
                    "  {}  {}",
                    crate::format::truncate(&ws.serial, 16),
                    ws.state
                ),
                t.muted(),
            ),
        ];
        let bp = super::layout::Bp::of(area.width);
        if let (Some(d), true) = (&ws.details, bp != super::layout::Bp::Compact) {
            let s = &d.system;
            v.push(Span::styled(
                format!("  Android {}", s.android_release),
                t.muted(),
            ));
            v.push(Span::styled(format!("  🔋{}%", d.battery.level), t.muted()));
            if bp == super::layout::Bp::Wide {
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
        if bp == super::layout::Bp::Wide {
            v.push(theme);
        }
        Line::from(v)
    } else {
        let f = m.fleet();
        if f.total == 0 {
            Line::from(vec![
                Span::styled("● ", t.status(Status::Error)),
                Span::raw("No device connected"),
                theme,
            ])
        } else {
            Line::from(vec![
                Span::styled("● ", t.status(Status::Success)),
                Span::raw(format!("{} online of {}", f.online, f.total)),
                theme,
            ])
        }
    };
    f.render_widget(Paragraph::new(line), inner);
}
