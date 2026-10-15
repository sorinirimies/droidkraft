//! Framework-light application model: routing, per-device state, and the
//! ordered [`Fleet`] that routes worker responses to the right device.
//!
//! Everything here is constructible without a running gpui app so the
//! ordering / selection / routing logic is unit-testable.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use gpui::{Entity, Subscription, WindowHandle};

use droidkraft_core::features::flash::RootStatus;
use droidkraft_core::features::rom::{
    DeviceProfile, DownloadProgress, FlashSession, InstallMethod, RomBuild,
};
use droidkraft_core::{
    AdbDeviceEntry, DeviceDetails, DeviceHistory, LogEntry, LogcatFilter, LogcatStream,
};
use droidkraft_mirror::{default_output_dir, RecordOptions, ScreenRecorder, ScreenStream};

use crate::mirror_view::{DetachedRoot, MirrorView};

/// Maximum number of log entries retained per device.
pub const MAX_LOG_ENTRIES: usize = 20_000;

/// Workspace tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Logs,
    Commands,
    Flash,
    Screen,
}

impl Tab {
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Logs => "Logs",
            Tab::Commands => "Commands",
            Tab::Flash => "Flash & Root",
            Tab::Screen => "Screen",
        }
    }

    /// Short label for narrow windows.
    pub fn short_label(&self) -> &'static str {
        match self {
            Tab::Overview => "Stats",
            Tab::Logs => "Logs",
            Tab::Commands => "Cmds",
            Tab::Flash => "Flash",
            Tab::Screen => "Screen",
        }
    }

    pub fn all() -> &'static [Tab] {
        &[
            Tab::Overview,
            Tab::Logs,
            Tab::Commands,
            Tab::Flash,
            Tab::Screen,
        ]
    }
}

/// What the main area is showing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// Fleet overview (home).
    Devices,
    /// A device workspace.
    Device(String),
    Settings,
}

/// What a saved capture is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureKind {
    Recording,
    Screenshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    pub kind: CaptureKind,
    pub path: PathBuf,
}

/// Candidate output folders for recordings / screenshots.
pub fn output_dir_choices() -> Vec<PathBuf> {
    let mut v = vec![default_output_dir()];
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        for sub in ["Desktop", "Downloads"] {
            let d = PathBuf::from(&home).join(sub);
            if d.is_dir() && !v.contains(&d) {
                v.push(d);
            }
        }
    }
    let tmp = std::env::temp_dir();
    if !v.contains(&tmp) {
        v.push(tmp);
    }
    v
}

/// Next bit-rate preset (Mbps), wrapping.
pub fn next_bitrate(cur: u32) -> u32 {
    const P: [u32; 5] = [4, 8, 12, 16, 24];
    P.iter().copied().find(|&p| p > cur).unwrap_or(P[0])
}

/// Next max-height preset (`None` = native), wrapping.
pub fn next_max_height(cur: Option<u32>) -> Option<u32> {
    match cur {
        None => Some(720),
        Some(720) => Some(1080),
        Some(1080) => Some(1440),
        _ => None,
    }
}

/// Next directory in `choices` after `cur` (first when `cur` is unknown).
pub fn next_dir(cur: &std::path::Path, choices: &[PathBuf]) -> PathBuf {
    let i = choices
        .iter()
        .position(|p| p == cur)
        .map(|i| i + 1)
        .unwrap_or(0);
    choices
        .get(i % choices.len().max(1))
        .cloned()
        .unwrap_or_else(|| cur.to_path_buf())
}

/// Everything that belongs to ONE device.
pub struct DeviceState {
    pub serial: String,
    /// adb state ("device", "unauthorized", "offline", …) or "disconnected".
    pub adb_state: String,
    pub connected: bool,
    pub details: Option<DeviceDetails>,
    pub history: DeviceHistory,
    pub tab: Tab,

    // Logs
    pub log_stream: LogcatStream,
    pub log_entries: Vec<LogEntry>,
    pub log_filter: LogcatFilter,
    pub log_streaming: bool,

    // Commands / flash
    pub last_output: Option<(String, Result<String, String>)>,
    pub root_status: Option<RootStatus>,
    pub busy_label: Option<String>,
    pub profile: Option<DeviceProfile>,
    pub rom_builds: Vec<RomBuild>,
    pub rom_status: Option<String>,
    pub download_progress: Option<DownloadProgress>,
    pub downloaded_path: Option<PathBuf>,
    pub pending_method: Option<InstallMethod>,
    pub flash_session: Option<FlashSession>,
    pub flash_busy: bool,

    // Screen
    pub stream: Arc<ScreenStream>,
    pub recorder: ScreenRecorder,
    pub rec_opts: RecordOptions,
    pub out_dir: PathBuf,
    pub captures: Vec<Capture>,
    pub capture_status: Option<String>,
    /// Whether the recorder's current `Done` result was already harvested.
    pub rec_harvested: bool,
    pub mirror: Option<Entity<MirrorView>>,
    pub detached: Option<WindowHandle<DetachedRoot>>,
    pub detach_subs: Vec<Subscription>,
    pub mirror_subs: Vec<Subscription>,
}

impl DeviceState {
    pub fn new(serial: impl Into<String>) -> Self {
        Self {
            serial: serial.into(),
            adb_state: "device".into(),
            connected: true,
            details: None,
            history: DeviceHistory::default(),
            tab: Tab::Overview,
            log_stream: LogcatStream::new(),
            log_entries: Vec::new(),
            log_filter: LogcatFilter::default(),
            log_streaming: false,
            last_output: None,
            root_status: None,
            busy_label: None,
            profile: None,
            rom_builds: Vec::new(),
            rom_status: None,
            download_progress: None,
            downloaded_path: None,
            pending_method: None,
            flash_session: None,
            flash_busy: false,
            stream: Arc::new(ScreenStream::new()),
            recorder: ScreenRecorder::new(),
            rec_opts: RecordOptions::default(),
            out_dir: default_output_dir(),
            captures: Vec::new(),
            capture_status: None,
            rec_harvested: true,
            mirror: None,
            detached: None,
            detach_subs: Vec::new(),
            mirror_subs: Vec::new(),
        }
    }

    pub fn is_online(&self) -> bool {
        self.connected && self.adb_state == "device"
    }

    /// "Manufacturer Model" once details are known, else the serial.
    pub fn display_name(&self) -> String {
        match &self.details {
            Some(d) if d.online => d.display_name(),
            _ => self.serial.clone(),
        }
    }

    /// Battery % when known.
    pub fn battery_pct(&self) -> Option<u8> {
        self.details
            .as_ref()
            .filter(|d| d.online)
            .map(|d| d.battery.level)
    }

    /// Append raw logcat lines, capping retained entries.
    pub fn push_log_lines(&mut self, lines: &[String]) {
        for l in lines {
            self.log_entries.push(LogEntry::parse(l));
        }
        if self.log_entries.len() > MAX_LOG_ENTRIES {
            let excess = self.log_entries.len() - MAX_LOG_ENTRIES;
            self.log_entries.drain(0..excess);
        }
    }

    /// Record finished recording files (deduplicated).
    pub fn add_recordings(&mut self, files: &[PathBuf]) {
        for f in files {
            let c = Capture {
                kind: CaptureKind::Recording,
                path: f.clone(),
            };
            if !self.captures.contains(&c) {
                self.captures.push(c);
            }
        }
    }

    pub fn add_screenshot(&mut self, path: PathBuf) {
        self.captures.push(Capture {
            kind: CaptureKind::Screenshot,
            path,
        });
    }
}

/// Sort rank for the device list: online first, then other connected, then gone.
fn rank(d: &DeviceState) -> u8 {
    if d.is_online() {
        0
    } else if d.connected {
        1
    } else {
        2
    }
}

/// All known devices in stable first-seen order.
#[derive(Default)]
pub struct Fleet {
    seen: Vec<String>,
    devices: HashMap<String, DeviceState>,
}

impl Fleet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.seen.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }

    pub fn get(&self, serial: &str) -> Option<&DeviceState> {
        self.devices.get(serial)
    }

    pub fn get_mut(&mut self, serial: &str) -> Option<&mut DeviceState> {
        self.devices.get_mut(serial)
    }

    /// Mutable access, creating the device if unknown.
    pub fn ensure(&mut self, serial: &str) -> &mut DeviceState {
        if !self.devices.contains_key(serial) {
            self.seen.push(serial.to_string());
            self.devices
                .insert(serial.to_string(), DeviceState::new(serial));
        }
        self.devices.get_mut(serial).expect("just inserted")
    }

    /// Display order: online, other connected, disconnected; stable within.
    pub fn ordered(&self) -> Vec<&DeviceState> {
        let mut v: Vec<&DeviceState> = self
            .seen
            .iter()
            .filter_map(|s| self.devices.get(s))
            .collect();
        v.sort_by_key(|d| rank(d));
        v
    }

    pub fn serials(&self) -> Vec<String> {
        self.ordered().iter().map(|d| d.serial.clone()).collect()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut DeviceState> {
        self.devices.values_mut()
    }

    /// Apply a fresh `adb devices` snapshot: new devices appear, missing
    /// ones are marked disconnected (their state is kept).
    pub fn apply_entries(&mut self, entries: &[AdbDeviceEntry]) {
        for e in entries {
            let d = self.ensure(&e.serial);
            d.connected = true;
            d.adb_state = e.state.clone();
        }
        for d in self.devices.values_mut() {
            if !entries.iter().any(|e| e.serial == d.serial) {
                d.connected = false;
                d.adb_state = "disconnected".into();
            }
        }
    }

    /// Store polled details + history for `serial`; ignored for unknown ones.
    pub fn apply_details(
        &mut self,
        serial: &str,
        details: DeviceDetails,
        history: DeviceHistory,
    ) -> bool {
        match self.devices.get_mut(serial) {
            Some(d) => {
                d.details = Some(details);
                d.history = history;
                true
            }
            None => false,
        }
    }

    /// Number of online devices.
    pub fn online_count(&self) -> usize {
        self.devices.values().filter(|d| d.is_online()).count()
    }

    /// Average battery of online devices that reported one.
    pub fn avg_battery(&self) -> Option<u8> {
        let v: Vec<u32> = self
            .devices
            .values()
            .filter(|d| d.is_online())
            .filter_map(|d| d.battery_pct())
            .map(u32::from)
            .collect();
        if v.is_empty() {
            None
        } else {
            Some((v.iter().sum::<u32>() / v.len() as u32) as u8)
        }
    }

    /// Total RAM in use (MiB) and total RAM (MiB) across online devices.
    pub fn ram_totals(&self) -> (u64, u64) {
        self.devices
            .values()
            .filter(|d| d.is_online())
            .filter_map(|d| d.details.as_ref())
            .fold((0, 0), |(u, t), d| {
                (u + d.memory.used_mib(), t + d.memory.total_mib)
            })
    }

    /// Resolve a route: a workspace for an unknown device falls back home.
    pub fn resolve(&self, route: &Route) -> Route {
        match route {
            Route::Device(s) if !self.devices.contains_key(s) => Route::Devices,
            r => r.clone(),
        }
    }

    /// Serial of the open workspace, if any.
    pub fn open_serial(&self, route: &Route) -> Option<String> {
        match self.resolve(route) {
            Route::Device(s) => Some(s),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(s: &str, st: &str) -> AdbDeviceEntry {
        AdbDeviceEntry {
            serial: s.into(),
            state: st.into(),
        }
    }

    #[test]
    fn ordering_online_first_stable() {
        let mut f = Fleet::new();
        f.apply_entries(&[
            entry("b", "unauthorized"),
            entry("a", "device"),
            entry("c", "device"),
        ]);
        assert_eq!(f.serials(), vec!["a", "c", "b"]);
    }

    #[test]
    fn disconnected_devices_kept_and_sorted_last() {
        let mut f = Fleet::new();
        f.apply_entries(&[entry("a", "device"), entry("b", "device")]);
        f.apply_entries(&[entry("b", "device")]);
        assert_eq!(f.serials(), vec!["b", "a"]);
        let a = f.get("a").unwrap();
        assert!(!a.connected && !a.is_online());
        // Reconnect restores.
        f.apply_entries(&[entry("a", "device"), entry("b", "device")]);
        assert_eq!(f.serials(), vec!["a", "b"]);
        assert!(f.get("a").unwrap().is_online());
    }

    #[test]
    fn per_device_state_is_isolated() {
        let mut f = Fleet::new();
        f.apply_entries(&[entry("a", "device"), entry("b", "device")]);
        f.get_mut("a").unwrap().last_output = Some(("x".into(), Ok("out".into())));
        f.get_mut("a").unwrap().tab = Tab::Logs;
        assert!(f.get("b").unwrap().last_output.is_none());
        assert_eq!(f.get("b").unwrap().tab, Tab::Overview);
    }

    #[test]
    fn details_route_by_serial() {
        let mut f = Fleet::new();
        f.apply_entries(&[entry("a", "device")]);
        let mut d = DeviceDetails {
            serial: "a".into(),
            online: true,
            ..Default::default()
        };
        d.battery.level = 77;
        assert!(f.apply_details("a", d.clone(), DeviceHistory::default()));
        assert!(!f.apply_details("zzz", d, DeviceHistory::default()));
        assert_eq!(f.get("a").unwrap().battery_pct(), Some(77));
    }

    #[test]
    fn route_resolution() {
        let mut f = Fleet::new();
        f.apply_entries(&[entry("a", "device")]);
        assert_eq!(
            f.resolve(&Route::Device("a".into())),
            Route::Device("a".into())
        );
        assert_eq!(f.resolve(&Route::Device("gone".into())), Route::Devices);
        assert_eq!(
            f.open_serial(&Route::Device("a".into())).as_deref(),
            Some("a")
        );
        assert_eq!(f.open_serial(&Route::Settings), None);
    }

    #[test]
    fn fleet_summary() {
        let mut f = Fleet::new();
        f.apply_entries(&[
            entry("a", "device"),
            entry("b", "device"),
            entry("c", "offline"),
        ]);
        for (s, lvl, tot) in [("a", 80u8, 4000u64), ("b", 40, 2000)] {
            let mut d = DeviceDetails {
                serial: s.into(),
                online: true,
                ..Default::default()
            };
            d.battery.level = lvl;
            d.memory.total_mib = tot;
            d.memory.available_mib = tot / 2;
            f.apply_details(s, d, DeviceHistory::default());
        }
        assert_eq!(f.online_count(), 2);
        assert_eq!(f.avg_battery(), Some(60));
        assert_eq!(f.ram_totals(), (3000, 6000));
    }

    #[test]
    fn logs_capped_and_recordings_deduped() {
        let mut d = DeviceState::new("a");
        d.push_log_lines(&vec!["I/Tag( 1): hi".to_string(); MAX_LOG_ENTRIES + 5]);
        assert_eq!(d.log_entries.len(), MAX_LOG_ENTRIES);
        let p = PathBuf::from("/tmp/x.mp4");
        d.add_recordings(std::slice::from_ref(&p));
        d.add_recordings(&[p]);
        assert_eq!(d.captures.len(), 1);
    }

    #[test]
    fn option_cycling() {
        assert_eq!(next_bitrate(8), 12);
        assert_eq!(next_bitrate(24), 4);
        assert_eq!(next_max_height(None), Some(720));
        assert_eq!(next_max_height(Some(1440)), None);
        let c = vec![PathBuf::from("/a"), PathBuf::from("/b")];
        assert_eq!(
            next_dir(std::path::Path::new("/a"), &c),
            PathBuf::from("/b")
        );
        assert_eq!(
            next_dir(std::path::Path::new("/b"), &c),
            PathBuf::from("/a")
        );
        assert_eq!(
            next_dir(std::path::Path::new("/zzz"), &c),
            PathBuf::from("/a")
        );
    }

    #[test]
    fn display_name_falls_back_to_serial() {
        let mut d = DeviceState::new("SER1");
        assert_eq!(d.display_name(), "SER1");
        let mut det = DeviceDetails {
            serial: "SER1".into(),
            online: true,
            ..Default::default()
        };
        det.system.manufacturer = "Google".into();
        det.system.model = "Pixel 7".into();
        d.details = Some(det);
        assert_eq!(d.display_name(), "Google Pixel 7");
    }
}
