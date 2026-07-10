//! The root gpui view: a left navigation rail plus one of five panels —
//! Dashboard, Logs, Commands, Flash/Root, and Screen mirror.

use std::time::Duration;

use gpui::{div, img, prelude::*, px, rgb, AnyElement, ClickEvent, Context, Window};

use droidtui_core::features::fastboot::FastbootCommand;
use droidtui_core::features::flash::{RebootTarget, RootStatus};
use droidtui_core::{DeviceStatus, LogEntry, LogLevel, LogcatFilter, LogcatStream};

use crate::commands::{by_category, CommandCategory, GuiCommand};
use crate::screen::ScreenStream;
use crate::theme;
use crate::worker::{Worker, WorkerRequest, WorkerResponse};

/// Which panel is currently shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Dashboard,
    Logs,
    Commands,
    Flash,
    Screen,
}

impl Panel {
    fn label(&self) -> &'static str {
        match self {
            Panel::Dashboard => "Dashboard",
            Panel::Logs => "Live Logs",
            Panel::Commands => "Commands",
            Panel::Flash => "Flash & Root",
            Panel::Screen => "Screen",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Panel::Dashboard => "▦",
            Panel::Logs => "≣",
            Panel::Commands => "⌘",
            Panel::Flash => "⚡",
            Panel::Screen => "▶",
        }
    }

    fn all() -> &'static [Panel] {
        &[
            Panel::Dashboard,
            Panel::Logs,
            Panel::Commands,
            Panel::Flash,
            Panel::Screen,
        ]
    }
}

/// Maximum number of log entries retained in the GUI.
const MAX_LOG_ENTRIES: usize = 20_000;
/// Maximum number of visible (filtered) log rows rendered per frame.
const MAX_VISIBLE_LOGS: usize = 500;

/// Root application state + view.
pub struct DroidGui {
    worker: Worker,
    active: Panel,
    status: DeviceStatus,

    log_stream: LogcatStream,
    log_entries: Vec<LogEntry>,
    log_filter: LogcatFilter,
    log_streaming: bool,

    screen: ScreenStream,

    last_output: Option<(String, Result<String, String>)>,
    root_status: Option<RootStatus>,
    busy_label: Option<String>,
}

impl DroidGui {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Periodic refresh loop (drains worker + logcat, then repaints).
        cx.spawn(async move |this, cx| loop {
            let alive = this.update(cx, |this, cx| {
                this.tick();
                cx.notify();
            });
            if alive.is_err() {
                break;
            }
            cx.background_executor()
                .timer(Duration::from_millis(250))
                .await;
        })
        .detach();

        Self {
            worker: Worker::spawn(),
            active: Panel::Dashboard,
            status: DeviceStatus::default(),
            log_stream: LogcatStream::new(),
            log_entries: Vec::new(),
            log_filter: LogcatFilter::default(),
            log_streaming: false,
            screen: ScreenStream::new(),
            last_output: None,
            root_status: None,
            busy_label: None,
        }
    }

    /// The active device serial, if connected and authorised.
    fn active_serial(&self) -> Option<String> {
        self.status.active().map(|d| d.serial.clone())
    }

    // ── per-tick data pump ────────────────────────────────────────────────

    fn tick(&mut self) {
        // Drain worker responses.
        for resp in self.worker.drain() {
            match resp {
                WorkerResponse::Status(s) => self.status = s,
                WorkerResponse::Output { label, result } => {
                    self.busy_label = None;
                    self.last_output = Some((label, result));
                }
                WorkerResponse::Root(result) => {
                    self.busy_label = None;
                    match result {
                        Ok(status) => self.root_status = Some(status),
                        Err(e) => self.last_output = Some(("Root check".into(), Err(e))),
                    }
                }
            }
        }

        // Drain the logcat stream.
        if self.log_stream.is_running() {
            let mut lines = Vec::new();
            let (_n, _status) = self.log_stream.drain_into(&mut lines, 2000);
            for line in lines {
                self.log_entries.push(LogEntry::parse(&line));
            }
            if self.log_entries.len() > MAX_LOG_ENTRIES {
                let excess = self.log_entries.len() - MAX_LOG_ENTRIES;
                self.log_entries.drain(0..excess);
            }
        }
    }

    // ── actions ───────────────────────────────────────────────────────────

    fn run_command(&mut self, cmd: &GuiCommand) {
        self.busy_label = Some(cmd.label.to_string());
        self.worker.send(WorkerRequest::Run {
            label: cmd.label.to_string(),
            action: cmd.action.clone(),
        });
    }

    fn toggle_log_stream(&mut self) {
        if self.log_streaming {
            self.log_stream.stop();
            self.log_streaming = false;
        } else if let Some(serial) = self.active_serial() {
            self.log_entries.clear();
            self.log_stream.start(serial);
            self.log_streaming = true;
        }
    }

    fn toggle_screen(&mut self) {
        if self.screen.is_running() {
            self.screen.stop();
        } else if let Some(serial) = self.active_serial() {
            self.screen.start(serial);
        }
    }

    fn detect_root(&mut self) {
        self.busy_label = Some("Root check".into());
        self.worker.send(WorkerRequest::DetectRoot);
    }

    fn reboot(&mut self, target: RebootTarget) {
        self.busy_label = Some(target.label().into());
        self.worker.send(WorkerRequest::Run {
            label: target.label().into(),
            action: crate::commands::CommandAction::Reboot(target),
        });
    }

    fn fastboot(&mut self, command: FastbootCommand) {
        let label = command.label().to_string();
        self.busy_label = Some(label.clone());
        self.worker.send(WorkerRequest::Fastboot { label, command });
    }

    // ── shared widgets ────────────────────────────────────────────────────

    fn pill(label: impl Into<String>, value: impl Into<String>, color: u32) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .px_3()
            .py_2()
            .min_w(px(120.))
            .rounded_md()
            .bg(rgb(theme::BG_ELEV))
            .border_1()
            .border_color(rgb(theme::BORDER))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(label.into()),
            )
            .child(div().text_lg().text_color(rgb(color)).child(value.into()))
            .into_any_element()
    }

    fn button(
        &self,
        id: &'static str,
        label: impl Into<String>,
        bg: u32,
        fg: u32,
        cx: &mut Context<Self>,
        on_click: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> AnyElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(rgb(bg))
            .text_sm()
            .text_color(rgb(fg))
            .cursor_pointer()
            .border_1()
            .border_color(rgb(theme::BORDER))
            .child(label.into())
            .on_click(cx.listener(move |this, _ev: &ClickEvent, _window, cx| {
                on_click(this, cx);
                cx.notify();
            }))
            .into_any_element()
    }

    fn section_title(text: &str) -> AnyElement {
        div()
            .text_xs()
            .text_color(rgb(theme::TEXT_FAINT))
            .child(text.to_uppercase())
            .into_any_element()
    }

    // ── panels ────────────────────────────────────────────────────────────

    fn render_nav(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut rail = div()
            .flex()
            .flex_col()
            .gap_1()
            .w(px(180.))
            .h_full()
            .p_2()
            .bg(rgb(theme::BG_PANEL))
            .border_r_1()
            .border_color(rgb(theme::BORDER))
            .child(
                div()
                    .px_2()
                    .py_2()
                    .text_lg()
                    .text_color(rgb(theme::ACCENT))
                    .child("DroidTUI"),
            );

        for panel in Panel::all() {
            let panel = *panel;
            let active = panel == self.active;
            let bg = if active {
                theme::BG_ELEV
            } else {
                theme::BG_PANEL
            };
            let fg = if active { theme::TEXT } else { theme::TEXT_DIM };
            rail = rail.child(
                div()
                    .id(panel.label())
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_2()
                    .rounded_md()
                    .bg(rgb(bg))
                    .text_color(rgb(fg))
                    .cursor_pointer()
                    .child(div().text_sm().child(panel.icon()))
                    .child(div().text_sm().child(panel.label()))
                    .on_click(cx.listener(move |this, _ev: &ClickEvent, _window, cx| {
                        this.active = panel;
                        cx.notify();
                    })),
            );
        }

        // Connection indicator.
        let (dot, label) = if self.status.is_connected() {
            (theme::OK, format!("{} online", self.status.devices.len()))
        } else {
            (theme::ERR, "no device".to_string())
        };
        rail = rail.child(div().flex_grow_1()).child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .px_2()
                .py_2()
                .child(div().size_2().rounded_full().bg(rgb(dot)))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme::TEXT_DIM))
                        .child(label),
                ),
        );

        rail.into_any_element()
    }

    fn render_dashboard(&self, cx: &mut Context<Self>) -> AnyElement {
        let s = &self.status;
        let connected = s.is_connected();

        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(theme::TEXT))
                    .child("Dashboard"),
            )
            .child(self.button(
                "refresh",
                "Refresh",
                theme::BG_ELEV,
                theme::TEXT,
                cx,
                |this, _cx| this.worker.send(WorkerRequest::RefreshStatus),
            ));

        let body: AnyElement = if connected {
            div()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_3()
                        .flex_wrap()
                        .child(Self::pill(
                            "Model",
                            nonempty(&s.model, "unknown"),
                            theme::TEXT,
                        ))
                        .child(Self::pill(
                            "Android",
                            nonempty(&s.android_version, "?"),
                            theme::TEXT,
                        ))
                        .child(Self::pill(
                            "Battery",
                            format!("{}%", s.battery_pct),
                            battery_color(s.battery_pct),
                        ))
                        .child(Self::pill(
                            "RAM",
                            format!("{} / {} MiB", s.ram_used_mib(), s.ram_total_mib),
                            theme::TEXT,
                        ))
                        .child(Self::pill(
                            "CPU load",
                            format!("{:.2}", s.cpu_load_1min),
                            theme::TEXT,
                        )),
                )
                .child(Self::section_title("Devices"))
                .children(s.devices.iter().enumerate().map(|(i, d)| {
                    let active = i == s.selected_idx;
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(rgb(if active {
                            theme::BG_ELEV
                        } else {
                            theme::BG_PANEL
                        }))
                        .child(div().size_2().rounded_full().bg(rgb(if d.is_online() {
                            theme::OK
                        } else {
                            theme::WARN
                        })))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme::TEXT))
                                .child(d.serial.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme::TEXT_DIM))
                                .child(d.state.clone()),
                        )
                        .into_any_element()
                }))
                .into_any_element()
        } else {
            empty_state(
                "No device connected",
                "Connect a device over USB with debugging enabled.",
            )
        };

        panel_container(Panel::Dashboard)
            .child(header)
            .child(body)
            .into_any_element()
    }

    fn render_commands(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut root = panel_container(Panel::Commands).child(
            div()
                .text_xl()
                .text_color(rgb(theme::TEXT))
                .child("Commands"),
        );

        for cat in CommandCategory::all() {
            let mut row = div().flex().flex_row().flex_wrap().gap_2();
            for cmd in by_category(*cat) {
                let (bg, fg) = if cmd.destructive {
                    (theme::DANGER, 0xffffff)
                } else {
                    (theme::BG_ELEV, theme::TEXT)
                };
                let id: &'static str = cmd.label;
                let cmd_clone = cmd.clone();
                row = row.child(self.button(id, cmd.label, bg, fg, cx, move |this, _cx| {
                    this.run_command(&cmd_clone)
                }));
            }
            root = root.child(Self::section_title(cat.label())).child(row);
        }

        root.child(self.render_output()).into_any_element()
    }

    fn render_output(&self) -> AnyElement {
        let content: AnyElement = match &self.last_output {
            None => div()
                .text_sm()
                .text_color(rgb(theme::TEXT_FAINT))
                .child("Command output will appear here.")
                .into_any_element(),
            Some((label, result)) => {
                let (color, text) = match result {
                    Ok(out) => (theme::OK, out.clone()),
                    Err(e) => (theme::ERR, e.clone()),
                };
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_xs().text_color(rgb(color)).child(label.clone()))
                    .child(
                        div()
                            .font_family("monospace")
                            .text_xs()
                            .text_color(rgb(theme::TEXT))
                            .child(truncate(&text, 4000)),
                    )
                    .into_any_element()
            }
        };

        div()
            .id("cmd-output")
            .mt_3()
            .p_3()
            .h(px(240.))
            .w_full()
            .rounded_md()
            .bg(rgb(theme::BG))
            .border_1()
            .border_color(rgb(theme::BORDER))
            .overflow_y_scroll()
            .child(content)
            .into_any_element()
    }

    fn render_flash(&self, cx: &mut Context<Self>) -> AnyElement {
        let root_line: AnyElement = match &self.root_status {
            None => div()
                .text_sm()
                .text_color(rgb(theme::TEXT_DIM))
                .child("Root status unknown — run a check.")
                .into_any_element(),
            Some(rs) => {
                let (color, text) = if rs.is_rooted {
                    (
                        theme::OK,
                        format!(
                            "Rooted via {}{}",
                            rs.method.label(),
                            rs.magisk_version
                                .as_ref()
                                .map(|v| format!(" (Magisk {v})"))
                                .unwrap_or_default()
                        ),
                    )
                } else {
                    (theme::WARN, "Not rooted".to_string())
                };
                div()
                    .text_sm()
                    .text_color(rgb(color))
                    .child(text)
                    .into_any_element()
            }
        };

        let reboots = RebootTarget::all().iter().fold(
            div().flex().flex_row().flex_wrap().gap_2(),
            |row, &target| {
                let id: &'static str = target.label();
                row.child(self.button(
                    id,
                    target.label(),
                    theme::BG_ELEV,
                    theme::TEXT,
                    cx,
                    move |this, _cx| this.reboot(target),
                ))
            },
        );

        let fastboot_cmds = [
            ("fb-unlock", "Unlock Bootloader", FastbootCommand::OemUnlock),
            ("fb-lock", "Lock Bootloader", FastbootCommand::OemLock),
            ("fb-wipe", "Wipe Data", FastbootCommand::WipeData),
            ("fb-reboot", "Fastboot Reboot", FastbootCommand::Reboot),
            ("fb-getvar", "Get Variables", FastbootCommand::GetVarAll),
        ];
        let mut fb_row = div().flex().flex_row().flex_wrap().gap_2();
        for (id, label, cmd) in fastboot_cmds {
            let destructive = cmd.is_destructive();
            let (bg, fg) = if destructive {
                (theme::DANGER, 0xffffff)
            } else {
                (theme::BG_ELEV, theme::TEXT)
            };
            let cmd_clone = cmd.clone();
            fb_row = fb_row.child(self.button(id, label, bg, fg, cx, move |this, _cx| {
                this.fastboot(cmd_clone.clone())
            }));
        }

        panel_container(Panel::Flash)
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(theme::TEXT))
                    .child("Flash & Root"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(self.button(
                        "root-check",
                        "Check Root",
                        theme::ACCENT_DIM,
                        0xffffff,
                        cx,
                        |this, _cx| this.detect_root(),
                    ))
                    .child(self.button(
                        "remount",
                        "Remount RW",
                        theme::BG_ELEV,
                        theme::TEXT,
                        cx,
                        |this, _cx| {
                            this.busy_label = Some("Remount".into());
                            this.worker.send(WorkerRequest::Remount);
                        },
                    ))
                    .child(root_line),
            )
            .child(Self::section_title("Reboot into"))
            .child(reboots)
            .child(Self::section_title(
                "Fastboot (device must be in bootloader)",
            ))
            .child(fb_row)
            .child(self.render_output())
            .into_any_element()
    }

    fn render_logs(&self, cx: &mut Context<Self>) -> AnyElement {
        let toolbar = div()
            .flex()
            .items_center()
            .gap_2()
            .child(self.button(
                "log-toggle",
                if self.log_streaming { "Stop" } else { "Start" },
                if self.log_streaming {
                    theme::DANGER
                } else {
                    theme::ACCENT_DIM
                },
                0xffffff,
                cx,
                |this, _cx| this.toggle_log_stream(),
            ))
            .child(self.button(
                "log-clear",
                "Clear",
                theme::BG_ELEV,
                theme::TEXT,
                cx,
                |this, _cx| this.log_entries.clear(),
            ))
            .child(self.button(
                "log-level",
                format!("Level ≥ {}", self.log_filter.min_level.as_char()),
                theme::BG_ELEV,
                theme::TEXT,
                cx,
                |this, _cx| this.log_filter.cycle_level(),
            ))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(format!("{} lines", self.log_entries.len())),
            );

        // Filtered, capped, newest-last.
        let mut rows: Vec<AnyElement> = Vec::new();
        for entry in self
            .log_entries
            .iter()
            .filter(|e| self.log_filter.matches(e))
        {
            rows.push(log_row(entry));
        }
        let start = rows.len().saturating_sub(MAX_VISIBLE_LOGS);
        let visible = rows.split_off(start);

        let list = div()
            .id("log-list")
            .flex()
            .flex_col()
            .w_full()
            .flex_grow_1()
            .mt_2()
            .p_2()
            .rounded_md()
            .bg(rgb(theme::BG))
            .border_1()
            .border_color(rgb(theme::BORDER))
            .overflow_y_scroll()
            .children(visible);

        panel_container(Panel::Logs)
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(theme::TEXT))
                    .child("Live Logs"),
            )
            .child(toolbar)
            .child(list)
            .into_any_element()
    }

    fn render_screen(&self, cx: &mut Context<Self>) -> AnyElement {
        let st = self.screen.state();
        let running = self.screen.is_running();

        let toolbar = div()
            .flex()
            .items_center()
            .gap_2()
            .child(self.button(
                "screen-toggle",
                if running {
                    "Stop Mirror"
                } else {
                    "Start Mirror"
                },
                if running {
                    theme::DANGER
                } else {
                    theme::ACCENT_DIM
                },
                0xffffff,
                cx,
                |this, _cx| this.toggle_screen(),
            ))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(format!("{:.1} fps · {} frames", st.fps, st.frame_count)),
            );

        let view: AnyElement = match (&st.latest_path, &st.error) {
            (Some(path), _) => img(path.clone())
                .max_h(px(640.))
                .rounded_md()
                .into_any_element(),
            (None, Some(err)) => empty_state("Capture error", err),
            (None, None) if running => empty_state("Waiting for first frame…", ""),
            _ => empty_state(
                "Screen mirror stopped",
                "Start the mirror to stream the device screen.",
            ),
        };

        panel_container(Panel::Screen)
            .child(div().text_xl().text_color(rgb(theme::TEXT)).child("Screen"))
            .child(toolbar)
            .child(
                div()
                    .flex()
                    .justify_center()
                    .items_center()
                    .w_full()
                    .flex_grow_1()
                    .mt_2()
                    .child(view),
            )
            .into_any_element()
    }
}

impl Render for DroidGui {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.active {
            Panel::Dashboard => self.render_dashboard(cx),
            Panel::Logs => self.render_logs(cx),
            Panel::Commands => self.render_commands(cx),
            Panel::Flash => self.render_flash(cx),
            Panel::Screen => self.render_screen(cx),
        };

        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(theme::BG))
            .text_color(rgb(theme::TEXT))
            .font_family("sans-serif")
            .child(self.render_nav(cx))
            .child(content)
    }
}

// ── free helpers ───────────────────────────────────────────────────────────

fn panel_container(_panel: Panel) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .size_full()
        .p_4()
        .overflow_hidden()
}

fn empty_state(title: &str, subtitle: &str) -> AnyElement {
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

fn log_row(entry: &LogEntry) -> AnyElement {
    let mut row = div().flex().flex_row().gap_2().py(px(1.));

    row = row.child(
        div()
            .w(px(14.))
            .text_xs()
            .text_color(theme::level_color(entry.level))
            .child(entry.level.as_char().to_string()),
    );

    if let Some(tag) = &entry.tag {
        row = row.child(
            div()
                .min_w(px(120.))
                .max_w(px(180.))
                .text_xs()
                .text_color(theme::tag_color(tag))
                .overflow_hidden()
                .child(tag.clone()),
        );
    }

    row.child(
        div()
            .flex_grow_1()
            .font_family("monospace")
            .text_xs()
            .text_color(theme::level_color(entry.level))
            .child(entry.message.clone()),
    )
    .into_any_element()
}

fn nonempty(s: &str, fallback: &str) -> String {
    if s.trim().is_empty() {
        fallback.to_string()
    } else {
        s.to_string()
    }
}

fn battery_color(pct: u8) -> u32 {
    if pct >= 50 {
        theme::OK
    } else if pct >= 20 {
        theme::WARN
    } else {
        theme::ERR
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…\n[truncated]", &s[..max])
    }
}

/// Suppress an unused-import warning for `LogLevel` when compiled without the
/// `all()` helper; used by the level filter.
#[allow(dead_code)]
fn _levels() -> &'static [LogLevel] {
    LogLevel::all()
}
