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
use ratatui::text::Span;
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
    if area.height < 5 || area.width < 12 {
        let msg = "Too small (min 12x5)";
        f.render_widget(
            Paragraph::new(layout::truncate_to(msg, area.width as usize)).style(m.themes.base()),
            area,
        );
        return;
    }
    let footer_h = statusbar::plan(m, area.width, area.height)
        .height()
        .min(area.height.saturating_sub(4).max(1));
    let rows = Layout::vertical([
        Constraint::Length(3.min(area.height)),
        Constraint::Min(1),
        Constraint::Length(footer_h),
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
    let banner_h = if offline {
        banner_text(m)
            .map(|t| layout::wrapped_height(&t, area.width as usize).min(3) as u16)
            .unwrap_or(1)
    } else {
        0
    };
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
    let names: Vec<&str> = Panel::ALL.iter().map(|p| p.label()).collect();
    let w = area.width as usize;
    let labels: Vec<(Panel, String)> = match layout::tab_labels(&names, w) {
        Some(l) => Panel::ALL.iter().copied().zip(l).collect(),
        // Too narrow even for single letters: show the active tab only.
        None => vec![(
            m.panel,
            layout::truncate_to(&format!(" {} ", m.panel.label()), w),
        )],
    };
    let mut x = area.x;
    for (p, label) in labels {
        let lw = layout::cells(&label) as u16;
        let r = Rect::new(x, area.y, lw.min(area.right().saturating_sub(x)), 1);
        let style = if p == m.panel {
            m.themes.selected()
        } else {
            m.themes.muted()
        };
        f.render_widget(Paragraph::new(label).style(style), r);
        m.tab_rects.push((p, r));
        x += lw + 1;
    }
}

fn banner_text(m: &Model) -> Option<String> {
    let w = m.active_ws()?;
    Some(format!(
        " ○ {} is {} — {}",
        w.name(),
        w.state,
        crate::model::state_hint(&w.state)
    ))
}

fn draw_banner(f: &mut Frame, area: Rect, m: &Model) {
    let Some(text) = banner_text(m) else { return };
    f.render_widget(
        Paragraph::new(text)
            .style(m.themes.status(Status::Warning))
            .wrap(ratatui::widgets::Wrap { trim: false }),
        area,
    );
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

    const SIZES: [(u16, u16); 8] = [
        (1, 1),
        (10, 3),
        (20, 5),
        (40, 12),
        (60, 20),
        (80, 24),
        (120, 30),
        (200, 60),
    ];

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

    mod responsive {
        use crate::model::Panel;
        use crate::ui::{layout, statusbar, test_util};

        fn rows(out: &str) -> Vec<String> {
            out.lines().map(|l| l.trim_end().to_string()).collect()
        }

        fn footer_of(r: &[String]) -> String {
            r[r.len().saturating_sub(3)..].join("\n")
        }

        #[test]
        fn footer_always_shows_help_and_quit() {
            for panel in Panel::ALL {
                for w in (20..=200).step_by(7) {
                    for h in [10u16, 14, 24, 40] {
                        let mut m = test_util::with_device(panel);
                        let r = rows(&test_util::render(&mut m, w, h));
                        assert!(
                            footer_of(&r).contains("q quit"),
                            "{panel:?} {w}x{h}:\n{}",
                            footer_of(&r)
                        );
                    }
                }
            }
        }

        #[test]
        fn footer_rows_never_exceed_budget() {
            let m = test_util::with_device(Panel::Logs);
            for w in 10..=200u16 {
                for h in [8u16, 14, 30] {
                    let p = statusbar::plan(&m, w, h);
                    assert!(p.height() as usize <= layout::footer_max_lines(h));
                    assert!(p.hints.iter().all(|l| layout::cells(l) <= w as usize));
                }
            }
        }

        #[test]
        fn message_is_visible_on_any_height() {
            for (w, h) in [(30u16, 10u16), (60, 20), (120, 40)] {
                let mut m = test_util::with_device(Panel::Overview);
                m.set_message("Refresh every 2s");
                let r = rows(&test_util::render(&mut m, w, h));
                assert!(footer_of(&r).contains("Refresh"), "{w}x{h}");
            }
        }

        #[test]
        fn header_keeps_name_and_never_overflows() {
            for w in 24..=200u16 {
                let mut m = test_util::with_device(Panel::Overview);
                let r = rows(&test_util::render(&mut m, w, 20));
                assert!(r[1].contains("TESTSE"), "{w}: header {:?}", r[1]);
                assert!(layout::cells(&r[1]) <= w as usize);
            }
        }

        #[test]
        fn tabs_degrade_but_active_stays_visible() {
            for w in 12..=200u16 {
                let mut m = test_util::with_device(Panel::Logs);
                let r = rows(&test_util::render(&mut m, w, 20));
                let top = r[..8].join("\n");
                assert!(top.contains("Log") || top.contains(" L "), "{w}:\n{top}");
            }
        }

        #[test]
        fn help_wraps_instead_of_clipping() {
            let mut m = test_util::with_device(Panel::Overview);
            m.help = true;
            for w in [20u16, 40, 70, 120] {
                let out = test_util::render(&mut m, w, 60);
                assert!(out.contains("Help"), "{w}");
            }
        }

        #[test]
        fn offline_banner_wraps() {
            let mut m = test_util::with_device(Panel::Overview);
            m.workspaces.get_mut("TESTSERIAL").unwrap().state = "offline".into();
            for w in [24u16, 40, 80] {
                let out = test_util::render(&mut m, w, 20);
                assert!(out.contains("offline"), "{w}");
            }
        }
    }
}
