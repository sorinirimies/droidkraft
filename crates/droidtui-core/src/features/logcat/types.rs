//! Logcat domain types: levels, entries, parsing, stats, and formatting.

use serde::Serialize;
use std::collections::VecDeque;
use std::fmt;

// ---------------------------------------------------------------------------
// SaveFormat
// ---------------------------------------------------------------------------

/// Format for saving log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFormat {
    /// Plain text — one raw logcat line per line.
    Text,
    /// JSON Lines — one JSON object per line.
    Json,
}

impl SaveFormat {
    pub fn cycle(&self) -> Self {
        match self {
            SaveFormat::Text => SaveFormat::Json,
            SaveFormat::Json => SaveFormat::Text,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            SaveFormat::Text => "TXT",
            SaveFormat::Json => "JSON",
        }
    }
    pub fn extension(&self) -> &'static str {
        match self {
            SaveFormat::Text => "log",
            SaveFormat::Json => "jsonl",
        }
    }
}

// ---------------------------------------------------------------------------
// LogLevel
// ---------------------------------------------------------------------------

/// Android logcat log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Unknown,
}

impl LogLevel {
    /// Parse a single character into a `LogLevel`.
    pub fn from_char(c: char) -> Self {
        match c {
            'V' => LogLevel::Verbose,
            'D' => LogLevel::Debug,
            'I' => LogLevel::Info,
            'W' => LogLevel::Warn,
            'E' => LogLevel::Error,
            'F' => LogLevel::Fatal,
            _ => LogLevel::Unknown,
        }
    }

    /// The single-character representation.
    pub fn as_char(&self) -> char {
        match self {
            LogLevel::Verbose => 'V',
            LogLevel::Debug => 'D',
            LogLevel::Info => 'I',
            LogLevel::Warn => 'W',
            LogLevel::Error => 'E',
            LogLevel::Fatal => 'F',
            LogLevel::Unknown => '?',
        }
    }

    /// Numeric ordering value used for filtering comparisons.
    /// Lower values are less severe.
    pub fn order(&self) -> u8 {
        match self {
            LogLevel::Verbose => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
            LogLevel::Fatal => 5,
            LogLevel::Unknown => 0,
        }
    }

    /// A static slice of all concrete log levels (V through F).
    pub fn all() -> &'static [LogLevel] {
        &[
            LogLevel::Verbose,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
            LogLevel::Fatal,
        ]
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

// ---------------------------------------------------------------------------
// LogEntry
// ---------------------------------------------------------------------------

/// Detect whether a message line looks like a stack trace continuation.
pub fn is_continuation_line(msg: &str) -> bool {
    let trimmed = msg.trim_start();
    trimmed.starts_with("at ")
        || trimmed.starts_with("Caused by:")
        || trimmed.starts_with("... ")
        || (trimmed.starts_with("java.")
            || trimmed.starts_with("kotlin.")
            || trimmed.starts_with("android.")
            || trimmed.starts_with("javax."))
            && (trimmed.contains("Exception") || trimmed.contains("Error"))
}

/// A single parsed logcat entry.
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    /// The original raw line from logcat.
    pub raw: String,
    /// Timestamp string (e.g. "01-15 12:34:56.789").
    pub timestamp: Option<String>,
    /// Process ID.
    pub pid: Option<String>,
    /// Thread ID.
    pub tid: Option<String>,
    /// Log level.
    pub level: LogLevel,
    /// Log tag (e.g. "ActivityManager").
    pub tag: Option<String>,
    /// The log message body.
    pub message: String,
    /// Whether this line is a stack trace continuation.
    pub is_stack_continuation: bool,
}

impl LogEntry {
    /// Parse a raw logcat line into a structured `LogEntry`.
    ///
    /// Supports the **threadtime** and **brief** formats; falls back to storing
    /// the whole line as the message with `LogLevel::Unknown`.
    pub fn parse(line: &str) -> Self {
        if let Some(entry) = Self::parse_threadtime(line) {
            return entry;
        }
        if let Some(entry) = Self::parse_brief(line) {
            return entry;
        }
        LogEntry {
            raw: line.to_string(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Unknown,
            tag: None,
            message: line.to_string(),
            is_stack_continuation: is_continuation_line(line),
        }
    }

    /// Parse the threadtime format:
    /// `MM-DD HH:MM:SS.mmm  PID  TID LEVEL TAG: message`
    fn parse_threadtime(line: &str) -> Option<LogEntry> {
        if line.len() < 30 {
            return None;
        }
        let bytes = line.as_bytes();
        if bytes.len() < 6 {
            return None;
        }
        if !(bytes[0].is_ascii_digit()
            && bytes[1].is_ascii_digit()
            && bytes[2] == b'-'
            && bytes[3].is_ascii_digit()
            && bytes[4].is_ascii_digit()
            && bytes[5] == b' ')
        {
            return None;
        }

        let timestamp = &line[0..18];
        let rest = line[18..].trim_start();

        let pid = rest.split_whitespace().next()?.to_string();
        let remaining = rest[pid.len()..].trim_start();

        let tid = remaining.split_whitespace().next()?.to_string();
        let remaining2 = remaining[tid.len()..].trim_start();

        let level_char = remaining2.chars().next()?;
        let level = LogLevel::from_char(level_char);

        let after_level = remaining2[level_char.len_utf8()..].trim_start();
        let colon_pos = after_level.find(": ")?;
        let tag = after_level[..colon_pos].trim().to_string();
        let message = after_level[colon_pos + 2..].to_string();

        Some(LogEntry {
            raw: line.to_string(),
            timestamp: Some(timestamp.to_string()),
            pid: Some(pid),
            tid: Some(tid),
            level,
            tag: if tag.is_empty() { None } else { Some(tag) },
            is_stack_continuation: is_continuation_line(&message),
            message,
        })
    }

    /// Parse the brief format: `LEVEL/TAG(PID): message`
    fn parse_brief(line: &str) -> Option<LogEntry> {
        let mut chars = line.chars();
        let level_char = chars.next()?;
        if chars.next()? != '/' {
            return None;
        }
        let level = LogLevel::from_char(level_char);
        let rest = &line[2..];

        let paren_open = rest.find('(')?;
        let tag = rest[..paren_open].to_string();

        let after_paren = &rest[paren_open + 1..];
        let paren_close = after_paren.find(')')?;
        let pid = after_paren[..paren_close].trim().to_string();

        let after_close = &after_paren[paren_close + 1..];
        let message = if let Some(stripped) = after_close.strip_prefix(": ") {
            stripped.to_string()
        } else if let Some(stripped) = after_close.strip_prefix(':') {
            stripped.trim_start().to_string()
        } else {
            after_close.to_string()
        };

        Some(LogEntry {
            raw: line.to_string(),
            timestamp: None,
            pid: Some(pid),
            tid: None,
            level,
            tag: if tag.is_empty() { None } else { Some(tag) },
            is_stack_continuation: is_continuation_line(&message),
            message,
        })
    }
}

// ---------------------------------------------------------------------------
// LogStats
// ---------------------------------------------------------------------------

/// Tracks per-level counts and lines-per-second rate for a stats bar.
#[derive(Debug, Clone)]
pub struct LogStats {
    /// Counts indexed by `LogLevel::order()` (0–6).
    pub counts: [u64; 7],
    /// Estimated lines received per second.
    pub lines_per_sec: f64,
    samples: VecDeque<u64>,
    _last_total: u64,
}

impl LogStats {
    pub fn new() -> Self {
        Self {
            counts: [0; 7],
            lines_per_sec: 0.0,
            samples: VecDeque::with_capacity(32),
            _last_total: 0,
        }
    }

    /// Record one entry at the given level.
    pub fn record(&mut self, level: &LogLevel) {
        self.counts[level.order() as usize] += 1;
    }

    /// Update the lines-per-second estimate from the running total.
    pub fn update_rate(&mut self, total_received: u64) {
        self.samples.push_back(total_received);
        if self.samples.len() > 30 {
            self.samples.pop_front();
        }
        if self.samples.len() >= 2 {
            let newest = *self.samples.back().unwrap();
            let oldest = *self.samples.front().unwrap();
            let window = self.samples.len() as f64;
            self.lines_per_sec = (newest - oldest) as f64 / (window / 30.0);
        }
    }

    /// Reset all stats.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for LogStats {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// Pretty-print embedded JSON in a log message, if present.
pub fn try_format_json(message: &str) -> Option<String> {
    let trimmed = message.trim();
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            return serde_json::to_string_pretty(&value).ok();
        }
    }
    for sep in [':', '='] {
        if let Some(pos) = trimmed.find(sep) {
            let after = trimmed[pos + 1..].trim();
            if (after.starts_with('{') && after.ends_with('}'))
                || (after.starts_with('[') && after.ends_with(']'))
            {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(after) {
                    let prefix = &trimmed[..pos + 1];
                    if let Ok(formatted) = serde_json::to_string_pretty(&value) {
                        return Some(format!("{} {}", prefix.trim(), formatted));
                    }
                }
            }
        }
    }
    None
}

/// Compute wrapped display lines for a log entry message.
///
/// The first line is unindented; continuation lines are indented by `indent`.
pub fn wrap_entry_message(message: &str, max_width: usize, indent: usize) -> Vec<String> {
    if max_width <= indent || message.is_empty() {
        return vec![message.to_string()];
    }

    let first_width = max_width;
    let cont_width = max_width.saturating_sub(indent);

    let mut lines = Vec::new();
    let mut remaining = message;
    let mut is_first = true;

    while !remaining.is_empty() {
        let width = if is_first { first_width } else { cont_width };
        if remaining.len() <= width {
            if is_first {
                lines.push(remaining.to_string());
            } else {
                lines.push(format!("{}{}", " ".repeat(indent), remaining));
            }
            break;
        }
        let break_at = remaining[..width].rfind(' ').unwrap_or(width);
        let break_at = if break_at == 0 { width } else { break_at };
        if is_first {
            lines.push(remaining[..break_at].to_string());
        } else {
            lines.push(format!("{}{}", " ".repeat(indent), &remaining[..break_at]));
        }
        remaining = remaining[break_at..].trim_start();
        is_first = false;
    }

    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_from_char() {
        assert_eq!(LogLevel::from_char('E'), LogLevel::Error);
        assert_eq!(LogLevel::from_char('X'), LogLevel::Unknown);
    }

    #[test]
    fn level_char_roundtrip() {
        for l in LogLevel::all() {
            assert_eq!(LogLevel::from_char(l.as_char()), *l);
        }
    }

    #[test]
    fn level_ordering() {
        assert!(LogLevel::Verbose < LogLevel::Error);
        assert!(LogLevel::Warn < LogLevel::Fatal);
    }

    #[test]
    fn save_format_cycle_and_ext() {
        assert_eq!(SaveFormat::Text.cycle(), SaveFormat::Json);
        assert_eq!(SaveFormat::Json.extension(), "jsonl");
    }

    #[test]
    fn parse_threadtime() {
        let line = "01-15 12:34:56.789  1234  5678 I ActivityManager: Start proc com.example";
        let e = LogEntry::parse(line);
        assert_eq!(e.timestamp.as_deref(), Some("01-15 12:34:56.789"));
        assert_eq!(e.pid.as_deref(), Some("1234"));
        assert_eq!(e.tid.as_deref(), Some("5678"));
        assert_eq!(e.level, LogLevel::Info);
        assert_eq!(e.tag.as_deref(), Some("ActivityManager"));
        assert_eq!(e.message, "Start proc com.example");
    }

    #[test]
    fn parse_brief() {
        let e = LogEntry::parse("E/AndroidRuntime(1234): FATAL EXCEPTION");
        assert_eq!(e.level, LogLevel::Error);
        assert_eq!(e.tag.as_deref(), Some("AndroidRuntime"));
        assert_eq!(e.pid.as_deref(), Some("1234"));
        assert_eq!(e.message, "FATAL EXCEPTION");
    }

    #[test]
    fn parse_fallback_unknown() {
        let e = LogEntry::parse("some random line");
        assert_eq!(e.level, LogLevel::Unknown);
        assert_eq!(e.message, "some random line");
    }

    #[test]
    fn continuation_detection() {
        assert!(is_continuation_line("    at com.example.Foo.bar(Foo.java:42)"));
        assert!(is_continuation_line("Caused by: java.lang.NullPointerException"));
        assert!(!is_continuation_line("just a normal message"));
    }

    #[test]
    fn stats_record_and_reset() {
        let mut s = LogStats::new();
        s.record(&LogLevel::Error);
        s.record(&LogLevel::Error);
        assert_eq!(s.counts[LogLevel::Error.order() as usize], 2);
        s.reset();
        assert_eq!(s.counts[LogLevel::Error.order() as usize], 0);
    }

    #[test]
    fn format_json_whole() {
        let out = try_format_json(r#"{"a":1}"#).unwrap();
        assert!(out.contains("\"a\": 1"));
    }

    #[test]
    fn format_json_none_for_plain() {
        assert!(try_format_json("no json here").is_none());
    }

    #[test]
    fn wrap_short_message_single_line() {
        assert_eq!(wrap_entry_message("hi", 80, 4), vec!["hi".to_string()]);
    }

    #[test]
    fn wrap_long_message_multiple_lines() {
        let msg = "word ".repeat(40);
        let lines = wrap_entry_message(msg.trim(), 40, 4);
        assert!(lines.len() > 1);
    }
}
