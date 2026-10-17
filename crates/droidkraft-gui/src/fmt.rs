//! Framework-free formatting / classification helpers (unit-tested).

use std::time::Duration;

use crate::theme;

/// `512 MiB` / `3.4 GiB`.
pub fn fmt_mib(mib: u64) -> String {
    if mib >= 1024 {
        format!("{:.1} GiB", mib as f64 / 1024.0)
    } else {
        format!("{mib} MiB")
    }
}

/// Byte count as `B`/`KB`/`MB`/`GB` (decimal).
pub fn fmt_bytes(b: u64) -> String {
    const U: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1000.0 && i < U.len() - 1 {
        v /= 1000.0;
        i += 1;
    }
    if i == 0 {
        format!("{b} B")
    } else {
        format!("{v:.1} {}", U[i])
    }
}

/// Device uptime: `3d 4h`, `2h 05m`, `5m 03s`, `42s`.
pub fn fmt_uptime(secs: u64) -> String {
    let (d, h, m, s) = (
        secs / 86_400,
        secs % 86_400 / 3600,
        secs % 3600 / 60,
        secs % 60,
    );
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m:02}m")
    } else if m > 0 {
        format!("{m}m {s:02}s")
    } else {
        format!("{s}s")
    }
}

/// Recording timer: `MM:SS` or `H:MM:SS`.
pub fn fmt_elapsed(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

pub fn nonempty(s: &str, fallback: &str) -> String {
    if s.trim().is_empty() {
        fallback.to_string()
    } else {
        s.to_string()
    }
}

/// Truncate to at most `max` bytes on a char boundary.
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…\n[truncated]", &s[..end])
}

/// Shorten to `max` chars with a trailing `…`.
pub fn ellipsize(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut o: String = s.chars().take(max.saturating_sub(1)).collect();
        o.push('…');
        o
    }
}

/// Battery level → colour (low is bad).
pub fn battery_color(pct: u8) -> u32 {
    if pct >= 50 {
        theme::OK
    } else if pct >= 20 {
        theme::WARN
    } else {
        theme::ERR
    }
}

/// Usage percentage → colour (high is bad).
pub fn usage_color(pct: f32) -> u32 {
    if pct < 60.0 {
        theme::OK
    } else if pct < 85.0 {
        theme::WARN
    } else {
        theme::ERR
    }
}

/// Temperature (°C) → colour.
pub fn temp_color(c: f32) -> u32 {
    if c < 38.0 {
        theme::OK
    } else if c < 45.0 {
        theme::WARN
    } else {
        theme::ERR
    }
}

pub use droidkraft_core::connection_kind;

/// Explanation + fix for devices that are not usable yet.
pub fn state_hint(state: &str) -> Option<&'static str> {
    match state {
        "device" => None,
        "unauthorized" => Some("Unlock the phone and accept the “Allow USB debugging?” prompt."),
        "offline" => Some("Replug the cable or run `adb reconnect`; check USB debugging is on."),
        "recovery" | "sideload" => Some("Device is in recovery. Reboot to system to monitor it."),
        "bootloader" | "fastboot" => {
            Some("Device is in fastboot. Use Flash & Root to flash or reboot.")
        }
        "disconnected" => Some("Device disconnected. Reconnect it to resume."),
        _ => Some("Device is not ready. Check the connection and USB debugging."),
    }
}

/// Bar heights (px) for a sparkline: values scaled against `max` into
/// `0..=height`, with a 1px floor so empty samples stay visible.
pub fn spark_heights(values: &[f64], max: f64, height: f32) -> Vec<f32> {
    let max = if max <= 0.0 { 1.0 } else { max };
    values
        .iter()
        .map(|v| ((*v / max).clamp(0.0, 1.0) as f32 * height).max(1.0))
        .collect()
}

/// Last `n` values of a series (for fitting a fixed number of columns).
pub fn tail<T: Copy>(v: &[T], n: usize) -> &[T] {
    &v[v.len().saturating_sub(n)..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mib_formatting() {
        assert_eq!(fmt_mib(512), "512 MiB");
        assert_eq!(fmt_mib(3584), "3.5 GiB");
    }

    #[test]
    fn bytes_formatting() {
        assert_eq!(fmt_bytes(999), "999 B");
        assert_eq!(fmt_bytes(1_500_000), "1.5 MB");
    }

    #[test]
    fn uptime_formatting() {
        assert_eq!(fmt_uptime(42), "42s");
        assert_eq!(fmt_uptime(303), "5m 03s");
        assert_eq!(fmt_uptime(7500), "2h 05m");
        assert_eq!(fmt_uptime(3 * 86_400 + 4 * 3600), "3d 4h");
    }

    #[test]
    fn elapsed_formatting() {
        assert_eq!(fmt_elapsed(Duration::from_secs(65)), "01:05");
        assert_eq!(fmt_elapsed(Duration::from_secs(3725)), "1:02:05");
    }

    #[test]
    fn truncate_is_char_safe() {
        assert_eq!(truncate("abc", 10), "abc");
        assert!(truncate("ééééé", 3).starts_with('é'));
    }

    #[test]
    fn ellipsize_chars() {
        assert_eq!(ellipsize("abcdef", 4), "abc…");
        assert_eq!(ellipsize("abc", 4), "abc");
    }

    #[test]
    fn colors_thresholds() {
        assert_eq!(battery_color(80), theme::OK);
        assert_eq!(battery_color(30), theme::WARN);
        assert_eq!(battery_color(5), theme::ERR);
        assert_eq!(usage_color(90.0), theme::ERR);
        assert_eq!(temp_color(30.0), theme::OK);
    }

    #[test]
    fn connection_kinds() {
        assert_eq!(connection_kind("R5CT1234"), "USB");
        assert_eq!(connection_kind("192.168.1.5:5555"), "Wi-Fi");
    }

    #[test]
    fn hints() {
        assert!(state_hint("device").is_none());
        assert!(state_hint("unauthorized")
            .unwrap()
            .contains("USB debugging"));
        assert!(state_hint("weird").is_some());
    }

    #[test]
    fn spark_scaling() {
        let h = spark_heights(&[0.0, 50.0, 100.0, 200.0], 100.0, 20.0);
        assert_eq!(h, vec![1.0, 10.0, 20.0, 20.0]);
        assert_eq!(spark_heights(&[5.0], 0.0, 10.0), vec![10.0]);
        assert_eq!(tail(&[1, 2, 3, 4], 2), &[3, 4]);
        assert_eq!(tail(&[1], 5), &[1]);
    }
}
