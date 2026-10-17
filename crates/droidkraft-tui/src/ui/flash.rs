//! Flash & Root panel: Fastboot / Root / ROM tabs with a destructive-action
//! confirmation modal. Long operations run on worker threads.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use droidkraft_core::features::flash::RootStatus;
use droidkraft_core::features::rom::StepStatus;
use droidkraft_fastboot::FastbootCommand;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Gauge, Paragraph};
use ratatui::Frame;
use tui_file_explorer::{ExplorerOutcome, FileExplorer, SelectionMode};

use super::common::{draw_explorer, draw_modal, hit, new_explorer, scroll_by, spinner};
use crate::message::Cmd;
use crate::model::Model;
use crate::rom_flash::RomFlashState;
use crate::worker::Req;

pub const HINTS: &str = "h/l tab  j/k select  Enter run  p partition  d detect  n next step";

const MAX_LOG: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Fastboot,
    Root,
    Rom,
}

impl Tab {
    const ALL: [Tab; 3] = [Tab::Fastboot, Tab::Root, Tab::Rom];
    fn label(self) -> &'static str {
        match self {
            Tab::Fastboot => "Fastboot",
            Tab::Root => "Root",
            Tab::Rom => "ROM",
        }
    }
    fn step(self, d: i32) -> Tab {
        Self::ALL[(self as i32 + d).rem_euclid(3) as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FbAction {
    GetVars,
    Unlock,
    Lock,
    Wipe,
    FlashPart,
    Erase,
    SlotA,
    SlotB,
    Reboot,
    RebootBootloader,
    RebootFastbootd,
    RebootRecovery,
    Factory,
}

pub const ACTIONS: [(FbAction, &str); 13] = [
    (FbAction::GetVars, "Get vars"),
    (FbAction::Unlock, "Unlock bootloader"),
    (FbAction::Lock, "Lock bootloader"),
    (FbAction::Wipe, "Wipe data"),
    (FbAction::FlashPart, "Flash partition…"),
    (FbAction::Erase, "Erase partition"),
    (FbAction::SlotA, "Set active slot A"),
    (FbAction::SlotB, "Set active slot B"),
    (FbAction::Reboot, "Reboot"),
    (FbAction::RebootBootloader, "Reboot bootloader"),
    (FbAction::RebootFastbootd, "Reboot fastbootd"),
    (FbAction::RebootRecovery, "Reboot recovery"),
    (FbAction::Factory, "Flash factory image…"),
];

#[derive(Debug, Clone)]
pub enum Pending {
    Fb(FastbootCommand),
    Factory(PathBuf),
    RomStep,
}

pub struct Confirm {
    pub pending: Pending,
    pub text: String,
    pub typed: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    FlashImage,
    Factory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    Tab(Tab),
    Action(usize),
    Build(usize),
}

#[derive(Default)]
pub struct FlashUi {
    pub tab: Tab,
    pub sel: usize,
    pub partition: String,
    pub editing_partition: bool,
    pub devices: Vec<String>,
    pub fb_available: Option<bool>,
    pub root: Option<Result<RootStatus, String>>,
    pub log: Vec<String>,
    /// Lines scrolled up from the bottom.
    pub log_scroll: u16,
    pub running: Option<String>,
    pub progress: Option<(usize, usize)>,
    pub confirm: Option<Confirm>,
    pub explorer: Option<(FileExplorer, Purpose)>,
    pub rom: Option<RomFlashState>,
    hits: Vec<(Hit, Rect)>,
    log_rect: Rect,
}

impl FlashUi {
    pub fn clear_hits(&mut self) {
        self.hits.clear();
    }

    fn push_log(&mut self, s: impl Into<String>) {
        for l in s.into().lines() {
            self.log.push(l.to_string());
        }
        if self.log.len() > MAX_LOG {
            let n = self.log.len() - MAX_LOG;
            self.log.drain(..n);
        }
        self.log_scroll = 0;
    }
}

// ---- worker events --------------------------------------------------------

pub fn tick(m: &mut Model) {
    if let Some(r) = m.flash.rom.as_mut() {
        r.poll();
    }
}

pub fn on_root(m: &mut Model, r: Result<RootStatus, String>) {
    m.flash.root = Some(r);
}

pub fn on_devices(m: &mut Model, d: Vec<String>, avail: bool) {
    m.flash.devices = d;
    m.flash.fb_available = Some(avail);
}

pub fn on_result(m: &mut Model, label: String, result: Result<String, String>) {
    let f = &mut m.flash;
    f.running = None;
    f.progress = None;
    match result {
        Ok(s) => {
            f.push_log(format!("✔ {label}"));
            if !s.trim().is_empty() {
                f.push_log(s);
            }
        }
        Err(e) => f.push_log(format!("✘ {label}: {e}")),
    }
}

pub fn on_progress(m: &mut Model, done: usize, total: usize, label: String) {
    m.flash.progress = Some((done, total));
    m.flash.push_log(format!("[{done}/{total}] {label}"));
}

// ---- actions --------------------------------------------------------------

fn exec_fb(m: &mut Model, cmd: FastbootCommand) -> Vec<Cmd> {
    let label = cmd.label().to_string();
    m.flash.running = Some(label.clone());
    m.flash.push_log(format!("> {label}"));
    vec![Cmd::Worker(Req::Fastboot {
        serial: m.serial(),
        label,
        command: cmd,
    })]
}

fn dispatch_fb(m: &mut Model, cmd: FastbootCommand) -> Vec<Cmd> {
    if m.flash.running.is_some() {
        m.set_message("Operation in progress");
        return vec![];
    }
    if cmd.is_destructive() {
        let text = format!("{} — {:?}", cmd.label(), cmd.args().join(" "));
        m.flash.confirm = Some(Confirm {
            pending: Pending::Fb(cmd),
            text,
            typed: String::new(),
        });
        vec![]
    } else {
        exec_fb(m, cmd)
    }
}

fn accept(m: &mut Model, p: Pending) -> Vec<Cmd> {
    match p {
        Pending::Fb(c) => exec_fb(m, c),
        Pending::Factory(path) => {
            m.flash.running = Some("Factory flash".into());
            m.flash
                .push_log(format!("> Factory flash {}", path.display()));
            vec![Cmd::Worker(Req::FactoryFlash {
                serial: m.serial(),
                source: path,
                wipe: false,
            })]
        }
        Pending::RomStep => {
            if let Some(r) = m.flash.rom.as_mut() {
                r.run_next_step();
            }
            vec![]
        }
    }
}

fn need_partition(m: &mut Model) -> Option<String> {
    let p = m.flash.partition.trim().to_string();
    if p.is_empty() {
        m.set_message("Set a partition name first (p)");
        None
    } else {
        Some(p)
    }
}

fn activate(m: &mut Model, i: usize) -> Vec<Cmd> {
    m.flash.sel = i;
    match ACTIONS[i].0 {
        FbAction::GetVars => dispatch_fb(m, FastbootCommand::GetVarAll),
        FbAction::Unlock => dispatch_fb(m, FastbootCommand::OemUnlock),
        FbAction::Lock => dispatch_fb(m, FastbootCommand::OemLock),
        FbAction::Wipe => dispatch_fb(m, FastbootCommand::WipeData),
        FbAction::Erase => match need_partition(m) {
            Some(partition) => dispatch_fb(m, FastbootCommand::Erase { partition }),
            None => vec![],
        },
        FbAction::SlotA => dispatch_fb(m, FastbootCommand::SetActiveSlot { slot: "a".into() }),
        FbAction::SlotB => dispatch_fb(m, FastbootCommand::SetActiveSlot { slot: "b".into() }),
        FbAction::Reboot => dispatch_fb(m, FastbootCommand::Reboot),
        FbAction::RebootBootloader => dispatch_fb(m, FastbootCommand::RebootBootloader),
        FbAction::RebootFastbootd => dispatch_fb(m, FastbootCommand::RebootFastbootd),
        FbAction::RebootRecovery => dispatch_fb(m, FastbootCommand::RebootRecovery),
        FbAction::FlashPart => {
            if need_partition(m).is_some() {
                m.flash.explorer =
                    Some((new_explorer(&[], SelectionMode::Files), Purpose::FlashImage));
            }
            vec![]
        }
        FbAction::Factory => {
            m.flash.explorer = Some((new_explorer(&[], SelectionMode::Any), Purpose::Factory));
            vec![]
        }
    }
}

fn explorer_selected(m: &mut Model, p: PathBuf, purpose: Purpose) -> Vec<Cmd> {
    match purpose {
        Purpose::FlashImage => {
            let Some(partition) = need_partition(m) else {
                return vec![];
            };
            dispatch_fb(
                m,
                FastbootCommand::FlashPartition {
                    partition,
                    image_path: p.display().to_string(),
                },
            )
        }
        Purpose::Factory => {
            if m.flash.running.is_some() {
                return vec![];
            }
            let text = format!(
                "Flash factory image {} (reflashes all partitions)",
                p.display()
            );
            m.flash.confirm = Some(Confirm {
                pending: Pending::Factory(p),
                text,
                typed: String::new(),
            });
            vec![]
        }
    }
}

fn set_tab(m: &mut Model, t: Tab) {
    m.flash.tab = t;
    m.flash.sel = 0;
    if t == Tab::Rom && m.flash.rom.is_none() {
        m.flash.rom = Some(RomFlashState::new());
    }
}

fn rom_next(m: &mut Model) {
    let Some(r) = m.flash.rom.as_mut() else {
        return;
    };
    if r.next_step_needs_confirmation() {
        let text = r
            .session
            .as_ref()
            .and_then(|s| s.next_step())
            .map(|(_, s)| format!("{}: {}", s.label(), s.description()))
            .unwrap_or_default();
        m.flash.confirm = Some(Confirm {
            pending: Pending::RomStep,
            text,
            typed: String::new(),
        });
    } else {
        r.run_next_step();
    }
}

// ---- drawing --------------------------------------------------------------

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    let block = super::panel_block(m, "Flash & Root");
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.height < 3 {
        // Too small to show (and thus safely answer) a modal: cancel it.
        m.flash.confirm = None;
        m.flash.explorer = None;
        return;
    }
    m.flash.hits.clear();
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
    let mut x = rows[0].x;
    for t in Tab::ALL {
        let label = format!(" {} ", t.label());
        let w = label.chars().count() as u16;
        let r = Rect::new(x, rows[0].y, w.min(rows[0].right().saturating_sub(x)), 1);
        let st = if t == m.flash.tab {
            m.themes.selected()
        } else {
            m.themes.muted()
        };
        f.render_widget(Paragraph::new(label).style(st), r);
        m.flash.hits.push((Hit::Tab(t), r));
        x += w + 1;
    }
    match m.flash.tab {
        Tab::Fastboot => draw_fastboot(f, rows[1], m),
        Tab::Root => draw_root(f, rows[1], m),
        Tab::Rom => draw_rom(f, rows[1], m),
    }
    if let Some(c) = &m.flash.confirm {
        let body = vec![
            Line::from(c.text.clone()),
            Line::from(""),
            Line::from("⚠ Destructive. Type “yes” + Enter, or press Y. Esc/N cancels."),
            Line::from(format!("> {}▏", c.typed)),
        ];
        if !draw_modal(f, area, m, "Confirm", body, 70, 9) {
            m.flash.confirm = None;
        }
    }
    if let Some((ex, _)) = m.flash.explorer.as_mut() {
        draw_explorer(f, area, ex);
    }
}

fn draw_log(f: &mut Frame, area: Rect, m: &mut Model) {
    m.flash.log_rect = area;
    let h = area.height as usize;
    let total = m.flash.log.len();
    let max_up = total.saturating_sub(h);
    m.flash.log_scroll = m.flash.log_scroll.min(max_up as u16);
    let end = total - m.flash.log_scroll as usize;
    let start = end.saturating_sub(h);
    let lines: Vec<Line> = m.flash.log[start..end]
        .iter()
        .map(|l| Line::raw(l.clone()))
        .collect();
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_fastboot(f: &mut Frame, area: Rect, m: &mut Model) {
    let cols = if super::layout::stack_panes(area.width) {
        let h = (ACTIONS.len() as u16).min(area.height / 2);
        Layout::vertical([Constraint::Length(h), Constraint::Min(1)]).split(area)
    } else {
        Layout::horizontal([
            Constraint::Length(26.min(area.width / 2)),
            Constraint::Min(5),
        ])
        .split(area)
    };
    let vis = cols[0].height as usize;
    let off = (m.flash.sel + 1).saturating_sub(vis);
    for (row, (i, (_, label))) in ACTIONS.iter().enumerate().skip(off).take(vis).enumerate() {
        let r = Rect::new(cols[0].x, cols[0].y + row as u16, cols[0].width, 1);
        let st = if i == m.flash.sel {
            m.themes.selected()
        } else {
            m.themes.base()
        };
        f.render_widget(Paragraph::new(format!(" {label}")).style(st), r);
        m.flash.hits.push((Hit::Action(i), r));
    }
    let right = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(cols[1]);
    let avail = match m.flash.fb_available {
        Some(true) => "available",
        Some(false) => "NOT available",
        None => "checking…",
    };
    let devs = if m.flash.devices.is_empty() {
        "none".to_string()
    } else {
        m.flash.devices.join(", ")
    };
    let cursor = if m.flash.editing_partition { "▏" } else { "" };
    let info = vec![
        Line::from(vec![
            Span::styled("fastboot: ", m.themes.title()),
            Span::raw(avail),
        ]),
        Line::from(vec![
            Span::styled("devices:  ", m.themes.title()),
            Span::raw(devs),
        ]),
        Line::from(vec![
            Span::styled("partition: ", m.themes.title()),
            Span::raw(format!("{}{cursor}", m.flash.partition)),
            Span::styled("  (p to edit)", m.themes.muted()),
        ]),
    ];
    f.render_widget(Paragraph::new(info), right[0]);
    if let Some((d, t)) = m.flash.progress {
        let ratio = if t == 0 {
            0.0
        } else {
            (d as f64 / t as f64).clamp(0.0, 1.0)
        };
        f.render_widget(
            Gauge::default().ratio(ratio).label(format!("{d}/{t}")),
            right[1],
        );
    } else if let Some(l) = m.flash.running.clone() {
        spinner(f, m, right[1].x, right[1].y);
        f.render_widget(
            Paragraph::new(format!("Running {l}…")).style(m.themes.muted()),
            Rect::new(
                right[1].x + 2,
                right[1].y,
                right[1].width.saturating_sub(2),
                1,
            ),
        );
    }
    draw_log(f, right[2], m);
}

fn draw_root(f: &mut Frame, area: Rect, m: &mut Model) {
    let rows = Layout::vertical([Constraint::Length(5), Constraint::Min(1)]).split(area);
    let mut lines = vec![];
    match &m.flash.root {
        None => lines.push(Line::raw("Detecting root…")),
        Some(Err(e)) => lines.push(Line::raw(format!("Detection failed: {e}"))),
        Some(Ok(s)) => {
            lines.push(Line::raw(format!(
                "Rooted: {}",
                if s.is_rooted { "yes" } else { "no" }
            )));
            lines.push(Line::raw(format!("Method: {}", s.method.label())));
            lines.push(Line::raw(format!(
                "Magisk: {}",
                s.magisk_version.as_deref().unwrap_or("-")
            )));
        }
    }
    lines.push(Line::styled(
        "d re-detect   m remount (needs root)",
        m.themes.muted(),
    ));
    f.render_widget(Paragraph::new(lines), rows[0]);
    if let Some(l) = m.flash.running.clone() {
        spinner(f, m, rows[1].x, rows[1].y);
        f.render_widget(
            Paragraph::new(format!("Running {l}…")),
            Rect::new(rows[1].x + 2, rows[1].y, rows[1].width.saturating_sub(2), 1),
        );
        let r = Rect::new(
            rows[1].x,
            rows[1].y + 1,
            rows[1].width,
            rows[1].height.saturating_sub(1),
        );
        draw_log(f, r, m);
    } else {
        draw_log(f, rows[1], m);
    }
}

fn draw_rom(f: &mut Frame, area: Rect, m: &mut Model) {
    let Some(rom) = m.flash.rom.as_ref() else {
        f.render_widget(
            Paragraph::new("Press d to detect device and list ROMs").style(m.themes.muted()),
            area,
        );
        return;
    };
    let list_h = (rom.builds.len() as u16).clamp(1, 6);
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(list_h),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(area);
    let prof = rom
        .profile
        .as_ref()
        .map(|p| p.display())
        .unwrap_or_else(|| "no device profile".into());
    f.render_widget(
        Paragraph::new(vec![
            Line::styled(prof, m.themes.title()),
            Line::raw(rom.status.clone()),
        ]),
        rows[0],
    );
    let vis = rows[1].height as usize;
    let off = (rom.selected + 1).saturating_sub(vis);
    let mut hits = vec![];
    for (row, (i, b)) in rom
        .builds
        .iter()
        .enumerate()
        .skip(off)
        .take(vis)
        .enumerate()
    {
        let r = Rect::new(rows[1].x, rows[1].y + row as u16, rows[1].width, 1);
        let st = if i == rom.selected {
            m.themes.selected()
        } else {
            m.themes.base()
        };
        f.render_widget(Paragraph::new(format!(" {}", b.file_name())).style(st), r);
        hits.push((Hit::Build(i), r));
    }
    if let Some(p) = rom.download_progress {
        let ratio = p.fraction().unwrap_or(0.0) as f64;
        f.render_widget(
            Gauge::default()
                .ratio(ratio)
                .label(format!("{:.0}%", ratio * 100.0)),
            rows[2],
        );
    }
    if let Some(s) = &rom.session {
        let lines: Vec<Line> = s
            .plan
            .steps
            .iter()
            .zip(&s.statuses)
            .map(|(step, st)| {
                let mark = match st {
                    StepStatus::Pending => "○",
                    StepStatus::Running => "●",
                    StepStatus::Done => "✔",
                    StepStatus::Failed(_) => "✘",
                };
                Line::raw(format!("{mark} {}", step.label()))
            })
            .collect();
        f.render_widget(Paragraph::new(lines), rows[3]);
    }
    m.flash.hits.extend(hits);
}

// ---- input ----------------------------------------------------------------

pub fn on_key(m: &mut Model, k: KeyEvent) -> (Vec<Cmd>, bool) {
    if let Some((ex, purpose)) = m.flash.explorer.as_mut() {
        let purpose = *purpose;
        match ex.handle_key(k) {
            ExplorerOutcome::Selected(p) => {
                m.flash.explorer = None;
                return (explorer_selected(m, p, purpose), true);
            }
            ExplorerOutcome::Dismissed => m.flash.explorer = None,
            _ => {}
        }
        return (vec![], true);
    }
    if let Some(c) = m.flash.confirm.as_mut() {
        match k.code {
            KeyCode::Char('Y') if c.typed.is_empty() => return confirm_accept(m),
            KeyCode::Enter if c.typed.trim().eq_ignore_ascii_case("yes") => {
                return confirm_accept(m)
            }
            KeyCode::Char('n') | KeyCode::Char('N') if c.typed.is_empty() => m.flash.confirm = None,
            KeyCode::Esc => m.flash.confirm = None,
            KeyCode::Backspace => {
                c.typed.pop();
            }
            KeyCode::Char(ch) if !k.modifiers.contains(KeyModifiers::CONTROL) => c.typed.push(ch),
            _ => {}
        }
        return (vec![], true);
    }
    if m.flash.editing_partition {
        match k.code {
            KeyCode::Esc | KeyCode::Enter => m.flash.editing_partition = false,
            KeyCode::Backspace => {
                m.flash.partition.pop();
            }
            KeyCode::Char(c) if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                m.flash.partition.push(c)
            }
            _ => {}
        }
        return (vec![], true);
    }
    match k.code {
        KeyCode::Left | KeyCode::Char('h') => {
            let t = m.flash.tab.step(-1);
            set_tab(m, t);
            return (vec![], true);
        }
        KeyCode::Right | KeyCode::Char('l') => {
            let t = m.flash.tab.step(1);
            set_tab(m, t);
            return (vec![], true);
        }
        _ => {}
    }
    match m.flash.tab {
        Tab::Fastboot => fb_key(m, k),
        Tab::Root => match k.code {
            KeyCode::Char('d') => (
                vec![Cmd::Worker(Req::DetectRoot { serial: m.serial() })],
                true,
            ),
            KeyCode::Char('m') => {
                m.flash.running = Some("Remount".into());
                (vec![Cmd::Worker(Req::Remount { serial: m.serial() })], true)
            }
            _ => (vec![], false),
        },
        Tab::Rom => rom_key(m, k),
    }
}

fn confirm_accept(m: &mut Model) -> (Vec<Cmd>, bool) {
    let p = m.flash.confirm.take().map(|c| c.pending);
    (p.map(|p| accept(m, p)).unwrap_or_default(), true)
}

fn fb_key(m: &mut Model, k: KeyEvent) -> (Vec<Cmd>, bool) {
    let n = ACTIONS.len();
    match k.code {
        KeyCode::Char('j') | KeyCode::Down => m.flash.sel = (m.flash.sel + 1).min(n - 1),
        KeyCode::Char('k') | KeyCode::Up => m.flash.sel = m.flash.sel.saturating_sub(1),
        KeyCode::Enter => return (activate(m, m.flash.sel), true),
        KeyCode::Char('p') => m.flash.editing_partition = true,
        KeyCode::Char('u') => {
            return (
                vec![Cmd::Worker(Req::FastbootDevices { serial: m.serial() })],
                true,
            )
        }
        KeyCode::PageUp => scroll_by(&mut m.flash.log_scroll, 5, usize::MAX),
        KeyCode::PageDown => scroll_by(&mut m.flash.log_scroll, -5, usize::MAX),
        _ => return (vec![], false),
    }
    (vec![], true)
}

fn rom_key(m: &mut Model, k: KeyEvent) -> (Vec<Cmd>, bool) {
    if m.flash.rom.is_none() {
        m.flash.rom = Some(RomFlashState::new());
    }
    match k.code {
        KeyCode::Char('d') => m.flash.rom.as_mut().map(|r| r.detect()).unwrap_or_default(),
        KeyCode::Char('j') | KeyCode::Down => m
            .flash
            .rom
            .as_mut()
            .map(|r| r.select_down())
            .unwrap_or_default(),
        KeyCode::Char('k') | KeyCode::Up => m
            .flash
            .rom
            .as_mut()
            .map(|r| r.select_up())
            .unwrap_or_default(),
        KeyCode::Enter => m
            .flash
            .rom
            .as_mut()
            .map(|r| r.download_selected())
            .unwrap_or_default(),
        KeyCode::Char('n') => rom_next(m),
        _ => return (vec![], false),
    }
    (vec![], true)
}

pub fn on_mouse(m: &mut Model, e: MouseEvent) -> Vec<Cmd> {
    if m.flash.explorer.is_some() || m.flash.confirm.is_some() {
        return vec![];
    }
    match e.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let h = m
                .flash
                .hits
                .iter()
                .find(|(_, r)| hit(*r, e.column, e.row))
                .map(|(h, _)| *h);
            match h {
                Some(Hit::Tab(t)) => set_tab(m, t),
                Some(Hit::Action(i)) => return activate(m, i),
                Some(Hit::Build(i)) => {
                    if let Some(r) = m.flash.rom.as_mut() {
                        r.selected = i;
                    }
                }
                None => {}
            }
        }
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            let up = e.kind == MouseEventKind::ScrollUp;
            if hit(m.flash.log_rect, e.column, e.row) {
                scroll_by(&mut m.flash.log_scroll, if up { 3 } else { -3 }, usize::MAX);
            } else if m.flash.tab == Tab::Rom {
                if let Some(r) = m.flash.rom.as_mut() {
                    if up {
                        r.select_up()
                    } else {
                        r.select_down()
                    }
                }
            } else {
                let n = ACTIONS.len();
                m.flash.sel = if up {
                    m.flash.sel.saturating_sub(1)
                } else {
                    (m.flash.sel + 1).min(n - 1)
                };
            }
        }
        _ => {}
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_util::{key, render};

    fn model() -> Model {
        crate::ui::test_util::with_device(crate::model::Panel::Flash)
    }

    fn idx(a: FbAction) -> usize {
        ACTIONS.iter().position(|(x, _)| *x == a).unwrap()
    }

    #[test]
    fn renders_all_tabs_both_sizes() {
        for (w, h) in [(80, 24), (200, 60)] {
            let mut m = model();
            let out = render(&mut m, w, h);
            assert!(out.contains("Fastboot") && out.contains("Unlock bootloader"));
            m.flash.tab = Tab::Root;
            assert!(render(&mut m, w, h).contains("Root"));
            m.flash.tab = Tab::Rom;
            assert!(render(&mut m, w, h).contains("ROM"));
        }
    }

    #[test]
    fn destructive_requires_yes() {
        let mut m = model();
        m.flash.sel = idx(FbAction::Unlock);
        let (c, _) = on_key(&mut m, key(KeyCode::Enter));
        assert!(c.is_empty() && m.flash.confirm.is_some());
        // "ye" + Enter is not enough
        for ch in "ye".chars() {
            on_key(&mut m, key(KeyCode::Char(ch)));
        }
        let (c, _) = on_key(&mut m, key(KeyCode::Enter));
        assert!(c.is_empty() && m.flash.confirm.is_some());
        on_key(&mut m, key(KeyCode::Char('s')));
        let (c, _) = on_key(&mut m, key(KeyCode::Enter));
        assert!(matches!(c.as_slice(), [Cmd::Worker(Req::Fastboot { .. })]));
        assert!(m.flash.confirm.is_none());
        assert!(m.flash.running.is_some());
    }

    #[test]
    fn capital_y_confirms_and_esc_cancels() {
        let mut m = model();
        m.flash.sel = idx(FbAction::Wipe);
        on_key(&mut m, key(KeyCode::Enter));
        on_key(&mut m, key(KeyCode::Esc));
        assert!(m.flash.confirm.is_none() && m.flash.running.is_none());
        on_key(&mut m, key(KeyCode::Enter));
        let (c, _) = on_key(&mut m, key(KeyCode::Char('Y')));
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn non_destructive_runs_directly_and_erase_needs_partition() {
        let mut m = model();
        m.flash.sel = idx(FbAction::GetVars);
        let (c, _) = on_key(&mut m, key(KeyCode::Enter));
        assert_eq!(c.len(), 1);
        m.flash.running = None;
        m.flash.sel = idx(FbAction::Erase);
        let (c, _) = on_key(&mut m, key(KeyCode::Enter));
        assert!(c.is_empty() && m.flash.confirm.is_none());
        m.flash.partition = "cache".into();
        on_key(&mut m, key(KeyCode::Enter));
        assert!(m.flash.confirm.is_some());
    }

    #[test]
    fn tab_click_switches() {
        let mut m = model();
        render(&mut m, 120, 30);
        let (_, r) = *m
            .flash
            .hits
            .iter()
            .find(|(h, _)| *h == Hit::Tab(Tab::Root))
            .unwrap();
        on_mouse(
            &mut m,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: r.x,
                row: r.y,
                modifiers: KeyModifiers::NONE,
            },
        );
        assert_eq!(m.flash.tab, Tab::Root);
    }

    #[test]
    fn result_appends_log() {
        let mut m = model();
        m.flash.running = Some("x".into());
        on_result(&mut m, "Get vars".into(), Ok("a: b".into()));
        assert!(m.flash.running.is_none());
        assert!(m.flash.log.iter().any(|l| l.contains("a: b")));
    }
}
