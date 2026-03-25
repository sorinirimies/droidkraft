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
use crate::model::{AppState, Model};
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
        AppState::Startup    => render_startup(model, area, buf),
        AppState::Menu       => render_menu(model, area, buf),
        AppState::Loading    => render_loading(model, area, buf),
        AppState::ShowResult => render_result(model, area, buf),
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
    let label   = model.menu.get_selected_label();

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
    let body_area   = outer[1];
    let desc_area   = outer[2];
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
        "  \u{2191}/\u{2193} j/k Navigate  Tab/S-Tab Section  Enter Execute  d Device  r Refresh  q Quit  ",
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
    let connected   = status.is_connected();
    let bdr_color   = if connected { Color::Rgb(40, 120, 40) } else { Color::Rgb(60, 50, 35) };
    let multi_hint  = if status.devices.len() > 1 { "  d \u{2013} cycle" } else { "" };

    let block = Block::bordered()
        .title(format!("  \u{1f4f1}  Devices{}  ", multi_hint))
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(bdr_color));

    let inner = block.inner(area);
    block.render(area, buf);

    if !connected {
        Paragraph::new(Line::from(vec![
            Span::styled(
                "  \u{25cb}  No device connected",
                Style::default().fg(Color::Rgb(160, 130, 60)),
            ),
        ]))
        .render(inner, buf);
        return;
    }

    let dim    = Color::Rgb(100, 100, 100);
    let sel_bg = Color::Rgb(40, 140, 40);
    let sel_fg = Color::Rgb(10, 10, 10);

    let lines: Vec<Line<'static>> = status
        .devices
        .iter()
        .enumerate()
        .take(inner.height as usize)
        .map(|(idx, dev)| {
            let selected = idx == status.selected_idx;
            let serial   = format!("{:<20}", dev.serial);
            let state    = dev.state.clone();

            if selected {
                Line::from(vec![
                    Span::styled(
                        "  \u{25b6}  ",
                        Style::default().fg(Color::Rgb(80, 220, 80)),
                    ),
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
    let connected  = status.is_connected();
    let bdr_color  = if connected { Color::Rgb(35, 70, 35) } else { Color::Rgb(55, 45, 35) };

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
        0..=20  => Color::Rgb(220, 60, 60),
        21..=50 => Color::Rgb(220, 160, 50),
        _       => Color::Rgb(80, 200, 80),
    };

    // RAM
    let ram_used = status.ram_total_mib.saturating_sub(status.ram_avail_mib);
    let ram_pct  = if status.ram_total_mib > 0 {
        ram_used as f32 / status.ram_total_mib as f32
    } else {
        0.0
    };
    let ram_color = match (ram_pct * 100.0) as u8 {
        0..=60  => Color::Rgb(80, 200, 80),
        61..=80 => Color::Rgb(220, 160, 50),
        _       => Color::Rgb(220, 60, 60),
    };

    // CPU — clamp load to 4.0 (4-core ceiling for bar display)
    let cpu_frac  = (status.cpu_load_1min / 4.0).clamp(0.0, 1.0);
    let cpu_color = match (cpu_frac * 100.0) as u8 {
        0..=60  => Color::Rgb(80, 200, 80),
        61..=80 => Color::Rgb(220, 160, 50),
        _       => Color::Rgb(220, 60, 60),
    };

    let dim    = Color::Rgb(90, 90, 90);
    let bright = Color::Rgb(205, 205, 205);

    let model_text = if status.model.is_empty() { "Unknown".to_string() } else { status.model.clone() };
    let ver_text   = if status.android_version.is_empty() {
        String::new()
    } else {
        format!("Android {}", status.android_version)
    };

    let mut lines: Vec<Line<'static>> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  \u{1f4f1}  ", Style::default().fg(dim)),
            Span::styled(model_text, Style::default().fg(bright).add_modifier(Modifier::BOLD)),
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
    let dim    = Color::Rgb(90, 90, 90);
    let amber  = Color::Rgb(160, 130, 60);
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
    label:  &str,
    value:  f32,
    max:    f32,
    bar_w:  usize,
    color:  Color,
    suffix: String,
) -> Line<'static> {
    let pct    = (value / max).clamp(0.0, 1.0);
    let filled = (pct * bar_w as f32) as usize;
    let empty  = bar_w.saturating_sub(filled);

    Line::from(vec![
        Span::styled(label.to_string(),       Style::default().fg(Color::Rgb(90, 90, 90))),
        Span::styled("\u{2588}".repeat(filled), Style::default().fg(color)),
        Span::styled("\u{2591}".repeat(empty),  Style::default().fg(Color::Rgb(40, 40, 40))),
        Span::styled(format!("  {}", suffix),  Style::default().fg(color)),
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
    let base_area      = centered_rect(82, 76, area);
    let slide_offset   = ((1.0 - slide_progress) * base_area.height as f32 * 0.4) as u16;

    let popup_area = if slide_offset > 0 {
        Rect {
            x:      base_area.x,
            y:      base_area.y + slide_offset,
            width:  base_area.width,
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
        x:     popup_area.x + popup_area.width.saturating_sub(3),
        width: 3,
        ..popup_area
    };

    let content_height = content_area.height.saturating_sub(4) as usize;
    let max_width      = content_area.width.saturating_sub(4) as usize;
    model.update_wrapped_lines(max_width);

    let total   = model.wrapped_lines.len();
    let start   = model.scroll_position;
    let end     = (start + content_height).min(total);
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
    area:        Rect,
    buf:         &mut Buffer,
    total_lines: usize,
    visible:     usize,
    pos:         usize,
    color:       Color,
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
    let thumb_pos  = if total_lines > visible {
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
