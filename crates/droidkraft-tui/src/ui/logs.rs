//! Logs panel: live logcat with level/tag/text filters, pause, save.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use tui_checkbox::Checkbox;
use tui_file_explorer::{ExplorerOutcome, FileExplorer, SelectionMode};

use super::common::{draw_explorer, hit, new_explorer};
use crate::logcat::{LogEntry, LogLevel, LogcatState, SaveFormat};
use crate::message::Cmd;
use crate::model::Model;

pub const HINTS: &str =
    "/ search  z fuzzy  f tag  l level  space pause  c clear  s save  m fmt  j/k PgUp/PgDn G";

/// Logs panel state.
pub struct LogsUi {
    pub state: LogcatState,
    pub fmt: SaveFormat,
    pub explorer: Option<FileExplorer>,
    /// Serial the stream was started for.
    pub serial: Option<String>,
    pub level_rects: Vec<(LogLevel, Rect)>,
}

impl Default for LogsUi {
    fn default() -> Self {
        Self {
            state: LogcatState::new(),
            fmt: SaveFormat::Text,
            explorer: None,
            serial: None,
            level_rects: vec![],
        }
    }
}

/// Called on every tick: (re)start the stream for the active device, ingest lines.
pub fn tick(m: &mut Model) {
    let serial = if m.active.is_some() {
        m.active_online_serial()
    } else {
        None
    };
    let l = &mut m.logs;
    match (&serial, &l.serial) {
        (Some(s), cur) if cur.as_ref() != Some(s) && m.panel == crate::model::Panel::Logs => {
            if l.serial.is_some() {
                l.state.stop_streaming();
            }
            l.state.start_streaming(s.clone());
            l.serial = Some(s.clone());
        }
        (None, Some(_)) => {
            l.state.stop_streaming();
            l.serial = None;
        }
        _ => {}
    }
    if l.state.is_streaming {
        l.state.poll_new_entries();
    }
}

fn rgb(c: droidkraft_core::Rgb) -> Color {
    let (r, g, b) = c.tuple();
    Color::Rgb(r, g, b)
}

/// Render one entry as a styled line.
pub fn entry_line(e: &LogEntry) -> Line<'static> {
    let lc = rgb(droidkraft_core::level_text_color(e.level));
    let mut spans = Vec::new();
    if let Some(t) = &e.timestamp {
        spans.push(Span::styled(
            format!("{t} "),
            Style::default().fg(Color::DarkGray),
        ));
    }
    spans.push(Span::styled(
        format!("{} ", e.level.as_char()),
        Style::default().fg(lc).add_modifier(Modifier::BOLD),
    ));
    if let Some(tag) = &e.tag {
        spans.push(Span::styled(
            format!("{tag}: "),
            Style::default().fg(rgb(droidkraft_core::tag_color(tag))),
        ));
    }
    spans.push(Span::styled(e.message.clone(), Style::default().fg(lc)));
    Line::from(spans)
}

fn input_span(label: &str, val: &str, active: bool, m: &Model) -> Vec<Span<'static>> {
    let st = if active {
        m.themes.selected()
    } else {
        m.themes.muted()
    };
    let cursor = if active { "▏" } else { "" };
    vec![
        Span::styled(format!(" {label}: "), m.themes.title()),
        Span::styled(format!("{val}{cursor}"), st),
    ]
}

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    let title = if m.logs.state.paused {
        "Logs (paused)"
    } else {
        "Logs"
    };
    let block = super::panel_block(m, title);
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.height < 4 {
        return;
    }
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(inner);

    // filter row
    let fl = &m.logs.state.filter;
    let mut spans = input_span(
        "search",
        &fl.search_query,
        fl.active_field == crate::logcat::FilterField::Search,
        m,
    );
    spans.extend(input_span(
        "tag",
        &fl.tag_filter,
        fl.active_field == crate::logcat::FilterField::Tag,
        m,
    ));
    spans.push(Span::styled(
        format!(
            "  mode:{} (z / ^F fuzzy · ^R regex)  fmt:{}",
            fl.mode_label(),
            m.logs.fmt.label()
        ),
        m.themes.muted(),
    ));
    f.render_widget(Paragraph::new(Line::from(spans)), rows[0]);

    // level checkboxes (checked = level passes min_level)
    m.logs.level_rects.clear();
    let min = m.logs.state.filter.min_level;
    let mut x = rows[1].x + 1;
    for lv in LogLevel::all() {
        let r = Rect::new(x, rows[1].y, 6.min(rows[1].right().saturating_sub(x)), 1);
        if r.width == 0 {
            break;
        }
        let on = lv.order() >= min.order();
        let c = rgb(droidkraft_core::level_text_color(*lv));
        f.render_widget(
            Checkbox::new(lv.as_char().to_string(), on)
                .checkbox_style(Style::default().fg(c))
                .label_style(Style::default().fg(c)),
            r,
        );
        m.logs.level_rects.push((*lv, r));
        x += 7;
    }

    // list
    let h = rows[2].height as usize;
    let idxs = m.logs.state.visible_entries(h);
    let lines: Vec<Line> = idxs
        .iter()
        .filter_map(|&i| m.logs.state.entries.get(i))
        .map(entry_line)
        .collect();
    if lines.is_empty() {
        let msg = if m.connected() {
            "Waiting for log lines…"
        } else {
            "No device connected."
        };
        f.render_widget(Paragraph::new(msg).style(m.themes.muted()), rows[2]);
    } else {
        f.render_widget(Paragraph::new(lines), rows[2]);
    }

    // stats
    let st = &m.logs.state.stats;
    let mut s = vec![Span::styled(
        format!(
            " {}/{} ",
            m.logs.state.entry_count(),
            m.logs.state.total_count()
        ),
        m.themes.muted(),
    )];
    for lv in LogLevel::all() {
        s.push(Span::styled(
            format!("{}:{} ", lv.as_char(), st.counts[lv.order() as usize]),
            Style::default().fg(rgb(droidkraft_core::level_text_color(*lv))),
        ));
    }
    s.push(Span::styled(
        format!("{:.0}/s ", st.lines_per_sec),
        m.themes.muted(),
    ));
    if !m.logs.state.auto_scroll {
        s.push(Span::styled("[scrolled — G to follow]", m.themes.muted()));
    }
    f.render_widget(Paragraph::new(Line::from(s)), rows[3]);

    if let Some(ex) = m.logs.explorer.as_mut() {
        draw_explorer(f, area, ex);
    }
}

fn refilter(m: &mut Model) {
    m.logs.state.rebuild_filtered();
}

fn save_to(m: &mut Model, sel: std::path::PathBuf) {
    let dir = if sel.is_dir() {
        sel
    } else {
        sel.parent().map(|p| p.to_path_buf()).unwrap_or_default()
    };
    let path = dir.join(LogcatState::default_save_filename(m.logs.fmt));
    let r = match m.logs.fmt {
        SaveFormat::Text => m.logs.state.save_filtered_to_file(&path),
        SaveFormat::Json => m.logs.state.save_filtered_to_json_file(&path),
    };
    match r {
        Ok(n) => m.set_message(format!("Saved {n} lines to {}", path.display())),
        Err(e) => m.set_message(format!("Save failed: {e}")),
    }
}

pub fn on_key(m: &mut Model, k: KeyEvent) -> (Vec<Cmd>, bool) {
    if let Some(ex) = m.logs.explorer.as_mut() {
        match ex.handle_key(k) {
            ExplorerOutcome::Selected(p) => {
                m.logs.explorer = None;
                save_to(m, p);
            }
            ExplorerOutcome::Dismissed => m.logs.explorer = None,
            _ => {}
        }
        return (vec![], true);
    }
    use crate::logcat::FilterField as F;
    if m.logs.state.filter.active_field != F::None {
        let f = &mut m.logs.state.filter;
        match k.code {
            KeyCode::Esc | KeyCode::Enter => f.active_field = F::None,
            KeyCode::Backspace => f.delete_char(),
            KeyCode::Delete => f.delete_char_forward(),
            KeyCode::Left => f.move_cursor_left(),
            KeyCode::Right => f.move_cursor_right(),
            KeyCode::Char('f') if k.modifiers.contains(KeyModifiers::CONTROL) => f.toggle_fuzzy(),
            KeyCode::Char('r') if k.modifiers.contains(KeyModifiers::CONTROL) => f.toggle_regex(),
            KeyCode::Char(c) if !k.modifiers.contains(KeyModifiers::CONTROL) => f.insert_char(c),
            _ => {}
        }
        refilter(m);
        return (vec![], true);
    }
    let h = m.logs.state.viewport_height.max(1);
    match k.code {
        KeyCode::Char('/') => m.logs.state.filter.active_field = F::Search,
        KeyCode::Char('z') => {
            m.logs.state.filter.toggle_fuzzy();
            refilter(m);
            let l = m.logs.state.filter.mode_label();
            m.set_message(format!("Search mode: {l}"));
        }
        KeyCode::Char('f') => m.logs.state.filter.active_field = F::Tag,
        KeyCode::Char('l') => {
            m.logs.state.filter.cycle_level();
            refilter(m);
        }
        KeyCode::Char(' ') => m.logs.state.toggle_pause(),
        KeyCode::Char('c') => m.logs.state.clear(),
        KeyCode::Char('m') => m.logs.fmt = m.logs.fmt.cycle(),
        KeyCode::Char('s') => m.logs.explorer = Some(new_explorer(&[], SelectionMode::Any)),
        KeyCode::Char('j') | KeyCode::Down => m.logs.state.scroll_down(1),
        KeyCode::Char('k') | KeyCode::Up => m.logs.state.scroll_up(1),
        KeyCode::PageUp => m.logs.state.scroll_up(h),
        KeyCode::PageDown => m.logs.state.scroll_down(h),
        KeyCode::Char('g') | KeyCode::Home => m.logs.state.scroll_to_top(),
        KeyCode::Char('G') | KeyCode::End => m.logs.state.scroll_to_bottom(),
        _ => return (vec![], false),
    }
    (vec![], true)
}

pub fn on_mouse(m: &mut Model, e: MouseEvent) -> Vec<Cmd> {
    if m.logs.explorer.is_some() {
        return vec![];
    }
    match e.kind {
        MouseEventKind::ScrollUp => m.logs.state.scroll_up(3),
        MouseEventKind::ScrollDown => m.logs.state.scroll_down(3),
        MouseEventKind::Down(MouseButton::Left) => {
            let hit_lv = m
                .logs
                .level_rects
                .iter()
                .find(|(_, r)| hit(*r, e.column, e.row))
                .map(|(l, _)| *l);
            if let Some(lv) = hit_lv {
                m.logs.state.filter.min_level = lv;
                refilter(m);
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
    use crossterm::event::KeyCode;

    fn feed(m: &mut Model, lines: &[&str]) {
        for l in lines {
            let e = LogEntry::parse(l);
            let i = m.logs.state.entries.len();
            m.logs.state.stats.record(&e.level);
            m.logs.state.entries.push(e);
            if m.logs.state.filter.matches(&m.logs.state.entries[i]) {
                m.logs.state.filtered_indices.push(i);
            }
        }
    }

    const L: [&str; 3] = [
        "01-15 12:34:56.789  100  200 I ActivityManager: started foo",
        "01-15 12:34:56.790  100  200 E AndroidRuntime: boom",
        "01-15 12:34:56.791  100  200 D Other: hello",
    ];

    #[test]
    fn renders_80x24_and_200x60() {
        for (w, h) in [(80, 24), (200, 60)] {
            let mut m = crate::ui::test_util::with_device(crate::model::Panel::Logs);
            feed(&mut m, &L);
            let out = render(&mut m, w, h);
            assert!(out.contains("Logs"));
            assert!(out.contains("boom"));
        }
    }

    #[test]
    fn text_filter_narrows() {
        let mut m = crate::ui::test_util::with_device(crate::model::Panel::Logs);
        feed(&mut m, &L);
        on_key(&mut m, key(KeyCode::Char('/')));
        for c in "boom".chars() {
            on_key(&mut m, key(KeyCode::Char(c)));
        }
        assert_eq!(m.logs.state.entry_count(), 1);
        on_key(&mut m, key(KeyCode::Esc));
        assert_eq!(
            m.logs.state.filter.active_field,
            crate::logcat::FilterField::None
        );
    }

    #[test]
    fn level_click_sets_min_level() {
        let mut m = crate::ui::test_util::with_device(crate::model::Panel::Logs);
        feed(&mut m, &L);
        render(&mut m, 120, 30);
        let (_, r) = *m
            .logs
            .level_rects
            .iter()
            .find(|(l, _)| *l == LogLevel::Error)
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
        assert_eq!(m.logs.state.entry_count(), 1);
    }

    #[test]
    fn pause_and_clear() {
        let mut m = crate::ui::test_util::with_device(crate::model::Panel::Logs);
        feed(&mut m, &L);
        on_key(&mut m, key(KeyCode::Char(' ')));
        assert!(m.logs.state.paused);
        on_key(&mut m, key(KeyCode::Char('c')));
        assert_eq!(m.logs.state.total_count(), 0);
    }
}
