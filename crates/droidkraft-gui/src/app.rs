//! The root gpui view: nav rail + either the fleet overview, a device
//! workspace, or settings. All device state lives in [`Fleet`]; this module
//! owns the worker, routing, and per-device actions.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    div, prelude::*, px, rgb, size, Bounds, Context, Entity, TitlebarOptions, Window, WindowBounds,
    WindowOptions,
};

use droidkraft_core::features::fastboot::FastbootCommand;
use droidkraft_core::features::flash::RebootTarget;
use droidkraft_core::features::rom::{
    build_plan, DownloadProgress, FlashOptions, FlashSession, InstallMethod, RomBuild, StepStatus,
};
use droidkraft_mirror::{timestamped_name, RecordStatus};

use crate::commands::{CommandAction, GuiCommand};
use crate::mirror_view::{Chrome, DetachedRoot, MirrorEvent, MirrorView, ToggleDetach};
use crate::state::{DeviceState, Fleet, Route, Tab};
use crate::theme;
use crate::worker::{Worker, WorkerRequest, WorkerResponse};

/// Root application state + view.
pub struct DroidGui {
    pub(crate) worker: Worker,
    pub(crate) fleet: Fleet,
    pub(crate) route: Route,
}

impl DroidGui {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Periodic refresh loop (drains worker + logcat, then repaints).
        cx.spawn(async move |this, cx| loop {
            let alive = this.update(cx, |this, cx| {
                this.tick(cx);
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

        // Main window closing releases this view → close detached mirrors too.
        cx.on_release(|this, cx| {
            for d in this.fleet.iter_mut() {
                d.detach_subs.clear();
                if let Some(h) = d.detached.take() {
                    let _ = h.update(cx, |_, w, _| w.remove_window());
                }
                d.stream.stop();
                d.log_stream.stop();
                d.recorder.stop();
            }
        })
        .detach();

        Self {
            worker: Worker::spawn(),
            fleet: Fleet::new(),
            route: Route::Devices,
        }
    }

    // ── routing ───────────────────────────────────────────────────────────

    /// Show a route and tell the worker which device is focused.
    pub fn set_route(&mut self, route: Route) {
        let focus = self.fleet.open_serial(&route);
        self.route = route;
        self.worker.send(WorkerRequest::Focus(focus));
    }

    /// Open a device workspace on `tab`.
    pub fn open_device(&mut self, serial: &str, tab: Tab, cx: &mut Context<Self>) {
        if let Some(d) = self.fleet.get_mut(serial) {
            d.tab = tab;
        }
        self.set_route(Route::Device(serial.to_string()));
        if tab == Tab::Screen {
            self.ensure_mirror(serial, cx);
        }
    }

    pub fn select_tab(&mut self, serial: &str, tab: Tab, cx: &mut Context<Self>) {
        self.open_device(serial, tab, cx);
    }

    pub(crate) fn dev(&mut self, serial: &str) -> Option<&mut DeviceState> {
        self.fleet.get_mut(serial)
    }

    // ── per-tick data pump ────────────────────────────────────────────────

    fn tick(&mut self, cx: &mut Context<Self>) {
        let open = self.fleet.open_serial(&self.route);
        let on_screen_tab = open
            .as_deref()
            .and_then(|s| self.fleet.get(s))
            .is_some_and(|d| d.tab == Tab::Screen);
        if let (Some(s), true) = (&open, on_screen_tab) {
            let s = s.clone();
            self.ensure_mirror(&s, cx);
        }

        for resp in self.worker.drain() {
            self.apply(resp);
        }

        for d in self.fleet.iter_mut() {
            // Keep each mirror pointed at its device; auto-start while its
            // Screen tab is open or its window is detached.
            if let Some(m) = &d.mirror {
                let auto = d.is_online()
                    && (d.detached.is_some()
                        || (open.as_deref() == Some(d.serial.as_str()) && d.tab == Tab::Screen));
                let serial = d.serial.clone();
                m.update(cx, |m, _| {
                    m.set_serial(Some(serial));
                    m.set_auto_start(auto);
                });
            }
            // Logcat.
            if d.log_stream.is_running() {
                let mut lines = Vec::new();
                let _ = d.log_stream.drain_into(&mut lines, 2000);
                d.push_log_lines(&lines);
            }
            // Recorder finished?
            match d.recorder.status() {
                RecordStatus::Recording { .. } | RecordStatus::Finishing => d.rec_harvested = false,
                RecordStatus::Done { files } if !d.rec_harvested => {
                    d.rec_harvested = true;
                    d.add_recordings(&files);
                    d.capture_status = Some(format!("Saved {} recording file(s)", files.len()));
                }
                RecordStatus::Failed(e) if !d.rec_harvested => {
                    d.rec_harvested = true;
                    d.capture_status = Some(format!("Recording failed: {e}"));
                }
                _ => {}
            }
        }
    }

    /// Route one worker response to its device.
    fn apply(&mut self, resp: WorkerResponse) {
        match resp {
            WorkerResponse::Devices(entries) => {
                self.fleet.apply_entries(&entries);
                // Stop work tied to devices that vanished.
                for d in self.fleet.iter_mut().filter(|d| !d.connected) {
                    if d.log_streaming {
                        d.log_stream.stop();
                        d.log_streaming = false;
                    }
                }
            }
            WorkerResponse::Details {
                serial,
                details,
                history,
            } => {
                self.fleet.apply_details(&serial, *details, *history);
            }
            WorkerResponse::Output {
                serial,
                label,
                result,
            } => {
                if let Some(d) = self.dev(&serial) {
                    d.busy_label = None;
                    d.last_output = Some((label, result));
                }
            }
            WorkerResponse::Root { serial, result } => {
                if let Some(d) = self.dev(&serial) {
                    d.busy_label = None;
                    match result {
                        Ok(s) => d.root_status = Some(s),
                        Err(e) => d.last_output = Some(("Root check".into(), Err(e))),
                    }
                }
            }
            WorkerResponse::Profile { serial, profile } => {
                let mut fetch = None;
                if let Some(d) = self.dev(&serial) {
                    if profile.codename.is_empty() {
                        d.rom_status = Some("Could not detect device codename.".into());
                    } else {
                        d.rom_status = Some(format!("Searching ROMs for {}…", profile.codename));
                        fetch = Some(profile.codename.clone());
                    }
                    d.profile = Some(profile);
                }
                if let Some(codename) = fetch {
                    self.worker
                        .send(WorkerRequest::FetchRoms { serial, codename });
                }
            }
            WorkerResponse::Roms { serial, result } => {
                if let Some(d) = self.dev(&serial) {
                    match result {
                        Ok(builds) => {
                            d.rom_status = builds.is_empty().then(|| {
                                "No downloadable builds found for this device.".to_string()
                            });
                            d.rom_builds = builds;
                        }
                        Err(e) => d.rom_status = Some(format!("ROM lookup failed: {e}")),
                    }
                }
            }
            WorkerResponse::DownloadProgress { serial, progress } => {
                if let Some(d) = self.dev(&serial) {
                    d.download_progress = Some(progress);
                }
            }
            WorkerResponse::Downloaded { serial, result } => {
                if let Some(d) = self.dev(&serial) {
                    d.download_progress = None;
                    match result {
                        Ok(path) => {
                            d.rom_status = Some(format!("Downloaded {}", path.display()));
                            d.downloaded_path = Some(path.clone());
                            build_flash_session(d, path);
                        }
                        Err(e) => d.rom_status = Some(format!("Download failed: {e}")),
                    }
                }
            }
            WorkerResponse::FlashStepDone {
                serial,
                idx,
                result,
            } => {
                if let Some(d) = self.dev(&serial) {
                    d.flash_busy = false;
                    if let Some(s) = &mut d.flash_session {
                        if let Some(slot) = s.statuses.get_mut(idx) {
                            *slot = match result {
                                Ok(_) => StepStatus::Done,
                                Err(e) => StepStatus::Failed(e),
                            };
                        }
                    }
                }
            }
            WorkerResponse::Screenshot { serial, result } => {
                if let Some(d) = self.dev(&serial) {
                    match result {
                        Ok(p) => {
                            d.capture_status = Some(format!("Screenshot saved: {}", p.display()));
                            d.add_screenshot(p);
                        }
                        Err(e) => d.capture_status = Some(format!("Screenshot failed: {e}")),
                    }
                }
            }
        }
    }

    // ── per-device actions ────────────────────────────────────────────────

    pub(crate) fn run_command(&mut self, serial: &str, cmd: &GuiCommand) {
        if let Some(d) = self.dev(serial) {
            d.busy_label = Some(cmd.label.to_string());
        }
        self.worker.send(WorkerRequest::Run {
            serial: serial.to_string(),
            label: cmd.label.to_string(),
            action: cmd.action.clone(),
        });
    }

    pub(crate) fn toggle_log_stream(&mut self, serial: &str) {
        if let Some(d) = self.dev(serial) {
            if d.log_streaming {
                d.log_stream.stop();
                d.log_streaming = false;
            } else if d.is_online() {
                d.log_entries.clear();
                d.log_stream.start(d.serial.clone());
                d.log_streaming = true;
            }
        }
    }

    pub(crate) fn detect_root(&mut self, serial: &str) {
        if let Some(d) = self.dev(serial) {
            d.busy_label = Some("Root check".into());
        }
        self.worker.send(WorkerRequest::DetectRoot {
            serial: serial.to_string(),
        });
    }

    pub(crate) fn remount(&mut self, serial: &str) {
        if let Some(d) = self.dev(serial) {
            d.busy_label = Some("Remount".into());
        }
        self.worker.send(WorkerRequest::Remount {
            serial: serial.to_string(),
        });
    }

    pub(crate) fn reboot(&mut self, serial: &str, target: RebootTarget) {
        if let Some(d) = self.dev(serial) {
            d.busy_label = Some(target.label().into());
        }
        self.worker.send(WorkerRequest::Run {
            serial: serial.to_string(),
            label: target.label().into(),
            action: CommandAction::Reboot(target),
        });
    }

    pub(crate) fn fastboot(&mut self, serial: &str, command: FastbootCommand) {
        let label = command.label().to_string();
        if let Some(d) = self.dev(serial) {
            d.busy_label = Some(label.clone());
        }
        self.worker.send(WorkerRequest::Fastboot {
            serial: serial.to_string(),
            label,
            command,
        });
    }

    // ── custom-ROM flashing ───────────────────────────────────────────────

    pub(crate) fn detect_roms(&mut self, serial: &str) {
        if let Some(d) = self.dev(serial) {
            d.rom_builds.clear();
            d.flash_session = None;
            d.downloaded_path = None;
            d.rom_status = Some("Detecting device…".into());
        }
        self.worker.send(WorkerRequest::DetectProfile {
            serial: serial.to_string(),
        });
    }

    pub(crate) fn download_build(&mut self, serial: &str, build: &RomBuild) {
        let dest = std::env::temp_dir()
            .join("droidkraft-roms")
            .join(build.file_name());
        if let Some(d) = self.dev(serial) {
            d.pending_method = Some(build.os.install_method());
            d.rom_status = Some(format!("Downloading {}…", build.file_name()));
            d.download_progress = Some(DownloadProgress {
                downloaded: 0,
                total: build.size_bytes,
            });
        }
        self.worker.send(WorkerRequest::Download {
            serial: serial.to_string(),
            build: build.clone(),
            dest,
        });
    }

    /// Run the next pending flash step (the caller has already consented).
    pub(crate) fn run_next_flash_step(&mut self, serial: &str) {
        let mut req = None;
        if let Some(d) = self.dev(serial) {
            if d.flash_busy {
                return;
            }
            if let Some(session) = &d.flash_session {
                if let Some((idx, step)) = session.next_step() {
                    req = Some(WorkerRequest::RunFlashStep {
                        serial: serial.to_string(),
                        idx,
                        step: step.clone(),
                    });
                }
            }
            if req.is_some() {
                d.flash_busy = true;
            }
        }
        if let Some(r) = req {
            self.worker.send(r);
        }
    }

    // ── recording / screenshots ───────────────────────────────────────────

    pub(crate) fn start_recording(&mut self, serial: &str) {
        if let Some(d) = self.dev(serial) {
            let name = timestamped_name(serial);
            d.capture_status =
                match d
                    .recorder
                    .start(serial, &d.out_dir.clone(), &name, d.rec_opts.clone())
                {
                    Ok(()) => None,
                    Err(e) => Some(format!("Cannot record: {e}")),
                };
        }
    }

    pub(crate) fn stop_recording(&mut self, serial: &str) {
        if let Some(d) = self.dev(serial) {
            d.recorder.stop();
        }
    }

    pub(crate) fn take_screenshot(&mut self, serial: &str) {
        let path = match self.dev(serial) {
            Some(d) => {
                d.capture_status = Some("Capturing screenshot…".into());
                d.out_dir.join(format!("{}.png", timestamped_name(serial)))
            }
            None => return,
        };
        self.worker.send(WorkerRequest::Screenshot {
            serial: serial.to_string(),
            path,
        });
    }

    // ── mirror / detach (per device) ──────────────────────────────────────

    /// Create the device's `MirrorView` on first use.
    pub(crate) fn ensure_mirror(
        &mut self,
        serial: &str,
        cx: &mut Context<Self>,
    ) -> Option<Entity<MirrorView>> {
        let d = self.fleet.get_mut(serial)?;
        if let Some(m) = &d.mirror {
            return Some(m.clone());
        }
        let stream = Arc::clone(&d.stream);
        let mirror = cx.new(|cx| MirrorView::new(stream, Chrome::Full, cx));
        let ser = serial.to_string();
        mirror.update(cx, |m, _| m.set_serial(Some(ser.clone())));
        let sub = cx.subscribe(&mirror, move |_this, _m, ev: &MirrorEvent, cx| match ev {
            MirrorEvent::AttachRequested => {
                // Deferred: we may be inside the detached window's update.
                let me = cx.entity();
                let ser = ser.clone();
                cx.defer(move |cx| me.update(cx, |t, cx| t.attach(&ser, cx)));
            }
        });
        d.mirror_subs.push(sub);
        d.mirror = Some(mirror.clone());
        Some(mirror)
    }

    /// Move a device's mirror view into its own window (stream keeps running).
    pub(crate) fn detach(&mut self, serial: &str, cx: &mut Context<Self>) {
        let Some(mirror) = self.ensure_mirror(serial, cx) else {
            return;
        };
        let Some(d) = self.fleet.get_mut(serial) else {
            return;
        };
        if d.detached.is_some() {
            return;
        }
        let name = d.display_name();
        let title = if name == serial {
            format!("DroidKraft — {serial}")
        } else {
            format!("DroidKraft — {name} ({serial})")
        };
        // Window aspect follows the device (portrait by default).
        let aspect = mirror
            .read(cx)
            .stream()
            .state()
            .device_size
            .map(|(w, h)| w as f32 / h.max(1) as f32)
            .unwrap_or(9.0 / 19.5)
            .clamp(0.3, 3.0);
        let (mut w, mut h) = (aspect * 780.0 + 16.0, 780.0 + 48.0);
        if w > 1100.0 {
            h *= 1100.0 / w;
            w = 1100.0;
        }
        let m2 = mirror.clone();
        let bounds = Bounds::centered(None, size(px(w), px(h)), cx);
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some(title.into()),
                    ..Default::default()
                }),
                window_min_size: Some(size(px(240.), px(320.))),
                ..Default::default()
            },
            |_, cx| cx.new(|_| DetachedRoot::new(m2)),
        );
        let Ok(handle) = opened else { return };
        mirror.update(cx, |m, cx| m.set_chrome(Chrome::Compact, true, cx));
        let ser = serial.to_string();
        let sub = handle
            .entity(cx)
            .ok()
            // Window closed by any means (close button, Cmd+W, …) → re-embed.
            .map(|root| {
                cx.observe_release(&root, move |this, _root, cx| {
                    this.on_detached_closed(&ser, cx)
                })
            });
        if let Some(d) = self.fleet.get_mut(serial) {
            d.detach_subs.extend(sub);
            d.detached = Some(handle);
        }
        let _ = handle.update(cx, |_, w, _| w.activate_window());
    }

    fn on_detached_closed(&mut self, serial: &str, cx: &mut Context<Self>) {
        let Some(d) = self.fleet.get_mut(serial) else {
            return;
        };
        if d.detached.take().is_some() {
            d.detach_subs.clear();
            if let Some(m) = &d.mirror {
                m.update(cx, |m, cx| m.set_chrome(Chrome::Full, false, cx));
            }
            cx.notify();
        }
    }

    /// Close the detached window and re-embed the view in the Screen tab.
    pub(crate) fn attach(&mut self, serial: &str, cx: &mut Context<Self>) {
        let Some(d) = self.fleet.get_mut(serial) else {
            return;
        };
        let Some(handle) = d.detached.take() else {
            return;
        };
        d.detach_subs.clear();
        if let Some(m) = &d.mirror {
            m.update(cx, |m, cx| m.set_chrome(Chrome::Full, false, cx));
        }
        let _ = handle.update(cx, |_, w, _| w.remove_window());
        self.open_device(serial, Tab::Screen, cx);
        cx.notify();
    }

    pub(crate) fn toggle_detach(&mut self, serial: &str, cx: &mut Context<Self>) {
        let detached = self.fleet.get(serial).is_some_and(|d| d.detached.is_some());
        if detached {
            self.attach(serial, cx);
        } else {
            self.detach(serial, cx);
        }
    }
}

/// Build a flash session for a downloaded ROM zip, honouring the detected
/// bootloader-lock state.
fn build_flash_session(d: &mut DeviceState, rom_zip: std::path::PathBuf) {
    let mut opts = match d.pending_method {
        Some(InstallMethod::FastbootFactory) => FlashOptions::factory(rom_zip),
        _ => FlashOptions::new(rom_zip),
    };
    if d.profile.as_ref().and_then(|p| p.bootloader_unlocked) == Some(false) {
        opts.unlock_bootloader = true;
    }
    d.flash_session = Some(FlashSession::new(build_plan(&opts), d.serial.clone()));
}

impl Render for DroidGui {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let route = self.fleet.resolve(&self.route);
        let vp = window.viewport_size();
        let lay = crate::layout::Layout::new(f32::from(vp.width), f32::from(vp.height));
        let content = match &route {
            Route::Devices => self.render_devices(&lay, cx),
            Route::Device(s) => self.render_workspace(s, &lay, cx),
            Route::Settings => self.render_settings(),
        };

        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(theme::BG))
            .text_color(rgb(theme::TEXT))
            .font_family("sans-serif")
            .on_action(cx.listener(|this, _: &ToggleDetach, _w, cx| {
                if let Some(s) = this.fleet.open_serial(&this.route) {
                    this.toggle_detach(&s, cx);
                }
            }))
            .child(self.render_nav(&route, &lay, cx))
            .child(div().flex_1().min_w_0().h_full().child(content))
    }
}
