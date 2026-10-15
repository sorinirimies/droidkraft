//! `update(model, message) -> commands`. Pure state transitions; no I/O.

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::message::{Cmd, Message};
use crate::model::{Model, NavTarget, Panel};
use crate::ui;
use crate::ui::common::hit;
use crate::worker::{Req, Res};

pub fn update(m: &mut Model, msg: Message) -> Vec<Cmd> {
    match msg {
        Message::Tick => {
            m.tick = m.tick.wrapping_add(1);
            ui::logs::tick(m);
            ui::flash::tick(m);
            ui::screen::tick(m);
            if let Some(msg) = m.tick_parked().pop() {
                m.set_message(msg);
            }
            vec![]
        }
        Message::Key(k) => on_key(m, k),
        Message::Mouse(e) => on_mouse(m, e),
        Message::Worker(r) => on_worker(m, r),
    }
}

fn on_key(m: &mut Model, k: KeyEvent) -> Vec<Cmd> {
    if k.kind == KeyEventKind::Release {
        return vec![];
    }
    if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) {
        return vec![Cmd::Quit];
    }
    if m.help {
        if matches!(
            k.code,
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
        ) {
            m.help = false;
        }
        return vec![];
    }
    // Alt+o/l/c/f/s always switches tab (plain letters belong to the panels).
    if k.modifiers.contains(KeyModifiers::ALT) && m.active.is_some() && !m.screen.capture {
        if let KeyCode::Char(c) = k.code {
            if let Some(p) = Panel::from_key(c) {
                return m.set_tab(p);
            }
        }
    }
    // Panels get first refusal so text inputs can consume plain characters.
    if m.active.is_some() {
        let (cmds, consumed) = ui::route_key(m, k);
        if consumed {
            return cmds;
        }
    }
    match k.code {
        KeyCode::Char('q') => vec![Cmd::Quit],
        KeyCode::Char('?') => {
            m.help = true;
            vec![]
        }
        KeyCode::Char('t') => {
            m.themes.cycle();
            let n = m.themes.name().to_string();
            m.set_message(format!("Theme: {n}"));
            vec![]
        }
        KeyCode::Char('r') => {
            m.set_message("Refreshing…");
            vec![Cmd::Worker(Req::Refresh)]
        }
        KeyCode::Char('>' | '+' | '=') => set_rate(m, true),
        KeyCode::Char('<' | '-' | '_') => set_rate(m, false),
        KeyCode::Char('d') => m.go_home(),
        KeyCode::Char(c @ '1'..='9') => m.open_nth(c as usize - '1' as usize),
        KeyCode::Char('[') => m.step_device(-1),
        KeyCode::Char(']') => m.step_device(1),
        KeyCode::Char(c) if m.active.is_some() => match Panel::from_key(c) {
            Some(p) => m.set_tab(p),
            None => vec![],
        },
        KeyCode::Tab if m.active.is_some() => {
            let p = m.panel.next();
            m.set_tab(p)
        }
        KeyCode::BackTab if m.active.is_some() => {
            let p = m.panel.prev();
            m.set_tab(p)
        }
        _ => ui::devices::on_key(m, k),
    }
}

/// Make graphs update faster (`+`) or slower (`-`).
fn set_rate(m: &mut Model, faster: bool) -> Vec<Cmd> {
    let r = if faster {
        m.refresh.faster()
    } else {
        m.refresh.slower()
    };
    if r == m.refresh {
        let end = if faster { "fastest" } else { "slowest" };
        m.set_message(format!("Refresh already {end} ({})", r.label()));
        return vec![];
    }
    m.refresh = r;
    m.set_message(format!("Refresh every {}", r.label()));
    vec![Cmd::Worker(Req::SetRate(r))]
}

fn on_mouse(m: &mut Model, e: MouseEvent) -> Vec<Cmd> {
    if let MouseEventKind::Down(MouseButton::Left) = e.kind {
        let nav = m
            .nav_rects
            .iter()
            .find(|(_, r)| hit(*r, e.column, e.row))
            .map(|(t, _)| t.clone());
        if let Some(t) = nav {
            return match t {
                NavTarget::Home => m.go_home(),
                NavTarget::Device(s) => {
                    let tab = if m.active.is_some() {
                        m.panel
                    } else {
                        Panel::Overview
                    };
                    m.open_device(&s, tab)
                }
                NavTarget::Theme => {
                    m.themes.cycle();
                    let n = m.themes.name().to_string();
                    m.set_message(format!("Theme: {n}"));
                    vec![]
                }
                NavTarget::Help => {
                    m.help = true;
                    vec![]
                }
            };
        }
        if m.active.is_some() {
            let tab = m
                .tab_rects
                .iter()
                .find(|(_, r)| hit(*r, e.column, e.row))
                .map(|(p, _)| *p);
            if let Some(p) = tab {
                return m.set_tab(p);
            }
        }
    }
    if m.active.is_some() {
        ui::route_mouse(m, e)
    } else {
        ui::devices::on_mouse(m, e)
    }
}

fn on_worker(m: &mut Model, r: Res) -> Vec<Cmd> {
    match r {
        Res::Devices(list) => {
            m.apply_devices(&list);
            vec![]
        }
        Res::Details(d) => {
            m.apply_details(*d);
            vec![]
        }
        Res::Output {
            serial,
            label,
            result,
        } => {
            match &result {
                Ok(_) => m.set_message(format!("{label}: done")),
                Err(e) => m.set_message(format!("{label} failed: {e}")),
            }
            m.with_ws(&serial, |m| ui::commands::on_output(m, &label, result));
            vec![]
        }
        Res::Root { serial, result } => {
            m.with_ws(&serial, |m| ui::flash::on_root(m, result));
            vec![]
        }
        Res::FastbootDevices {
            serial,
            devices,
            available,
        } => {
            m.with_ws(&serial, |m| ui::flash::on_devices(m, devices, available));
            vec![]
        }
        Res::Flash {
            serial,
            label,
            result,
        } => {
            m.with_ws(&serial, |m| ui::flash::on_result(m, label, result));
            vec![]
        }
        Res::Screenshot { serial, result } => {
            m.with_ws(&serial, |m| ui::screen::on_screenshot(m, result));
            vec![]
        }
        Res::Progress {
            serial,
            done,
            total,
            label,
        } => {
            m.with_ws(&serial, |m| ui::flash::on_progress(m, done, total, label));
            vec![]
        }
    }
}
