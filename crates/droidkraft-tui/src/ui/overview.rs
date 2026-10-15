//! Per-device Overview: battery, memory, CPU, history, storage, thermal,
//! display, network and system info in a responsive card grid.

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use droidkraft_core::{BatteryStatus, DeviceDetails, DeviceHistory, PlugType};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Sparkline};
use ratatui::Frame;
use tui_chart::{Axis, LineChart, Series};
use tui_nuances::Status;
use tui_piechart::{PieChart, PieSlice, Resolution};
use tui_spinner::FluxSpinner;

use crate::format::{fmt_mib, fmt_uptime, opt, truncate};
use crate::message::Cmd;
use crate::model::{state_hint, Model};

pub const HINTS: &str = "j/k PgUp/PgDn scroll ·";

const CARD_H: u16 = 11;
const SYS_H: u16 = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Card {
    Battery,
    Memory,
    Cpu,
    History,
    Storage,
    Thermal,
    Display,
    Network,
    System,
}

const CARDS: [Card; 9] = [
    Card::Battery,
    Card::Memory,
    Card::Cpu,
    Card::History,
    Card::Storage,
    Card::Thermal,
    Card::Display,
    Card::Network,
    Card::System,
];

impl Card {
    fn title(self) -> &'static str {
        match self {
            Card::Battery => "Battery",
            Card::Memory => "Memory",
            Card::Cpu => "CPU",
            Card::History => "CPU % / Memory % history",
            Card::Storage => "Storage (/data)",
            Card::Thermal => "Thermal",
            Card::Display => "Display",
            Card::Network => "Network",
            Card::System => "System",
        }
    }
    fn height(self) -> u16 {
        if self == Card::System {
            SYS_H
        } else {
            CARD_H
        }
    }
}

/// Number of grid columns for a given width (pure, tested).
pub fn columns(width: u16) -> usize {
    if width < 60 {
        return 1;
    }
    ((width / 27) as usize).clamp(1, 3)
}

/// Group cards into rows of `cols`: `(row_height, cards)`.
fn rows(cols: usize) -> Vec<(u16, Vec<Card>)> {
    CARDS
        .chunks(cols)
        .map(|c| {
            (
                c.iter().map(|x| x.height()).max().unwrap_or(CARD_H),
                c.to_vec(),
            )
        })
        .collect()
}

/// Rows that fit starting at `scroll`; the last may be clipped. Returns `(first, rect heights)`.
pub fn layout_rows(heights: &[u16], scroll: usize, avail: u16) -> Vec<(usize, u16)> {
    let mut out = Vec::new();
    let mut left = avail;
    for (i, h) in heights.iter().enumerate().skip(scroll) {
        if left == 0 {
            break;
        }
        let give = (*h).min(left);
        out.push((i, give));
        left -= give;
    }
    out
}

pub fn level_bands(m: &Model, invert: bool) -> Vec<(f64, Color)> {
    let t = &m.themes;
    let (ok, warn, bad) = (
        t.status_color(Status::Success),
        t.status_color(Status::Warning),
        t.status_color(Status::Error),
    );
    if invert {
        vec![(20.0, bad), (50.0, warn), (100.0, ok)]
    } else {
        vec![(60.0, ok), (85.0, warn), (100.0, bad)]
    }
}

/// A themed gauge. `invert`: low is bad (battery).
pub fn mini_gauge(f: &mut Frame, area: Rect, m: &Model, pct: f64, label: String, invert: bool) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    super::common::smooth_gauge(f, area, m, pct, &label, invert);
}

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    let Some(ws) = m.active_ws() else { return };
    let Some(d) = ws.details.clone() else {
        let inner = super::common::card_block(f, area, m, "Overview", false);
        if ws.online() {
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
                Paragraph::new("Loading device…").style(m.themes.muted()),
                Rect::new(
                    inner.x + 3,
                    inner.y,
                    inner.width.saturating_sub(3),
                    1.min(inner.height),
                ),
            );
        } else {
            super::common::notice(
                f,
                inner,
                m,
                &format!("Device is {}", ws.state),
                state_hint(&ws.state),
                true,
            );
        }
        return;
    };
    let hist = m.active_ws().map(|w| w.history.clone()).unwrap_or_default();
    let cols = columns(area.width);
    let grid = rows(cols);
    let heights: Vec<u16> = grid.iter().map(|r| r.0).collect();
    m.ov_scroll = m.ov_scroll.min(grid.len().saturating_sub(1));
    let placed = layout_rows(&heights, m.ov_scroll, area.height);
    let card_w = area.width / cols as u16;
    let mut y = area.y;
    for (ri, h) in &placed {
        for (ci, card) in grid[*ri].1.iter().enumerate() {
            let r = Rect::new(area.x + ci as u16 * card_w, y, card_w, *h);
            draw_card(f, r, m, *card, &d, &hist);
        }
        y += h;
    }
    if m.ov_scroll + placed.len() < grid.len() {
        let hint = " ▼ more · PgDn ";
        let w = hint.chars().count() as u16;
        if w < area.width && area.height > 0 {
            f.render_widget(
                Paragraph::new(hint).style(m.themes.muted()),
                Rect::new(area.x + area.width - w, area.y + area.height - 1, w, 1),
            );
        }
    }
}

fn draw_card(
    f: &mut Frame,
    area: Rect,
    m: &Model,
    card: Card,
    d: &DeviceDetails,
    h: &DeviceHistory,
) {
    let mut title = card.title().to_string();
    if card == Card::System && !d.errors.is_empty() {
        title.push_str(&format!(" · ⚠ {}", d.errors.len()));
    }
    let inner = super::common::card_block(f, area, m, &title, false);
    if inner.width < 4 || inner.height == 0 {
        return;
    }
    match card {
        Card::Battery => battery(f, inner, m, d, h),
        Card::Memory => memory(f, inner, m, d, h),
        Card::Cpu => cpu(f, inner, m, d, h),
        Card::History => history(f, inner, m, h),
        Card::Storage => storage(f, inner, m, d),
        Card::Thermal => thermal(f, inner, m, d, h),
        Card::Display => display(f, inner, m, d),
        Card::Network => network(f, inner, m, d),
        Card::System => system(f, inner, m, d),
    }
}

/// `label  value` lines, aligned, truncated to `w`.
fn kv_lines(rows: &[(&str, String)], w: u16, m: &Model) -> Vec<Line<'static>> {
    let kw = rows
        .iter()
        .map(|(k, _)| k.chars().count())
        .max()
        .unwrap_or(0);
    rows.iter()
        .map(|(k, v)| {
            let vw = (w as usize).saturating_sub(kw + 1);
            Line::from(vec![
                ratatui::text::Span::styled(format!("{k:<kw$} "), m.themes.muted()),
                ratatui::text::Span::raw(truncate(v, vw)),
            ])
        })
        .collect()
}

/// Gauge on row 0, key/values below, sparkline on the last row.
#[allow(clippy::too_many_arguments)]
fn gauge_card(
    f: &mut Frame,
    inner: Rect,
    m: &Model,
    pct: f64,
    label: String,
    invert: bool,
    rows: &[(&str, String)],
    spark: &[f64],
) {
    mini_gauge(
        f,
        Rect::new(inner.x, inner.y, inner.width, 1),
        m,
        pct,
        label,
        invert,
    );
    let spark_h = u16::from(inner.height >= 6 && spark.len() > 1);
    let body_h = inner.height.saturating_sub(1 + spark_h);
    if body_h > 0 {
        let r = Rect::new(inner.x, inner.y + 1, inner.width, body_h);
        f.render_widget(Paragraph::new(kv_lines(rows, inner.width, m)), r);
    }
    if spark_h > 0 {
        let data: Vec<u64> = spark.iter().map(|v| v.max(0.0) as u64).collect();
        f.render_widget(
            Sparkline::default()
                .data(&data)
                .max(100)
                .style(Style::default().fg(m.themes.accent())),
            Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1),
        );
    }
}

fn battery(f: &mut Frame, inner: Rect, m: &Model, d: &DeviceDetails, h: &DeviceHistory) {
    let b = &d.battery;
    let status = match b.status {
        BatteryStatus::Charging => "Charging ⚡",
        BatteryStatus::Discharging => "Discharging",
        BatteryStatus::Full => "Full",
        BatteryStatus::NotCharging => "Not charging",
        BatteryStatus::Unknown => "Unknown",
    };
    let plug = match b.plugged {
        PlugType::Ac => "AC",
        PlugType::Usb => "USB",
        PlugType::Wireless => "Wireless",
        PlugType::None => "unplugged",
        PlugType::Unknown => "-",
    };
    let rows = [
        ("Status", status.to_string()),
        ("Plug", plug.to_string()),
        (
            "Temp",
            b.temperature_c.map_or("-".into(), |t| format!("{t:.1} °C")),
        ),
        (
            "Voltage",
            b.voltage_mv
                .map_or("-".into(), |v| format!("{:.2} V", v as f64 / 1000.0)),
        ),
        (
            "Health",
            if b.health.is_empty() {
                "-".into()
            } else {
                b.health.clone()
            },
        ),
        ("Cycles", opt(b.cycle_count)),
    ];
    let label = format!(
        "{}%{}",
        b.level,
        if b.status == BatteryStatus::Charging {
            " ⚡"
        } else {
            ""
        }
    );
    gauge_card(
        f,
        inner,
        m,
        b.level as f64,
        label,
        true,
        &rows,
        &h.battery.as_vec_f64(),
    );
}

fn memory(f: &mut Frame, inner: Rect, m: &Model, d: &DeviceDetails, h: &DeviceHistory) {
    let mem = &d.memory;
    let swap_used = mem.swap_total_mib.saturating_sub(mem.swap_free_mib);
    let rows = [
        (
            "Used",
            format!("{} / {}", fmt_mib(mem.used_mib()), fmt_mib(mem.total_mib)),
        ),
        ("Avail", fmt_mib(mem.available_mib)),
        ("Cached", fmt_mib(mem.cached_mib)),
        (
            "Swap",
            format!("{} / {}", fmt_mib(swap_used), fmt_mib(mem.swap_total_mib)),
        ),
    ];
    gauge_card(
        f,
        inner,
        m,
        mem.used_pct() as f64,
        format!("{:.0}%", mem.used_pct()),
        false,
        &rows,
        &h.ram_pct.as_vec_f64(),
    );
}

fn cpu(f: &mut Frame, inner: Rect, m: &Model, d: &DeviceDetails, h: &DeviceHistory) {
    let c = &d.cpu;
    let pct = h.cpu_pct.last().copied().unwrap_or(0.0);
    let freq = match (c.cur_freq_mhz, c.max_freq_mhz) {
        (Some(a), Some(b)) => format!("{a}/{b} MHz"),
        (None, Some(b)) => format!("max {b} MHz"),
        (Some(a), None) => format!("{a} MHz"),
        _ => "-".into(),
    };
    let rows = [
        (
            "Load",
            format!("{:.2} {:.2} {:.2}", c.load_1, c.load_5, c.load_15),
        ),
        ("Cores", c.cores.to_string()),
        ("Freq", freq),
        (
            "SoC",
            if c.hardware.is_empty() {
                "-".into()
            } else {
                c.hardware.clone()
            },
        ),
        (
            "ABI",
            if c.abi.is_empty() {
                "-".into()
            } else {
                c.abi.clone()
            },
        ),
    ];
    gauge_card(
        f,
        inner,
        m,
        pct,
        format!("{pct:.0}%"),
        false,
        &rows,
        &h.cpu_pct.as_vec_f64(),
    );
}

fn series(name: &str, v: &[f64], cap: usize) -> Series {
    let off = cap.saturating_sub(v.len());
    Series::new(name).data(v.iter().enumerate().map(|(i, y)| ((i + off) as f64, *y)))
}

fn history(f: &mut Frame, inner: Rect, m: &Model, h: &DeviceHistory) {
    if inner.width < 10 || inner.height < 4 || h.cpu_pct.len() < 2 {
        let msg = if h.cpu_pct.len() < 2 {
            "collecting samples…"
        } else {
            "(enlarge to show chart)"
        };
        f.render_widget(Paragraph::new(msg).style(m.themes.muted()), inner);
        return;
    }
    let cap = h.cpu_pct.capacity();
    let chart = LineChart::new(vec![
        series("CPU", &h.cpu_pct.as_vec_f64(), cap),
        series("Mem", &h.ram_pct.as_vec_f64(), cap),
    ])
    .x_axis(Axis::new().bounds(0.0, cap as f64))
    .y_axis(Axis::new().bounds(0.0, 100.0));
    f.render_widget(chart, inner);
}

fn storage(f: &mut Frame, inner: Rect, m: &Model, d: &DeviceDetails) {
    let s = &d.storage;
    if s.total_mib == 0 {
        f.render_widget(Paragraph::new("n/a").style(m.themes.muted()), inner);
        return;
    }
    let t = &m.themes;
    if inner.height < 4 || inner.width < 12 {
        // Too small for a chart: text only.
        let txt = format!(
            "{:.0}% · {} / {}",
            s.used_pct(),
            fmt_mib(s.used_mib),
            fmt_mib(s.total_mib)
        );
        f.render_widget(Paragraph::new(truncate(&txt, inner.width as usize)), inner);
        return;
    }
    let gib = |v: u64| v as f64 / 1024.0;
    let slices = vec![
        PieSlice::new("Used", gib(s.used_mib), t.status_color(Status::Warning)),
        PieSlice::new(
            "Free",
            gib(s.avail_mib.max(s.total_mib.saturating_sub(s.used_mib))),
            t.status_color(Status::Success),
        ),
    ];
    let pie_h = inner.height.saturating_sub(1).max(1);
    f.render_widget(
        PieChart::new(slices)
            .resolution(Resolution::Braille)
            .show_legend(true)
            .show_percentages(true),
        Rect::new(inner.x, inner.y, inner.width, pie_h),
    );
    if inner.height > 1 {
        f.render_widget(
            Paragraph::new(format!(
                "{} / {}",
                fmt_mib(s.used_mib),
                fmt_mib(s.total_mib)
            ))
            .style(t.muted()),
            Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1),
        );
    }
}

fn temp(v: Option<f32>) -> String {
    v.map_or("-".into(), |t| format!("{t:.1} °C"))
}

fn thermal(f: &mut Frame, inner: Rect, m: &Model, d: &DeviceDetails, h: &DeviceHistory) {
    let t = &d.thermal;
    let rows = [
        ("CPU", temp(t.cpu_temp_c)),
        ("Skin", temp(t.skin_temp_c)),
        (
            "Battery",
            temp(t.battery_temp_c.or(d.battery.temperature_c)),
        ),
        (
            "Throttling",
            if t.throttling {
                "YES".into()
            } else {
                "no".into()
            },
        ),
    ];
    let lines = kv_lines(&rows, inner.width, m);
    let n = lines.len() as u16;
    f.render_widget(
        Paragraph::new(lines),
        Rect::new(inner.x, inner.y, inner.width, n.min(inner.height)),
    );
    if inner.height > n + 1 && h.temp.len() > 1 {
        let data: Vec<u64> = h.temp.iter().map(|v| *v as u64).collect();
        let max = data.iter().copied().max().unwrap_or(1).max(1);
        f.render_widget(
            Sparkline::default()
                .data(&data)
                .max(max)
                .style(Style::default().fg(m.themes.status_color(Status::Warning))),
            Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1),
        );
    }
}

fn display(f: &mut Frame, inner: Rect, m: &Model, d: &DeviceDetails) {
    let s = &d.display;
    let rows = [
        (
            "Resolution",
            if s.width > 0 {
                format!("{}×{}", s.width, s.height)
            } else {
                "-".into()
            },
        ),
        (
            "Density",
            if s.density > 0 {
                format!("{} dpi", s.density)
            } else {
                "-".into()
            },
        ),
        (
            "Refresh",
            s.refresh_hz.map_or("-".into(), |h| format!("{h:.0} Hz")),
        ),
    ];
    let lines = kv_lines(&rows, inner.width, m);
    let n = lines.len() as u16;
    f.render_widget(
        Paragraph::new(lines),
        Rect::new(inner.x, inner.y, inner.width, n.min(inner.height)),
    );
    if let (Some(b), true) = (s.brightness_pct, inner.height > n + 1) {
        f.render_widget(
            Paragraph::new("Brightness").style(m.themes.muted()),
            Rect::new(inner.x, inner.y + n + 1, inner.width, 1),
        );
        if inner.height > n + 2 {
            mini_gauge(
                f,
                Rect::new(inner.x, inner.y + n + 2, inner.width, 1),
                m,
                b as f64,
                format!("{b}%"),
                true,
            );
        }
    }
}

fn network(f: &mut Frame, inner: Rect, m: &Model, d: &DeviceDetails) {
    let n = &d.network;
    let rows = [
        ("SSID", n.wifi_ssid.clone().unwrap_or_else(|| "-".into())),
        ("IP", n.wifi_ip.clone().unwrap_or_else(|| "-".into())),
        (
            "RSSI",
            n.rssi_dbm.map_or("-".into(), |r| format!("{r} dBm")),
        ),
        (
            "Link",
            n.link_mbps.map_or("-".into(), |l| format!("{l} Mbps")),
        ),
        (
            "Operator",
            n.mobile_operator.clone().unwrap_or_else(|| "-".into()),
        ),
        (
            "Airplane",
            if n.airplane {
                "on".into()
            } else {
                "off".into()
            },
        ),
    ];
    f.render_widget(Paragraph::new(kv_lines(&rows, inner.width, m)), inner);
}

fn system(f: &mut Frame, inner: Rect, m: &Model, d: &DeviceDetails) {
    let s = &d.system;
    let dash = |v: &str| {
        if v.is_empty() {
            "-".to_string()
        } else {
            v.to_string()
        }
    };
    let yn = |v: Option<bool>| match v {
        Some(true) => "yes",
        Some(false) => "no",
        None => "?",
    };
    let rows = [
        ("Maker", dash(&s.manufacturer)),
        ("Brand", dash(&s.brand)),
        ("Model", dash(&s.model)),
        ("Codename", dash(&s.codename)),
        (
            "Android",
            format!("{} (SDK {})", dash(&s.android_release), s.sdk),
        ),
        ("Patch", dash(&s.security_patch)),
        ("Build", dash(&s.build_id)),
        ("Fingerprint", dash(&s.build_fingerprint)),
        ("Kernel", dash(&s.kernel)),
        ("Bootloader", dash(&s.bootloader)),
        ("Baseband", dash(&s.baseband)),
        ("Uptime", fmt_uptime(s.uptime_secs)),
        ("Verified boot", dash(&s.verified_boot_state)),
        ("Treble", yn(Some(s.treble)).to_string()),
        ("Rooted", yn(s.is_rooted).to_string()),
        ("BL locked", yn(s.flash_locked).to_string()),
    ];
    f.render_widget(Paragraph::new(kv_lines(&rows, inner.width, m)), inner);
}

pub fn on_key(m: &mut Model, k: KeyEvent) -> (Vec<Cmd>, bool) {
    match k.code {
        KeyCode::Char('j') | KeyCode::Down | KeyCode::PageDown => m.ov_scroll += 1,
        KeyCode::Char('k') | KeyCode::Up | KeyCode::PageUp => {
            m.ov_scroll = m.ov_scroll.saturating_sub(1)
        }
        KeyCode::Home | KeyCode::Char('g') => m.ov_scroll = 0,
        _ => return (vec![], false),
    }
    (vec![], true)
}

pub fn on_mouse(m: &mut Model, e: MouseEvent) -> Vec<Cmd> {
    match e.kind {
        MouseEventKind::ScrollDown => m.ov_scroll += 1,
        MouseEventKind::ScrollUp => m.ov_scroll = m.ov_scroll.saturating_sub(1),
        _ => {}
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_counts() {
        assert_eq!(columns(20), 1);
        assert_eq!(columns(59), 1);
        assert_eq!(columns(60), 2);
        assert_eq!(columns(174), 3);
        assert_eq!(columns(400), 3);
    }

    #[test]
    fn rows_cover_all_cards() {
        for c in 1..=3 {
            let n: usize = rows(c).iter().map(|r| r.1.len()).sum();
            assert_eq!(n, CARDS.len());
        }
    }

    #[test]
    fn layout_clips_last_row() {
        assert_eq!(
            layout_rows(&[11, 11, 17], 0, 25),
            vec![(0, 11), (1, 11), (2, 3)]
        );
        assert_eq!(layout_rows(&[11, 11, 17], 2, 25), vec![(2, 17)]);
        assert_eq!(layout_rows(&[11], 0, 0), vec![]);
    }
}
