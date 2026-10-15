//! Application state: per-device workspaces + the open view.

use std::collections::HashMap;

use droidkraft_core::{DeviceDetails, DeviceHistory};
use ratatui::layout::Rect;

use crate::message::Cmd;
use crate::theme::Themes;
use crate::worker::{ListedDevice, Req};

/// Workspace tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Overview,
    Logs,
    Commands,
    Flash,
    Screen,
}

impl Panel {
    pub const ALL: [Panel; 5] = [
        Panel::Overview,
        Panel::Logs,
        Panel::Commands,
        Panel::Flash,
        Panel::Screen,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Panel::Overview => "Overview",
            Panel::Logs => "Logs",
            Panel::Commands => "Commands",
            Panel::Flash => "Flash & Root",
            Panel::Screen => "Screen",
        }
    }
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|p| *p == self).unwrap_or(0)
    }
    pub fn next(self) -> Panel {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }
    pub fn prev(self) -> Panel {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }
    /// Mnemonic key (`o l c f s`).
    pub fn from_key(c: char) -> Option<Panel> {
        match c {
            'o' => Some(Panel::Overview),
            'l' => Some(Panel::Logs),
            'c' => Some(Panel::Commands),
            'f' => Some(Panel::Flash),
            's' => Some(Panel::Screen),
            _ => None,
        }
    }
}

/// Clickable targets in the nav rail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavTarget {
    Home,
    Device(String),
    Theme,
    Help,
}

/// Quick-action buttons on a device card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardBtn {
    Overview,
    Logs,
    Commands,
    Mirror,
    Record,
    Flash,
}

impl CardBtn {
    pub const ALL: [CardBtn; 6] = [
        CardBtn::Overview,
        CardBtn::Logs,
        CardBtn::Commands,
        CardBtn::Mirror,
        CardBtn::Record,
        CardBtn::Flash,
    ];
    pub fn label(self, short: bool) -> &'static str {
        match (self, short) {
            (CardBtn::Overview, false) => "Overview",
            (CardBtn::Logs, false) => "Logs",
            (CardBtn::Commands, false) => "Commands",
            (CardBtn::Mirror, false) => "Mirror",
            (CardBtn::Record, false) => "Record",
            (CardBtn::Flash, false) => "Flash",
            (CardBtn::Overview, true) => "Ovw",
            (CardBtn::Logs, true) => "Log",
            (CardBtn::Commands, true) => "Cmd",
            (CardBtn::Mirror, true) => "Mir",
            (CardBtn::Record, true) => "Rec",
            (CardBtn::Flash, true) => "Fls",
        }
    }
    /// Workspace tab the button opens.
    pub fn tab(self) -> Panel {
        match self {
            CardBtn::Overview => Panel::Overview,
            CardBtn::Logs => Panel::Logs,
            CardBtn::Commands => Panel::Commands,
            CardBtn::Mirror | CardBtn::Record => Panel::Screen,
            CardBtn::Flash => Panel::Flash,
        }
    }
}

/// UI state of the four device-scoped panels.
#[derive(Default)]
pub struct PanelState {
    pub logs: crate::ui::logs::LogsUi,
    pub cmds: crate::ui::commands::CmdUi,
    pub flash: crate::ui::flash::FlashUi,
    pub screen: crate::ui::screen::ScreenUi,
}

/// Everything the TUI knows about one device.
pub struct DeviceWorkspace {
    pub serial: String,
    /// adb state (`device`, `unauthorized`, `offline`, …) or `disconnected`.
    pub state: String,
    /// Present in the latest adb device list.
    pub listed: bool,
    pub details: Option<DeviceDetails>,
    pub history: DeviceHistory,
    /// Panel state while this workspace is not the open one.
    pub panels: Option<PanelState>,
}

impl DeviceWorkspace {
    pub fn new(serial: &str) -> Self {
        Self {
            serial: serial.to_string(),
            state: "disconnected".into(),
            listed: false,
            details: None,
            history: DeviceHistory::default(),
            panels: None,
        }
    }
    pub fn online(&self) -> bool {
        self.listed && self.state == "device"
    }
    pub fn name(&self) -> String {
        self.details
            .as_ref()
            .map(|d| d.display_name())
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| self.serial.clone())
    }
    pub fn battery_pct(&self) -> Option<u8> {
        self.details
            .as_ref()
            .filter(|_| self.online())
            .map(|d| d.battery.level)
    }
    /// adb-over-network serials look like `host:port`.
    pub fn is_wifi(&self) -> bool {
        self.serial.contains(':') || self.serial.contains("._adb-tls-connect")
    }
    pub fn connection(&self) -> &'static str {
        if self.is_wifi() {
            "Wi-Fi"
        } else {
            "USB"
        }
    }
}

/// Fix hint for devices that are listed but unusable.
pub fn state_hint(state: &str) -> &'static str {
    match state {
        "unauthorized" => "Unlock the phone and accept the “Allow USB debugging” prompt.",
        "offline" => "Replug the cable or run `adb kill-server && adb start-server`.",
        "bootloader" | "fastboot" => "In fastboot mode: use Flash & Root, or `fastboot reboot`.",
        "recovery" | "sideload" => "In recovery: reboot the device to use adb shell features.",
        "disconnected" => "Not connected. The workspace is kept until the device returns.",
        _ => "Device not ready. Check the cable and USB debugging.",
    }
}

pub struct Model {
    /// Open tab inside a workspace.
    pub panel: Panel,
    /// Serial of the open workspace; `None` = Devices home.
    pub active: Option<String>,
    pub workspaces: HashMap<String, DeviceWorkspace>,
    /// Selected card on the Devices home.
    pub home_sel: usize,
    pub themes: Themes,
    /// True until the first device list arrives.
    pub loading: bool,
    pub tick: u64,
    pub help: bool,
    pub message: Option<String>,
    /// Filled while drawing, for mouse hit-testing.
    pub nav_rects: Vec<(NavTarget, Rect)>,
    pub tab_rects: Vec<(Panel, Rect)>,
    /// Home cards: `(serial, None = card body / Some = button, rect)`.
    pub card_rects: Vec<(String, Option<CardBtn>, Rect)>,
    /// Cards per row on the Devices home (set while drawing).
    pub home_cols: usize,
    /// First visible row on the device Overview.
    pub ov_scroll: usize,
    pub quit: bool,
    // Live panel state of the OPEN workspace (swapped in/out on switch).
    pub logs: crate::ui::logs::LogsUi,
    pub cmds: crate::ui::commands::CmdUi,
    pub flash: crate::ui::flash::FlashUi,
    pub screen: crate::ui::screen::ScreenUi,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            panel: Panel::Overview,
            active: None,
            workspaces: HashMap::new(),
            home_sel: 0,
            themes: Themes::default(),
            loading: true,
            tick: 0,
            help: false,
            message: None,
            nav_rects: Vec::new(),
            tab_rects: Vec::new(),
            card_rects: Vec::new(),
            home_cols: 1,
            ov_scroll: 0,
            quit: false,
            logs: Default::default(),
            cmds: Default::default(),
            flash: Default::default(),
            screen: Default::default(),
        }
    }
}

impl Model {
    /// Stable device order: by serial.
    pub fn ordered_serials(&self) -> Vec<String> {
        let mut v: Vec<String> = self.workspaces.keys().cloned().collect();
        v.sort();
        v
    }

    pub fn ws(&self, serial: &str) -> Option<&DeviceWorkspace> {
        self.workspaces.get(serial)
    }

    pub fn active_ws(&self) -> Option<&DeviceWorkspace> {
        self.active.as_deref().and_then(|s| self.workspaces.get(s))
    }

    /// Serial of the open workspace (empty on home).
    pub fn serial(&self) -> String {
        self.active.clone().unwrap_or_default()
    }

    /// Open workspace exists and its device is online.
    pub fn connected(&self) -> bool {
        self.active_ws().is_some_and(|w| w.online())
    }

    /// Serial of the open workspace when online.
    pub fn active_online_serial(&self) -> Option<String> {
        self.active_ws()
            .filter(|w| w.online())
            .map(|w| w.serial.clone())
    }

    pub fn set_message(&mut self, s: impl Into<String>) {
        self.message = Some(s.into());
    }

    // ---- selection / navigation -------------------------------------------

    fn swap_panels(&mut self, p: &mut PanelState) {
        std::mem::swap(&mut self.logs, &mut p.logs);
        std::mem::swap(&mut self.cmds, &mut p.cmds);
        std::mem::swap(&mut self.flash, &mut p.flash);
        std::mem::swap(&mut self.screen, &mut p.screen);
        // One shared graphics backend (detected once at startup) follows the visible screen.
        std::mem::swap(&mut self.screen.gfx, &mut p.screen.gfx);
    }

    /// Park the open workspace's panel state in its workspace.
    fn park_active(&mut self) {
        let Some(prev) = self.active.take() else {
            return;
        };
        // The live mirror is only for the visible device.
        self.screen.stream.stop();
        self.screen.started_for = None;
        self.screen.capture = false;
        let mut p = PanelState::default();
        self.swap_panels(&mut p);
        if let Some(ws) = self.workspaces.get_mut(&prev) {
            ws.panels = Some(p);
        }
    }

    /// Open `serial`'s workspace on `tab`. Returns worker commands.
    pub fn open_device(&mut self, serial: &str, tab: Panel) -> Vec<Cmd> {
        if !self.workspaces.contains_key(serial) {
            return vec![];
        }
        if self.active.as_deref() != Some(serial) {
            self.park_active();
            let mut p = self
                .workspaces
                .get_mut(serial)
                .and_then(|w| w.panels.take())
                .unwrap_or_default();
            self.swap_panels(&mut p);
            self.active = Some(serial.to_string());
            if let Some(i) = self.ordered_serials().iter().position(|s| s == serial) {
                self.home_sel = i;
            }
        }
        self.set_tab(tab)
    }

    /// Switch tab inside the open workspace.
    pub fn set_tab(&mut self, tab: Panel) -> Vec<Cmd> {
        self.panel = tab;
        self.screen.capture = false;
        let serial = self.serial();
        let mut cmds = vec![Cmd::Worker(Req::Focus(self.active.clone()))];
        if tab == Panel::Flash && self.connected() {
            cmds.push(Cmd::Worker(Req::FastbootDevices {
                serial: serial.clone(),
            }));
            cmds.push(Cmd::Worker(Req::DetectRoot { serial }));
        }
        if tab == Panel::Screen {
            crate::ui::screen::ensure_started(self);
        }
        cmds
    }

    pub fn go_home(&mut self) -> Vec<Cmd> {
        self.park_active();
        self.panel = Panel::Overview;
        vec![Cmd::Worker(Req::Focus(None))]
    }

    /// Open the `n`-th (0-based) device, keeping the current tab if already in a workspace.
    pub fn open_nth(&mut self, n: usize) -> Vec<Cmd> {
        let Some(s) = self.ordered_serials().get(n).cloned() else {
            return vec![];
        };
        let tab = if self.active.is_some() {
            self.panel
        } else {
            Panel::Overview
        };
        self.open_device(&s, tab)
    }

    /// Move to the previous/next device (wrapping); from home opens the first/last.
    pub fn step_device(&mut self, delta: i32) -> Vec<Cmd> {
        let order = self.ordered_serials();
        if order.is_empty() {
            return vec![];
        }
        let n = order.len() as i32;
        let idx = match &self.active {
            Some(a) => order
                .iter()
                .position(|s| s == a)
                .map_or(0, |i| (i as i32 + delta).rem_euclid(n)),
            None => {
                if delta >= 0 {
                    0
                } else {
                    n - 1
                }
            }
        };
        self.open_nth(idx as usize)
    }

    // ---- worker data ------------------------------------------------------

    /// Apply a fresh `adb devices` list: unknown serials get a workspace,
    /// missing ones are kept but flagged disconnected.
    pub fn apply_devices(&mut self, list: &[ListedDevice]) {
        self.loading = false;
        for d in list {
            let ws = self
                .workspaces
                .entry(d.serial.clone())
                .or_insert_with(|| DeviceWorkspace::new(&d.serial));
            ws.listed = true;
            ws.state = d.state.clone();
        }
        for ws in self.workspaces.values_mut() {
            if !list.iter().any(|d| d.serial == ws.serial) {
                ws.listed = false;
                ws.state = "disconnected".into();
            }
        }
        let n = self.workspaces.len();
        if n > 0 && self.home_sel >= n {
            self.home_sel = n - 1;
        }
    }

    pub fn apply_details(&mut self, d: DeviceDetails) {
        let ws = self
            .workspaces
            .entry(d.serial.clone())
            .or_insert_with(|| DeviceWorkspace::new(&d.serial));
        if d.online {
            ws.history.record(&d);
        }
        ws.details = Some(d);
        self.loading = false;
    }

    /// Run `f` with `serial`'s panel state live in `self.{logs,cmds,flash,screen}`.
    pub fn with_ws<R>(&mut self, serial: &str, f: impl FnOnce(&mut Model) -> R) -> Option<R> {
        if self.active.as_deref() == Some(serial) {
            return Some(f(self));
        }
        let mut p = self
            .workspaces
            .get_mut(serial)?
            .panels
            .take()
            .unwrap_or_default();
        self.swap_panels(&mut p);
        let r = f(self);
        self.swap_panels(&mut p);
        if let Some(ws) = self.workspaces.get_mut(serial) {
            ws.panels = Some(p);
        }
        Some(r)
    }

    /// Is a screen recording running for `serial`?
    pub fn is_recording(&self, serial: &str) -> bool {
        if self.active.as_deref() == Some(serial) {
            return self.screen.recorder.is_recording();
        }
        self.workspaces
            .get(serial)
            .and_then(|w| w.panels.as_ref())
            .is_some_and(|p| p.screen.recorder.is_recording())
    }

    /// Stop every background stream/recording (app exit).
    pub fn shutdown_all(&mut self) {
        self.screen.stream.stop();
        self.screen.recorder.stop();
        for ws in self.workspaces.values_mut() {
            if let Some(p) = ws.panels.as_mut() {
                p.screen.stream.stop();
                p.screen.recorder.stop();
                p.logs.state.stop_streaming();
            }
        }
    }

    /// Per-tick housekeeping for parked workspaces (logcat drain, recorder).
    pub fn tick_parked(&mut self) -> Vec<String> {
        let mut msgs = Vec::new();
        for ws in self.workspaces.values_mut() {
            if let Some(p) = ws.panels.as_mut() {
                if p.logs.state.is_streaming {
                    p.logs.state.poll_new_entries();
                }
                if let Some(msg) = p.screen.poll_recorder() {
                    msgs.push(format!("{}: {msg}", ws.serial));
                }
            }
        }
        msgs
    }

    // ---- fleet summary ----------------------------------------------------

    pub fn fleet(&self) -> Fleet {
        let mut f = Fleet::default();
        let mut bat = 0u32;
        for ws in self.workspaces.values() {
            f.total += 1;
            if !ws.online() {
                continue;
            }
            f.online += 1;
            if let Some(d) = &ws.details {
                bat += d.battery.level as u32;
                f.battery_n += 1;
                f.ram_used_mib += d.memory.used_mib();
                f.ram_total_mib += d.memory.total_mib;
            }
        }
        f.avg_battery = bat.checked_div(f.battery_n);
        f
    }
}

/// Aggregates shown on the Devices home.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Fleet {
    pub total: usize,
    pub online: usize,
    pub battery_n: u32,
    pub avg_battery: Option<u32>,
    pub ram_used_mib: u64,
    pub ram_total_mib: u64,
}
