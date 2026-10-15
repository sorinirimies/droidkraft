//! `?` overlay.

use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

use crate::model::Model;

pub const LINES: &[&str] = &[
    "d                       Devices home (device cards)",
    "1-9                     open the n-th device workspace (keeps current tab)",
    "[ / ]                   previous / next device",
    "Tab / Shift-Tab         cycle tabs in a workspace (or click the tab bar)",
    "o l c f s               Overview/Logs/Commands/Flash/Screen (alt-<key> always works)",
    "t                       cycle theme      r  refresh devices now",
    "> / <                   graphs faster / slower (0.5s-10s; + / - too, except on Screen)",
    "?                       toggle this help q / Ctrl-C  quit",
    "",
    "Home:      arrows select card · Enter open · o l c m R f = open on that tab",
    "           (m Mirror → Screen, R Record → Screen + start recording)",
    "           mouse: click a card or its [Overview][Logs][Commands][Mirror][Record][Flash] buttons",
    "Overview:  live battery / memory / CPU / storage / thermal / display / network / system",
    "Logs:      see status bar (filter, level, pause, clear, scroll)",
    "Commands:  see status bar (select, run, filter, output scroll)",
    "Flash:     see status bar (fastboot / root / ROM actions)",
    "Screen:    b back · h home · r recents · s mirror start/stop · p power · +/- volume",
    "           R record start/stop · S screenshot · B bit-rate · H max height · T touches",
    "           O open recordings folder · w external mirror window · Ctrl-G input capture",
    "           mouse: drag = touch · right = Back · middle = Home · wheel = scroll",
];

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    let w = 92.min(area.width);
    let h = (LINES.len() as u16 + 4).min(area.height);
    let [r] = Layout::horizontal([Constraint::Length(w)])
        .flex(Flex::Center)
        .areas(area);
    let [r] = Layout::vertical([Constraint::Length(h)])
        .flex(Flex::Center)
        .areas(r);
    f.render_widget(Clear, r);
    let block = super::panel_block(m, "Help");
    let inner = block.inner(r);
    f.render_widget(block, r);
    f.render_widget(
        Paragraph::new(LINES.join("\n")).style(m.themes.base()),
        inner,
    );
}
