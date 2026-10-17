//! Devices home: fleet summary + one card per device with quick actions.

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Sparkline, Wrap};
use ratatui::Frame;
use tui_nuances::Status;
use tui_spinner::FluxSpinner;

use super::common::hit;
use super::overview::mini_gauge;
use crate::format::{fmt_mib, truncate};
use crate::message::Cmd;
use crate::model::{state_hint, CardBtn, DeviceWorkspace, Model};

pub const CARD_H: u16 = 11;

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    if m.loading {
        let block = super::panel_block(m, "Devices");
        let inner = block.inner(area);
        f.render_widget(block, area);
        let sp = FluxSpinner::new(m.tick).color(m.themes.accent());
        f.render_widget(
            &sp,
            Rect::new(
                inner.x + 1,
                inner.y,
                1.min(inner.width),
                1.min(inner.height),
            ),
        );
        f.render_widget(
            Paragraph::new("Looking for devices…").style(m.themes.muted()),
            Rect::new(
                inner.x + 3,
                inner.y,
                inner.width.saturating_sub(3),
                1.min(inner.height),
            ),
        );
        return;
    }
    if m.workspaces.is_empty() {
        let block = super::panel_block(m, "Devices");
        let inner = block.inner(area);
        f.render_widget(block, area);
        let t = &m.themes;
        f.render_widget(
            Paragraph::new(vec![
                Line::styled("No device connected", t.status(Status::Warning)),
                Line::styled(
                    "Plug in a device with USB debugging enabled; press r to refresh.",
                    t.muted(),
                ),
            ])
            .wrap(ratatui::widgets::Wrap { trim: false }),
            inner,
        );
        return;
    }
    if area.height < 14 {
        cards(f, area, m);
        return;
    }
    let rows = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(area);
    fleet_row(f, rows[0], m);
    cards(f, rows[1], m);
}

fn fleet_row(f: &mut Frame, area: Rect, m: &Model) {
    let fl = m.fleet();
    let cols = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Percentage(35),
        Constraint::Percentage(35),
    ])
    .split(area);
    let t = &m.themes;
    let b = super::panel_block(m, "Online");
    let i = b.inner(cols[0]);
    f.render_widget(b, cols[0]);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("{}/{}", fl.online, fl.total), t.title()),
            Span::styled(" devices", t.muted()),
        ])),
        i,
    );
    let b = super::panel_block(m, "Avg battery");
    let i = b.inner(cols[1]);
    f.render_widget(b, cols[1]);
    match fl.avg_battery {
        Some(p) => mini_gauge(f, i, m, p as f64, format!("{p}%"), true),
        None => f.render_widget(Paragraph::new("n/a").style(t.muted()), i),
    }
    let b = super::panel_block(m, "RAM in use");
    let i = b.inner(cols[2]);
    f.render_widget(b, cols[2]);
    if fl.ram_total_mib > 0 {
        let p = fl.ram_used_mib as f64 * 100.0 / fl.ram_total_mib as f64;
        mini_gauge(
            f,
            i,
            m,
            p,
            format!(
                "{} / {}",
                fmt_mib(fl.ram_used_mib),
                fmt_mib(fl.ram_total_mib)
            ),
            false,
        );
    } else {
        f.render_widget(Paragraph::new("n/a").style(t.muted()), i);
    }
}

/// Columns for a given width (pure, tested).
pub fn grid_cols(width: u16) -> usize {
    super::layout::card_columns(width) as usize
}

/// Card body rows to show for `n` inner rows, in drop-last priority order
/// (sparkline, then secondary gauges, then android line, buttons last).
pub fn card_rows(n: u16) -> Vec<Row> {
    use Row::*;
    [Head, Bat, Buttons, Android, Ram, Cpu, Sto, Spark]
        .into_iter()
        .take(n as usize)
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    Head,
    Android,
    Bat,
    Ram,
    Cpu,
    Sto,
    Spark,
    Buttons,
}

/// First visible card row so the selected row stays on screen.
pub fn first_row(sel_row: usize, visible_rows: usize) -> usize {
    (sel_row + 1).saturating_sub(visible_rows.max(1))
}

fn cards(f: &mut Frame, area: Rect, m: &mut Model) {
    let order = m.ordered_serials();
    let cols = grid_cols(area.width);
    m.home_cols = cols;
    let card_w = area.width / cols as u16;
    let card_h = CARD_H.min(area.height).max(1);
    let vis_rows = ((area.height / card_h) as usize).max(1);
    let first = first_row(m.home_sel / cols, vis_rows);
    for (i, serial) in order.iter().enumerate() {
        let (r, c) = (i / cols, i % cols);
        if r < first || r >= first + vis_rows {
            continue;
        }
        let rect = Rect::new(
            area.x + c as u16 * card_w,
            area.y + (r - first) as u16 * card_h,
            card_w,
            card_h,
        );
        card(f, rect, m, serial, i == m.home_sel);
    }
    if order.len() > cols * vis_rows {
        let hint = format!(" ▼ {} devices — arrows scroll ", order.len());
        let w = hint.chars().count() as u16;
        if w < area.width && area.height > 0 {
            f.render_widget(
                Paragraph::new(hint).style(m.themes.muted()),
                Rect::new(area.x + area.width - w, area.y + area.height - 1, w, 1),
            );
        }
    }
}

fn card(f: &mut Frame, area: Rect, m: &mut Model, serial: &str, selected: bool) {
    let Some(ws) = m.ws(serial) else { return };
    let online = ws.online();
    let mut title = ws.name();
    if m.is_recording(serial) {
        title.push_str("  ● REC");
    }
    let inner = super::common::card_block(f, area, m, &title, selected);
    m.card_rects.push((serial.to_string(), None, area));
    if inner.height == 0 {
        return;
    }
    let ws: &DeviceWorkspace = m.ws(serial).unwrap();
    let t = &m.themes;
    let st = super::nav::state_status(online, &ws.state);
    let head = Line::from(vec![
        Span::styled("● ", t.status(st)),
        Span::raw(format!(
            "{} · {} · {}",
            truncate(&ws.serial, 20),
            ws.connection(),
            ws.state
        )),
    ]);
    let Some(d) = ws.details.as_ref().filter(|_| online) else {
        let lines = vec![
            head,
            Line::raw(""),
            Line::styled(state_hint(&ws.state), t.status(Status::Warning)),
        ];
        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
        return;
    };
    let rows = card_rows(inner.height);
    let line = |i: usize| Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
    let mut extras = String::new();
    if matches!(d.battery.status, droidkraft_core::BatteryStatus::Charging) {
        extras.push_str(" ⚡");
    }
    if let Some(tc) = d.battery.temperature_c {
        extras.push_str(&format!(" {tc:.0}°C"));
    }
    let cpu = ws.history.cpu_pct.last().copied().unwrap_or(0.0);
    let gauge = |row: Row| -> (&'static str, f64, String, bool) {
        match row {
            Row::Bat => (
                "BAT",
                d.battery.level as f64,
                format!("{}%{extras}", d.battery.level),
                true,
            ),
            Row::Ram => (
                "RAM",
                d.memory.used_pct() as f64,
                format!(
                    "{:.0}% {}",
                    d.memory.used_pct(),
                    fmt_mib(d.memory.used_mib())
                ),
                false,
            ),
            Row::Cpu => ("CPU", cpu, format!("{cpu:.0}%"), false),
            _ => (
                "STO",
                d.storage.used_pct() as f64,
                format!(
                    "{:.0}% {}",
                    d.storage.used_pct(),
                    fmt_mib(d.storage.used_mib)
                ),
                false,
            ),
        }
    };
    let patch = if d.system.security_patch.is_empty() {
        "-"
    } else {
        &d.system.security_patch
    };
    let android = format!(
        "Android {} (SDK {}) · patch {}",
        d.system.android_release, d.system.sdk, patch
    );
    let data: Vec<u64> = ws.history.cpu_pct.iter().map(|v| *v as u64).collect();
    let accent = t.accent();
    let muted = t.muted();
    let mut btn_row = None;
    // Display order is fixed; priority only decides which rows survive.
    let order = [
        Row::Head,
        Row::Android,
        Row::Bat,
        Row::Ram,
        Row::Cpu,
        Row::Sto,
        Row::Spark,
        Row::Buttons,
    ];
    for (y, r) in order.into_iter().filter(|r| rows.contains(r)).enumerate() {
        match r {
            Row::Head => f.render_widget(Paragraph::new(head.clone()), line(y)),
            Row::Android => f.render_widget(
                Paragraph::new(truncate(&android, inner.width as usize)).style(muted),
                line(y),
            ),
            Row::Spark => f.render_widget(
                Sparkline::default()
                    .data(&data)
                    .max(100)
                    .style(ratatui::style::Style::default().fg(accent)),
                line(y),
            ),
            Row::Buttons => btn_row = Some(line(y)),
            g => {
                let (name, pct, label, invert) = gauge(g);
                f.render_widget(
                    Paragraph::new(name).style(muted),
                    Rect::new(inner.x, inner.y + y as u16, 4.min(inner.width), 1),
                );
                let gw = inner.width.saturating_sub(4);
                super::common::smooth_gauge(
                    f,
                    Rect::new(inner.x + 4, inner.y + y as u16, gw, 1),
                    m,
                    pct,
                    &label,
                    invert,
                );
            }
        }
    }
    if let Some(r) = btn_row {
        draw_buttons(f, r, m, serial);
    }
}

fn draw_buttons(f: &mut Frame, r: Rect, m: &mut Model, serial: &str) {
    let full: u16 = CardBtn::ALL
        .iter()
        .map(|b| b.label(false).len() as u16 + 3)
        .sum();
    let short = r.width < full;
    let mut x = r.x;
    for b in CardBtn::ALL {
        let label = format!("[{}]", b.label(short));
        let w = label.chars().count() as u16;
        if x + w > r.x + r.width {
            break;
        }
        let rect = Rect::new(x, r.y, w, 1);
        let style = if b == CardBtn::Record {
            m.themes.status(Status::Error)
        } else {
            ratatui::style::Style::default().fg(m.themes.accent())
        };
        f.render_widget(Paragraph::new(label).style(style), rect);
        m.card_rects.push((serial.to_string(), Some(b), rect));
        x += w + 1;
    }
}

/// Open a device on a card button's tab (Record also starts recording).
pub fn open_card(m: &mut Model, serial: &str, btn: Option<CardBtn>) -> Vec<Cmd> {
    let tab = btn.map_or(crate::model::Panel::Overview, CardBtn::tab);
    let cmds = m.open_device(serial, tab);
    if btn == Some(CardBtn::Record) && m.connected() && !m.screen.recorder.is_recording() {
        super::screen::toggle_record(m);
    }
    cmds
}

/// Selection after an arrow key (pure, tested).
pub fn move_sel(sel: usize, n: usize, cols: usize, code: KeyCode) -> usize {
    if n == 0 {
        return 0;
    }
    let cols = cols.max(1);
    match code {
        KeyCode::Left => sel.saturating_sub(1),
        KeyCode::Right => (sel + 1).min(n - 1),
        KeyCode::Up => sel.saturating_sub(cols),
        KeyCode::Down => (sel + cols).min(n - 1),
        _ => sel,
    }
}

pub fn on_key(m: &mut Model, k: KeyEvent) -> Vec<Cmd> {
    let order = m.ordered_serials();
    if order.is_empty() {
        return vec![];
    }
    let cur = order[m.home_sel.min(order.len() - 1)].clone();
    let btn = match k.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
            m.home_sel = move_sel(m.home_sel, order.len(), m.home_cols, k.code);
            return vec![];
        }
        KeyCode::Enter | KeyCode::Char('o') => CardBtn::Overview,
        KeyCode::Char('l') => CardBtn::Logs,
        KeyCode::Char('c') => CardBtn::Commands,
        KeyCode::Char('m') | KeyCode::Char('s') => CardBtn::Mirror,
        KeyCode::Char('R') => CardBtn::Record,
        KeyCode::Char('f') => CardBtn::Flash,
        _ => return vec![],
    };
    open_card(m, &cur, Some(btn))
}

pub fn on_mouse(m: &mut Model, e: MouseEvent) -> Vec<Cmd> {
    if !matches!(e.kind, MouseEventKind::Down(MouseButton::Left)) {
        return vec![];
    }
    let found = m
        .card_rects
        .iter()
        .rev()
        .find(|(_, _, r)| hit(*r, e.column, e.row))
        .map(|(s, b, _)| (s.clone(), *b));
    match found {
        Some((serial, btn)) => {
            if let Some(i) = m.ordered_serials().iter().position(|s| *s == serial) {
                m.home_sel = i;
            }
            open_card(m, &serial, btn)
        }
        None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_math() {
        assert_eq!(grid_cols(30), 1);
        assert_eq!(grid_cols(100), 2);
        assert_eq!(card_rows(3), vec![Row::Head, Row::Bat, Row::Buttons]);
        assert!(!card_rows(7).contains(&Row::Spark));
        assert!(card_rows(8).contains(&Row::Spark));
        assert_eq!(first_row(0, 2), 0);
        assert_eq!(first_row(3, 2), 2);
    }

    #[test]
    fn selection_moves() {
        assert_eq!(move_sel(0, 5, 2, KeyCode::Right), 1);
        assert_eq!(move_sel(0, 5, 2, KeyCode::Left), 0);
        assert_eq!(move_sel(1, 5, 2, KeyCode::Down), 3);
        assert_eq!(move_sel(4, 5, 2, KeyCode::Down), 4);
        assert_eq!(move_sel(3, 5, 2, KeyCode::Up), 1);
        assert_eq!(move_sel(0, 0, 2, KeyCode::Down), 0);
    }
}
