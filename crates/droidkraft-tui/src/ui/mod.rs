//! Draw dispatcher + input routing.
//!
//! Panel module convention: `draw(f, area, m)`, `on_key(m, key) -> (Vec<Cmd>, bool)`,
//! `on_mouse(m, event) -> Vec<Cmd>` and a `HINTS` status-bar string.

pub mod commands;
pub mod common;
pub mod devices;
pub mod flash;
pub mod header;
pub mod help;
pub mod layout;
pub mod logs;
pub mod nav;
pub mod overview;
pub mod screen;
pub mod statusbar;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use tui_nuances::Status;

use crate::message::Cmd;
use crate::model::{Model, Panel};

/// Themed bordered block with a title, shared by every panel.
pub fn panel_block<'a>(m: &Model, title: &str) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(m.themes.border(false))
        .title(Span::styled(format!(" {title} "), m.themes.title()))
}

/// Draw the whole UI.
pub fn draw(f: &mut Frame, m: &mut Model) {
    let area = f.area();
    m.screen.gfx.begin_frame();
    f.render_widget(Block::default().style(m.themes.base()), area);
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);
    header::draw(f, rows[0], m);
    let cols = Layout::horizontal([
        Constraint::Length(layout::nav_width(area.width)),
        Constraint::Min(1),
    ])
    .split(rows[1]);
    nav::draw(f, cols[0], m);
    draw_main(f, cols[1], m);
    statusbar::draw(f, rows[2], m);
    if m.help {
        help::draw(f, area, m);
    }
    m.screen.gfx.end_frame();
}

fn draw_main(f: &mut Frame, area: Rect, m: &mut Model) {
    m.tab_rects.clear();
    if m.active.is_none() {
        m.card_rects.clear();
        devices::draw(f, area, m);
        return;
    }
    let offline = !m.connected();
    let banner_h = u16::from(offline);
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(banner_h),
        Constraint::Min(1),
    ])
    .split(area);
    draw_tabs(f, rows[0], m);
    if offline {
        draw_banner(f, rows[1], m);
    }
    match m.panel {
        Panel::Overview => overview::draw(f, rows[2], m),
        Panel::Logs => logs::draw(f, rows[2], m),
        Panel::Commands => commands::draw(f, rows[2], m),
        Panel::Flash => flash::draw(f, rows[2], m),
        Panel::Screen => screen::draw(f, rows[2], m),
    }
}

fn draw_tabs(f: &mut Frame, area: Rect, m: &mut Model) {
    let mut x = area.x;
    for p in Panel::ALL {
        let label = if area.width < 60 {
            format!(" {} ", p.label().chars().take(3).collect::<String>())
        } else {
            format!(" {} ", p.label())
        };
        let w = label.chars().count() as u16;
        if x + w > area.x + area.width {
            break;
        }
        let r = Rect::new(x, area.y, w, 1);
        let style = if p == m.panel {
            m.themes.selected()
        } else {
            m.themes.muted()
        };
        f.render_widget(Paragraph::new(label).style(style), r);
        m.tab_rects.push((p, r));
        x += w + 1;
    }
}

fn draw_banner(f: &mut Frame, area: Rect, m: &Model) {
    let (state, name) = m
        .active_ws()
        .map(|w| (w.state.clone(), w.name()))
        .unwrap_or_default();
    let line = Line::from(vec![
        Span::styled(
            format!(" ○ {name} is {state} — "),
            m.themes.status(Status::Warning),
        ),
        Span::styled(crate::model::state_hint(&state), m.themes.muted()),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

pub fn hints(p: Panel) -> &'static str {
    match p {
        Panel::Overview => overview::HINTS,
        Panel::Logs => logs::HINTS,
        Panel::Commands => commands::HINTS,
        Panel::Flash => flash::HINTS,
        Panel::Screen => screen::HINTS,
    }
}

/// Route a key to the active panel. Returns (cmds, consumed).
pub fn route_key(m: &mut Model, k: KeyEvent) -> (Vec<Cmd>, bool) {
    match m.panel {
        Panel::Overview => overview::on_key(m, k),
        Panel::Logs => logs::on_key(m, k),
        Panel::Commands => commands::on_key(m, k),
        Panel::Flash => flash::on_key(m, k),
        Panel::Screen => screen::on_key(m, k),
    }
}

pub fn route_mouse(m: &mut Model, e: MouseEvent) -> Vec<Cmd> {
    match m.panel {
        Panel::Overview => overview::on_mouse(m, e),
        Panel::Logs => logs::on_mouse(m, e),
        Panel::Commands => commands::on_mouse(m, e),
        Panel::Flash => flash::on_mouse(m, e),
        Panel::Screen => screen::on_mouse(m, e),
    }
}

#[cfg(test)]
pub mod test_util {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{backend::TestBackend, Terminal};

    use crate::model::Model;

    /// Model with one online device `TESTSERIAL` open on `panel`.
    pub fn with_device(panel: crate::model::Panel) -> Model {
        let mut m = Model {
            loading: false,
            ..Default::default()
        };
        let mut ws = crate::model::DeviceWorkspace::new("TESTSERIAL");
        ws.listed = true;
        ws.state = "device".into();
        m.workspaces.insert("TESTSERIAL".into(), ws);
        m.open_device("TESTSERIAL", panel);
        m
    }

    pub fn key(c: KeyCode) -> KeyEvent {
        KeyEvent::new(c, KeyModifiers::NONE)
    }

    /// Render the full UI to a string at `w`x`h`.
    pub fn render(m: &mut Model, w: u16, h: u16) -> String {
        let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
        t.draw(|f| super::draw(f, m)).unwrap();
        let buf = t.backend().buffer().clone();
        let mut out = String::new();
        for y in 0..h {
            for x in 0..w {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod render_tests {
    use super::test_util::*;
    use crate::model::Panel;

    #[test]
    fn all_panels_all_sizes_no_panic() {
        for panel in Panel::ALL {
            for (w, h) in [(40, 12), (60, 20), (80, 24), (120, 30), (200, 60)] {
                let mut m = with_device(panel);
                let out = render(&mut m, w, h);
                assert_eq!(out.lines().count(), h as usize);
            }
        }
        let mut m = crate::model::Model {
            loading: false,
            ..Default::default()
        };
        for (w, h) in [(40, 12), (80, 24), (200, 60)] {
            render(&mut m, w, h);
        }
    }

    #[test]
    fn wide_shows_labels_compact_hides() {
        let mut m = with_device(Panel::Overview);
        assert!(render(&mut m, 120, 30).contains("Devices"));
        assert!(!render(&mut m, 40, 12).contains("DroidKraft"));
    }
}

#[cfg(test)]
mod size_tests {
    use super::test_util::*;
    use crate::model::{DeviceWorkspace, Model, Panel};

    const SIZES: [(u16, u16); 5] = [(40, 12), (60, 20), (80, 24), (120, 30), (200, 60)];

    fn home(n: usize) -> Model {
        let mut m = Model {
            loading: false,
            ..Default::default()
        };
        for i in 0..n {
            let serial = format!("FAKE{i}");
            let mut ws = DeviceWorkspace::new(&serial);
            ws.listed = true;
            ws.state = "device".into();
            let mut d = droidkraft_core::DeviceDetails::default();
            d.system.manufacturer = "Acme".into();
            d.system.model = format!("Phone{i}");
            d.system.android_release = "14".into();
            d.battery.level = 77;
            d.memory.total_mib = 8000;
            d.storage.total_mib = 128000;
            ws.details = Some(d);
            m.workspaces.insert(serial, ws);
        }
        m
    }

    #[test]
    fn home_renders_at_all_sizes() {
        for n in [1, 3] {
            for (w, h) in SIZES {
                let mut m = home(n);
                let out = render(&mut m, w, h);
                assert_eq!(out.lines().count(), h as usize);
                if w >= 60 && h >= 20 {
                    assert!(out.contains("Phone0"), "{w}x{h} n={n}");
                }
                for (_, _, r) in &m.card_rects {
                    assert!(
                        r.right() <= w && r.bottom() <= h,
                        "rect out of bounds {r:?} at {w}x{h}"
                    );
                }
            }
        }
    }

    #[test]
    fn overview_renders_at_all_sizes() {
        for (w, h) in SIZES {
            let mut m = home(1);
            m.open_device("FAKE0", Panel::Overview);
            let out = render(&mut m, w, h);
            assert_eq!(out.lines().count(), h as usize);
            if w >= 60 {
                assert!(out.contains("Battery"), "{w}x{h}");
            }
        }
    }
}
