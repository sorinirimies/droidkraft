//! Reusable screen-mirror viewport (+ optional control chrome) as a gpui entity.
//!
//! One [`MirrorView`] owns the zoom/pan/drag state, the currently shown frame,
//! the repaint loop and auto-start logic. It can be embedded in the main
//! window's Screen panel, hosted in a detached window, or be the only content
//! of a standalone `--mirror` window. An entity can only be rendered by one
//! window at a time, so hosts must not render it while it lives elsewhere.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gpui::{
    actions, canvas, div, img, prelude::*, px, rgb, AnyElement, App, Bounds, ClickEvent, Context,
    EventEmitter, FocusHandle, KeyBinding, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ObjectFit, Pixels, Point, RenderImage, ScrollDelta,
    ScrollWheelEvent, Window,
};

use crate::screen::{layout, map_to_device, oriented_device, ScreenStream, Zoom};
use crate::theme;

actions!(droidkraft, [ToggleDetach, CloseWindow]);

/// Register the mirror-related key bindings (Cmd/Ctrl+D, Cmd/Ctrl+W).
pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("secondary-d", ToggleDetach, None),
        KeyBinding::new("secondary-w", CloseWindow, None),
    ]);
}

/// How much control UI surrounds the viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chrome {
    /// Viewport + full side panel (nav, hardware keys, zoom, stats, help).
    Full,
    /// Viewport + top control bar (zoom, nav, start/stop, stats toggle).
    Compact,
    /// Viewport + thin strip (zoom, Back/Home, start/stop).
    Minimal,
}

/// Events a host window may react to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorEvent {
    /// The user asked to re-embed a detached view (Attach button).
    AttachRequested,
}

impl EventEmitter<MirrorEvent> for MirrorView {}

type ViewRect = ((f32, f32), (f32, f32, f32, f32));

pub struct MirrorView {
    stream: Arc<ScreenStream>,
    serial: Option<String>,
    chrome: Chrome,
    /// Show the "Attach" button in the control bar.
    attach_button: bool,
    show_stats: bool,
    /// Auto-start the stream when a serial is known (unless user stopped it).
    auto_start: bool,
    /// Standalone: look up the first connected device when no serial is set.
    discover: bool,
    discovered: Arc<Mutex<Option<String>>>,
    discovering: Arc<AtomicBool>,
    user_stopped: bool,
    /// Frame currently handed to gpui (dropped from the GPU atlas when replaced).
    shown: Option<(u64, Arc<RenderImage>)>,
    /// Bounds of the viewport, written during prepaint.
    bounds: Rc<Cell<Bounds<Pixels>>>,
    focus: FocusHandle,
    /// Active live-touch gesture: last device position + time of last MOVE.
    drag: Option<((u32, u32), Instant)>,
    zoom: Zoom,
    pan: (f32, f32),
    last_scroll: Instant,
}

impl MirrorView {
    pub fn new(stream: Arc<ScreenStream>, chrome: Chrome, cx: &mut Context<Self>) -> Self {
        // Fast repaint loop: converts + notifies only when a new frame arrived.
        // Also does the (slower-cadence) auto-start check.
        cx.spawn(async move |this, cx| {
            let mut last_check = Instant::now() - Duration::from_secs(1);
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(8))
                    .await;
                let alive = this.update(cx, |this, cx| {
                    if last_check.elapsed() >= Duration::from_millis(250) {
                        last_check = Instant::now();
                        this.poll_auto_start();
                    }
                    let seq = this.stream.latest_seq();
                    if seq != 0 && this.shown.as_ref().map(|(s, _)| *s) != Some(seq) {
                        if let Some(f) = this.stream.state().frame {
                            let img = to_render_image(&f.image);
                            if let Some((_, old)) = this.shown.replace((seq, img)) {
                                cx.drop_image(old, None);
                            }
                            cx.notify();
                        }
                    } else if seq == 0 && this.shown.is_some() {
                        if let Some((_, old)) = this.shown.take() {
                            cx.drop_image(old, None);
                        }
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        })
        .detach();

        Self {
            stream,
            serial: None,
            chrome,
            attach_button: false,
            show_stats: false,
            auto_start: false,
            discover: false,
            discovered: Arc::new(Mutex::new(None)),
            discovering: Arc::new(AtomicBool::new(false)),
            user_stopped: false,
            shown: None,
            bounds: Rc::new(Cell::new(Bounds::default())),
            focus: cx.focus_handle(),
            drag: None,
            zoom: Zoom::Fit,
            pan: (0.0, 0.0),
            last_scroll: Instant::now(),
        }
    }

    // ── configuration ─────────────────────────────────────────────────────

    pub fn stream(&self) -> &Arc<ScreenStream> {
        &self.stream
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    pub fn set_chrome(&mut self, chrome: Chrome, attach_button: bool, cx: &mut Context<Self>) {
        self.chrome = chrome;
        self.attach_button = attach_button;
        cx.notify();
    }

    pub fn set_auto_start(&mut self, on: bool) {
        self.auto_start = on;
    }

    /// Standalone mode: pick the first connected device if no serial is set.
    pub fn set_discover(&mut self, on: bool) {
        self.discover = on;
    }

    pub fn user_stopped(&self) -> bool {
        self.user_stopped
    }

    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }

    /// Set the device to mirror. If the device changed while streaming, the
    /// stream is restarted on the new one.
    pub fn set_serial(&mut self, serial: Option<String>) {
        if self.serial == serial {
            return;
        }
        let was_running = self.stream.is_running();
        self.serial = serial;
        if was_running {
            self.stream.stop();
            self.shown = None;
            if let Some(s) = self.serial.clone() {
                self.stream.start(s);
            }
        }
    }

    fn poll_auto_start(&mut self) {
        if self.serial.is_none() && self.discover {
            let found = self.discovered.lock().ok().and_then(|mut d| d.take());
            if found.is_some() {
                self.serial = found;
            } else if !self.discovering.swap(true, Ordering::SeqCst) {
                let slot = self.discovered.clone();
                let busy = self.discovering.clone();
                std::thread::spawn(move || {
                    let mut adb = droidkraft_core::AdbManager::new();
                    let st = adb.fetch_device_status();
                    if let (Some(d), Ok(mut s)) = (st.active(), slot.lock()) {
                        *s = Some(d.serial.clone());
                    }
                    busy.store(false, Ordering::SeqCst);
                });
            }
        }
        // Don't start when the user stopped it or it stopped itself with an
        // error (e.g. ffmpeg missing).
        if self.auto_start
            && !self.user_stopped
            && !self.stream.is_running()
            && self.stream.state().error.is_none()
        {
            if let Some(serial) = self.serial.clone() {
                self.stream.start(serial);
            }
        }
    }

    pub fn toggle_stream(&mut self) {
        if self.stream.is_running() {
            self.stream.stop();
            self.shown = None;
            self.user_stopped = true;
        } else if let Some(serial) = self.serial.clone() {
            self.user_stopped = false;
            self.stream.start(serial);
        }
    }

    // ── geometry ──────────────────────────────────────────────────────────

    /// Viewport size and the frame rectangle currently drawn inside it.
    fn rect(&self) -> Option<ViewRect> {
        let st = self.stream.state();
        let f = st.frame.as_ref()?;
        let b = self.bounds.get();
        let view = (f32::from(b.size.width), f32::from(b.size.height));
        let dev = oriented_device((f.width, f.height), st.device_size);
        Some((
            view,
            layout(view, (f.width, f.height), dev, self.zoom, self.pan),
        ))
    }

    /// Map a window-space point to device pixels via the viewport bounds.
    fn to_device(&self, pos: Point<Pixels>) -> Option<(u32, u32)> {
        let st = self.stream.state();
        let f = st.frame.as_ref()?;
        let b = self.bounds.get();
        let (_, rect) = self.rect()?;
        map_to_device(
            (f32::from(pos.x - b.origin.x), f32::from(pos.y - b.origin.y)),
            rect,
            (f.width, f.height),
            st.device_size,
        )
    }

    /// Multiply the effective zoom (from Fit's current scale if fitting).
    fn zoom_by(&mut self, k: f32) {
        let st = self.stream.state();
        let Some(f) = st.frame.as_ref() else { return };
        let Some((_, (_, _, w, _))) = self.rect() else {
            return;
        };
        let dev = oriented_device((f.width, f.height), st.device_size);
        let cur = match self.zoom {
            Zoom::Fit => w / dev.0.max(1) as f32,
            Zoom::Scale(z) => z,
        };
        self.zoom = Zoom::Scale((cur * k).clamp(0.2, 4.0));
    }

    fn set_zoom(&mut self, z: Zoom) {
        self.zoom = z;
        self.pan = (0.0, 0.0);
    }

    // ── input ─────────────────────────────────────────────────────────────

    fn mouse_down(&mut self, ev: &MouseDownEvent, window: &mut Window) {
        window.focus(&self.focus);
        match ev.button {
            MouseButton::Left => {
                if let Some(pt) = self.to_device(ev.position) {
                    self.stream.touch("DOWN", pt.0, pt.1);
                    self.drag = Some((pt, Instant::now()));
                }
            }
            MouseButton::Right => self.stream.key("KEYCODE_BACK"),
            MouseButton::Middle => self.stream.key("KEYCODE_HOME"),
            _ => {}
        }
    }

    fn mouse_move(&mut self, ev: &MouseMoveEvent) {
        let Some((_, last)) = self.drag else { return };
        if ev.pressed_button != Some(MouseButton::Left)
            || last.elapsed() < Duration::from_millis(16)
        {
            return;
        }
        if let Some(pt) = self.to_device(ev.position) {
            self.stream.touch("MOVE", pt.0, pt.1);
            self.drag = Some((pt, Instant::now()));
        }
    }

    fn mouse_up(&mut self, ev: Option<&MouseUpEvent>) {
        let Some((last_pt, _)) = self.drag.take() else {
            return;
        };
        let pt = ev
            .and_then(|e| self.to_device(e.position))
            .unwrap_or(last_pt);
        self.stream.touch("UP", pt.0, pt.1);
    }

    fn scroll(&mut self, ev: &ScrollWheelEvent) {
        let (dx, dy) = match ev.delta {
            ScrollDelta::Pixels(p) => (f32::from(p.x), f32::from(p.y)),
            ScrollDelta::Lines(l) => (l.x * 40.0, l.y * 40.0),
        };
        if ev.modifiers.platform || ev.modifiers.control {
            self.zoom_by(if dy > 0.0 { 1.1 } else { 1.0 / 1.1 });
            return;
        }
        if ev.modifiers.shift {
            self.pan = (self.pan.0 + dx, self.pan.1 + dy);
            return;
        }
        if self.last_scroll.elapsed() < Duration::from_millis(90) {
            return;
        }
        let Some((x, y)) = self.to_device(ev.position) else {
            return;
        };
        if dy.abs() < 1.0 {
            return;
        }
        self.last_scroll = Instant::now();
        // Wheel down = content up = finger swipes up.
        let span = (dy.abs() * 6.0).clamp(120.0, 700.0) as u32;
        let (y1, y2) = if dy < 0.0 {
            (y, y.saturating_add(span))
        } else {
            (y.saturating_add(span), y)
        };
        let (y1, y2) = (y1.min(u32::MAX / 2), y2.min(u32::MAX / 2));
        self.stream.swipe(x, y1, x, y2, 120);
    }

    fn key_down(&mut self, ev: &KeyDownEvent) {
        let ks = &ev.keystroke;
        // Never hijack Cmd/Ctrl shortcuts (copy, detach, close window, …).
        if ks.modifiers.platform || ks.modifiers.control {
            return;
        }
        let code = match ks.key.as_str() {
            "enter" => "KEYCODE_ENTER",
            "backspace" => "KEYCODE_DEL",
            "delete" => "KEYCODE_FORWARD_DEL",
            "escape" => "KEYCODE_BACK",
            "tab" => "KEYCODE_TAB",
            "up" => "KEYCODE_DPAD_UP",
            "down" => "KEYCODE_DPAD_DOWN",
            "left" => "KEYCODE_DPAD_LEFT",
            "right" => "KEYCODE_DPAD_RIGHT",
            _ => {
                if let Some(c) = ks.key_char.as_deref() {
                    self.stream.text(c);
                }
                return;
            }
        };
        self.stream.key(code);
    }

    // ── rendering ─────────────────────────────────────────────────────────

    fn btn(
        &self,
        id: &'static str,
        label: impl Into<String>,
        bg: u32,
        fg: u32,
        cx: &mut Context<Self>,
        on_click: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> AnyElement {
        let compact = self.chrome != Chrome::Full;
        let mut d = div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .bg(rgb(bg))
            .text_color(rgb(fg))
            .cursor_pointer()
            .border_1()
            .border_color(rgb(theme::BORDER))
            .child(label.into());
        d = if compact {
            d.px_2().py_1().text_xs()
        } else {
            d.px_3().py_2().text_sm()
        };
        d.on_click(cx.listener(move |this, _ev: &ClickEvent, _w, cx| {
            on_click(this, cx);
            cx.notify();
        }))
        .into_any_element()
    }

    fn key_btn(
        &self,
        id: &'static str,
        label: &'static str,
        code: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.btn(id, label, theme::BG_ELEV, theme::TEXT, cx, move |t, _| {
            t.stream.key(code)
        })
    }

    fn section_title(text: &str) -> AnyElement {
        div()
            .text_xs()
            .text_color(rgb(theme::TEXT_FAINT))
            .child(text.to_uppercase())
            .into_any_element()
    }

    fn zoom_buttons(
        &self,
        presets: &[(&'static str, &'static str, Zoom)],
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut out = vec![
            self.btn("z-out", "−", theme::BG_ELEV, theme::TEXT, cx, |t, _| {
                t.zoom_by(1.0 / 1.25)
            }),
            self.btn("z-in", "+", theme::BG_ELEV, theme::TEXT, cx, |t, _| {
                t.zoom_by(1.25)
            }),
        ];
        for &(id, label, z) in presets {
            let on = self.zoom == z;
            out.push(self.btn(
                id,
                label,
                if on {
                    theme::ACCENT_DIM
                } else {
                    theme::BG_ELEV
                },
                theme::TEXT,
                cx,
                move |t, _| t.set_zoom(z),
            ));
        }
        out
    }

    fn toggle_button(&self, cx: &mut Context<Self>) -> AnyElement {
        let running = self.stream.is_running();
        self.btn(
            "screen-toggle",
            match (self.chrome, running) {
                (Chrome::Full, true) => "■ Stop Mirror",
                (Chrome::Full, false) => "▶ Start Mirror",
                (_, true) => "■ Stop",
                (_, false) => "▶ Start",
            },
            if running {
                theme::DANGER
            } else {
                theme::ACCENT_DIM
            },
            0xffffff,
            cx,
            |t, _| t.toggle_stream(),
        )
    }

    fn viewport(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let st = self.stream.state();
        let running = self.stream.is_running();
        let res = resolution(&st);
        let bounds_cell = self.bounds.clone();
        let focused = self.focus.is_focused(window);
        let inner: AnyElement = match (&self.shown, &st.error) {
            (Some((_, image)), _) => {
                let (x, y, w, h) = self.rect().map(|(_, r)| r).unwrap_or_default();
                img(image.clone())
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .w(px(w))
                    .h(px(h))
                    .object_fit(ObjectFit::Fill)
                    .into_any_element()
            }
            (None, Some(err)) => empty_state("Mirror error", err),
            (None, None) if running => {
                empty_state("Waiting for first frame…", "Needs adb + ffmpeg.")
            }
            _ if self.serial.is_none() => empty_state("No device", "Connect an authorised device."),
            _ => empty_state(
                "Screen mirror stopped",
                "Press Start to stream the device screen.",
            ),
        };
        let mut vp = div()
            .id("screen-viewport")
            .relative()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .rounded_lg()
            .bg(rgb(0x05060a))
            .border_1()
            .border_color(rgb(if focused {
                theme::ACCENT
            } else {
                theme::BORDER
            }))
            .overflow_hidden()
            .track_focus(&self.focus)
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, ev: &MouseDownEvent, window, cx| {
                    this.mouse_down(ev, window);
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, ev: &MouseDownEvent, window, _cx| this.mouse_down(ev, window)),
            )
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(|this, ev: &MouseDownEvent, window, _cx| this.mouse_down(ev, window)),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, ev: &MouseUpEvent, _w, _cx| this.mouse_up(Some(ev))),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _ev: &MouseUpEvent, _w, _cx| this.mouse_up(None)),
            )
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _w, _cx| this.mouse_move(ev)))
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, _w, _cx| this.scroll(ev)))
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _w, _cx| this.key_down(ev)))
            .child(
                canvas(move |b, _w, _cx| bounds_cell.set(b), |_b, _s, _w, _cx| {})
                    .absolute()
                    .size_full(),
            )
            .child(div().absolute().top_0().left_0().size_full().child(inner))
            .child(
                div()
                    .absolute()
                    .top_2()
                    .left_2()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(gpui::rgba(0x000000b0))
                    .text_xs()
                    .text_color(rgb(if st.is_still {
                        theme::TEXT_DIM
                    } else {
                        theme::OK
                    }))
                    .child(if running {
                        format!("{:.0} fps · {} · {} frames", st.fps, res, st.frame_count)
                    } else {
                        "stopped".to_string()
                    }),
            );
        if self.chrome == Chrome::Compact && self.show_stats {
            vp = vp.child(
                div()
                    .absolute()
                    .top_2()
                    .right_2()
                    .w(px(170.))
                    .child(stats_card(&st, &res)),
            );
        }
        vp.into_any_element()
    }

    fn side_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let st = self.stream.state();
        let res = resolution(&st);
        div()
            .flex()
            .flex_col()
            .gap_3()
            .w(px(220.))
            .flex_none()
            .child(self.toggle_button(cx))
            .child(Self::section_title("Navigation"))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(self.key_btn("k-back", "◀ Back", "KEYCODE_BACK", cx))
                    .child(self.key_btn("k-home", "● Home", "KEYCODE_HOME", cx))
                    .child(self.key_btn("k-recent", "▣", "KEYCODE_APP_SWITCH", cx)),
            )
            .child(Self::section_title("Hardware"))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(self.key_btn("k-power", "⏻ Power", "KEYCODE_POWER", cx))
                    .child(self.key_btn("k-volup", "Vol +", "KEYCODE_VOLUME_UP", cx))
                    .child(self.key_btn("k-voldn", "Vol −", "KEYCODE_VOLUME_DOWN", cx))
                    .child(self.key_btn("k-notif", "Notifs", "KEYCODE_NOTIFICATION", cx)),
            )
            .child(Self::section_title("Zoom"))
            .child(div().flex().flex_wrap().gap_2().children(self.zoom_buttons(
                &[
                    ("z-fit", "Fit", Zoom::Fit),
                    ("z-50", "50%", Zoom::Scale(0.5)),
                    ("z-100", "1:1", Zoom::Scale(1.0)),
                    ("z-150", "150%", Zoom::Scale(1.5)),
                    ("z-200", "200%", Zoom::Scale(2.0)),
                ],
                cx,
            )))
            .child(Self::section_title("Stats"))
            .child(stats_card(&st, &res))
            .child(Self::section_title("Controls"))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_DIM))
                    .child("Click = tap · drag = swipe · hold = long-press")
                    .child(div().child("Wheel = scroll · right-click = Back"))
                    .child(div().child("Middle-click = Home · click view, then type"))
                    .child(div().child("⌘/Ctrl+wheel = zoom · Shift+wheel = pan")),
            )
            .into_any_element()
    }

    fn control_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let compact = self.chrome == Chrome::Compact;
        let mut bar = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap_1()
            .child(self.toggle_button(cx))
            .child(self.key_btn("k-back", "◀", "KEYCODE_BACK", cx))
            .child(self.key_btn("k-home", "●", "KEYCODE_HOME", cx));
        if compact {
            bar = bar.child(self.key_btn("k-recent", "▣", "KEYCODE_APP_SWITCH", cx));
        }
        let presets: &[(&str, &str, Zoom)] = if compact {
            &[
                ("z-fit", "Fit", Zoom::Fit),
                ("z-100", "1:1", Zoom::Scale(1.0)),
            ]
        } else {
            &[("z-fit", "Fit", Zoom::Fit)]
        };
        bar = bar
            .child(div().w(px(8.)))
            .children(self.zoom_buttons(presets, cx));
        if compact {
            let on = self.show_stats;
            bar = bar.child(self.btn(
                "stats-toggle",
                "Stats",
                if on {
                    theme::ACCENT_DIM
                } else {
                    theme::BG_ELEV
                },
                theme::TEXT,
                cx,
                |t, _| t.show_stats = !t.show_stats,
            ));
        }
        if self.attach_button {
            bar = bar.child(div().flex_1()).child(self.btn(
                "attach",
                "⇲ Attach",
                theme::ACCENT_DIM,
                0xffffff,
                cx,
                |_, cx| cx.emit(MirrorEvent::AttachRequested),
            ));
        }
        bar.into_any_element()
    }
}

impl gpui::Render for MirrorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport = self.viewport(window, cx);
        match self.chrome {
            Chrome::Full => div()
                .flex()
                .flex_1()
                .min_h_0()
                .size_full()
                .gap_4()
                .child(viewport)
                .child(self.side_panel(cx)),
            _ => div()
                .flex()
                .flex_col()
                .size_full()
                .gap_2()
                .p_2()
                .child(self.control_bar(cx))
                .child(viewport),
        }
    }
}

fn resolution(st: &droidkraft_mirror::ScreenState) -> String {
    st.frame
        .as_ref()
        .map(|f| format!("{}×{}", f.width, f.height))
        .unwrap_or_else(|| "—".into())
}

fn stats_card(st: &droidkraft_mirror::ScreenState, res: &str) -> AnyElement {
    let stat = |k: &str, v: String| {
        div()
            .flex()
            .justify_between()
            .text_xs()
            .child(div().text_color(rgb(theme::TEXT_DIM)).child(k.to_string()))
            .child(div().text_color(rgb(theme::TEXT)).child(v))
    };
    div()
        .flex()
        .flex_col()
        .gap_1()
        .p_2()
        .rounded_md()
        .bg(rgb(theme::BG_ELEV))
        .child(stat("FPS", format!("{:.0}", st.fps)))
        .child(stat("Resolution", res.to_string()))
        .child(stat("Frames", st.frame_count.to_string()))
        .child(stat("Dropped", st.dropped.to_string()))
        .child(stat("Decode", format!("{:.1} ms", st.decode_ms)))
        .child(stat("Data", format!("{:.1} MB/s", st.mbps)))
        .child(stat(
            "Source",
            if st.is_still { "still" } else { "video" }.into(),
        ))
        .into_any_element()
}

pub(crate) fn empty_state(title: &str, subtitle: &str) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .size_full()
        .child(
            div()
                .text_lg()
                .text_color(rgb(theme::TEXT_DIM))
                .child(title.to_string()),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(theme::TEXT_FAINT))
                .child(subtitle.to_string()),
        )
        .into_any_element()
}

/// RGBA frame → gpui `RenderImage` (BGRA, gpui's native layout).
fn to_render_image(rgba: &image::RgbaImage) -> Arc<RenderImage> {
    let mut bgra = rgba.clone();
    let raw: &mut [u8] = &mut bgra;
    for px in raw.as_chunks_mut::<4>().0 {
        px.swap(0, 2);
    }
    Arc::new(RenderImage::new(vec![image::Frame::new(bgra)]))
}

/// Root view of the detached mirror window: hosts the shared [`MirrorView`]
/// plus a capture bar (record / stop / screenshot).
pub struct DetachedRoot {
    mirror: gpui::Entity<MirrorView>,
    app: gpui::WeakEntity<crate::app::DroidGui>,
    serial: String,
}

impl DetachedRoot {
    pub fn new(
        mirror: gpui::Entity<MirrorView>,
        app: gpui::WeakEntity<crate::app::DroidGui>,
        serial: String,
        cx: &mut Context<Self>,
    ) -> Self {
        // Repaint (elapsed timer) whenever the mirror repaints.
        cx.observe(&mirror, |_, _, cx| cx.notify()).detach();
        Self {
            mirror,
            app,
            serial,
        }
    }
}

impl gpui::Render for DetachedRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use droidkraft_mirror::RecordStatus;
        let (status, elapsed, note) = self
            .app
            .upgrade()
            .and_then(|a| {
                a.read(cx).fleet.get(&self.serial).map(|d| {
                    (
                        d.recorder.status(),
                        d.recorder.elapsed(),
                        d.capture_status.clone(),
                    )
                })
            })
            .unwrap_or((RecordStatus::Idle, None, None));
        let recording = matches!(status, RecordStatus::Recording { .. });
        let busy = recording || matches!(status, RecordStatus::Finishing);

        let bar_btn =
            |id: &'static str, label: String, bg: u32, cb: fn(&mut crate::app::DroidGui, &str)| {
                let app = self.app.clone();
                let ser = self.serial.clone();
                div()
                    .id(id)
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .bg(rgb(bg))
                    .text_xs()
                    .text_color(rgb(0xffffff))
                    .cursor_pointer()
                    .child(label)
                    .on_click(move |_, _, cx| {
                        let _ = app.update(cx, |t, cx| {
                            cb(t, &ser);
                            cx.notify();
                        });
                    })
            };
        let rec = if busy {
            bar_btn("d-stop", "■ Stop".into(), theme::DANGER, |t, s| {
                t.stop_recording(s)
            })
        } else {
            bar_btn("d-rec", "● Record".into(), theme::ACCENT_DIM, |t, s| {
                t.start_recording(s)
            })
        };
        let shot = bar_btn(
            "d-shot",
            "📷 Screenshot".into(),
            theme::BG_ELEV,
            |t, s| t.take_screenshot(s),
        );
        let label = if recording {
            elapsed
                .map(|e| format!("● REC {}", crate::fmt::fmt_elapsed(e)))
                .unwrap_or_default()
        } else {
            note.unwrap_or_default()
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(theme::BG))
            .text_color(rgb(theme::TEXT))
            .font_family("sans-serif")
            .on_action(cx.listener(|this, _: &ToggleDetach, _w, cx| {
                this.mirror
                    .update(cx, |_, cx| cx.emit(MirrorEvent::AttachRequested));
            }))
            .on_action(cx.listener(|_, _: &CloseWindow, window, _cx| window.remove_window()))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .child(rec)
                    .child(shot)
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(theme::TEXT_DIM))
                            .child(label),
                    ),
            )
            .child(div().flex_1().min_h_0().child(self.mirror.clone()))
    }
}

/// Root view of the standalone `--mirror` window: mirror + capture bar
/// (record / stop / screenshot) with its own recorder.
pub struct StandaloneRoot {
    mirror: gpui::Entity<MirrorView>,
    recorder: droidkraft_mirror::ScreenRecorder,
    note: Option<String>,
    shot: Arc<Mutex<Option<String>>>,
}

impl StandaloneRoot {
    pub fn new(mirror: gpui::Entity<MirrorView>, cx: &mut Context<Self>) -> Self {
        cx.observe(&mirror, |_, _, cx| cx.notify()).detach();
        Self {
            mirror,
            recorder: droidkraft_mirror::ScreenRecorder::new(),
            note: None,
            shot: Arc::new(Mutex::new(None)),
        }
    }

    fn start(&mut self, serial: &str) {
        let name = droidkraft_mirror::timestamped_name(serial);
        self.note = self
            .recorder
            .start(
                serial,
                &droidkraft_mirror::default_output_dir(),
                &name,
                Default::default(),
            )
            .err()
            .map(|e| format!("Cannot record: {e}"));
    }

    fn screenshot(&mut self, serial: &str) {
        let path = droidkraft_mirror::default_output_dir().join(format!(
            "{}.png",
            droidkraft_mirror::timestamped_name(serial)
        ));
        self.note = Some("Capturing screenshot…".into());
        let slot = self.shot.clone();
        std::thread::spawn(move || {
            let mut adb = droidkraft_core::AdbManager::new();
            let msg = match crate::worker::screenshot(&mut adb, &path) {
                Ok(p) => format!("Saved {}", p.display()),
                Err(e) => format!("Screenshot failed: {e}"),
            };
            *slot.lock().unwrap() = Some(msg);
        });
    }
}

impl gpui::Render for StandaloneRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use droidkraft_mirror::RecordStatus;
        if let Some(m) = self.shot.lock().unwrap().take() {
            self.note = Some(m);
        }
        let status = self.recorder.status();
        let recording = matches!(status, RecordStatus::Recording { .. });
        let busy = recording || matches!(status, RecordStatus::Finishing);
        let serial = self.mirror.read(cx).serial().map(str::to_string);
        let mk = |id: &'static str, label: &str, bg: u32| {
            div()
                .id(id)
                .px_2()
                .py_1()
                .rounded_md()
                .bg(rgb(bg))
                .text_xs()
                .text_color(rgb(0xffffff))
                .cursor_pointer()
                .child(label.to_string())
        };
        let ser = serial.clone();
        let rec = if busy {
            mk("s-stop", "■ Stop", theme::DANGER).on_click(cx.listener(|t, _, _, cx| {
                t.recorder.stop();
                cx.notify();
            }))
        } else {
            mk("s-rec", "● Record", theme::ACCENT_DIM).on_click(cx.listener(move |t, _, _, cx| {
                if let Some(s) = &ser {
                    t.start(s);
                }
                cx.notify();
            }))
        };
        let ser = serial;
        let shot = mk("s-shot", "📷 Screenshot", theme::BG_ELEV).on_click(cx.listener(
            move |t, _, _, cx| {
                if let Some(s) = &ser {
                    t.screenshot(s);
                }
                cx.notify();
            },
        ));
        let label = if recording {
            self.recorder
                .elapsed()
                .map(|e| format!("● REC {}", crate::fmt::fmt_elapsed(e)))
                .unwrap_or_default()
        } else if let RecordStatus::Failed(e) = &status {
            format!("Recording failed: {e}")
        } else {
            self.note.clone().unwrap_or_default()
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(theme::BG))
            .text_color(rgb(theme::TEXT))
            .font_family("sans-serif")
            .on_action(cx.listener(|_, _: &CloseWindow, window, _cx| window.remove_window()))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .child(rec)
                    .child(shot)
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(theme::TEXT_DIM))
                            .child(label),
                    ),
            )
            .child(div().flex_1().min_h_0().child(self.mirror.clone()))
    }
}
