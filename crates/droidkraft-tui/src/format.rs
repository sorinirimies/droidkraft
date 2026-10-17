//! Pure formatting helpers shared by the device views.

use std::time::Duration;

/// `1536` MiB → `1.5 GiB`; below 1 GiB → `512 MiB`.
pub fn fmt_mib(mib: u64) -> String {
    if mib >= 1024 {
        format!("{:.1} GiB", mib as f64 / 1024.0)
    } else {
        format!("{mib} MiB")
    }
}

/// Uptime seconds → `3d 4h`, `2h 05m`, `7m`, `42s`.
pub fn fmt_uptime(secs: u64) -> String {
    let (d, h, m) = (secs / 86_400, secs % 86_400 / 3600, secs % 3600 / 60);
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m:02}m")
    } else if m > 0 {
        format!("{m}m")
    } else {
        format!("{secs}s")
    }
}

/// Elapsed recording time as `MM:SS` (or `H:MM:SS` past an hour).
pub fn fmt_timer(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

/// Truncate to `max` chars with an ellipsis.
pub fn truncate(s: &str, max: usize) -> String {
    crate::ui::layout::truncate_to(s, max)
}

/// Optional value → text or `-`.
pub fn opt<T: std::fmt::Display>(v: Option<T>) -> String {
    v.map_or_else(|| "-".into(), |v| v.to_string())
}

/// Last path component for display.
pub fn file_name(p: &std::path::Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mib_formats() {
        assert_eq!(fmt_mib(512), "512 MiB");
        assert_eq!(fmt_mib(1536), "1.5 GiB");
    }

    #[test]
    fn uptime_formats() {
        assert_eq!(fmt_uptime(42), "42s");
        assert_eq!(fmt_uptime(7 * 60 + 3), "7m");
        assert_eq!(fmt_uptime(2 * 3600 + 5 * 60), "2h 05m");
        assert_eq!(fmt_uptime(3 * 86_400 + 4 * 3600), "3d 4h");
    }

    #[test]
    fn timer_formats() {
        assert_eq!(fmt_timer(Duration::from_secs(65)), "01:05");
        assert_eq!(fmt_timer(Duration::from_secs(3725)), "1:02:05");
    }

    #[test]
    fn truncate_and_opt() {
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("abc", 4), "abc");
        assert_eq!(opt(Some(3)), "3");
        assert_eq!(opt::<u8>(None), "-");
    }
}
