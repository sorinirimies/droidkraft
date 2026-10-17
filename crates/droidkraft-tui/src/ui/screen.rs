//! Screen panel: in-terminal scrcpy-like mirror (half-block render + input injection).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::path::PathBuf;

use droidkraft_mirror::{
    default_output_dir, oriented_device, timestamped_name, RecordOptions, RecordStatus,
    ScreenRecorder, ScreenState, ScreenStream,
};
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use tui_nuances::Status;

use crate::format::{file_name, fmt_timer, truncate};
use crate::gfx::{self, Gfx};
use crate::halfblock::{self, cell_to_device_aspect, cell_to_device_clamped_aspect, ScaleCache};
use crate::message::Cmd;
use crate::model::{Model, Panel};
use crate::worker::Req;

pub const HINTS: &str =
    "R rec · S shot · s mirror · b/h/r keys · p power · +/- vol · B/H/T rec opts · O folder · ^G capture ·";

pub const BITRATES: [u32; 4] = [4, 8, 12, 20];
pub const HEIGHTS: [Option<u32>; 4] = [None, Some(1080), Some(720), Some(480)];

/// Clickable controls of the recording side panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecAction {
    Toggle,
    Screenshot,
    Mirror,
    Bitrate,
    Height,
    Touches,
    Folder,
    Open(usize),
}

pub struct ScreenUi {
    pub stream: ScreenStream,
    /// Ctrl-G: forward typed keys to the device.
    pub capture: bool,
    /// User pressed stop; suppress auto-start.
    pub user_stopped: bool,
    /// Serial the stream was last started for.
    pub started_for: Option<String>,
    pub cache: ScaleCache,
    /// Real-image renderer (falls back to half-blocks).
    pub gfx: Gfx,
    /// Cell height/width ratio of the current frame_rect (2.0 for half-blocks).
    pub frame_ratio: f32,
    /// Cells occupied by the rendered frame (set by `draw`).
    pub frame_rect: Rect,
    /// Pixel size of the displayed frame.
    pub frame_px: (u32, u32),
    pub device: Option<(u32, u32)>,
    pub pressed: bool,
    last_seq: u64,
    /// Replaces the live stream state when set (tests / demos).
    pub state_override: Option<ScreenState>,
    // ---- recording (independent of the live mirror) ----
    pub recorder: ScreenRecorder,
    pub rec_opts: RecordOptions,
    pub out_dir: PathBuf,
    /// Finished recordings and screenshots (newest last).
    pub recordings: Vec<PathBuf>,
    pub rec_error: Option<String>,
    /// Terminal recorder status already folded into `recordings`.
    rec_absorbed: bool,
    pub rec_rects: Vec<(RecAction, Rect)>,
}

impl Default for ScreenUi {
    fn default() -> Self {
        Self {
            stream: ScreenStream::default(),
            capture: false,
            user_stopped: false,
            started_for: None,
            cache: ScaleCache::default(),
            gfx: Gfx::default(),
            frame_ratio: 2.0,
            frame_rect: Rect::default(),
            frame_px: (0, 0),
            device: None,
            pressed: false,
            last_seq: 0,
            state_override: None,
            recorder: ScreenRecorder::new(),
            rec_opts: RecordOptions::default(),
            out_dir: default_output_dir(),
            recordings: Vec::new(),
            rec_error: None,
            rec_absorbed: true,
            rec_rects: Vec::new(),
        }
    }
}

impl ScreenUi {
    /// Fold a recorder status into the finished-files list. Returns a status message once.
    pub fn absorb(&mut self, st: RecordStatus) -> Option<String> {
        match st {
            RecordStatus::Recording { .. } | RecordStatus::Finishing => {
                self.rec_absorbed = false;
                None
            }
            RecordStatus::Done { files } if !self.rec_absorbed => {
                self.rec_absorbed = true;
                self.rec_error = None;
                let n = files.len();
                self.recordings.extend(files);
                Some(format!(
                    "Recording saved ({n} file{})",
                    if n == 1 { "" } else { "s" }
                ))
            }
            RecordStatus::Failed(e) if !self.rec_absorbed => {
                self.rec_absorbed = true;
                self.rec_error = Some(e.clone());
                Some(format!("Recording failed: {e}"))
            }
            _ => None,
        }
    }

    pub fn poll_recorder(&mut self) -> Option<String> {
        let st = self.recorder.status();
        self.absorb(st)
    }

    pub fn cycle_bitrate(&mut self) {
        let i = BITRATES
            .iter()
            .position(|b| *b == self.rec_opts.bit_rate_mbps)
            .map_or(0, |i| i + 1);
        self.rec_opts.bit_rate_mbps = BITRATES[i % BITRATES.len()];
    }

    pub fn cycle_height(&mut self) {
        let i = HEIGHTS
            .iter()
            .position(|h| *h == self.rec_opts.max_height)
            .map_or(0, |i| i + 1);
        self.rec_opts.max_height = HEIGHTS[i % HEIGHTS.len()];
    }
}

impl ScreenUi {
    /// Stream sequence changed since last call (for redraw throttling).
    pub fn seq_changed(&mut self) -> bool {
        let s = self.stream.latest_seq();
        let ch = s != self.last_seq;
        self.last_seq = s;
        ch
    }
}

/// Input to inject, derived from a mouse event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Touch(&'static str, u32, u32),
    Key(&'static str),
    Swipe(u32, u32, u32, u32),
}

/// Pure mouse → device-input mapping.
pub fn mouse_input(
    rect: Rect,
    frame: (u32, u32),
    device: Option<(u32, u32)>,
    pressed: bool,
    e: &MouseEvent,
) -> Option<Input> {
    mouse_input_aspect(rect, frame, device, pressed, e, 2.0)
}

/// [`mouse_input`] for cells whose height is `ratio` × width.
pub fn mouse_input_aspect(
    rect: Rect,
    frame: (u32, u32),
    device: Option<(u32, u32)>,
    pressed: bool,
    e: &MouseEvent,
    ratio: f32,
) -> Option<Input> {
    let inside = |r| {
        cell_to_device_aspect(e.column, e.row, rect, frame, device, ratio).map(|(x, y)| (r, x, y))
    };
    match e.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            inside("DOWN").map(|(a, x, y)| Input::Touch(a, x, y))
        }
        MouseEventKind::Drag(MouseButton::Left) if pressed => {
            cell_to_device_clamped_aspect(e.column, e.row, rect, frame, device, ratio)
                .map(|(x, y)| Input::Touch("MOVE", x, y))
        }
        MouseEventKind::Up(MouseButton::Left) if pressed => {
            cell_to_device_clamped_aspect(e.column, e.row, rect, frame, device, ratio)
                .map(|(x, y)| Input::Touch("UP", x, y))
        }
        MouseEventKind::Down(MouseButton::Right) => inside("").map(|_| Input::Key("KEYCODE_BACK")),
        MouseEventKind::Down(MouseButton::Middle) => inside("").map(|_| Input::Key("KEYCODE_HOME")),
        MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
            let (_, x, y) = inside("")?;
            let (_, dh) = oriented_device(frame, device);
            let d = (dh / 4).max(1);
            let (from, to) = if e.kind == MouseEventKind::ScrollDown {
                let f = (y + d / 2).min(dh.saturating_sub(1));
                (f, f.saturating_sub(d))
            } else {
                let f = y.saturating_sub(d / 2);
                (f, (f + d).min(dh.saturating_sub(1)))
            };
            Some(Input::Swipe(x, from, x, to))
        }
        _ => None,
    }
}

fn send(s: &ScreenStream, i: Input) {
    match i {
        Input::Touch(a, x, y) => s.touch(a, x, y),
        Input::Key(k) => s.key(k),
        Input::Swipe(a, b, c, d) => s.swipe(a, b, c, d, 200),
    }
}

fn active_serial(m: &Model) -> Option<String> {
    m.active_online_serial()
}

/// Auto-start the stream when on the Screen panel (unless the user stopped it).
pub fn ensure_started(m: &mut Model) {
    if m.active.is_none() || m.panel != Panel::Screen || m.screen.user_stopped {
        return;
    }
    let Some(serial) = active_serial(m) else {
        return;
    };
    if m.screen.started_for.as_deref() != Some(serial.as_str()) {
        m.screen.stream.stop();
        m.screen.started_for = Some(serial.clone());
        m.screen.stream.start(serial);
    }
}

pub fn tick(m: &mut Model) {
    ensure_started(m);
    if let Some(msg) = m.screen.poll_recorder() {
        m.set_message(msg);
    }
}

/// Start or stop recording the open device.
pub fn toggle_record(m: &mut Model) {
    if m.screen.recorder.is_recording() {
        m.screen.recorder.stop();
        m.set_message("Stopping recording…");
        return;
    }
    let Some(serial) = active_serial(m) else {
        m.set_message("No online device to record");
        return;
    };
    let sc = &mut m.screen;
    match sc.recorder.start(
        &serial,
        &sc.out_dir,
        &timestamped_name(&serial),
        sc.rec_opts.clone(),
    ) {
        Ok(()) => {
            sc.rec_absorbed = false;
            sc.rec_error = None;
            m.set_message("Recording…");
        }
        Err(e) => m.set_message(format!("Record failed: {e}")),
    }
}

/// Request a PNG screenshot saved beside the recordings.
pub fn screenshot(m: &mut Model) -> Vec<Cmd> {
    let Some(serial) = active_serial(m) else {
        m.set_message("No online device");
        return vec![];
    };
    let path = m
        .screen
        .out_dir
        .join(format!("{}.png", timestamped_name(&serial)));
    m.set_message("Taking screenshot…");
    vec![Cmd::Worker(Req::Screenshot { serial, path })]
}

/// Worker result of a screenshot.
pub fn on_screenshot(m: &mut Model, result: Result<PathBuf, String>) {
    match result {
        Ok(p) => {
            m.set_message(format!("Screenshot saved: {}", p.display()));
            m.screen.recordings.push(p);
        }
        Err(e) => m.set_message(format!("Screenshot failed: {e}")),
    }
}

fn do_action(m: &mut Model, a: RecAction) -> Vec<Cmd> {
    match a {
        RecAction::Toggle => toggle_record(m),
        RecAction::Screenshot => return screenshot(m),
        RecAction::Mirror => start_stop(m),
        RecAction::Bitrate => m.screen.cycle_bitrate(),
        RecAction::Height => m.screen.cycle_height(),
        RecAction::Touches => m.screen.rec_opts.show_touches ^= true,
        RecAction::Folder => return vec![Cmd::Open(m.screen.out_dir.clone())],
        RecAction::Open(i) => {
            if let Some(p) = m.screen.recordings.get(i) {
                return vec![Cmd::Open(p.clone())];
            }
        }
    }
    vec![]
}

/// Stop the stream (app exit).
pub fn shutdown(m: &mut Model) {
    m.screen.stream.stop();
}

fn fmt_err(e: &str) -> Vec<String> {
    let mut v = vec![e.to_string()];
    if e.to_lowercase().contains("ffmpeg") {
        v.push(String::new());
        v.push("ffmpeg is required for the live stream. Install it:".into());
        v.push("  macOS:  brew install ffmpeg".into());
        v.push("  Linux:  sudo apt install ffmpeg".into());
    }
    v
}

pub fn draw(f: &mut Frame, area: Rect, m: &mut Model) {
    use ratatui::layout::{Constraint, Layout};
    let cols = if super::layout::stack_panes(area.width) {
        let h = 8.min(area.height / 2);
        Layout::vertical([Constraint::Min(1), Constraint::Length(h)]).split(area)
    } else {
        Layout::horizontal([
            Constraint::Min(1),
            Constraint::Length(32.min(area.width / 2)),
        ])
        .split(area)
    };
    draw_mirror(f, cols[0], m);
    draw_rec(f, cols[1], m);
}

fn draw_rec(f: &mut Frame, area: Rect, m: &mut Model) {
    let block = super::panel_block(m, "Record");
    let inner = block.inner(area);
    f.render_widget(block, area);
    m.screen.rec_rects.clear();
    let t = &m.themes;
    let sc = &m.screen;
    let w = inner.width as usize;
    let mut rows: Vec<(Line, Option<RecAction>)> = Vec::new();
    let st = sc.recorder.status();
    let rec = sc.recorder.is_recording();
    let badge = match (&st, sc.recorder.elapsed()) {
        (_, Some(e)) => Line::styled(
            format!(" ● REC {} ", fmt_timer(e)),
            t.status(Status::Error)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        ),
        (RecordStatus::Finishing, _) => Line::styled(" ◌ finishing… ", t.status(Status::Warning)),
        _ => Line::styled("○ not recording", t.muted()),
    };
    rows.push((badge, None));
    rows.push((
        Line::styled(
            if rec {
                "[R] ■ Stop"
            } else {
                "[R] ● Record"
            },
            t.status(if rec { Status::Error } else { Status::Success }),
        ),
        Some(RecAction::Toggle),
    ));
    rows.push((Line::raw("[S] ▣ Screenshot"), Some(RecAction::Screenshot)));
    let mirror_on = sc.stream.is_running();
    rows.push((
        Line::raw(format!(
            "[s] ▶ Mirror: {}",
            if mirror_on { "on" } else { "off" }
        )),
        Some(RecAction::Mirror),
    ));
    rows.push((Line::styled("─ Options", t.title()), None));
    rows.push((
        Line::raw(format!("[B] Bit-rate  {} Mbps", sc.rec_opts.bit_rate_mbps)),
        Some(RecAction::Bitrate),
    ));
    let h = sc
        .rec_opts
        .max_height
        .map_or("native".to_string(), |h| format!("{h}p"));
    rows.push((
        Line::raw(format!("[H] Max height {h}")),
        Some(RecAction::Height),
    ));
    rows.push((
        Line::raw(format!(
            "[T] Touches [{}]",
            if sc.rec_opts.show_touches { "x" } else { " " }
        )),
        Some(RecAction::Touches),
    ));
    rows.push((
        Line::raw(format!(
            "[O] 📁 {}",
            truncate(&sc.out_dir.display().to_string(), w.saturating_sub(7))
        )),
        Some(RecAction::Folder),
    ));
    if let Some(e) = &sc.rec_error {
        rows.push((
            Line::styled(truncate(&format!("✘ {e}"), w), t.status(Status::Error)),
            None,
        ));
    }
    rows.push((
        Line::styled(format!("─ Files ({})", sc.recordings.len()), t.title()),
        None,
    ));
    let used = rows.len();
    let room = (inner.height as usize).saturating_sub(used);
    let skip = sc.recordings.len().saturating_sub(room);
    for (i, p) in sc.recordings.iter().enumerate().skip(skip).rev() {
        rows.push((
            Line::raw(format!(
                "↗ {}",
                truncate(&file_name(p), w.saturating_sub(2))
            )),
            Some(RecAction::Open(i)),
        ));
    }
    // Short panel: drop section headers, then fall back to a key-hint row so
    // the hidden options stay discoverable.
    if rows.len() > inner.height as usize {
        let mut i = 0;
        rows.retain(|(l, a)| {
            i += 1;
            i == 1 || a.is_some() || !l.to_string().starts_with('─')
        });
    }
    if rows.len() > inner.height as usize && inner.height >= 2 {
        rows.truncate(inner.height as usize - 1);
        rows.push((Line::styled("keys: B H T O", m.themes.muted()), None));
    }
    for (i, (line, act)) in rows.into_iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        let r = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
        f.render_widget(Paragraph::new(line).style(m.themes.base()), r);
        if let Some(a) = act {
            m.screen.rec_rects.push((a, r));
        }
    }
}

fn draw_mirror(f: &mut Frame, area: Rect, m: &mut Model) {
    let proto = if m.screen.gfx.active() {
        let (w, h) = m.screen.gfx.font;
        format!(
            "{} {w}x{h}px/cell · frame {}x{}",
            m.screen.gfx.name(),
            m.screen.frame_px.0,
            m.screen.frame_px.1
        )
    } else {
        m.screen.gfx.name().to_string()
    };
    let title = if m.screen.capture {
        format!("Screen · {proto} · CAPTURING")
    } else {
        format!("Screen · {proto}")
    };
    let block = super::panel_block(m, &title);
    let inner = block.inner(area);
    f.render_widget(block, area);
    m.screen.frame_rect = Rect::default();
    let st = m
        .screen
        .state_override
        .clone()
        .unwrap_or_else(|| m.screen.stream.state());
    let t = &m.themes;

    if let Some(err) = &st.error {
        // Retry hint first so it survives short panels.
        let mut lines = vec![Line::styled(
            "Screen stream error · s retry",
            t.status(Status::Error),
        )];
        lines.extend(fmt_err(err).into_iter().map(Line::raw));
        f.render_widget(
            Paragraph::new(lines)
                .style(t.base())
                .wrap(ratatui::widgets::Wrap { trim: false }),
            inner,
        );
        return;
    }
    let Some(frame) = st.frame.as_ref() else {
        let msg = if !m.connected() {
            "No device connected. Plug one in with USB debugging enabled."
        } else if m.screen.user_stopped || !m.screen.stream.is_running() {
            "Stream stopped. Press s to start."
        } else {
            "Waiting for first frame…"
        };
        f.render_widget(
            Paragraph::new(msg)
                .style(t.muted())
                .wrap(ratatui::widgets::Wrap { trim: false }),
            inner,
        );
        return;
    };

    // Real image protocols are never drawn under the help overlay (Sixel/Kitty would bleed through).
    let use_gfx = m.screen.gfx.active() && !m.help;
    let rect = if use_gfx {
        gfx::fit_cells_font(inner, frame.width, frame.height, m.screen.gfx.font)
    } else {
        halfblock::fit_cells(inner, frame.width, frame.height)
    };
    if rect.width > 0 && rect.height > 0 {
        if use_gfx {
            m.screen.gfx.render(f, rect, frame.seq, &frame.image);
        } else {
            let scaled = m
                .screen
                .cache
                .get(frame.seq, &frame.image, rect.width, rect.height)
                .clone();
            halfblock::paint(f.buffer_mut(), rect, &scaled);
        }
        m.screen.frame_rect = rect;
        m.screen.frame_ratio = if use_gfx {
            m.screen.gfx.cell_ratio()
        } else {
            2.0
        };
        m.screen.frame_px = (frame.width, frame.height);
        m.screen.device = st.device_size;
        if !use_gfx {
            let chip = format!(
                " {:.0} fps · {}×{} · {} frames{} ",
                st.fps,
                frame.width,
                frame.height,
                st.frame_count,
                if st.is_still { " · still" } else { "" }
            );
            // Overlay on the panel's top row, not the (possibly tiny) frame.
            let w = (chip.chars().count() as u16).min(inner.width);
            f.render_widget(
                Paragraph::new(chip).style(t.base().add_modifier(Modifier::REVERSED)),
                Rect::new(inner.x, inner.y, w, 1),
            );
        }
    }
    if m.screen.capture {
        let b = " CAPTURING · ^G to release ";
        let w = (b.chars().count() as u16).min(inner.width);
        f.render_widget(
            Paragraph::new(b).style(
                t.status(Status::Error)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            ),
            Rect::new(inner.x + inner.width - w, inner.y, w, 1),
        );
    }
}

fn start_stop(m: &mut Model) {
    if m.screen.stream.is_running() {
        m.screen.user_stopped = true;
        m.screen.started_for = None;
        m.screen.stream.stop();
        m.set_message("Screen stream stopped");
    } else {
        m.screen.user_stopped = false;
        m.screen.started_for = None;
        ensure_started(m);
        m.set_message("Screen stream starting…");
    }
}

pub fn on_key(m: &mut Model, k: KeyEvent) -> (Vec<Cmd>, bool) {
    let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
    if ctrl && k.code == KeyCode::Char('g') {
        m.screen.capture = !m.screen.capture;
        m.set_message(if m.screen.capture {
            "Input capture ON (Ctrl-G to release)"
        } else {
            "Input capture off"
        });
        return (vec![], true);
    }
    let s = &m.screen.stream;
    if m.screen.capture {
        if ctrl {
            return (vec![], true);
        }
        match k.code {
            KeyCode::Char(c) => s.text(&c.to_string()),
            KeyCode::Enter => s.key("KEYCODE_ENTER"),
            KeyCode::Backspace => s.key("KEYCODE_DEL"),
            KeyCode::Up => s.key("KEYCODE_DPAD_UP"),
            KeyCode::Down => s.key("KEYCODE_DPAD_DOWN"),
            KeyCode::Left => s.key("KEYCODE_DPAD_LEFT"),
            KeyCode::Right => s.key("KEYCODE_DPAD_RIGHT"),
            KeyCode::Esc => s.key("KEYCODE_BACK"),
            KeyCode::Tab => s.key("KEYCODE_TAB"),
            _ => {}
        }
        return (vec![], true);
    }
    if ctrl {
        return (vec![], false);
    }
    match k.code {
        KeyCode::Char('b') => s.key("KEYCODE_BACK"),
        KeyCode::Char('h') => s.key("KEYCODE_HOME"),
        KeyCode::Char('r') => s.key("KEYCODE_APP_SWITCH"),
        KeyCode::Char('p') => s.key("KEYCODE_POWER"),
        KeyCode::Char('+') | KeyCode::Char('=') => s.key("KEYCODE_VOLUME_UP"),
        KeyCode::Char('-') => s.key("KEYCODE_VOLUME_DOWN"),
        KeyCode::Char('s') => start_stop(m),
        KeyCode::Char('R') => {
            do_action(m, RecAction::Toggle);
        }
        KeyCode::Char('S') => return (screenshot(m), true),
        KeyCode::Char('B') => {
            do_action(m, RecAction::Bitrate);
        }
        KeyCode::Char('H') => {
            do_action(m, RecAction::Height);
        }
        KeyCode::Char('T') => {
            do_action(m, RecAction::Touches);
        }
        KeyCode::Char('O') => return (do_action(m, RecAction::Folder), true),
        KeyCode::Char('w') => {
            let serial = active_serial(m);
            return (vec![Cmd::Mirror { serial, size: None }], true);
        }
        _ => return (vec![], false),
    }
    (vec![], true)
}

pub fn on_mouse(m: &mut Model, e: MouseEvent) -> Vec<Cmd> {
    if let MouseEventKind::Down(MouseButton::Left) = e.kind {
        let hit = m
            .screen
            .rec_rects
            .iter()
            .find(|(_, r)| super::common::hit(*r, e.column, e.row))
            .map(|(a, _)| *a);
        if let Some(a) = hit {
            return do_action(m, a);
        }
    }
    let sc = &mut m.screen;
    if let Some(i) = mouse_input_aspect(
        sc.frame_rect,
        sc.frame_px,
        sc.device,
        sc.pressed,
        &e,
        sc.frame_ratio,
    ) {
        match &i {
            Input::Touch("DOWN", ..) => sc.pressed = true,
            Input::Touch("UP", ..) => sc.pressed = false,
            _ => {}
        }
        send(&sc.stream, i);
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_util::{key, render};
    use crossterm::event::KeyModifiers;
    use droidkraft_mirror::{ScreenState, VideoFrame};
    use image::{Rgba, RgbaImage};
    use std::sync::Arc;

    fn me(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn screen_model() -> Model {
        crate::ui::test_util::with_device(crate::model::Panel::Screen)
    }

    #[test]
    fn mapping_touch_and_buttons() {
        let r = Rect::new(10, 5, 20, 20);
        let f = (1080, 2400);
        let d = Some((1080, 2400));
        match mouse_input(
            r,
            f,
            d,
            false,
            &me(MouseEventKind::Down(MouseButton::Left), 20, 15),
        ) {
            Some(Input::Touch("DOWN", x, y)) => {
                assert!(
                    (x as i32 - 540).abs() < 60 && (y as i32 - 1200).abs() < 80,
                    "{x},{y}"
                )
            }
            o => panic!("{o:?}"),
        }
        assert_eq!(
            mouse_input(
                r,
                f,
                d,
                false,
                &me(MouseEventKind::Down(MouseButton::Left), 0, 0)
            ),
            None
        );
        assert_eq!(
            mouse_input(
                r,
                f,
                d,
                false,
                &me(MouseEventKind::Drag(MouseButton::Left), 20, 15)
            ),
            None
        );
        assert!(matches!(
            mouse_input(r, f, d, true, &me(MouseEventKind::Up(MouseButton::Left), 99, 99)),
            Some(Input::Touch("UP", x, y)) if x > 1000 && y > 2300
        ));
        assert_eq!(
            mouse_input(
                r,
                f,
                d,
                false,
                &me(MouseEventKind::Down(MouseButton::Right), 20, 15)
            ),
            Some(Input::Key("KEYCODE_BACK"))
        );
        assert_eq!(
            mouse_input(
                r,
                f,
                d,
                false,
                &me(MouseEventKind::Down(MouseButton::Middle), 20, 15)
            ),
            Some(Input::Key("KEYCODE_HOME"))
        );
        match mouse_input(r, f, d, false, &me(MouseEventKind::ScrollDown, 20, 15)) {
            Some(Input::Swipe(x1, y1, x2, y2)) => assert!(x1 == x2 && y2 < y1),
            o => panic!("{o:?}"),
        }
        match mouse_input(r, f, d, false, &me(MouseEventKind::ScrollUp, 20, 15)) {
            Some(Input::Swipe(_, y1, _, y2)) => assert!(y2 > y1),
            o => panic!("{o:?}"),
        }
    }

    fn with_frame(m: &mut Model) {
        let img = Arc::new(RgbaImage::from_pixel(108, 240, Rgba([200, 30, 30, 255])));
        let st = ScreenState {
            frame: Some(VideoFrame {
                image: img,
                width: 108,
                height: 240,
                seq: 1,
            }),
            frame_count: 7,
            fps: 30.0,
            device_size: Some((108, 240)),
            ..Default::default()
        };
        m.screen.state_override = Some(st);
    }

    #[test]
    fn render_without_frame() {
        for (w, h) in [(80, 24), (200, 60)] {
            let mut m = screen_model();
            m.workspaces.get_mut("TESTSERIAL").unwrap().state = "offline".into();
            m.screen.state_override = Some(ScreenState::default());
            let out = render(&mut m, w, h);
            assert!(out.contains("Screen"));
            assert!(out.contains("No device connected"));
        }
    }

    #[test]
    fn render_with_frame_sets_rect() {
        for (w, h) in [(80, 24), (200, 60)] {
            let mut m = screen_model();
            with_frame(&mut m);
            let out = render(&mut m, w, h);
            assert!(out.contains('▀'));
            assert!(out.contains("fps"), "{w}x{h}:\n{out}");
            assert!(m.screen.frame_rect.width > 0 && m.screen.frame_rect.height > 0);
            assert_eq!(m.screen.frame_px, (108, 240));
        }
    }

    #[test]
    fn render_error_card() {
        let mut m = screen_model();
        m.screen.state_override = Some(ScreenState {
            error: Some("ffmpeg not found".into()),
            ..Default::default()
        });
        let out = render(&mut m, 120, 30);
        assert!(out.contains("brew install ffmpeg"));
    }

    #[test]
    fn capture_toggle_consumes_keys() {
        let mut m = screen_model();
        let ctrl_g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert!(on_key(&mut m, ctrl_g).1 && m.screen.capture);
        assert!(on_key(&mut m, key(KeyCode::Tab)).1);
        assert!(on_key(&mut m, ctrl_g).1 && !m.screen.capture);
        assert!(!on_key(&mut m, key(KeyCode::Tab)).1);
        let (c, ok) = on_key(&mut m, key(KeyCode::Char('w')));
        assert!(ok && matches!(c[0], Cmd::Mirror { .. }));
    }
}
