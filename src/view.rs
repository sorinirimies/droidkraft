//! View layer — pure rendering from the current `Model`.
//!
//! Layout:
//!   ┌── Header ───────────────────────────────────────────────┐
//!   │ (3 rows) app title                                      │
//!   ├── Body ─────────────────────────────────────────────────┤
//!   │ Commands (58%)          │  Device panel (42%)           │
//!   │  • Section cards        │  ┌─ Devices ──────────────┐   │
//!   │  • Scrollable           │  │ device selector list   │   │
//!   │                         │  └────────────────────────┘   │
//!   │                         │  ┌─ Stats / No-Device ────┐   │
//!   │                         │  │ model · version ·      │   │
//!   │                         │  │ battery / RAM / CPU    │   │
//!   │                         │  └────────────────────────┘   │
//!   ├── Description ──────────────────────────────────────────┤
//!   │ (3 rows) selected command description                   │
//!   ├── Footer ───────────────────────────────────────────────┤
//!   │ (3 rows) key hint bar                                   │
//!   └─────────────────────────────────────────────────────────┘

use crate::adb::DeviceStatus;
use crate::effects::{get_loading_spinner, RevealWidget};
use crate::logcat::{tag_color, FilterField, LogcatState};
use crate::model::{AppState, Model};
use crate::theme::Theme;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

// ── Public entry point ────────────────────────────────────────────────────────

pub fn render(model: &mut Model, area: Rect, buf: &mut Buffer) {
    match model.state {
        AppState::Startup => render_startup(model, area, buf),
        AppState::Menu => render_menu(model, area, buf),
        AppState::Loading => render_loading(model, area, buf),
        AppState::ShowResult => render_result(model, area, buf),
        AppState::Logcat => render_logcat(model, area, buf),
    }

    // ── Global overlays ───────────────────────────────────────────────
    if model.theme_selector.open {
        render_theme_selector(model, area, buf);
    }
}

// ── Startup ───────────────────────────────────────────────────────────────────

fn render_startup(model: &mut Model, area: Rect, buf: &mut Buffer) {
    RevealWidget::new(&mut model.effects, "DroidTUI", "Android ADB & Root Toolkit")
        .render(area, buf);
}

// ── Loading ───────────────────────────────────────────────────────────────────

fn render_loading(model: &Model, area: Rect, buf: &mut Buffer) {
    let spinner = get_loading_spinner(model.loading_counter);
    let label = model.menu.get_selected_label();

    Paragraph::new(format!(
        "\n   {}  Executing\u{2026}\n\n   {}\n\n   Esc  \u{00b7}  cancel",
        spinner, label
    ))
    .block(
        Block::bordered()
            .title("  \u{26a1} Running  ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(190, 160, 30))),
    )
    .style(Style::default().fg(Color::Rgb(230, 205, 60)))
    .alignment(Alignment::Center)
    .render(centered_rect(44, 30, area), buf);
}

// ── Menu (full layout) ────────────────────────────────────────────────────────

fn render_menu(model: &mut Model, area: Rect, buf: &mut Buffer) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(0),    // body
            Constraint::Length(3), // description
            Constraint::Length(3), // footer
        ])
        .split(area);

    let header_area = outer[0];
    let body_area = outer[1];
    let desc_area = outer[2];
    let footer_area = outer[3];

    // ── Header ────────────────────────────────────────────────────────────────
    Paragraph::new("  Android ADB \u{26a1} Root Toolkit  ")
        .block(
            Block::bordered()
                .title("  \u{1f916}  DroidTUI  ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(40, 160, 40))),
        )
        .style(Style::default().fg(Color::Rgb(120, 200, 120)))
        .alignment(Alignment::Center)
        .render(header_area, buf);

    // ── Body: commands left | device panel right ──────────────────────────────
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(58), // command cards
            Constraint::Percentage(42), // device dashboard
        ])
        .split(body_area);

    let cmd_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(35, 70, 35)));
    let cmd_inner = cmd_block.inner(body[0]);
    cmd_block.render(body[0], buf);
    model.menu.render(cmd_inner, buf);

    render_device_panel(&model.device_status, body[1], buf);

    // ── Description ───────────────────────────────────────────────────────────
    Paragraph::new(format!("  {}", model.menu.get_selected_description()))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(50, 50, 70))),
        )
        .style(Style::default().fg(Color::Rgb(150, 150, 175)))
        .render(desc_area, buf);

    // ── Footer ────────────────────────────────────────────────────────────────
    Paragraph::new(
        "  \u{2191}/\u{2193} j/k Navigate  Tab/S-Tab Section  Enter Execute  L Logcat  T Theme  d Device  r Refresh  q Quit  ",
    )
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(50, 50, 70))),
    )
    .style(Style::default().fg(Color::Rgb(100, 100, 120)))
    .alignment(Alignment::Center)
    .render(footer_area, buf);
}

// ── Device panel (right column) ───────────────────────────────────────────────

fn render_device_panel(status: &DeviceStatus, area: Rect, buf: &mut Buffer) {
    // Device selector height: 2 borders + max(1, num_devices) content rows, capped at 6
    let list_rows = (status.devices.len().max(1) as u16 + 2).min(6);

    let panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(list_rows), // device selector
            Constraint::Min(0),            // live stats
        ])
        .split(area);

    render_device_list(status, panel[0], buf);
    render_device_stats(status, panel[1], buf);
}

// ── Device selector ───────────────────────────────────────────────────────────

fn render_device_list(status: &DeviceStatus, area: Rect, buf: &mut Buffer) {
    let connected = status.is_connected();
    let bdr_color = if connected {
        Color::Rgb(40, 120, 40)
    } else {
        Color::Rgb(60, 50, 35)
    };
    let multi_hint = if status.devices.len() > 1 {
        "  d \u{2013} cycle"
    } else {
        ""
    };

    let block = Block::bordered()
        .title(format!("  \u{1f4f1}  Devices{}  ", multi_hint))
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(bdr_color));

    let inner = block.inner(area);
    block.render(area, buf);

    if !connected {
        Paragraph::new(Line::from(vec![Span::styled(
            "  \u{25cb}  No device connected",
            Style::default().fg(Color::Rgb(160, 130, 60)),
        )]))
        .render(inner, buf);
        return;
    }

    let dim = Color::Rgb(100, 100, 100);
    let sel_bg = Color::Rgb(40, 140, 40);
    let sel_fg = Color::Rgb(10, 10, 10);

    let lines: Vec<Line<'static>> = status
        .devices
        .iter()
        .enumerate()
        .take(inner.height as usize)
        .map(|(idx, dev)| {
            let selected = idx == status.selected_idx;
            let serial = format!("{:<20}", dev.serial);
            let state = dev.state.clone();

            if selected {
                Line::from(vec![
                    Span::styled("  \u{25b6}  ", Style::default().fg(Color::Rgb(80, 220, 80))),
                    Span::styled(
                        serial,
                        Style::default()
                            .fg(sel_fg)
                            .bg(sel_bg)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {}", state),
                        Style::default().fg(sel_fg).bg(sel_bg),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled("     ", Style::default()),
                    Span::styled(serial, Style::default().fg(Color::Rgb(180, 180, 180))),
                    Span::styled(format!(" {}", state), Style::default().fg(dim)),
                ])
            }
        })
        .collect();

    Paragraph::new(lines).render(inner, buf);
}

// ── Device stats ──────────────────────────────────────────────────────────────

fn render_device_stats(status: &DeviceStatus, area: Rect, buf: &mut Buffer) {
    let connected = status.is_connected();
    let bdr_color = if connected {
        Color::Rgb(35, 70, 35)
    } else {
        Color::Rgb(55, 45, 35)
    };

    let title = status
        .active()
        .map(|d| format!("  {}  ", d.serial))
        .unwrap_or_else(|| "  No Device  ".to_string());

    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(bdr_color));

    let inner = block.inner(area);
    block.render(area, buf);

    if !connected {
        render_no_device_content(inner, buf);
        return;
    }

    let bar_w = (inner.width as usize).saturating_sub(22).clamp(4, 16);

    // Battery
    let batt_color = match status.battery_pct {
        0..=20 => Color::Rgb(220, 60, 60),
        21..=50 => Color::Rgb(220, 160, 50),
        _ => Color::Rgb(80, 200, 80),
    };

    // RAM
    let ram_used = status.ram_total_mib.saturating_sub(status.ram_avail_mib);
    let ram_pct = if status.ram_total_mib > 0 {
        ram_used as f32 / status.ram_total_mib as f32
    } else {
        0.0
    };
    let ram_color = match (ram_pct * 100.0) as u8 {
        0..=60 => Color::Rgb(80, 200, 80),
        61..=80 => Color::Rgb(220, 160, 50),
        _ => Color::Rgb(220, 60, 60),
    };

    // CPU — clamp load to 4.0 (4-core ceiling for bar display)
    let cpu_frac = (status.cpu_load_1min / 4.0).clamp(0.0, 1.0);
    let cpu_color = match (cpu_frac * 100.0) as u8 {
        0..=60 => Color::Rgb(80, 200, 80),
        61..=80 => Color::Rgb(220, 160, 50),
        _ => Color::Rgb(220, 60, 60),
    };

    let dim = Color::Rgb(90, 90, 90);
    let bright = Color::Rgb(205, 205, 205);

    let model_text = if status.model.is_empty() {
        "Unknown".to_string()
    } else {
        status.model.clone()
    };
    let ver_text = if status.android_version.is_empty() {
        String::new()
    } else {
        format!("Android {}", status.android_version)
    };

    let mut lines: Vec<Line<'static>> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  \u{1f4f1}  ", Style::default().fg(dim)),
            Span::styled(
                model_text,
                Style::default().fg(bright).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    if !ver_text.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  \u{1f916}  ", Style::default().fg(dim)),
            Span::styled(ver_text, Style::default().fg(bright)),
        ]));
    }

    lines.push(Line::from(""));

    // Battery bar
    lines.push(stat_bar(
        "  \u{1f50b}  Batt ",
        status.battery_pct as f32,
        100.0,
        bar_w,
        batt_color,
        format!("{}%", status.battery_pct),
    ));

    // RAM bar (only if data available)
    if status.ram_total_mib > 0 {
        lines.push(stat_bar(
            "  \u{1f4be}  RAM  ",
            ram_pct * 100.0,
            100.0,
            bar_w,
            ram_color,
            format!("{} / {}", fmt_mib(ram_used), fmt_mib(status.ram_total_mib)),
        ));
    }

    // CPU load bar
    if status.cpu_load_1min > 0.0 {
        lines.push(stat_bar(
            "  \u{1f4ca}  CPU  ",
            cpu_frac * 100.0,
            100.0,
            bar_w,
            cpu_color,
            format!("{:.2}", status.cpu_load_1min),
        ));
    }

    Paragraph::new(lines).render(inner, buf);
}

// ── No-device content ─────────────────────────────────────────────────────────

fn render_no_device_content(area: Rect, buf: &mut Buffer) {
    let dim = Color::Rgb(90, 90, 90);
    let amber = Color::Rgb(160, 130, 60);
    let subtle = Color::Rgb(80, 80, 80);

    let lines: Vec<Line<'static>> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  \u{26aa}  No device connected",
            Style::default().fg(amber),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  \u{2022}  Enable USB debugging on the device",
            Style::default().fg(dim),
        )),
        Line::from(Span::styled(
            "  \u{2022}  Run  adb start-server",
            Style::default().fg(dim),
        )),
        Line::from(Span::styled(
            "  \u{2022}  Connect via USB or WiFi",
            Style::default().fg(dim),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Press  r  to retry",
            Style::default().fg(subtle),
        )),
    ];

    Paragraph::new(lines).render(area, buf);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a single progress-bar stat line.
fn stat_bar(
    label: &str,
    value: f32,
    max: f32,
    bar_w: usize,
    color: Color,
    suffix: String,
) -> Line<'static> {
    let pct = (value / max).clamp(0.0, 1.0);
    let filled = (pct * bar_w as f32) as usize;
    let empty = bar_w.saturating_sub(filled);

    Line::from(vec![
        Span::styled(
            label.to_string(),
            Style::default().fg(Color::Rgb(90, 90, 90)),
        ),
        Span::styled("\u{2588}".repeat(filled), Style::default().fg(color)),
        Span::styled(
            "\u{2591}".repeat(empty),
            Style::default().fg(Color::Rgb(40, 40, 40)),
        ),
        Span::styled(format!("  {}", suffix), Style::default().fg(color)),
    ])
}

/// Format MiB as human-readable (GiB when >= 1 GiB, else MiB).
fn fmt_mib(mib: u64) -> String {
    if mib >= 1024 {
        format!("{:.1}G", mib as f64 / 1024.0)
    } else {
        format!("{}M", mib)
    }
}

// ── Result view (unchanged) ───────────────────────────────────────────────────

fn render_result(model: &mut Model, area: Rect, buf: &mut Buffer) {
    let (icon, border_color) = if model.command_result.is_some() {
        ("\u{2705}", Color::Rgb(40, 160, 40))
    } else {
        ("\u{274c}", Color::Rgb(160, 40, 40))
    };

    let label = model
        .last_command_label
        .clone()
        .unwrap_or_else(|| "Result".to_string());

    let slide_progress = model.effects.get_slide_in_progress();
    let base_area = centered_rect(82, 76, area);
    let slide_offset = ((1.0 - slide_progress) * base_area.height as f32 * 0.4) as u16;

    let popup_area = if slide_offset > 0 {
        Rect {
            x: base_area.x,
            y: base_area.y + slide_offset,
            width: base_area.width,
            height: base_area.height.saturating_sub(slide_offset),
        }
    } else {
        base_area
    };

    if popup_area.height < 5 {
        return;
    }

    let content_area = Rect {
        width: popup_area.width.saturating_sub(3),
        ..popup_area
    };
    let scrollbar_area = Rect {
        x: popup_area.x + popup_area.width.saturating_sub(3),
        width: 3,
        ..popup_area
    };

    let content_height = content_area.height.saturating_sub(4) as usize;
    let max_width = content_area.width.saturating_sub(4) as usize;
    model.update_wrapped_lines(max_width);

    let total = model.wrapped_lines.len();
    let start = model.scroll_position;
    let end = (start + content_height).min(total);
    let visible = model.wrapped_lines[start..end].to_vec();

    let scroll_hint = if total > content_height {
        format!(
            "  [{}/{}]  \u{2191}\u{2193} Scroll  PgUp/PgDn Fast  Home/End Jump",
            start + 1,
            total
        )
    } else {
        "  Esc \u{00b7} q \u{00b7} Enter  return to menu".to_string()
    };

    let title = format!("  {} {}{}  ", icon, label.as_str(), scroll_hint);

    let mut display = visible.join("\n");
    if total > content_height {
        let pad = "\n".repeat(content_height.saturating_sub(visible.len()).max(1));
        display.push_str(&pad);
        if start > 0 {
            display.push_str("\u{25b2} more above");
        }
        if end < total {
            if start > 0 {
                display.push_str("  \u{00b7}  ");
            }
            display.push_str("\u{25bc} more below");
        }
    } else if !visible.is_empty() {
        let pad = "\n".repeat(content_height.saturating_sub(visible.len()).max(1));
        display.push_str(&pad);
        display.push_str("  Esc \u{00b7} q \u{00b7} Enter  return to menu");
    }

    Paragraph::new(display)
        .block(
            Block::bordered()
                .title(title)
                .title_alignment(Alignment::Left)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color)),
        )
        .style(Style::default().fg(Color::Rgb(220, 220, 220)))
        .render(content_area, buf);

    if total > content_height {
        render_scrollbar(
            scrollbar_area,
            buf,
            total,
            content_height,
            start,
            border_color,
        );
    }
}

// ── Scrollbar ─────────────────────────────────────────────────────────────────

fn render_scrollbar(
    area: Rect,
    buf: &mut Buffer,
    total_lines: usize,
    visible: usize,
    pos: usize,
    color: Color,
) {
    if area.height < 3 {
        return;
    }

    let scroll_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(color));
    let inner = scroll_block.inner(area);
    scroll_block.render(area, buf);

    let h = inner.height as usize;
    if h == 0 {
        return;
    }

    let thumb_size = ((visible as f64 / total_lines as f64) * h as f64).max(1.0) as usize;
    let thumb_pos = if total_lines > visible {
        ((pos as f64 / (total_lines - visible) as f64) * (h - thumb_size) as f64) as usize
    } else {
        0
    };

    for y in 0..h {
        if let Some(cell) = buf.cell_mut((inner.x, inner.y + y as u16)) {
            if y >= thumb_pos && y < thumb_pos + thumb_size {
                cell.set_char('\u{2588}');
                cell.set_fg(color);
            } else {
                cell.set_char('\u{2591}');
                cell.set_fg(Color::DarkGray);
            }
        }
    }
}

// ── centered_rect ─────────────────────────────────────────────────────────────

// ── Logcat view ───────────────────────────────────────────────────────────────

fn render_logcat(model: &mut Model, area: Rect, buf: &mut Buffer) {
    let theme = model.theme.clone();
    let state = &mut model.logcat;

    // ── Layout ────────────────────────────────────────────────────────────
    //  ┌─ Filter bar ──────────────────────────────────────────────────────┐
    //  │ 🔍 search… │ 🏷 tag… │ 📦 pkg… │ Level: I+ │ ⏸ Paused │ 1234  │
    //  ├─ Log lines ───────────────────────────────────────────────────────┤
    //  │ 03-25 12:00:00.000  1234  5678 I MyTag   : Hello world          │
    //  │ …                                                                │
    //  ├─ Footer / help ───────────────────────────────────────────────────┤
    //  │ /Search  tTag  pPkg  lLevel  Space Pause  cClear  q Close       │
    //  └──────────────────────────────────────────────────────────────────┘
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // filter bar
            Constraint::Min(1),    // log lines
            Constraint::Length(3), // footer
        ])
        .split(area);

    let filter_area = outer[0];
    let log_area = outer[1];
    let footer_area = outer[2];

    render_logcat_filter_bar(state, filter_area, buf);
    render_logcat_lines(state, log_area, buf);
    render_logcat_footer(state, &theme, footer_area, buf);

    // ── Save dialog overlay ───────────────────────────────────────────────
    if model.logcat_save_active {
        render_logcat_save_dialog(model, area, buf);
    }

    // ── Line detail popup ─────────────────────────────────────────────────
    if model.logcat.detail_open {
        render_logcat_detail(&model.logcat, area, buf);
    }
}

// ── Logcat filter bar ─────────────────────────────────────────────────────────

fn render_logcat_filter_bar(state: &LogcatState, area: Rect, buf: &mut Buffer) {
    let _editing = state.filter.active_field != FilterField::None;

    // Split into segments: search | tag | package | level | status | counts
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(22), // search
            Constraint::Percentage(16), // tag
            Constraint::Percentage(14), // package
            Constraint::Percentage(16), // exclude
            Constraint::Length(12),     // level
            Constraint::Min(10),        // status + stats
        ])
        .split(area);

    // ── Search field ──────────────────────────────────────────────────────
    let search_active = state.filter.active_field == FilterField::Search;
    let search_border = if search_active {
        Color::Rgb(80, 200, 255)
    } else if !state.filter.search_query.is_empty() {
        Color::Rgb(60, 140, 60)
    } else {
        Color::Rgb(50, 50, 70)
    };
    let search_text = if state.filter.search_query.is_empty() && !search_active {
        "  f find…".to_string()
    } else {
        let cursor = if search_active {
            let pos = state.filter.search_cursor;
            let (before, after) = state.filter.search_query.split_at(
                state
                    .filter
                    .search_query
                    .char_indices()
                    .nth(pos)
                    .map(|(i, _)| i)
                    .unwrap_or(state.filter.search_query.len()),
            );
            format!("  {}▏{}", before, after)
        } else {
            format!("  {}", state.filter.search_query)
        };
        cursor
    };
    let search_title = if state.filter.use_regex {
        "  \u{1f50d}  Regex Find  "
    } else {
        "  \u{1f50d}  Find  "
    };
    Paragraph::new(search_text)
        .block(
            Block::bordered()
                .title(search_title)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(search_border)),
        )
        .style(Style::default().fg(if search_active {
            Color::White
        } else {
            Color::Rgb(180, 180, 180)
        }))
        .render(cols[0], buf);

    // ── Tag filter ────────────────────────────────────────────────────────
    let tag_active = state.filter.active_field == FilterField::Tag;
    let tag_border = if tag_active {
        Color::Rgb(80, 200, 255)
    } else if !state.filter.tag_filter.is_empty() {
        Color::Rgb(60, 140, 60)
    } else {
        Color::Rgb(50, 50, 70)
    };
    let tag_text = if state.filter.tag_filter.is_empty() && !tag_active {
        "  t tag…".to_string()
    } else {
        let cursor = if tag_active {
            let pos = state.filter.tag_cursor;
            let (before, after) = state.filter.tag_filter.split_at(
                state
                    .filter
                    .tag_filter
                    .char_indices()
                    .nth(pos)
                    .map(|(i, _)| i)
                    .unwrap_or(state.filter.tag_filter.len()),
            );
            format!("  {}▏{}", before, after)
        } else {
            format!("  {}", state.filter.tag_filter)
        };
        cursor
    };
    Paragraph::new(tag_text)
        .block(
            Block::bordered()
                .title("  \u{1f3f7}  Tag  ")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(tag_border)),
        )
        .style(Style::default().fg(if tag_active {
            Color::White
        } else {
            Color::Rgb(180, 180, 180)
        }))
        .render(cols[1], buf);

    // ── Package filter ────────────────────────────────────────────────────
    let pkg_active = state.filter.active_field == FilterField::Package;
    let pkg_border = if pkg_active {
        Color::Rgb(80, 200, 255)
    } else if !state.filter.package_filter.is_empty() {
        Color::Rgb(60, 140, 60)
    } else {
        Color::Rgb(50, 50, 70)
    };
    let pkg_text = if state.filter.package_filter.is_empty() && !pkg_active {
        "  p pid…".to_string()
    } else {
        let cursor = if pkg_active {
            let pos = state.filter.package_cursor;
            let (before, after) = state.filter.package_filter.split_at(
                state
                    .filter
                    .package_filter
                    .char_indices()
                    .nth(pos)
                    .map(|(i, _)| i)
                    .unwrap_or(state.filter.package_filter.len()),
            );
            format!("  {}▏{}", before, after)
        } else {
            format!("  {}", state.filter.package_filter)
        };
        cursor
    };
    Paragraph::new(pkg_text)
        .block(
            Block::bordered()
                .title("  \u{1f4e6}  PID  ")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(pkg_border)),
        )
        .style(Style::default().fg(if pkg_active {
            Color::White
        } else {
            Color::Rgb(180, 180, 180)
        }))
        .render(cols[2], buf);

    // ── Exclude filter ────────────────────────────────────────────────
    let excl_active = state.filter.active_field == FilterField::Exclude;
    let excl_border = if excl_active {
        Color::Rgb(255, 100, 80)
    } else if !state.filter.exclude_query.is_empty() {
        Color::Rgb(160, 60, 60)
    } else {
        Color::Rgb(50, 50, 70)
    };
    let excl_text = if state.filter.exclude_query.is_empty() && !excl_active {
        "  e exclude…".to_string()
    } else if excl_active {
        let pos = state.filter.exclude_cursor;
        let (before, after) = state.filter.exclude_query.split_at(
            state
                .filter
                .exclude_query
                .char_indices()
                .nth(pos)
                .map(|(i, _)| i)
                .unwrap_or(state.filter.exclude_query.len()),
        );
        format!("  {}▏{}", before, after)
    } else {
        format!("  {}", state.filter.exclude_query)
    };
    Paragraph::new(excl_text)
        .block(
            Block::bordered()
                .title("  ✕  Exclude  ")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(excl_border)),
        )
        .style(Style::default().fg(if excl_active {
            Color::White
        } else {
            Color::Rgb(180, 130, 130)
        }))
        .render(cols[3], buf);

    // ── Level badge ───────────────────────────────────────────────────────
    let lvl = &state.filter.min_level;
    let lvl_char = lvl.as_char();
    let lvl_color = lvl.label_color();
    let lvl_label = format!("  {}+", lvl_char);
    Paragraph::new(lvl_label)
        .block(
            Block::bordered()
                .title("  Lvl  ")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(50, 50, 70))),
        )
        .style(Style::default().fg(lvl_color).add_modifier(Modifier::BOLD))
        .render(cols[4], buf);

    // ── Status / counts ───────────────────────────────────────────────────
    let pause_icon = if state.paused { "⏸ " } else { "▶ " };
    let stream_color = if state.paused {
        Color::Rgb(200, 160, 50)
    } else if state.is_streaming {
        Color::Rgb(80, 200, 80)
    } else {
        Color::Rgb(120, 120, 120)
    };

    let rate = state.stats.lines_per_sec;
    let rate_str = if rate > 0.0 {
        format!("{:.0}/s", rate)
    } else {
        String::new()
    };
    let status_text = format!(
        " {} {}/{} {}",
        pause_icon,
        state.entry_count(),
        state.total_count(),
        rate_str,
    );

    Paragraph::new(status_text)
        .block(
            Block::bordered()
                .title(if state.is_streaming {
                    "  \u{1f4e1}  Live  "
                } else {
                    "  \u{25cb}  Idle  "
                })
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(50, 50, 70))),
        )
        .style(Style::default().fg(stream_color))
        .render(cols[5], buf);
}

// ── Logcat log lines ──────────────────────────────────────────────────────────

fn render_logcat_lines(state: &mut LogcatState, area: Rect, buf: &mut Buffer) {
    let border_color = if state.is_streaming && !state.paused {
        Color::Rgb(35, 70, 35)
    } else if state.paused {
        Color::Rgb(70, 60, 25)
    } else {
        Color::Rgb(50, 50, 50)
    };

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    block.render(area, buf);

    let visible_height = inner.height as usize;
    if visible_height == 0 {
        return;
    }

    // If there are no entries, show a placeholder
    if state.entries.is_empty() {
        let msg = state
            .status_message
            .clone()
            .unwrap_or_else(|| "Waiting for logcat output…".to_string());
        Paragraph::new(format!("\n  {}", msg))
            .style(Style::default().fg(Color::Rgb(120, 120, 120)))
            .render(inner, buf);
        return;
    }

    if state.filtered_indices.is_empty() {
        Paragraph::new("\n  No entries match current filters.")
            .style(Style::default().fg(Color::Rgb(160, 130, 60)))
            .render(inner, buf);
        return;
    }

    let indices = state.visible_entries(visible_height);
    let max_width = inner.width as usize;

    let search_lower = state.filter.search_query.to_lowercase();
    let has_search = !search_lower.is_empty();

    for (row, &idx) in indices.iter().enumerate() {
        let y = inner.y + row as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let entry = &state.entries[idx];
        let level_color = entry.level.color();
        let level_label_color = entry.level.label_color();

        // Selected line highlight
        let is_selected = {
            let filtered_pos = if state.auto_scroll {
                let total = state.filtered_indices.len();
                let start = total.saturating_sub(visible_height);
                start + row
            } else {
                state.scroll_position.min(
                    state
                        .filtered_indices
                        .len()
                        .saturating_sub(visible_height)
                        .max(0),
                ) + row
            };
            filtered_pos == state.selected_line
        };
        if is_selected {
            for x in inner.x..inner.x + inner.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(Color::Rgb(30, 40, 50));
                }
            }
        }

        // Build spans for this line
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(8);
        let mut used_width: usize = 0;

        // Bookmark indicator
        let is_bm = state.is_bookmarked(idx);
        if is_bm {
            if let Some(cell) = buf.cell_mut((inner.x, y)) {
                cell.set_char('●');
                cell.set_fg(Color::Rgb(255, 200, 50));
            }
        }

        if !state.compact {
            // Timestamp (dimmed)
            if let Some(ref ts) = entry.timestamp {
                let ts_display: &str = if ts.len() > 18 { &ts[..18] } else { ts };
                let ts_str = format!(" {} ", ts_display);
                used_width += ts_str.len();
                spans.push(Span::styled(
                    ts_str,
                    Style::default().fg(Color::Rgb(100, 100, 100)),
                ));
            }

            // PID/TID (dimmed)
            if let Some(ref pid) = entry.pid {
                let pid_str = format!("{:>5}", pid);
                used_width += pid_str.len() + 1;
                spans.push(Span::styled(
                    format!("{} ", pid_str),
                    Style::default().fg(Color::Rgb(90, 90, 90)),
                ));
            }
        }

        // Fold indicator
        let is_fold_head = !entry.is_stack_continuation && {
            let next_idx = idx + 1;
            next_idx < state.entries.len() && state.entries[next_idx].is_stack_continuation
        };
        if is_fold_head {
            let is_folded = state.folded_groups.contains(&idx);
            let fold_char = if is_folded { "▶ " } else { "▼ " };
            spans.insert(
                0,
                Span::styled(
                    fold_char.to_string(),
                    Style::default().fg(Color::Rgb(120, 120, 140)),
                ),
            );
            used_width += 2;
        }

        // Level badge (colored, bold)
        let level_badge = format!(" {} ", entry.level.as_char());
        used_width += level_badge.len() + 1;
        spans.push(Span::styled(
            level_badge,
            Style::default()
                .fg(Color::Rgb(15, 15, 15))
                .bg(level_label_color)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));

        // Tag (colored by tag hash)
        if let Some(ref tag) = entry.tag {
            let tag_display: &str = if tag.len() > 20 { &tag[..20] } else { tag };
            let tag_str = format!("{:<20} ", tag_display);
            used_width += tag_str.len();
            spans.push(Span::styled(
                tag_str,
                Style::default()
                    .fg(tag_color(tag))
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Horizontal scroll
        let msg_to_display = if state.h_scroll > 0 && !state.word_wrap {
            let char_count = entry.message.chars().count();
            if state.h_scroll < char_count {
                entry
                    .message
                    .chars()
                    .skip(state.h_scroll)
                    .collect::<String>()
            } else {
                String::new()
            }
        } else {
            entry.message.clone()
        };

        // Message (truncated to fit, with search highlight)
        let remaining = max_width.saturating_sub(used_width);
        let msg = if msg_to_display.len() > remaining {
            format!("{}…", &msg_to_display[..remaining.saturating_sub(1)])
        } else {
            msg_to_display
        };

        if has_search {
            // Highlight search matches in the message
            let msg_lower = msg.to_lowercase();
            let mut last_end = 0;
            let mut search_start = 0;
            while let Some(pos) = msg_lower[search_start..].find(&search_lower) {
                let abs_pos = search_start + pos;
                // Text before match
                if abs_pos > last_end {
                    spans.push(Span::styled(
                        msg[last_end..abs_pos].to_string(),
                        Style::default().fg(level_color),
                    ));
                }
                // Highlighted match
                let match_end = abs_pos + search_lower.len();
                spans.push(Span::styled(
                    msg[abs_pos..match_end].to_string(),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Rgb(255, 200, 50))
                        .add_modifier(Modifier::BOLD),
                ));
                last_end = match_end;
                search_start = match_end;
            }
            // Remaining text after last match
            if last_end < msg.len() {
                spans.push(Span::styled(
                    msg[last_end..].to_string(),
                    Style::default().fg(level_color),
                ));
            }
        } else {
            spans.push(Span::styled(msg, Style::default().fg(level_color)));
        }

        // Render the line
        let line = Line::from(spans);
        let line_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: 1,
        };
        Paragraph::new(line).render(line_area, buf);
    }

    // ── Scrollbar ─────────────────────────────────────────────────────────
    let total = state.filtered_indices.len();
    if total > visible_height {
        let sb_x = inner.x + inner.width.saturating_sub(1);
        let h = inner.height as usize;
        let thumb_size = ((visible_height as f64 / total as f64) * h as f64).max(1.0) as usize;
        let max_scroll = total.saturating_sub(visible_height);
        let thumb_pos = if max_scroll > 0 {
            ((state.scroll_position as f64 / max_scroll as f64) * (h - thumb_size) as f64) as usize
        } else {
            0
        };

        for row in 0..h {
            if let Some(cell) = buf.cell_mut((sb_x, inner.y + row as u16)) {
                if row >= thumb_pos && row < thumb_pos + thumb_size {
                    cell.set_char('\u{2588}');
                    cell.set_fg(Color::Rgb(80, 160, 80));
                } else {
                    cell.set_char('\u{2591}');
                    cell.set_fg(Color::Rgb(35, 35, 35));
                }
            }
        }
    }

    // Auto-scroll indicator
    if state.auto_scroll && total > visible_height {
        let indicator = " ↓ AUTO ";
        let x = inner.x + inner.width.saturating_sub(indicator.len() as u16 + 2);
        let y = inner.y + inner.height.saturating_sub(1);
        for (i, ch) in indicator.chars().enumerate() {
            if let Some(cell) = buf.cell_mut((x + i as u16, y)) {
                cell.set_char(ch);
                cell.set_fg(Color::Rgb(80, 200, 80));
                cell.set_bg(Color::Rgb(20, 40, 20));
            }
        }
    }
}

// ── Logcat footer ─────────────────────────────────────────────────────────────

fn render_logcat_footer(state: &LogcatState, theme: &Theme, area: Rect, buf: &mut Buffer) {
    let editing = state.filter.active_field != FilterField::None;

    if editing {
        // Simple editing hint bar
        let content = Line::from(vec![
            Span::styled("  Type to filter  ", Style::default().fg(theme.fg)),
            Span::styled("·  ", Style::default().fg(theme.border)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(theme.key_hint)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("/", Style::default().fg(theme.dim)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.key_hint)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" confirm", Style::default().fg(theme.dim)),
        ]);
        Paragraph::new(content)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border)),
            )
            .alignment(Alignment::Center)
            .render(area, buf);
        return;
    }

    // Split footer into two bordered sections like tui-file-explorer
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(62), // Navigate + Filters
            Constraint::Percentage(38), // Actions
        ])
        .split(area);

    // Helper to build a key hint span pair: highlighted key + dimmed label
    fn kh<'a>(key: &'a str, label: &'a str, key_color: Color, dim_color: Color) -> Vec<Span<'a>> {
        vec![
            Span::styled(
                key,
                Style::default().fg(key_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {}  ", label), Style::default().fg(dim_color)),
        ]
    }

    // ── Left group: Navigate + Filters ────────────────────────────────────
    let mut left_spans: Vec<Span<'static>> = Vec::new();
    left_spans.push(Span::raw(" "));
    // Navigation
    left_spans.extend(
        kh("↑/k", "up", theme.key_hint, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    left_spans.extend(
        kh("↓/j", "down", theme.key_hint, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    left_spans.push(Span::styled("│ ", Style::default().fg(theme.border)));
    // Filters
    left_spans.extend(
        kh("f", "find", theme.key_hint, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    left_spans.extend(
        kh("e", "excl", theme.key_hint, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    left_spans.extend(
        kh("t", "tag", theme.key_hint, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    left_spans.extend(
        kh("p", "pid", theme.key_hint, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    left_spans.extend(
        kh("l", "level", theme.key_hint, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    left_spans.extend(
        kh("r", "regex", theme.key_hint, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );

    let left_line = Line::from(left_spans);
    Paragraph::new(left_line)
        .block(
            Block::bordered()
                .title(" Navigate ")
                .title_style(Style::default().fg(theme.dim))
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border)),
        )
        .render(cols[0], buf);

    // ── Right group: Actions ──────────────────────────────────────────────
    let mut right_spans: Vec<Span<'static>> = Vec::new();
    right_spans.push(Span::raw(" "));
    right_spans.extend(
        kh("y", "copy", theme.accent, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    right_spans.extend(
        kh("s", "save", theme.accent, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    right_spans.extend(
        kh("m", "mark", theme.accent, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    right_spans.extend(
        kh("F", "fold", theme.accent, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    right_spans.extend(
        kh("x", "cmpct", theme.accent, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );
    right_spans.extend(
        kh("Esc", "close", theme.error, theme.dim)
            .into_iter()
            .map(|s| Span::styled(s.content.to_string(), s.style)),
    );

    let right_line = Line::from(right_spans);
    Paragraph::new(right_line)
        .block(
            Block::bordered()
                .title(" Actions ")
                .title_style(Style::default().fg(theme.dim))
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border)),
        )
        .render(cols[1], buf);
}

// ── Logcat line detail popup ──────────────────────────────────────────────────

fn render_logcat_detail(state: &LogcatState, area: Rect, buf: &mut Buffer) {
    let popup = centered_rect(80, 60, area);

    // Solid background
    let bg = Color::Rgb(20, 22, 28);
    for y in popup.top()..popup.bottom() {
        for x in popup.left()..popup.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_bg(bg);
            }
        }
    }

    let entry = match state.selected_entry() {
        Some(e) => e,
        None => {
            Paragraph::new("  No line selected")
                .style(Style::default().fg(Color::Rgb(120, 120, 120)).bg(bg))
                .render(popup, buf);
            return;
        }
    };

    let level_color = entry.level.label_color();
    let mut lines: Vec<Line<'static>> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Level: ", Style::default().fg(Color::Rgb(100, 100, 120))),
            Span::styled(
                format!("{}", entry.level.as_char()),
                Style::default()
                    .fg(level_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    if let Some(ref ts) = entry.timestamp {
        lines.push(Line::from(vec![
            Span::styled("  Time:  ", Style::default().fg(Color::Rgb(100, 100, 120))),
            Span::styled(ts.clone(), Style::default().fg(Color::Rgb(200, 200, 200))),
        ]));
    }
    if let Some(ref tag) = entry.tag {
        lines.push(Line::from(vec![
            Span::styled("  Tag:   ", Style::default().fg(Color::Rgb(100, 100, 120))),
            Span::styled(tag.clone(), Style::default().fg(tag_color(tag))),
        ]));
    }
    if let Some(ref pid) = entry.pid {
        lines.push(Line::from(vec![
            Span::styled("  PID:   ", Style::default().fg(Color::Rgb(100, 100, 120))),
            Span::styled(pid.clone(), Style::default().fg(Color::Rgb(200, 200, 200))),
        ]));
    }
    if let Some(ref tid) = entry.tid {
        lines.push(Line::from(vec![
            Span::styled("  TID:   ", Style::default().fg(Color::Rgb(100, 100, 120))),
            Span::styled(tid.clone(), Style::default().fg(Color::Rgb(200, 200, 200))),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Message:",
        Style::default().fg(Color::Rgb(100, 100, 120)),
    )));

    // Word-wrap the message to fit popup width
    let max_w = popup.width.saturating_sub(4) as usize;
    let msg = &entry.message;
    for chunk_start in (0..msg.len()).step_by(max_w.max(1)) {
        let chunk_end = (chunk_start + max_w).min(msg.len());
        let chunk = &msg[chunk_start..chunk_end];
        lines.push(Line::from(Span::styled(
            format!("  {}", chunk),
            Style::default().fg(entry.level.color()),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  y ",
            Style::default()
                .fg(Color::Rgb(80, 200, 80))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("copy  ", Style::default().fg(Color::Rgb(90, 90, 100))),
        Span::styled(
            "m ",
            Style::default()
                .fg(Color::Rgb(255, 200, 50))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("bookmark  ", Style::default().fg(Color::Rgb(90, 90, 100))),
        Span::styled(
            "Esc/Enter ",
            Style::default()
                .fg(Color::Rgb(200, 100, 80))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("close", Style::default().fg(Color::Rgb(90, 90, 100))),
    ]));

    let title = "  \u{1f4cb}  Line Detail  ".to_string();
    Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(title)
                .title_alignment(Alignment::Left)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(level_color)),
        )
        .style(Style::default().bg(bg))
        .render(popup, buf);
}

// ── Logcat save dialog ────────────────────────────────────────────────────────

fn render_logcat_save_dialog(model: &Model, area: Rect, buf: &mut Buffer) {
    use crate::model::LogcatSaveMode;

    // ── Solid opaque background ───────────────────────────────────────────
    let bg = Color::Rgb(18, 18, 24);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_bg(bg);
                cell.set_fg(Color::Rgb(50, 50, 60));
            }
        }
    }

    match model.logcat_save_mode {
        LogcatSaveMode::PathInput => render_save_path_input(model, area, buf, bg),
        LogcatSaveMode::FileBrowser => render_save_file_browser(model, area, buf, bg),
    }
}

/// Path-input sub-dialog for saving logs.
fn render_save_path_input(model: &Model, area: Rect, buf: &mut Buffer, bg: Color) {
    let popup = centered_rect(65, 40, area);
    let popup = if popup.height < 9 {
        centered_rect(90, 70, area)
    } else {
        popup
    };

    let kind = if model.logcat_save_filtered_only {
        "filtered"
    } else {
        "all"
    };
    let count = if model.logcat_save_filtered_only {
        model.logcat.entry_count()
    } else {
        model.logcat.total_count()
    };

    let title = format!("  \u{1f4be}  Save {} entries ({})  ", count, kind);

    // Build the path input with cursor
    let path = &model.logcat_save_path;
    let cursor_pos = model.logcat_save_cursor;
    let byte_idx = path
        .char_indices()
        .nth(cursor_pos)
        .map(|(i, _)| i)
        .unwrap_or(path.len());
    let (before, after) = path.split_at(byte_idx);
    let path_display = format!("{}▏{}", before, after);

    let toggle_hint = if model.logcat_save_filtered_only {
        "Tab \u{2192} save all"
    } else {
        "Tab \u{2192} save filtered only"
    };

    let lines: Vec<Line<'static>> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Path: ", Style::default().fg(Color::Rgb(140, 140, 160))),
            Span::styled(
                path_display,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", toggle_hint),
            Style::default().fg(Color::Rgb(120, 120, 140)),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Enter ",
                Style::default()
                    .fg(Color::Rgb(80, 200, 80))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "save  \u{00b7}  ",
                Style::default().fg(Color::Rgb(90, 90, 100)),
            ),
            Span::styled(
                "F2 ",
                Style::default()
                    .fg(Color::Rgb(80, 200, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Save As\u{2026}  \u{00b7}  ",
                Style::default().fg(Color::Rgb(90, 90, 100)),
            ),
            Span::styled(
                "Esc ",
                Style::default()
                    .fg(Color::Rgb(200, 100, 80))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("cancel", Style::default().fg(Color::Rgb(90, 90, 100))),
        ]),
    ];

    Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(title)
                .title_alignment(Alignment::Left)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(80, 200, 255))),
        )
        .style(Style::default().fg(Color::Rgb(220, 220, 220)).bg(bg))
        .render(popup, buf);
}

/// File-explorer sub-dialog for "Save As…" browsing.
fn render_save_file_browser(model: &Model, area: Rect, buf: &mut Buffer, bg: Color) {
    let popup = centered_rect(80, 80, area);
    let popup = if popup.height < 10 { area } else { popup };

    let explorer = match &model.logcat_file_explorer {
        Some(e) => e,
        None => return,
    };

    // Outer block
    let title = format!(
        "  \u{1f4c2}  Save As\u{2026}  \u{2014}  {}  ",
        explorer.current_dir.display()
    );
    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(80, 200, 255)));
    let inner = block.inner(popup);
    block.render(popup, buf);

    // Fill inner bg
    for y in inner.top()..inner.bottom() {
        for x in inner.left()..inner.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_bg(bg);
            }
        }
    }

    if inner.height < 3 {
        return;
    }

    // Reserve last 2 rows for footer hints
    let list_area = Rect {
        height: inner.height.saturating_sub(2),
        ..inner
    };
    let footer_area = Rect {
        x: inner.x,
        y: inner.y + list_area.height,
        width: inner.width,
        height: 2.min(inner.height),
    };

    // ── Render entries ────────────────────────────────────────────────────
    let visible_height = list_area.height as usize;
    let total = explorer.entries.len();

    // Compute scroll so cursor is always visible.
    // scroll_offset is private, so we derive it from cursor position.
    let scroll = if explorer.cursor >= visible_height {
        explorer.cursor.saturating_sub(visible_height - 1)
    } else {
        0
    };

    if total == 0 {
        Paragraph::new("  (empty directory)")
            .style(Style::default().fg(Color::Rgb(120, 120, 120)).bg(bg))
            .render(list_area, buf);
    } else {
        for (row, idx) in (scroll..total).take(visible_height).enumerate() {
            let entry = &explorer.entries[idx];
            let is_selected = idx == explorer.cursor;

            let icon = if entry.is_dir {
                "\u{1f4c1} "
            } else {
                "\u{1f4c4} "
            };
            let name = &entry.name;
            let size_str = if entry.is_dir {
                String::new()
            } else {
                entry.size.map(format_file_size).unwrap_or_default()
            };

            let y = list_area.y + row as u16;
            let line_area = Rect {
                x: list_area.x,
                y,
                width: list_area.width,
                height: 1,
            };

            let name_max = list_area.width as usize - 14;
            let display_name: String = if name.len() > name_max {
                format!("{}\u{2026}", &name[..name_max.saturating_sub(1)])
            } else {
                name.clone()
            };

            let (fg, entry_bg) = if is_selected {
                (Color::Rgb(15, 15, 15), Color::Rgb(60, 160, 220))
            } else if entry.is_dir {
                (Color::Rgb(100, 200, 255), bg)
            } else {
                (Color::Rgb(200, 200, 210), bg)
            };

            let line = Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(fg).bg(entry_bg)),
                Span::styled(
                    format!("{:<width$}", display_name, width = name_max),
                    Style::default()
                        .fg(fg)
                        .bg(entry_bg)
                        .add_modifier(if is_selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::styled(
                    format!("{:>8}", size_str),
                    Style::default()
                        .fg(if is_selected {
                            Color::Rgb(30, 30, 30)
                        } else {
                            Color::Rgb(100, 100, 110)
                        })
                        .bg(entry_bg),
                ),
            ]);

            Paragraph::new(line).render(line_area, buf);
        }
    }

    // ── Search bar (if active) ────────────────────────────────────────────
    let search_line = if explorer.search_active {
        Line::from(vec![
            Span::styled(
                "  \u{1f50d} ",
                Style::default().fg(Color::Rgb(80, 200, 255)),
            ),
            Span::styled(
                format!("{}▏", explorer.search_query),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  \u{00b7}  Esc clear",
                Style::default().fg(Color::Rgb(80, 80, 100)),
            ),
        ])
    } else if explorer.mkdir_active {
        Line::from(vec![
            Span::styled(
                "  \u{1f4c1} New folder: ",
                Style::default().fg(Color::Rgb(255, 200, 80)),
            ),
            Span::styled(
                format!("{}▏", explorer.mkdir_input),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                "  \u{2191}\u{2193} navigate  Enter select  ",
                Style::default().fg(Color::Rgb(80, 80, 100)),
            ),
            Span::styled(
                "S Save Here  ",
                Style::default()
                    .fg(Color::Rgb(80, 200, 80))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "/search  n mkdir  Esc back  ",
                Style::default().fg(Color::Rgb(80, 80, 100)),
            ),
        ])
    };

    // Scrollbar indicator
    let scroll_hint = if total > visible_height {
        format!(" {}/{} ", scroll + 1, total)
    } else {
        String::new()
    };

    let footer_lines = vec![
        search_line,
        Line::from(vec![
            Span::styled(
                "  h/\u{2190} parent  l/\u{2192}/Enter dir  .hidden  s sort",
                Style::default().fg(Color::Rgb(65, 65, 80)),
            ),
            Span::styled(
                format!("  {}", scroll_hint),
                Style::default().fg(Color::Rgb(80, 80, 100)),
            ),
        ]),
    ];

    Paragraph::new(footer_lines)
        .style(Style::default().bg(bg))
        .render(footer_area, buf);
}

/// Format bytes into a human-readable size string.
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

// ── Theme selector overlay ────────────────────────────────────────────────────

/// Render the theme selector overlay panel (right side).
fn render_theme_selector(model: &Model, area: Rect, buf: &mut Buffer) {
    let sel = &model.theme_selector;
    let presets = crate::theme::Theme::all_presets();

    // Panel on the right side, about 30 chars wide
    let panel_width = 32.min(area.width);
    let panel = Rect {
        x: area.x + area.width - panel_width,
        y: area.y,
        width: panel_width,
        height: area.height,
    };

    // Solid background
    let bg = Color::Rgb(20, 22, 28);
    for y in panel.top()..panel.bottom() {
        for x in panel.left()..panel.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_bg(bg);
            }
        }
    }

    let block = Block::bordered()
        .title("  🎨  Themes  ")
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(100, 100, 140)));
    let inner = block.inner(panel);
    block.render(panel, buf);

    if inner.height < 3 {
        return;
    }

    // Header hints
    let header = Line::from(vec![
        Span::styled(
            "  ↑ ",
            Style::default()
                .fg(Color::Rgb(130, 180, 255))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "↓ prev/next  ",
            Style::default().fg(Color::Rgb(100, 100, 120)),
        ),
        Span::styled(
            "Enter ",
            Style::default()
                .fg(Color::Rgb(130, 180, 255))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("apply", Style::default().fg(Color::Rgb(100, 100, 120))),
    ]);
    let header_area = Rect { height: 1, ..inner };
    Paragraph::new(header).render(header_area, buf);

    // List area
    let list_area = Rect {
        y: inner.y + 2,
        height: inner.height.saturating_sub(4),
        ..inner
    };

    // Compute scroll
    let visible = list_area.height as usize;
    let scroll = if sel.cursor >= visible {
        sel.cursor - visible + 1
    } else {
        0
    };

    for (row, idx) in (scroll..presets.len()).take(visible).enumerate() {
        let (name, _desc, _theme) = &presets[idx];
        let is_cursor = idx == sel.cursor;
        let is_active = idx == sel.active;

        let y = list_area.y + row as u16;
        let prefix = if is_active { "→ " } else { "  " };
        let num = format!("{}{}. ", prefix, idx + 1);

        let fg = if is_cursor {
            Color::Rgb(15, 15, 15)
        } else if is_active {
            Color::Rgb(255, 200, 80)
        } else {
            Color::Rgb(180, 180, 190)
        };
        let row_bg = if is_cursor {
            Color::Rgb(60, 140, 220)
        } else {
            bg
        };

        let line = Line::from(vec![
            Span::styled(
                num,
                Style::default()
                    .fg(if is_cursor {
                        fg
                    } else {
                        Color::Rgb(80, 80, 100)
                    })
                    .bg(row_bg),
            ),
            Span::styled(
                format!(
                    "{:<width$}",
                    name,
                    width = (inner.width as usize).saturating_sub(6)
                ),
                Style::default()
                    .fg(fg)
                    .bg(row_bg)
                    .add_modifier(if is_cursor {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ]);
        let line_area = Rect {
            x: list_area.x,
            y,
            width: list_area.width,
            height: 1,
        };
        Paragraph::new(line).render(line_area, buf);
    }

    // Footer with current theme name + description
    let footer_area = Rect {
        y: inner.y + inner.height.saturating_sub(2),
        height: 2.min(inner.height),
        ..inner
    };
    let (active_name, active_desc, _) = &presets[sel.active];
    let footer_lines = vec![
        Line::from(Span::styled(
            format!("  {}", active_name),
            Style::default()
                .fg(Color::Rgb(255, 200, 80))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(
                "  {}",
                if active_desc.len() > inner.width as usize - 4 {
                    &active_desc[..inner.width as usize - 4]
                } else {
                    active_desc
                }
            ),
            Style::default().fg(Color::Rgb(100, 100, 120)),
        )),
    ];
    Paragraph::new(footer_lines)
        .style(Style::default().bg(bg))
        .render(footer_area, buf);
}

// ── centered_rect ─────────────────────────────────────────────────────────────

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}
