//! Commands panel: clickable command list (left), scrollable result (right),
//! shell prompt with history, APK install via file explorer.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;
use tui_file_explorer::{ExplorerOutcome, FileExplorer, SelectionMode};

use super::common::{
    centered, draw_explorer, draw_modal, hit, new_explorer, scroll_by, spinner, wrapped_lines,
};
use crate::commands::{catalog, CmdAction, Command};
use crate::message::Cmd;
use crate::model::Model;
use crate::worker::Req;
use droidkraft_core::AdbCommand;

pub const HINTS: &str = "j/k select  Enter run  : shell  PgUp/PgDn scroll result";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    Cmd(usize),
    Install,
    Uninstall,
    Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    Shell,
    Uninstall,
}

pub struct CmdUi {
    pub items: Vec<Command>,
    pub entries: Vec<Entry>,
    pub sel: usize,
    pub title: String,
    pub result: String,
    pub running: Option<String>,
    pub scroll: u16,
    pub input: Option<(InputKind, String)>,
    pub history: Vec<String>,
    pub hist_pos: Option<usize>,
    pub explorer: Option<FileExplorer>,
    /// Index into `items` awaiting confirmation.
    pub confirm: Option<usize>,
    pub hits: Vec<(usize, Rect)>,
    pub result_rect: Rect,
}

impl Default for CmdUi {
    fn default() -> Self {
        let items = catalog();
        let mut entries = Vec::new();
        for (i, c) in items.iter().enumerate() {
            entries.push(Entry::Cmd(i));
            // Package tools follow the last Packages command.
            let last_pkg = c.category == crate::commands::Category::Packages
                && items.get(i + 1).is_none_or(|n| n.category != c.category);
            if last_pkg {
                entries.push(Entry::Install);
                entries.push(Entry::Uninstall);
            }
        }
        entries.push(Entry::Shell);
        Self {
            items,
            entries,
            sel: 0,
            title: "Result".into(),
            result: "Select a command and press Enter.".into(),
            running: None,
            scroll: 0,
            input: None,
            history: vec![],
            hist_pos: None,
            explorer: None,
            confirm: None,
            hits: vec![],
            result_rect: Rect::default(),
        }
    }
}

impl CmdUi {
    fn label(&self, e: Entry) -> String {
        match e {
            Entry::Cmd(i) => self.items[i].label.to_string(),
            Entry::Install => "Install APK…".into(),
            Entry::Uninstall => "Uninstall Package…".into(),
            Entry::Shell => "Shell prompt (:)".into(),
        }
    }
    fn group(&self, e: Entry) -> &'static str {
        match e {
            Entry::Cmd(i) => self.items[i].category.label(),
            Entry::Install | Entry::Uninstall => "Packages",
            Entry::Shell => "Shell",
        }
    }
}

/// Worker result for a command run from this panel.
pub fn on_output(m: &mut Model, label: &str, result: Result<String, String>) {
    if m.cmds.running.is_none() {
        return;
    }
    m.cmds.running = None;
    m.cmds.title = label.to_string();
    m.cmds.scroll = 0;
    m.cmds.result = match result {
        Ok(s) if s.trim().is_empty() => "(no output)".into(),
        Ok(s) => s,
        Err(e) => format!("Error: {e}"),
    };
}

fn run(m: &mut Model, label: String, action: CmdAction) -> Vec<Cmd> {
    if !m.connected() {
        m.set_message("No device connected");
        return vec![];
    }
    m.cmds.running = Some(label.clone());
    m.cmds.result.clear();
    vec![Cmd::Worker(Req::Run {
        serial: m.serial(),
        label,
        action,
    })]
}

fn activate(m: &mut Model, idx: usize) -> Vec<Cmd> {
    m.cmds.sel = idx;
    match m.cmds.entries[idx] {
        Entry::Cmd(i) => {
            if m.cmds.items[i].destructive {
                m.cmds.confirm = Some(i);
                vec![]
            } else {
                let c = m.cmds.items[i].clone();
                run(m, c.label.to_string(), c.action)
            }
        }
        Entry::Install => {
            m.cmds.explorer = Some(new_explorer(&["apk"], SelectionMode::Files));
            vec![]
        }
        Entry::Uninstall => {
            m.cmds.input = Some((InputKind::Uninstall, String::new()));
            vec![]
        }
        Entry::Shell => {
            m.cmds.input = Some((InputKind::Shell, String::new()));
            vec![]
        }
    }
}

fn submit(m: &mut Model, kind: InputKind, text: String) -> Vec<Cmd> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return vec![];
    }
    match kind {
        InputKind::Shell => {
            m.cmds.history.push(text.clone());
            m.cmds.hist_pos = None;
            if !m.connected() {
                m.set_message("No device connected");
                return vec![];
            }
            let label = format!("$ {text}");
            m.cmds.running = Some(label.clone());
            m.cmds.result.clear();
            vec![Cmd::Worker(Req::Shell {
                serial: m.serial(),
                label,
                command: text,
            })]
        }
        InputKind::Uninstall => run(
            m,
            format!("Uninstall {text}"),
            CmdAction::Adb(AdbCommand::UninstallPackage { package_name: text }),
        ),
    }
}

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    let cols = if super::layout::stack_panes(area.width) {
        Layout::vertical([Constraint::Percentage(45), Constraint::Min(3)]).split(area)
    } else {
        Layout::horizontal([
            Constraint::Length(28.min(area.width / 3).max(12)),
            Constraint::Min(10),
        ])
        .split(area)
    };
    // list
    let block = super::panel_block(m, "Commands");
    let inner = block.inner(cols[0]);
    f.render_widget(block, cols[0]);
    m.cmds.hits.clear();
    let mut lines: Vec<(Line, Option<usize>)> = Vec::new();
    let mut last = "";
    for (i, e) in m.cmds.entries.iter().enumerate() {
        let g = m.cmds.group(*e);
        if g != last {
            lines.push((Line::styled(format!("─ {g}"), m.themes.title()), None));
            last = g;
        }
        let st = if i == m.cmds.sel {
            m.themes.selected()
        } else {
            m.themes.base()
        };
        lines.push((Line::styled(format!("  {}", m.cmds.label(*e)), st), Some(i)));
    }
    let h = inner.height as usize;
    let sel_row = lines
        .iter()
        .position(|(_, i)| *i == Some(m.cmds.sel))
        .unwrap_or(0);
    let off = (sel_row + 1).saturating_sub(h);
    for (row, (line, idx)) in lines.into_iter().skip(off).take(h).enumerate() {
        let r = Rect::new(inner.x, inner.y + row as u16, inner.width, 1);
        if let Some(i) = idx {
            m.cmds.hits.push((i, r));
        }
        f.render_widget(Paragraph::new(line), r);
    }

    // result
    let title = m.cmds.title.clone();
    let rb = super::panel_block(m, &title);
    let rin = rb.inner(cols[1]);
    f.render_widget(rb, cols[1]);
    let prompt_h = u16::from(m.cmds.input.is_some());
    let body = Rect::new(rin.x, rin.y, rin.width, rin.height.saturating_sub(prompt_h));
    m.cmds.result_rect = body;
    if let Some(l) = m.cmds.running.clone() {
        spinner(f, m, body.x, body.y);
        f.render_widget(
            Paragraph::new(format!("Running {l}…")).style(m.themes.muted()),
            Rect::new(body.x + 2, body.y, body.width.saturating_sub(2), 1),
        );
    } else {
        let max = wrapped_lines(&m.cmds.result, body.width).saturating_sub(body.height as usize);
        scroll_by(&mut m.cmds.scroll, 0, max);
        f.render_widget(
            Paragraph::new(m.cmds.result.clone())
                .wrap(Wrap { trim: false })
                .scroll((m.cmds.scroll, 0)),
            body,
        );
    }
    if let Some((k, text)) = &m.cmds.input {
        let p = if *k == InputKind::Shell {
            "$ "
        } else {
            "pkg> "
        };
        let avail = (rin.width as usize).saturating_sub(p.len() + 1);
        let line = Line::from(vec![
            Span::styled(p, m.themes.title()),
            Span::raw(format!("{}▏", super::layout::tail_to(text, avail))),
        ]);
        if rin.height > 0 {
            f.render_widget(
                Paragraph::new(line),
                Rect::new(rin.x, rin.bottom().saturating_sub(1), rin.width, 1),
            );
        }
    }

    if let Some(i) = m.cmds.confirm {
        let body = vec![
            Line::from(format!("Run “{}”?", m.cmds.items[i].label)),
            Line::from(""),
            Line::from("Y / Enter = yes     N / Esc = cancel"),
        ];
        if !draw_modal(f, area, m, "Confirm", body, 44, 6) {
            m.cmds.confirm = None;
        }
    }
    if let Some(ex) = m.cmds.explorer.as_mut() {
        draw_explorer(f, area, ex);
    }
    let _ = centered;
}

pub fn on_key(m: &mut Model, k: KeyEvent) -> (Vec<Cmd>, bool) {
    if let Some(ex) = m.cmds.explorer.as_mut() {
        match ex.handle_key(k) {
            ExplorerOutcome::Selected(p) => {
                m.cmds.explorer = None;
                let label = format!(
                    "Install {}",
                    p.file_name().and_then(|s| s.to_str()).unwrap_or("apk")
                );
                let c = run(
                    m,
                    label,
                    CmdAction::Adb(AdbCommand::InstallPackage {
                        apk_path: p.display().to_string(),
                    }),
                );
                return (c, true);
            }
            ExplorerOutcome::Dismissed => m.cmds.explorer = None,
            _ => {}
        }
        return (vec![], true);
    }
    if let Some(i) = m.cmds.confirm {
        match k.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                m.cmds.confirm = None;
                let c = m.cmds.items[i].clone();
                return (run(m, c.label.to_string(), c.action), true);
            }
            _ => m.cmds.confirm = None,
        }
        return (vec![], true);
    }
    if let Some((kind, mut text)) = m.cmds.input.take() {
        match k.code {
            KeyCode::Esc => {}
            KeyCode::Enter => return (submit(m, kind, text), true),
            KeyCode::Backspace => {
                text.pop();
                m.cmds.input = Some((kind, text));
            }
            KeyCode::Up if kind == InputKind::Shell && !m.cmds.history.is_empty() => {
                let pos = m
                    .cmds
                    .hist_pos
                    .map_or(m.cmds.history.len() - 1, |p| p.saturating_sub(1));
                m.cmds.hist_pos = Some(pos);
                m.cmds.input = Some((kind, m.cmds.history[pos].clone()));
            }
            KeyCode::Down if kind == InputKind::Shell => {
                match m.cmds.hist_pos {
                    Some(p) if p + 1 < m.cmds.history.len() => {
                        m.cmds.hist_pos = Some(p + 1);
                        text = m.cmds.history[p + 1].clone();
                    }
                    _ => {
                        m.cmds.hist_pos = None;
                        text.clear();
                    }
                }
                m.cmds.input = Some((kind, text));
            }
            KeyCode::Char(c) if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                text.push(c);
                m.cmds.input = Some((kind, text));
            }
            _ => m.cmds.input = Some((kind, text)),
        }
        return (vec![], true);
    }
    let n = m.cmds.entries.len();
    let page = m.cmds.result_rect.height.max(1) as i32;
    match k.code {
        KeyCode::Char('j') | KeyCode::Down => m.cmds.sel = (m.cmds.sel + 1).min(n - 1),
        KeyCode::Char('k') | KeyCode::Up => m.cmds.sel = m.cmds.sel.saturating_sub(1),
        KeyCode::Enter => return (activate(m, m.cmds.sel), true),
        KeyCode::Char(':') | KeyCode::Char('!') => {
            m.cmds.input = Some((InputKind::Shell, String::new()))
        }
        KeyCode::PageDown => scroll_by(&mut m.cmds.scroll, page, usize::MAX),
        KeyCode::PageUp => scroll_by(&mut m.cmds.scroll, -page, usize::MAX),
        _ => return (vec![], false),
    }
    (vec![], true)
}

pub fn on_mouse(m: &mut Model, e: MouseEvent) -> Vec<Cmd> {
    if m.cmds.explorer.is_some() || m.cmds.confirm.is_some() {
        return vec![];
    }
    match e.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let h = m
                .cmds
                .hits
                .iter()
                .find(|(_, r)| hit(*r, e.column, e.row))
                .map(|(i, _)| *i);
            if let Some(i) = h {
                return activate(m, i);
            }
        }
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            let d = if e.kind == MouseEventKind::ScrollUp {
                -3
            } else {
                3
            };
            if hit(m.cmds.result_rect, e.column, e.row) {
                scroll_by(&mut m.cmds.scroll, d, usize::MAX);
            } else {
                let n = m.cmds.entries.len() as i32;
                m.cmds.sel = (m.cmds.sel as i32 + d.signum()).clamp(0, n - 1) as usize;
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
        crate::ui::test_util::with_device(crate::model::Panel::Commands)
    }

    #[test]
    fn renders_both_sizes() {
        for (w, h) in [(80, 24), (200, 60)] {
            let mut m = model();
            let out = render(&mut m, w, h);
            assert!(out.contains("Commands"));
            assert!(out.contains("Battery") || w == 80);
            assert!(out.contains("Result"));
        }
    }

    #[test]
    fn covers_gui_command_set() {
        let ui = CmdUi::default();
        for want in [
            "Battery",
            "Screenshot",
            "Install APK…",
            "Uninstall Package…",
            "Reboot",
            "Network",
            "Properties",
        ] {
            assert!(ui.entries.iter().any(|e| ui.label(*e) == want), "{want}");
        }
    }

    #[test]
    fn shell_history_up_down() {
        let mut m = model();
        m.cmds.history = vec!["ls".into(), "pwd".into()];
        on_key(&mut m, key(KeyCode::Char(':')));
        on_key(&mut m, key(KeyCode::Up));
        assert_eq!(m.cmds.input.as_ref().unwrap().1, "pwd");
        on_key(&mut m, key(KeyCode::Up));
        assert_eq!(m.cmds.input.as_ref().unwrap().1, "ls");
        on_key(&mut m, key(KeyCode::Down));
        assert_eq!(m.cmds.input.as_ref().unwrap().1, "pwd");
        on_key(&mut m, key(KeyCode::Esc));
        assert!(m.cmds.input.is_none());
    }

    #[test]
    fn destructive_needs_confirm() {
        let mut m = model();
        let idx = m
            .cmds
            .entries
            .iter()
            .position(|e| matches!(e, Entry::Cmd(i) if m.cmds.items[*i].destructive))
            .unwrap();
        m.cmds.sel = idx;
        let (c, _) = on_key(&mut m, key(KeyCode::Enter));
        assert!(c.is_empty());
        assert!(m.cmds.confirm.is_some());
        on_key(&mut m, key(KeyCode::Esc));
        assert!(m.cmds.confirm.is_none());
    }

    #[test]
    fn output_clears_spinner() {
        let mut m = model();
        m.cmds.running = Some("x".into());
        on_output(&mut m, "x", Ok("hi".into()));
        assert!(m.cmds.running.is_none());
        assert_eq!(m.cmds.result, "hi");
    }
}
