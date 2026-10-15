//! Rich per-device details for multi-device dashboards.
//!
//! [`collect_device_details`] gathers everything in a single batched shell
//! round-trip; the `parse_*` helpers are pure and unit-tested.

use adb_client::{ADBDeviceExt, ADBServer};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::SystemTime;

use crate::client::ADB_PORT;

const SEP: &str = "---SEP---";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    NotCharging,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlugType {
    None,
    Ac,
    Usb,
    Wireless,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub level: u8,
    pub status: BatteryStatus,
    pub plugged: PlugType,
    pub health: String,
    pub temperature_c: Option<f32>,
    pub voltage_mv: Option<u32>,
    pub technology: String,
    pub charge_counter_uah: Option<u64>,
    pub cycle_count: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_mib: u64,
    pub available_mib: u64,
    pub swap_total_mib: u64,
    pub swap_free_mib: u64,
    pub cached_mib: u64,
}

impl MemoryInfo {
    pub fn used_mib(&self) -> u64 {
        self.total_mib.saturating_sub(self.available_mib)
    }
    pub fn used_pct(&self) -> f32 {
        if self.total_mib == 0 {
            0.0
        } else {
            self.used_mib() as f32 * 100.0 / self.total_mib as f32
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageInfo {
    pub total_mib: u64,
    pub used_mib: u64,
    pub avail_mib: u64,
}

impl StorageInfo {
    pub fn used_pct(&self) -> f32 {
        if self.total_mib == 0 {
            0.0
        } else {
            self.used_mib as f32 * 100.0 / self.total_mib as f32
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuInfo {
    pub cores: u32,
    pub load_1: f32,
    pub load_5: f32,
    pub load_15: f32,
    pub abi: String,
    pub hardware: String,
    pub max_freq_mhz: Option<u32>,
    pub cur_freq_mhz: Option<u32>,
    pub usage_pct: Option<f32>,
}

/// Aggregate `/proc/stat` counters used for CPU-usage deltas.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuSample {
    pub total: u64,
    pub idle: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub width: u32,
    pub height: u32,
    pub density: u32,
    pub refresh_hz: Option<f32>,
    pub brightness_pct: Option<u8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub wifi_ssid: Option<String>,
    pub wifi_ip: Option<String>,
    pub rssi_dbm: Option<i32>,
    pub link_mbps: Option<u32>,
    pub mobile_operator: Option<String>,
    pub airplane: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThermalInfo {
    pub cpu_temp_c: Option<f32>,
    pub skin_temp_c: Option<f32>,
    pub battery_temp_c: Option<f32>,
    pub throttling: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInfo {
    pub manufacturer: String,
    pub brand: String,
    pub model: String,
    pub codename: String,
    pub android_release: String,
    pub sdk: u32,
    pub security_patch: String,
    pub build_id: String,
    pub build_fingerprint: String,
    pub kernel: String,
    pub bootloader: String,
    pub baseband: String,
    pub timezone: String,
    pub locale: String,
    pub uptime_secs: u64,
    pub verified_boot_state: String,
    pub flash_locked: Option<bool>,
    pub treble: bool,
    pub is_rooted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDetails {
    pub serial: String,
    pub state: String,
    pub online: bool,
    pub system: SystemInfo,
    pub battery: BatteryInfo,
    pub memory: MemoryInfo,
    pub storage: StorageInfo,
    pub cpu: CpuInfo,
    pub display: DisplayInfo,
    pub network: NetworkInfo,
    pub thermal: ThermalInfo,
    pub collected_at: SystemTime,
    pub errors: Vec<String>,
}

impl Default for DeviceDetails {
    fn default() -> Self {
        Self {
            serial: String::new(),
            state: String::new(),
            online: false,
            system: SystemInfo::default(),
            battery: BatteryInfo::default(),
            memory: MemoryInfo::default(),
            storage: StorageInfo::default(),
            cpu: CpuInfo::default(),
            display: DisplayInfo::default(),
            network: NetworkInfo::default(),
            thermal: ThermalInfo::default(),
            collected_at: SystemTime::UNIX_EPOCH,
            errors: Vec::new(),
        }
    }
}

impl DeviceDetails {
    /// "Manufacturer Model", or the serial when unknown.
    pub fn display_name(&self) -> String {
        let s = &self.system;
        let name = format!("{} {}", s.manufacturer, s.model).trim().to_string();
        if name.is_empty() {
            self.serial.clone()
        } else {
            name
        }
    }

    /// One-line summary, e.g. "Google Pixel 7 · Android 14 · 85% · RAM 62%".
    pub fn summary_line(&self) -> String {
        format!(
            "{} · Android {} · {}% · RAM {:.0}%",
            self.display_name(),
            self.system.android_release,
            self.battery.level,
            self.memory.used_pct()
        )
    }
}

// ── Ring buffer / history ──────────────────────────────────────────────────

/// Fixed-capacity ring buffer (oldest dropped on overflow).
#[derive(Debug, Clone)]
pub struct Ring<T> {
    cap: usize,
    buf: VecDeque<T>,
}

impl<T> Ring<T> {
    pub fn new(cap: usize) -> Self {
        let cap = cap.max(1);
        Self {
            cap,
            buf: VecDeque::with_capacity(cap),
        }
    }
    pub fn push(&mut self, v: T) {
        if self.buf.len() == self.cap {
            self.buf.pop_front();
        }
        self.buf.push_back(v);
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buf.iter()
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
    pub fn last(&self) -> Option<&T> {
        self.buf.back()
    }
    pub fn capacity(&self) -> usize {
        self.cap
    }
}

impl Ring<f64> {
    pub fn as_vec_f64(&self) -> Vec<f64> {
        self.buf.iter().copied().collect()
    }
}

/// Default history length (samples).
pub const HISTORY_LEN: usize = 60;

/// Per-device sparkline histories.
#[derive(Debug, Clone)]
pub struct DeviceHistory {
    pub battery: Ring<f64>,
    pub ram_pct: Ring<f64>,
    pub cpu_pct: Ring<f64>,
    pub temp: Ring<f64>,
}

impl Default for DeviceHistory {
    fn default() -> Self {
        Self::with_capacity(HISTORY_LEN)
    }
}

impl DeviceHistory {
    pub fn with_capacity(n: usize) -> Self {
        Self {
            battery: Ring::new(n),
            ram_pct: Ring::new(n),
            cpu_pct: Ring::new(n),
            temp: Ring::new(n),
        }
    }

    pub fn record(&mut self, d: &DeviceDetails) {
        self.battery.push(d.battery.level as f64);
        self.ram_pct.push(d.memory.used_pct() as f64);
        let cpu = d
            .cpu
            .usage_pct
            .unwrap_or(d.cpu.load_1 * 100.0 / d.cpu.cores.max(1) as f32);
        self.cpu_pct.push(cpu.clamp(0.0, 100.0) as f64);
        let t = d
            .battery
            .temperature_c
            .or(d.thermal.battery_temp_c)
            .or(d.thermal.cpu_temp_c);
        if let Some(t) = t {
            self.temp.push(t as f64);
        }
    }
}

// ── Pure parsers ───────────────────────────────────────────────────────────

fn kv(line: &str, sep: char) -> Option<(&str, &str)> {
    let (k, v) = line.split_once(sep)?;
    Some((k.trim(), v.trim()))
}

pub(crate) fn parse_dumpsys_battery(text: &str) -> BatteryInfo {
    let mut b = BatteryInfo::default();
    let (mut ac, mut usb, mut wl) = (false, false, false);
    let mut plugged_code = None;
    for line in text.lines() {
        let Some((k, v)) = kv(line, ':') else {
            continue;
        };
        match k {
            "level" => b.level = v.parse().unwrap_or(0),
            "status" => {
                b.status = match v {
                    "2" => BatteryStatus::Charging,
                    "3" => BatteryStatus::Discharging,
                    "4" => BatteryStatus::NotCharging,
                    "5" => BatteryStatus::Full,
                    _ => BatteryStatus::Unknown,
                }
            }
            "AC powered" => ac = v == "true",
            "USB powered" => usb = v == "true",
            "Wireless powered" => wl = v == "true",
            "plugged" => plugged_code = v.parse::<u32>().ok(),
            "health" => {
                b.health = match v {
                    "2" => "Good",
                    "3" => "Overheat",
                    "4" => "Dead",
                    "5" => "Over voltage",
                    "6" => "Failure",
                    "7" => "Cold",
                    other => other,
                }
                .to_string()
            }
            "temperature" => b.temperature_c = v.parse::<f32>().ok().map(|t| t / 10.0),
            "voltage" => b.voltage_mv = v.parse().ok(),
            "technology" => b.technology = v.to_string(),
            "Charge counter" => b.charge_counter_uah = v.parse().ok(),
            "Cycle count" | "cycle count" => b.cycle_count = v.parse().ok(),
            _ => {}
        }
    }
    b.plugged = if ac {
        PlugType::Ac
    } else if usb {
        PlugType::Usb
    } else if wl {
        PlugType::Wireless
    } else {
        match plugged_code {
            Some(0) => PlugType::None,
            Some(1) => PlugType::Ac,
            Some(2) => PlugType::Usb,
            Some(4) => PlugType::Wireless,
            _ if text.contains("AC powered") => PlugType::None,
            _ => PlugType::Unknown,
        }
    };
    b
}

fn kib_to_mib(v: &str) -> u64 {
    v.split_whitespace()
        .next()
        .and_then(|n| n.parse::<u64>().ok())
        .unwrap_or(0)
        / 1024
}

pub(crate) fn parse_meminfo_full(text: &str) -> MemoryInfo {
    let mut m = MemoryInfo::default();
    for line in text.lines() {
        let Some((k, v)) = kv(line, ':') else {
            continue;
        };
        match k {
            "MemTotal" => m.total_mib = kib_to_mib(v),
            "MemAvailable" => m.available_mib = kib_to_mib(v),
            "SwapTotal" => m.swap_total_mib = kib_to_mib(v),
            "SwapFree" => m.swap_free_mib = kib_to_mib(v),
            "Cached" => m.cached_mib = kib_to_mib(v),
            _ => {}
        }
    }
    m
}

/// Parse `df -k /data` (last data line: fs, 1K-blocks, used, avail, …).
pub(crate) fn parse_df(text: &str) -> StorageInfo {
    for line in text.lines().rev() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() >= 4 {
            if let (Ok(t), Ok(u), Ok(a)) = (
                f[1].parse::<u64>(),
                f[2].parse::<u64>(),
                f[3].parse::<u64>(),
            ) {
                return StorageInfo {
                    total_mib: t / 1024,
                    used_mib: u / 1024,
                    avail_mib: a / 1024,
                };
            }
        }
    }
    StorageInfo::default()
}

/// Parse the aggregate `cpu ` line of `/proc/stat`.
pub(crate) fn parse_proc_stat(text: &str) -> Option<CpuSample> {
    let line = text.lines().find(|l| l.starts_with("cpu "))?;
    let v: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|n| n.parse().ok())
        .collect();
    if v.len() < 4 {
        return None;
    }
    // user nice system idle iowait irq softirq steal
    let idle = v[3] + v.get(4).copied().unwrap_or(0);
    let total: u64 = v.iter().take(8).sum();
    Some(CpuSample { total, idle })
}

/// CPU usage percent between two samples.
pub(crate) fn cpu_delta(prev: &CpuSample, cur: &CpuSample) -> Option<f32> {
    let dt = cur.total.checked_sub(prev.total)?;
    let di = cur.idle.checked_sub(prev.idle)?;
    if dt == 0 {
        return None;
    }
    Some((dt.saturating_sub(di)) as f32 * 100.0 / dt as f32)
}

pub(crate) fn parse_loadavg(text: &str) -> (f32, f32, f32) {
    let mut it = text
        .split_whitespace()
        .map(|v| v.parse::<f32>().unwrap_or(0.0));
    (
        it.next().unwrap_or(0.0),
        it.next().unwrap_or(0.0),
        it.next().unwrap_or(0.0),
    )
}

/// `Physical size: 1080x2400` → (w, h).
pub(crate) fn parse_wm_size(text: &str) -> Option<(u32, u32)> {
    // Prefer "Override size" if present.
    let line = text
        .lines()
        .find(|l| l.contains("Override size"))
        .or_else(|| text.lines().find(|l| l.contains("Physical size")))?;
    let (w, h) = line.split(':').nth(1)?.trim().split_once('x')?;
    Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
}

pub(crate) fn parse_wm_density(text: &str) -> Option<u32> {
    let line = text
        .lines()
        .find(|l| l.contains("Override density"))
        .or_else(|| text.lines().find(|l| l.contains("Physical density")))?;
    line.split(':').nth(1)?.trim().parse().ok()
}

/// Extract (ssid, ip, rssi, link_mbps) from `dumpsys wifi` `mWifiInfo` line.
pub(crate) fn parse_dumpsys_wifi(
    text: &str,
) -> (Option<String>, Option<String>, Option<i32>, Option<u32>) {
    let line = text
        .lines()
        .find(|l| l.contains("mWifiInfo") && l.contains("SSID"))
        .or_else(|| text.lines().find(|l| l.contains("SSID:")));
    let Some(line) = line else {
        return (None, None, None, None);
    };
    let field = |key: &str| -> Option<String> {
        let i = line.find(key)? + key.len();
        let rest = line[i..].trim_start();
        let rest = rest.strip_prefix(' ').unwrap_or(rest);
        let end = rest.find(", ").unwrap_or(rest.len());
        Some(rest[..end].trim().trim_matches('"').to_string())
    };
    let ssid = field("SSID:").filter(|s| !s.is_empty() && s != "<unknown ssid>" && s != "0x");
    let ip = field("IP:").map(|s| s.trim_start_matches('/').to_string());
    let ip = ip.filter(|s| !s.is_empty() && s != "<none>" && s != "null");
    let rssi = field("RSSI:")
        .and_then(|s| s.parse::<i32>().ok())
        .filter(|r| *r > -127);
    let link =
        field("Link speed:").and_then(|s| s.trim_end_matches("Mbps").trim().parse::<u32>().ok());
    (ssid, ip, rssi, link)
}

/// Parse thermal lines of the form `<type> <millidegrees or degrees>`
/// (from `/sys/class/thermal`) or `dumpsys thermalservice` `Temperature{mValue=..., mType=..}`.
pub(crate) fn parse_thermal(text: &str) -> ThermalInfo {
    let mut t = ThermalInfo::default();
    let norm = |v: f32| if v.abs() >= 1000.0 { v / 1000.0 } else { v };
    for line in text.lines() {
        let line = line.trim();
        let (name, val) = if let Some(i) = line.find("mValue=") {
            let val = line[i + 7..]
                .split([',', '}'])
                .next()
                .and_then(|s| s.trim().parse::<f32>().ok());
            let name = line
                .find("mName=")
                .map(|j| {
                    line[j + 6..]
                        .split([',', '}'])
                        .next()
                        .unwrap_or("")
                        .to_lowercase()
                })
                .unwrap_or_default();
            (name, val)
        } else {
            let mut it = line.rsplitn(2, ' ');
            let v = it.next().and_then(|s| s.trim().parse::<f32>().ok());
            (it.next().unwrap_or("").to_lowercase(), v)
        };
        let Some(v) = val.map(norm) else { continue };
        if v <= 0.0 || v > 150.0 {
            continue;
        }
        if name.contains("cpu") || name.contains("soc") || name.contains("tsens") {
            t.cpu_temp_c = Some(t.cpu_temp_c.map_or(v, |o| o.max(v)));
        } else if name.contains("skin") || name.contains("shell") || name.contains("case") {
            t.skin_temp_c.get_or_insert(v);
        } else if name.contains("battery") || name.contains("batt") {
            t.battery_temp_c.get_or_insert(v);
        }
    }
    for line in text.lines() {
        if let Some(i) = line.find("Thermal Status:") {
            t.throttling = line[i + 15..].trim().parse::<u32>().is_ok_and(|s| s >= 2);
        }
    }
    t
}

/// `/proc/uptime` → whole seconds.
pub(crate) fn parse_uptime(text: &str) -> u64 {
    text.split_whitespace()
        .next()
        .and_then(|v| v.parse::<f64>().ok())
        .map_or(0, |v| v as u64)
}

/// Parse `getprop` dump (`[key]: [value]`) into a lookup closure input.
pub(crate) fn parse_getprop(text: &str) -> std::collections::HashMap<String, String> {
    text.lines()
        .filter_map(|l| {
            let (k, v) = l.split_once("]: [")?;
            Some((
                k.trim().trim_start_matches('[').to_string(),
                v.trim_end().trim_end_matches(']').to_string(),
            ))
        })
        .collect()
}

// ── Collection ─────────────────────────────────────────────────────────────

const SCRIPT: &str = concat!(
    "getprop; echo ---SEP---;",
    "dumpsys battery; echo ---SEP---;",
    "cat /proc/meminfo; echo ---SEP---;",
    "df -k /data; echo ---SEP---;",
    "head -1 /proc/stat; echo ---SEP---;",
    "cat /proc/loadavg; echo ---SEP---;",
    "wm size; wm density; echo ---SEP---;",
    "dumpsys wifi | grep -m2 -E 'mWifiInfo|SSID:'; echo ---SEP---;",
    "for z in /sys/class/thermal/thermal_zone*; do echo \"$(cat $z/type 2>/dev/null) $(cat $z/temp 2>/dev/null)\"; done; echo ---SEP---;",
    "cat /proc/uptime; echo ---SEP---;",
    "nproc; cat /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq; cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq; echo ---SEP---;",
    "uname -r; echo ---SEP---;",
    "settings get global airplane_mode_on; settings get system screen_brightness; echo ---SEP---;",
    "dumpsys display | grep -m1 -o 'fps=[0-9.]*'; echo ---SEP---;",
    "which su 2>/dev/null; echo ---SEP---;",
    "dumpsys thermalservice 2>/dev/null | grep -E 'Thermal Status:|mValue='"
);

fn pick<'a>(parts: &[&'a str], i: usize) -> &'a str {
    parts.get(i).copied().unwrap_or("")
}

/// Fill [`DeviceDetails`] from the split script output (pure; testable).
pub(crate) fn assemble(
    serial: &str,
    out: &str,
    prev_cpu: Option<&CpuSample>,
) -> (DeviceDetails, CpuSample) {
    let parts: Vec<&str> = out.split(SEP).collect();
    let mut d = DeviceDetails {
        serial: serial.to_string(),
        state: "device".into(),
        online: true,
        collected_at: SystemTime::now(),
        ..Default::default()
    };
    if parts.len() < 5 {
        d.errors.push("incomplete shell output".into());
    }
    let props = parse_getprop(pick(&parts, 0));
    let p = |k: &str| props.get(k).cloned().unwrap_or_default();
    let pn = |k: &str| props.get(k).filter(|v| !v.is_empty()).cloned();
    let s = &mut d.system;
    s.manufacturer = p("ro.product.manufacturer");
    s.brand = p("ro.product.brand");
    s.model = p("ro.product.model");
    s.codename = p("ro.product.device");
    s.android_release = p("ro.build.version.release");
    s.sdk = p("ro.build.version.sdk").parse().unwrap_or(0);
    s.security_patch = p("ro.build.version.security_patch");
    s.build_id = p("ro.build.id");
    s.build_fingerprint = p("ro.build.fingerprint");
    s.bootloader = p("ro.bootloader");
    s.baseband = p("gsm.version.baseband");
    s.timezone = p("persist.sys.timezone");
    s.locale = pn("persist.sys.locale")
        .or_else(|| pn("ro.product.locale"))
        .unwrap_or_default();
    s.verified_boot_state = p("ro.boot.verifiedbootstate");
    s.flash_locked = pn("ro.boot.flash.locked").map(|v| v == "1");
    s.treble = p("ro.treble.enabled") == "true";
    s.kernel = pick(&parts, 11).trim().to_string();
    s.uptime_secs = parse_uptime(pick(&parts, 9));
    let su = pick(&parts, 14).trim();
    s.is_rooted = if parts.len() > 14 {
        Some(!su.is_empty())
    } else {
        None
    };

    d.battery = parse_dumpsys_battery(pick(&parts, 1));
    d.memory = parse_meminfo_full(pick(&parts, 2));
    d.storage = parse_df(pick(&parts, 3));

    let sample = parse_proc_stat(pick(&parts, 4)).unwrap_or_default();
    let (l1, l5, l15) = parse_loadavg(pick(&parts, 5));
    let cpu_lines: Vec<&str> = pick(&parts, 10)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    d.cpu = CpuInfo {
        cores: cpu_lines.first().and_then(|v| v.parse().ok()).unwrap_or(0),
        load_1: l1,
        load_5: l5,
        load_15: l15,
        abi: p("ro.product.cpu.abi"),
        hardware: pn("ro.soc.model")
            .or_else(|| pn("ro.hardware"))
            .unwrap_or_default(),
        max_freq_mhz: cpu_lines
            .get(1)
            .and_then(|v| v.parse::<u32>().ok())
            .map(|k| k / 1000),
        cur_freq_mhz: cpu_lines
            .get(2)
            .and_then(|v| v.parse::<u32>().ok())
            .map(|k| k / 1000),
        usage_pct: prev_cpu.and_then(|p| cpu_delta(p, &sample)),
    };

    let wm = pick(&parts, 6);
    if let Some((w, h)) = parse_wm_size(wm) {
        d.display.width = w;
        d.display.height = h;
    }
    d.display.density = parse_wm_density(wm).unwrap_or(0);
    d.display.refresh_hz = pick(&parts, 13)
        .trim()
        .strip_prefix("fps=")
        .and_then(|v| v.parse().ok());
    let misc: Vec<&str> = pick(&parts, 12).lines().map(str::trim).collect();
    d.network.airplane = misc.first() == Some(&"1");
    d.display.brightness_pct = misc
        .get(1)
        .and_then(|v| v.parse::<u32>().ok())
        .map(|b| ((b * 100 + 127) / 255).min(100) as u8);

    let (ssid, ip, rssi, link) = parse_dumpsys_wifi(pick(&parts, 7));
    d.network.wifi_ssid = ssid;
    d.network.wifi_ip = ip;
    d.network.rssi_dbm = rssi;
    d.network.link_mbps = link;
    d.network.mobile_operator = pn("gsm.operator.alpha");

    let mut th = parse_thermal(&format!("{}\n{}", pick(&parts, 8), pick(&parts, 15)));
    th.battery_temp_c = th.battery_temp_c.or(d.battery.temperature_c);
    d.thermal = th;
    (d, sample)
}

/// Collect rich details for one device in a single shell round-trip.
///
/// Never panics; failures are recorded in `errors` and `online` is `false`.
pub fn collect_device_details(
    serial: &str,
    prev_cpu: Option<&CpuSample>,
) -> (DeviceDetails, CpuSample) {
    let fail = |msg: String| {
        let d = DeviceDetails {
            serial: serial.to_string(),
            state: "offline".into(),
            collected_at: SystemTime::now(),
            errors: vec![msg],
            ..Default::default()
        };
        (d, prev_cpu.copied().unwrap_or_default())
    };
    let mut server = ADBServer::new(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), ADB_PORT));
    let mut dev = match server.get_device_by_name(serial) {
        Ok(d) => d,
        Err(e) => return fail(format!("connect: {e}")),
    };
    let mut buf = Vec::new();
    if let Err(e) = dev.shell_command(&["sh", "-c", SCRIPT], &mut buf) {
        return fail(format!("shell: {e}"));
    }
    assemble(serial, &String::from_utf8_lossy(&buf), prev_cpu)
}

/// Collect details for several devices (no CPU delta).
pub fn list_all_details(serials: &[String]) -> Vec<DeviceDetails> {
    serials
        .iter()
        .map(|s| collect_device_details(s, None).0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery() {
        let t = "Current Battery Service state:\n  AC powered: false\n  USB powered: true\n  Wireless powered: false\n  status: 2\n  health: 2\n  level: 85\n  voltage: 4123\n  temperature: 312\n  technology: Li-ion\n  Charge counter: 3000000\n";
        let b = parse_dumpsys_battery(t);
        assert_eq!(b.level, 85);
        assert_eq!(b.status, BatteryStatus::Charging);
        assert_eq!(b.plugged, PlugType::Usb);
        assert_eq!(b.temperature_c, Some(31.2));
        assert_eq!(b.voltage_mv, Some(4123));
        assert_eq!(b.health, "Good");
        assert_eq!(b.charge_counter_uah, Some(3_000_000));
    }

    #[test]
    fn mem_df() {
        let m = parse_meminfo_full("MemTotal: 8192000 kB\nMemAvailable: 4096000 kB\nSwapTotal: 2048000 kB\nSwapFree: 1024000 kB\nCached: 1024000 kB\n");
        assert_eq!(m.total_mib, 8000);
        assert_eq!(m.used_mib(), 4000);
        assert!((m.used_pct() - 50.0).abs() < 0.1);
        let s = parse_df("Filesystem 1K-blocks Used Available Use% Mounted on\n/dev/block/dm-9 102400 51200 51200 50% /data\n");
        assert_eq!((s.total_mib, s.used_mib, s.avail_mib), (100, 50, 50));
        assert!((s.used_pct() - 50.0).abs() < 0.1);
    }

    #[test]
    fn cpu() {
        let a = parse_proc_stat("cpu  100 0 100 800 0 0 0 0 0 0\ncpu0 1 1 1 1").unwrap();
        let b = parse_proc_stat("cpu  200 0 200 1000 0 0 0 0 0 0").unwrap();
        assert_eq!(
            a,
            CpuSample {
                total: 1000,
                idle: 800
            }
        );
        assert_eq!(cpu_delta(&a, &b), Some(50.0));
        assert_eq!(cpu_delta(&b, &b), None);
        assert_eq!(parse_loadavg("1.5 2.5 3.5 1/200 99"), (1.5, 2.5, 3.5));
        assert_eq!(parse_uptime("1234.56 999.0"), 1234);
    }

    #[test]
    fn display_wifi() {
        let wm = "Physical size: 1080x2400\nPhysical density: 420\nOverride density: 440\n";
        assert_eq!(parse_wm_size(wm), Some((1080, 2400)));
        assert_eq!(parse_wm_density(wm), Some(440));
        let w = "  mWifiInfo SSID: \"Home\", BSSID: aa:bb, MAC: x, IP: /192.168.1.5, Security type: 2, Supplicant state: COMPLETED, RSSI: -55, Link speed: 433Mbps, Tx Link speed: 1Mbps";
        let (ssid, ip, rssi, link) = parse_dumpsys_wifi(w);
        assert_eq!(ssid.as_deref(), Some("Home"));
        assert_eq!(ip.as_deref(), Some("192.168.1.5"));
        assert_eq!(rssi, Some(-55));
        assert_eq!(link, Some(433));
        assert_eq!(parse_dumpsys_wifi("").0, None);
    }

    #[test]
    fn thermal() {
        let t = parse_thermal(
            "cpu-0-0 45000\nskin-therm 33500\nbattery 30100\nfoo 0\nThermal Status: 2\n",
        );
        assert_eq!(t.cpu_temp_c, Some(45.0));
        assert_eq!(t.skin_temp_c, Some(33.5));
        assert_eq!(t.battery_temp_c, Some(30.1));
        assert!(t.throttling);
    }

    #[test]
    fn ring_history_assemble() {
        let mut r = Ring::new(2);
        r.push(1.0);
        r.push(2.0);
        r.push(3.0);
        assert_eq!(r.as_vec_f64(), vec![2.0, 3.0]);
        assert_eq!(r.len(), 2);
        assert_eq!(r.last(), Some(&3.0));
        let out = "[ro.product.model]: [Pixel 7]\n[ro.product.manufacturer]: [Google]\n---SEP---\n  level: 50\n---SEP---\nMemTotal: 1024 kB\n";
        let (d, _) = assemble("abc", out, None);
        assert_eq!(d.display_name(), "Google Pixel 7");
        assert_eq!(d.battery.level, 50);
        let mut h = DeviceHistory::default();
        h.record(&d);
        assert_eq!(h.battery.last(), Some(&50.0));
        assert!(!d.summary_line().is_empty());
        assert_eq!(DeviceDetails::default().display_name(), "");
    }
}

/// How often the focused device's stats/graphs are polled.
///
/// Steps through fixed speeds so UIs can offer "faster / slower" controls.
/// Background devices poll at twice this interval (at least 3 s).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefreshRate(usize);

impl RefreshRate {
    /// Selectable intervals in milliseconds, fastest first.
    pub const STEPS_MS: [u64; 7] = [500, 1000, 1500, 2000, 3000, 5000, 10_000];
    /// Default (1.5 s).
    pub const DEFAULT: RefreshRate = RefreshRate(2);

    /// Interval of the focused device.
    pub fn focused(self) -> std::time::Duration {
        std::time::Duration::from_millis(Self::STEPS_MS[self.0])
    }

    /// Interval of devices that are not focused.
    pub fn background(self) -> std::time::Duration {
        (self.focused() * 2).max(std::time::Duration::from_secs(3))
    }

    /// One step faster (saturates at 0.5 s).
    pub fn faster(self) -> Self {
        Self(self.0.saturating_sub(1))
    }

    /// One step slower (saturates at 10 s).
    pub fn slower(self) -> Self {
        Self((self.0 + 1).min(Self::STEPS_MS.len() - 1))
    }

    pub fn is_fastest(self) -> bool {
        self.0 == 0
    }

    pub fn is_slowest(self) -> bool {
        self.0 == Self::STEPS_MS.len() - 1
    }

    /// Short label such as `1.5s` or `500ms`.
    pub fn label(self) -> String {
        let ms = Self::STEPS_MS[self.0];
        if ms < 1000 {
            format!("{ms}ms")
        } else if ms % 1000 == 0 {
            format!("{}s", ms / 1000)
        } else {
            format!("{:.1}s", ms as f64 / 1000.0)
        }
    }
}

impl Default for RefreshRate {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod refresh_rate_tests {
    use super::RefreshRate;

    #[test]
    fn steps_and_saturation() {
        let mut r = RefreshRate::default();
        assert_eq!(r.label(), "1.5s");
        for _ in 0..10 {
            r = r.faster();
        }
        assert!(r.is_fastest());
        assert_eq!(r.label(), "500ms");
        for _ in 0..10 {
            r = r.slower();
        }
        assert!(r.is_slowest());
        assert_eq!(r.label(), "10s");
    }

    #[test]
    fn background_is_slower() {
        let r = RefreshRate::default().faster().faster();
        assert_eq!(r.focused().as_millis(), 500);
        assert_eq!(r.background().as_secs(), 3);
        assert_eq!(
            RefreshRate::default()
                .slower()
                .slower()
                .background()
                .as_secs(),
            6
        );
    }
}
