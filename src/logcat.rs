//! Logcat Viewer Module
//!
//! This module provides a comprehensive logcat viewer for the TUI application.
//! It handles streaming logcat output from connected Android devices via ADB,
//! parsing log entries, filtering, and managing the view state.

use adb_client::ADBServerDevice;
use ratatui::style::Color;
use std::fmt;
use std::io::{self, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::mpsc;

// ---------------------------------------------------------------------------
// LogLevel
// ---------------------------------------------------------------------------

/// Represents Android logcat log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// Return the single-character representation.
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

    /// Color used for the log message text.
    pub fn color(&self) -> Color {
        match self {
            LogLevel::Verbose => Color::DarkGray,
            LogLevel::Debug => Color::Cyan,
            LogLevel::Info => Color::Green,
            LogLevel::Warn => Color::Yellow,
            LogLevel::Error => Color::Red,
            LogLevel::Fatal => Color::LightRed,
            LogLevel::Unknown => Color::Gray,
        }
    }

    /// Brighter color used for the level badge / label.
    pub fn label_color(&self) -> Color {
        match self {
            LogLevel::Verbose => Color::Gray,
            LogLevel::Debug => Color::LightCyan,
            LogLevel::Info => Color::LightGreen,
            LogLevel::Warn => Color::LightYellow,
            LogLevel::Error => Color::LightRed,
            LogLevel::Fatal => Color::Magenta,
            LogLevel::Unknown => Color::White,
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

    /// Returns a static slice of all concrete log levels (V through F).
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

/// A single parsed logcat entry.
#[derive(Debug, Clone)]
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
}

impl LogEntry {
    /// Parse a raw logcat line into a structured `LogEntry`.
    ///
    /// Supports two formats:
    ///
    /// 1. **threadtime** (default for `logcat -v threadtime`):
    ///    `MM-DD HH:MM:SS.mmm  PID  TID LEVEL TAG: message`
    ///
    /// 2. **brief** (`logcat -v brief`):
    ///    `LEVEL/TAG(PID): message`
    ///
    /// If neither format matches, the entire line is stored as the message
    /// with `LogLevel::Unknown`.
    pub fn parse(line: &str) -> Self {
        // Try threadtime format first
        if let Some(entry) = Self::parse_threadtime(line) {
            return entry;
        }

        // Try brief format
        if let Some(entry) = Self::parse_brief(line) {
            return entry;
        }

        // Fallback: raw line as message
        LogEntry {
            raw: line.to_string(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Unknown,
            tag: None,
            message: line.to_string(),
        }
    }

    /// Attempt to parse the threadtime format:
    /// `MM-DD HH:MM:SS.mmm  PID  TID LEVEL TAG     : message`
    ///
    /// The timestamp portion is `MM-DD HH:MM:SS.mmm` (18 chars).
    /// After that there are spaces, PID, spaces, TID, a space, a single
    /// level char, a space, then the tag (possibly with trailing spaces)
    /// followed by `: ` and the message.
    fn parse_threadtime(line: &str) -> Option<LogEntry> {
        // Minimum sanity length check
        if line.len() < 30 {
            return None;
        }

        // Expect date pattern: DD-DD at start (MM-DD)
        let bytes = line.as_bytes();
        if bytes.len() < 5 {
            return None;
        }

        // Check for date-like prefix: digit digit '-' digit digit ' '
        if !(bytes[0].is_ascii_digit()
            && bytes[1].is_ascii_digit()
            && bytes[2] == b'-'
            && bytes[3].is_ascii_digit()
            && bytes[4].is_ascii_digit()
            && bytes[5] == b' ')
        {
            return None;
        }

        // Find the timestamp portion (MM-DD HH:MM:SS.mmm) — 18 chars
        // e.g. "01-15 12:34:56.789"
        let timestamp = &line[0..18];

        // Rest after timestamp: should start with spaces then PID
        let rest = &line[18..];
        let rest = rest.trim_start();

        // Split on whitespace to get PID, TID, LEVEL, then tag: message
        let mut parts = rest.splitn(4, char::is_whitespace);

        let pid = parts.next()?.to_string();
        // After PID there may be extra spaces before TID
        let remaining = rest[pid.len()..].trim_start();
        let mut parts2 = remaining.splitn(3, char::is_whitespace);

        let tid = parts2.next()?.to_string();
        let remaining2 = remaining[tid.len()..].trim_start();

        // Next char should be the level
        let level_char = remaining2.chars().next()?;
        let level = LogLevel::from_char(level_char);
        if level == LogLevel::Unknown && level_char != '?' {
            // If we didn't recognise the level char and it's not '?',
            // this probably isn't threadtime format.
            // But let's be lenient for 'S' (silent) etc.
        }

        let after_level = &remaining2[level_char.len_utf8()..].trim_start();

        // Now we expect "TAG     : message" — find the ": " separator
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
            message,
        })
    }

    /// Attempt to parse the brief format:
    /// `LEVEL/TAG(PID): message`
    fn parse_brief(line: &str) -> Option<LogEntry> {
        // Must start with a single level char followed by '/'
        let mut chars = line.chars();
        let level_char = chars.next()?;
        let slash = chars.next()?;
        if slash != '/' {
            return None;
        }

        let level = LogLevel::from_char(level_char);

        let rest = &line[2..]; // after "L/"

        // Find '(' for PID
        let paren_open = rest.find('(')?;
        let tag = rest[..paren_open].to_string();

        let after_paren = &rest[paren_open + 1..];
        let paren_close = after_paren.find(')')?;
        let pid = after_paren[..paren_close].trim().to_string();

        // After ')' expect ": " then message
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
            message,
        })
    }
}

// ---------------------------------------------------------------------------
// FilterField
// ---------------------------------------------------------------------------

/// Which input field is currently active for keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterField {
    Search,
    Tag,
    Package,
    None,
}

// ---------------------------------------------------------------------------
// LogcatFilter
// ---------------------------------------------------------------------------

/// Filter state for the logcat viewer.
#[derive(Debug, Clone)]
pub struct LogcatFilter {
    /// Free-text search (case-insensitive substring match).
    pub search_query: String,
    /// Cursor position within the search field.
    pub search_cursor: usize,
    /// Minimum log level to display.
    pub min_level: LogLevel,
    /// Tag substring filter.
    pub tag_filter: String,
    /// Cursor position within the tag filter field.
    pub tag_cursor: usize,
    /// PID / package filter.
    pub package_filter: String,
    /// Cursor position within the package filter field.
    pub package_cursor: usize,
    /// Which input field is currently receiving keyboard input.
    pub active_field: FilterField,
}

impl Default for LogcatFilter {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            search_cursor: 0,
            min_level: LogLevel::Verbose,
            tag_filter: String::new(),
            tag_cursor: 0,
            package_filter: String::new(),
            package_cursor: 0,
            active_field: FilterField::None,
        }
    }
}

impl LogcatFilter {
    /// Check whether a log entry passes all active filters.
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Level filter
        if entry.level.order() < self.min_level.order() {
            return false;
        }

        // Search query (case-insensitive substring on raw line)
        if !self.search_query.is_empty() {
            let query_lower = self.search_query.to_lowercase();
            let raw_lower = entry.raw.to_lowercase();
            if !raw_lower.contains(&query_lower) {
                return false;
            }
        }

        // Tag filter (case-insensitive substring on tag)
        if !self.tag_filter.is_empty() {
            let tag_lower = self.tag_filter.to_lowercase();
            match &entry.tag {
                Some(tag) => {
                    if !tag.to_lowercase().contains(&tag_lower) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Package / PID filter (case-insensitive substring on PID)
        if !self.package_filter.is_empty() {
            let pkg_lower = self.package_filter.to_lowercase();
            match &entry.pid {
                Some(pid) => {
                    if !pid.to_lowercase().contains(&pkg_lower) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }

    /// Clear the search query and reset its cursor.
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_cursor = 0;
    }

    /// Clear the tag filter and reset its cursor.
    pub fn clear_tag(&mut self) {
        self.tag_filter.clear();
        self.tag_cursor = 0;
    }

    /// Clear the package filter and reset its cursor.
    pub fn clear_package(&mut self) {
        self.package_filter.clear();
        self.package_cursor = 0;
    }

    /// Insert a character at the cursor position of the currently active field.
    pub fn insert_char(&mut self, c: char) {
        let (field, cursor) = self.active_field_mut();
        let byte_idx = char_to_byte_index(field, *cursor);
        field.insert(byte_idx, c);
        *cursor += 1;
    }

    /// Delete the character before the cursor (backspace) in the active field.
    pub fn delete_char(&mut self) {
        let (field, cursor) = self.active_field_mut();
        if *cursor > 0 {
            *cursor -= 1;
            let byte_idx = char_to_byte_index(field, *cursor);
            field.remove(byte_idx);
        }
    }

    /// Delete the character at the cursor (forward delete) in the active field.
    pub fn delete_char_forward(&mut self) {
        let (field, cursor) = self.active_field_mut();
        let char_count = field.chars().count();
        if *cursor < char_count {
            let byte_idx = char_to_byte_index(field, *cursor);
            field.remove(byte_idx);
        }
    }

    /// Move the cursor one position to the left in the active field.
    pub fn move_cursor_left(&mut self) {
        let (_field, cursor) = self.active_field_mut();
        *cursor = cursor.saturating_sub(1);
    }

    /// Move the cursor one position to the right in the active field.
    pub fn move_cursor_right(&mut self) {
        let (field, cursor) = self.active_field_mut();
        let char_count = field.chars().count();
        if *cursor < char_count {
            *cursor += 1;
        }
    }

    /// Cycle the minimum log level through V → D → I → W → E → F → V.
    pub fn cycle_level(&mut self) {
        self.min_level = match self.min_level {
            LogLevel::Verbose => LogLevel::Debug,
            LogLevel::Debug => LogLevel::Info,
            LogLevel::Info => LogLevel::Warn,
            LogLevel::Warn => LogLevel::Error,
            LogLevel::Error => LogLevel::Fatal,
            LogLevel::Fatal => LogLevel::Verbose,
            LogLevel::Unknown => LogLevel::Verbose,
        };
    }

    /// Returns a mutable reference to the active field string and its cursor.
    /// If no field is active, returns the search field by default.
    fn active_field_mut(&mut self) -> (&mut String, &mut usize) {
        match self.active_field {
            FilterField::Search => (&mut self.search_query, &mut self.search_cursor),
            FilterField::Tag => (&mut self.tag_filter, &mut self.tag_cursor),
            FilterField::Package => (&mut self.package_filter, &mut self.package_cursor),
            FilterField::None => (&mut self.search_query, &mut self.search_cursor),
        }
    }
}

/// Convert a char-based cursor index to a byte index within a string.
fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

// ---------------------------------------------------------------------------
// ChannelWriter
// ---------------------------------------------------------------------------

/// A [`Write`] implementation that buffers incoming bytes, splits on newline
/// boundaries, and sends each complete line through an `mpsc::SyncSender<String>`.
///
/// When the bounded channel is full the `write` call blocks, applying natural
/// backpressure to the ADB streaming thread so memory usage stays bounded.
pub struct ChannelWriter {
    sender: mpsc::SyncSender<String>,
    buffer: Vec<u8>,
}

impl ChannelWriter {
    /// Create a new `ChannelWriter` wrapping the given sender.
    pub fn new(sender: mpsc::SyncSender<String>) -> Self {
        Self {
            sender,
            buffer: Vec::with_capacity(4096),
        }
    }
}

impl Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        // Send all complete lines (delimited by '\n')
        while let Some(newline_pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line_bytes: Vec<u8> = self.buffer.drain(..=newline_pos).collect();
            // Trim the trailing newline (and optional \r)
            let line = String::from_utf8_lossy(&line_bytes)
                .trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string();
            if self.sender.send(line).is_err() {
                // Receiver dropped — signal the caller to stop
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "logcat receiver dropped",
                ));
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            let remaining = String::from_utf8_lossy(&self.buffer).to_string();
            self.buffer.clear();
            if !remaining.is_empty() {
                let _ = self.sender.send(remaining);
            }
        }
        Ok(())
    }
}

impl fmt::Debug for ChannelWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChannelWriter")
            .field("buffer_len", &self.buffer.len())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// LogcatState
// ---------------------------------------------------------------------------

/// Maximum number of log entries kept in memory.
const MAX_ENTRIES: usize = 50_000;

/// Maximum number of lines drained from the channel per tick to keep the UI
/// responsive.
const MAX_DRAIN_PER_TICK: usize = 500;

/// Capacity of the bounded channel between the streaming thread and the UI.
/// When the channel is full the background thread blocks, applying natural
/// backpressure so memory usage stays bounded even during logcat bursts.
const CHANNEL_CAPACITY: usize = 10_000;

/// Full state of the logcat viewer.
pub struct LogcatState {
    /// All received log entries (ring buffer capped at `MAX_ENTRIES`).
    pub entries: Vec<LogEntry>,
    /// Indices into `entries` that match the current filter.
    pub filtered_indices: Vec<usize>,
    /// Current filter settings.
    pub filter: LogcatFilter,
    /// Current scroll offset within the filtered view (index of the first visible line).
    pub scroll_position: usize,
    /// Whether the view should automatically stick to the bottom.
    pub auto_scroll: bool,
    /// Whether ingestion of new entries is paused.
    pub paused: bool,
    /// Channel receiver for lines coming from the background streaming thread.
    receiver: Option<mpsc::Receiver<String>>,
    /// Number of entries trimmed from the front of `entries` since the
    /// beginning.  Used to adjust `filtered_indices` in-place instead of
    /// doing a full rebuild on every trim.
    trimmed_total: usize,
    /// Whether the background streaming thread is (believed to be) running.
    pub is_streaming: bool,
    /// Total number of raw lines received since streaming started.
    pub total_received: u64,
    /// Optional status or error message to display in the UI.
    pub status_message: Option<String>,
    /// Whether long lines should be word-wrapped in the view.
    pub word_wrap: bool,
    /// Last known viewport height (set by the view layer each frame).
    /// Used to compute the correct `scroll_position` when transitioning
    /// out of `auto_scroll` mode.
    pub viewport_height: usize,
}

impl LogcatState {
    /// Create a new, empty `LogcatState`.
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(1024),
            filtered_indices: Vec::with_capacity(1024),
            filter: LogcatFilter::default(),
            scroll_position: 0,
            auto_scroll: true,
            paused: false,
            receiver: None,
            is_streaming: false,
            total_received: 0,
            status_message: None,
            word_wrap: false,
            viewport_height: 30,
            trimmed_total: 0,
        }
    }

    /// Start streaming logcat from the device identified by `serial`.
    ///
    /// Spawns a background `std::thread` that connects to the ADB server at
    /// `127.0.0.1:5037`, creates an `ADBServerDevice`, and calls
    /// `device.get_logs(writer)` where `writer` is a [`ChannelWriter`] that
    /// sends complete lines back to this state via an `mpsc` channel.
    pub fn start_streaming(&mut self, serial: String) {
        // Stop any existing stream first
        self.stop_streaming();

        let (tx, rx) = mpsc::sync_channel::<String>(CHANNEL_CAPACITY);
        self.receiver = Some(rx);
        self.is_streaming = true;
        self.status_message = Some(format!("Streaming logcat from {}…", serial));

        std::thread::spawn(move || {
            let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);
            let mut device = ADBServerDevice::new(serial, Some(addr));
            let writer = ChannelWriter::new(tx.clone());

            if let Err(e) = device.get_logs(writer) {
                let _ = tx.send(format!("--- LOGCAT ERROR: {} ---", e));
            }
        });
    }

    /// Stop the current logcat stream by dropping the receiver, which causes
    /// the background thread's next send to fail with `BrokenPipe`.
    pub fn stop_streaming(&mut self) {
        self.receiver = None;
        self.is_streaming = false;
        self.status_message = Some("Logcat streaming stopped.".to_string());
    }

    /// Poll the channel for new log lines. Should be called on each UI tick.
    ///
    /// Drains up to [`MAX_DRAIN_PER_TICK`] lines to avoid blocking the render
    /// loop. Each line is parsed into a [`LogEntry`], appended to `entries`,
    /// and the filtered index list is updated incrementally.
    pub fn poll_new_entries(&mut self) {
        let receiver = match &self.receiver {
            Some(rx) => rx,
            None => return,
        };

        let mut new_count: usize = 0;

        for _ in 0..MAX_DRAIN_PER_TICK {
            match receiver.try_recv() {
                Ok(line) => {
                    self.total_received += 1;

                    if self.paused {
                        // Still consume to avoid backpressure, but don't store.
                        continue;
                    }

                    let entry = LogEntry::parse(&line);
                    let idx = self.entries.len();
                    self.entries.push(entry);
                    new_count += 1;

                    // Incremental filter update: check the new entry
                    if self.filter.matches(&self.entries[idx]) {
                        self.filtered_indices.push(idx);
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.is_streaming = false;
                    self.status_message =
                        Some("Logcat stream ended (thread disconnected).".to_string());
                    break;
                }
            }
        }

        // Trim oldest entries if we exceed the cap
        if self.entries.len() > MAX_ENTRIES {
            let excess = self.entries.len() - MAX_ENTRIES;
            self.entries.drain(0..excess);
            self.trimmed_total += excess;

            // Adjust filtered_indices in-place: subtract `excess` from each
            // index and drop any that pointed into the trimmed region.
            // This is O(filtered_len) instead of O(entries_len).
            let mut write = 0;
            for read in 0..self.filtered_indices.len() {
                let idx = self.filtered_indices[read];
                if idx >= excess {
                    self.filtered_indices[write] = idx - excess;
                    write += 1;
                }
            }
            self.filtered_indices.truncate(write);

            // Clamp scroll_position
            let len = self.filtered_indices.len();
            if self.scroll_position >= len && len > 0 {
                self.scroll_position = len.saturating_sub(self.viewport_height);
            } else if len == 0 {
                self.scroll_position = 0;
            }
        }

        if new_count > 0 && self.auto_scroll {
            // When auto_scroll is on, visible_entries will show the last
            // `height` entries regardless of scroll_position.  Keep it
            // near the end so transitioning out of auto_scroll doesn't jump.
            self.scroll_position = self.filtered_indices.len();
        }
    }

    /// Recompute `filtered_indices` from scratch based on the current filter.
    pub fn rebuild_filtered(&mut self) {
        self.filtered_indices.clear();
        for (i, entry) in self.entries.iter().enumerate() {
            if self.filter.matches(entry) {
                self.filtered_indices.push(i);
            }
        }

        // Only clamp if out of bounds, don't force auto scroll position
        let len = self.filtered_indices.len();
        if len == 0 {
            self.scroll_position = 0;
        } else if self.scroll_position >= len {
            self.scroll_position = len.saturating_sub(1);
        }
    }

    /// Return a slice of `filtered_indices` that represents the currently
    /// visible window given the viewport `height`.
    ///
    /// `scroll_position` always represents the index of the **first** visible
    /// line. When `auto_scroll` is enabled the last `height` entries are shown
    /// regardless of `scroll_position`.
    pub fn visible_entries(&mut self, height: usize) -> Vec<usize> {
        self.viewport_height = height;

        if self.filtered_indices.is_empty() || height == 0 {
            return vec![];
        }

        let total = self.filtered_indices.len();

        let start = if self.auto_scroll {
            let s = total.saturating_sub(height);
            // Keep scroll_position in sync so that transitioning out of
            // auto_scroll doesn't jump.
            self.scroll_position = s;
            s
        } else {
            // Clamp so that we never start beyond the point where a full
            // page of `height` entries can be shown (when possible).
            self.scroll_position
                .min(total.saturating_sub(height).max(0))
        };

        let end = (start + height).min(total);
        self.filtered_indices[start..end].to_vec()
    }

    /// Scroll up by `n` lines. Disables auto-scroll.
    pub fn scroll_up(&mut self, n: usize) {
        if self.auto_scroll {
            // Snapshot current position before leaving auto-scroll so
            // the view stays where it was instead of jumping to 0.
            let total = self.filtered_indices.len();
            self.scroll_position = total.saturating_sub(self.viewport_height);
        }
        self.auto_scroll = false;
        self.scroll_position = self.scroll_position.saturating_sub(n);
    }

    /// Scroll down by `n` lines. Disables auto-scroll.
    pub fn scroll_down(&mut self, n: usize) {
        if self.auto_scroll {
            // Already pinned to the bottom — nothing below to scroll to.
            return;
        }
        let max_scroll = self
            .filtered_indices
            .len()
            .saturating_sub(self.viewport_height);
        self.scroll_position = (self.scroll_position + n).min(max_scroll);
    }

    /// Jump to the bottom and re-enable auto-scroll.
    pub fn scroll_to_bottom(&mut self) {
        self.auto_scroll = true;
        let total = self.filtered_indices.len();
        self.scroll_position = total.saturating_sub(self.viewport_height);
    }

    /// Jump to the top of the log. Disables auto-scroll.
    pub fn scroll_to_top(&mut self) {
        self.auto_scroll = false;
        self.scroll_position = 0;
    }

    /// Clear all entries and filtered indices.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.filtered_indices.clear();
        self.scroll_position = 0;
        self.total_received = 0;
        self.trimmed_total = 0;
        self.auto_scroll = true;
    }

    /// Return the directory path that would be used for "Save Here".
    /// If a file explorer is active, returns its `current_dir`; otherwise
    /// falls back to the process working directory.
    pub fn default_save_dir() -> std::path::PathBuf {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    }

    /// Build a timestamped default filename for saving logs.
    pub fn default_save_filename() -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("logcat_{}.log", now)
    }

    /// Toggle the paused state.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if self.paused {
            self.status_message = Some("Logcat paused.".to_string());
        } else {
            self.status_message = Some("Logcat resumed.".to_string());
        }
    }

    /// Number of entries matching the current filter.
    pub fn entry_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Total number of entries stored (before filtering).
    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    /// Save all entries to a file at the given path.
    /// Returns the number of lines written or an error.
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<usize> {
        use std::io::BufWriter;
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        let mut count = 0;
        for entry in &self.entries {
            writeln!(writer, "{}", entry.raw)?;
            count += 1;
        }
        writer.flush()?;
        Ok(count)
    }

    /// Save only filtered entries to a file at the given path.
    pub fn save_filtered_to_file(&self, path: &std::path::Path) -> std::io::Result<usize> {
        use std::io::BufWriter;
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        let mut count = 0;
        for &idx in &self.filtered_indices {
            if let Some(entry) = self.entries.get(idx) {
                writeln!(writer, "{}", entry.raw)?;
                count += 1;
            }
        }
        writer.flush()?;
        Ok(count)
    }

    /// Toggle word wrapping for long lines.
    pub fn toggle_word_wrap(&mut self) {
        self.word_wrap = !self.word_wrap;
    }
}

impl Default for LogcatState {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for LogcatState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogcatState")
            .field("entries_len", &self.entries.len())
            .field("filtered_len", &self.filtered_indices.len())
            .field("filter", &self.filter)
            .field("scroll_position", &self.scroll_position)
            .field("auto_scroll", &self.auto_scroll)
            .field("paused", &self.paused)
            .field(
                "receiver",
                &if self.receiver.is_some() {
                    "Some(..)"
                } else {
                    "None"
                },
            )
            .field("is_streaming", &self.is_streaming)
            .field("total_received", &self.total_received)
            .field("trimmed_total", &self.trimmed_total)
            .field("status_message", &self.status_message)
            .field("word_wrap", &self.word_wrap)
            .field("viewport_height", &self.viewport_height)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- LogLevel tests -----------------------------------------------------

    #[test]
    fn test_log_level_from_char() {
        assert_eq!(LogLevel::from_char('V'), LogLevel::Verbose);
        assert_eq!(LogLevel::from_char('D'), LogLevel::Debug);
        assert_eq!(LogLevel::from_char('I'), LogLevel::Info);
        assert_eq!(LogLevel::from_char('W'), LogLevel::Warn);
        assert_eq!(LogLevel::from_char('E'), LogLevel::Error);
        assert_eq!(LogLevel::from_char('F'), LogLevel::Fatal);
        assert_eq!(LogLevel::from_char('X'), LogLevel::Unknown);
    }

    #[test]
    fn test_log_level_as_char_roundtrip() {
        for level in LogLevel::all() {
            assert_eq!(LogLevel::from_char(level.as_char()), *level);
        }
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Verbose < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Fatal);
    }

    #[test]
    fn test_log_level_all_count() {
        assert_eq!(LogLevel::all().len(), 6);
    }

    // -- LogEntry parsing tests ---------------------------------------------

    #[test]
    fn test_parse_threadtime_format() {
        let line = "01-15 12:34:56.789  1234  5678 I ActivityManager: Start proc com.example";
        let entry = LogEntry::parse(line);
        assert_eq!(entry.timestamp.as_deref(), Some("01-15 12:34:56.789"));
        assert_eq!(entry.pid.as_deref(), Some("1234"));
        assert_eq!(entry.tid.as_deref(), Some("5678"));
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.tag.as_deref(), Some("ActivityManager"));
        assert_eq!(entry.message, "Start proc com.example");
    }

    #[test]
    fn test_parse_brief_format() {
        let line = "I/ActivityManager( 1234): Start proc com.example";
        let entry = LogEntry::parse(line);
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.tag.as_deref(), Some("ActivityManager"));
        assert_eq!(entry.pid.as_deref(), Some("1234"));
        assert_eq!(entry.message, "Start proc com.example");
    }

    #[test]
    fn test_parse_unknown_format() {
        let line = "--- some random logcat line ---";
        let entry = LogEntry::parse(line);
        assert_eq!(entry.level, LogLevel::Unknown);
        assert_eq!(entry.message, line);
        assert!(entry.timestamp.is_none());
    }

    #[test]
    fn test_parse_empty_line() {
        let entry = LogEntry::parse("");
        assert_eq!(entry.level, LogLevel::Unknown);
        assert_eq!(entry.message, "");
    }

    // -- LogcatFilter tests -------------------------------------------------

    #[test]
    fn test_filter_matches_level() {
        let mut filter = LogcatFilter::default();
        filter.min_level = LogLevel::Warn;

        let info_entry = LogEntry {
            raw: String::new(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Info,
            tag: None,
            message: "hello".to_string(),
        };
        let warn_entry = LogEntry {
            raw: String::new(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Warn,
            tag: None,
            message: "warning".to_string(),
        };

        assert!(!filter.matches(&info_entry));
        assert!(filter.matches(&warn_entry));
    }

    #[test]
    fn test_filter_matches_search() {
        let mut filter = LogcatFilter::default();
        filter.search_query = "hello".to_string();

        let matching = LogEntry {
            raw: "01-01 00:00:00.000 I Hello World".to_string(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Info,
            tag: None,
            message: "Hello World".to_string(),
        };
        let not_matching = LogEntry {
            raw: "01-01 00:00:00.000 I Goodbye".to_string(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Info,
            tag: None,
            message: "Goodbye".to_string(),
        };

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&not_matching));
    }

    #[test]
    fn test_filter_matches_tag() {
        let mut filter = LogcatFilter::default();
        filter.tag_filter = "Activity".to_string();

        let matching = LogEntry {
            raw: String::new(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Info,
            tag: Some("ActivityManager".to_string()),
            message: "test".to_string(),
        };
        let not_matching = LogEntry {
            raw: String::new(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Info,
            tag: Some("WindowManager".to_string()),
            message: "test".to_string(),
        };

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&not_matching));
    }

    #[test]
    fn test_filter_cycle_level() {
        let mut filter = LogcatFilter::default();
        assert_eq!(filter.min_level, LogLevel::Verbose);
        filter.cycle_level();
        assert_eq!(filter.min_level, LogLevel::Debug);
        filter.cycle_level();
        assert_eq!(filter.min_level, LogLevel::Info);
        filter.cycle_level();
        assert_eq!(filter.min_level, LogLevel::Warn);
        filter.cycle_level();
        assert_eq!(filter.min_level, LogLevel::Error);
        filter.cycle_level();
        assert_eq!(filter.min_level, LogLevel::Fatal);
        filter.cycle_level();
        assert_eq!(filter.min_level, LogLevel::Verbose);
    }

    #[test]
    fn test_filter_insert_and_delete() {
        let mut filter = LogcatFilter::default();
        filter.active_field = FilterField::Search;

        filter.insert_char('h');
        filter.insert_char('i');
        assert_eq!(filter.search_query, "hi");
        assert_eq!(filter.search_cursor, 2);

        filter.delete_char();
        assert_eq!(filter.search_query, "h");
        assert_eq!(filter.search_cursor, 1);

        filter.insert_char('e');
        filter.insert_char('y');
        assert_eq!(filter.search_query, "hey");

        filter.move_cursor_left();
        filter.move_cursor_left();
        filter.delete_char_forward();
        assert_eq!(filter.search_query, "hy");
    }

    #[test]
    fn test_filter_clear() {
        let mut filter = LogcatFilter::default();
        filter.search_query = "test".to_string();
        filter.search_cursor = 4;
        filter.clear_search();
        assert_eq!(filter.search_query, "");
        assert_eq!(filter.search_cursor, 0);
    }

    // -- ChannelWriter tests ------------------------------------------------

    #[test]
    fn test_channel_writer_sends_lines() {
        let (tx, rx) = mpsc::sync_channel(100);
        let mut writer = ChannelWriter::new(tx);

        writer.write_all(b"hello\nworld\n").unwrap();

        assert_eq!(rx.recv().unwrap(), "hello");
        assert_eq!(rx.recv().unwrap(), "world");
    }

    #[test]
    fn test_channel_writer_partial_lines() {
        let (tx, rx) = mpsc::sync_channel(100);
        let mut writer = ChannelWriter::new(tx);

        writer.write_all(b"hel").unwrap();
        writer.write_all(b"lo\n").unwrap();

        assert_eq!(rx.recv().unwrap(), "hello");
    }

    #[test]
    fn test_channel_writer_flush_sends_remaining() {
        let (tx, rx) = mpsc::sync_channel(100);
        let mut writer = ChannelWriter::new(tx);

        writer.write_all(b"partial").unwrap();
        assert!(rx.try_recv().is_err()); // nothing sent yet

        writer.flush().unwrap();
        assert_eq!(rx.recv().unwrap(), "partial");
    }

    #[test]
    fn test_channel_writer_strips_cr_lf() {
        let (tx, rx) = mpsc::sync_channel(100);
        let mut writer = ChannelWriter::new(tx);

        writer.write_all(b"line\r\n").unwrap();
        assert_eq!(rx.recv().unwrap(), "line");
    }

    // -- LogcatState tests --------------------------------------------------

    #[test]
    fn test_logcat_state_new() {
        let state = LogcatState::new();
        assert!(state.entries.is_empty());
        assert!(state.filtered_indices.is_empty());
        assert!(state.auto_scroll);
        assert!(!state.paused);
        assert!(!state.is_streaming);
        assert_eq!(state.total_received, 0);
    }

    #[test]
    fn test_logcat_state_clear() {
        let mut state = LogcatState::new();
        state.entries.push(LogEntry::parse("test line"));
        state.filtered_indices.push(0);
        state.total_received = 5;
        state.scroll_position = 3;
        state.auto_scroll = false;

        state.clear();
        assert!(state.entries.is_empty());
        assert!(state.filtered_indices.is_empty());
        assert_eq!(state.total_received, 0);
        assert_eq!(state.scroll_position, 0);
        assert!(state.auto_scroll);
    }

    #[test]
    fn test_logcat_state_toggle_pause() {
        let mut state = LogcatState::new();
        assert!(!state.paused);
        state.toggle_pause();
        assert!(state.paused);
        state.toggle_pause();
        assert!(!state.paused);
    }

    #[test]
    fn test_logcat_state_scroll() {
        let mut state = LogcatState::new();
        state.viewport_height = 10; // simulate a 10-row viewport
                                    // Add some filtered entries
        for i in 0..20 {
            state.entries.push(LogEntry::parse(&format!("line {}", i)));
            state.filtered_indices.push(i);
        }

        // auto_scroll is true by default.  scroll_up should snapshot the
        // current bottom position (total - viewport = 20 - 10 = 10) and
        // then subtract n.
        state.scroll_up(5);
        assert!(!state.auto_scroll);
        assert_eq!(state.scroll_position, 5); // 10 - 5

        state.scroll_down(3);
        assert_eq!(state.scroll_position, 8);

        state.scroll_to_top();
        assert_eq!(state.scroll_position, 0);

        state.scroll_to_bottom();
        assert!(state.auto_scroll);
        // scroll_to_bottom sets position to total - viewport_height = 10
        assert_eq!(state.scroll_position, 10);
    }

    #[test]
    fn test_logcat_state_rebuild_filtered() {
        let mut state = LogcatState::new();
        state.entries.push(LogEntry {
            raw: "info line".to_string(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Info,
            tag: None,
            message: "info line".to_string(),
        });
        state.entries.push(LogEntry {
            raw: "debug line".to_string(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Debug,
            tag: None,
            message: "debug line".to_string(),
        });
        state.entries.push(LogEntry {
            raw: "warn line".to_string(),
            timestamp: None,
            pid: None,
            tid: None,
            level: LogLevel::Warn,
            tag: None,
            message: "warn line".to_string(),
        });

        // Filter to Warn and above
        state.filter.min_level = LogLevel::Warn;
        state.rebuild_filtered();

        assert_eq!(state.filtered_indices.len(), 1);
        assert_eq!(state.filtered_indices[0], 2);
    }

    #[test]
    fn test_logcat_state_visible_entries() {
        let mut state = LogcatState::new();
        for i in 0..10 {
            state.entries.push(LogEntry::parse(&format!("line {}", i)));
            state.filtered_indices.push(i);
        }
        state.auto_scroll = false;
        state.scroll_position = 3;

        let visible = state.visible_entries(5);
        assert_eq!(visible.len(), 5);
        assert_eq!(visible[0], 3);
        assert_eq!(visible[4], 7);

        // Auto-scroll shows the last `height` entries
        state.auto_scroll = true;
        let visible = state.visible_entries(4);
        assert_eq!(visible.len(), 4);
        assert_eq!(visible[0], 6);
        assert_eq!(visible[3], 9);
        // scroll_position is kept in sync
        assert_eq!(state.scroll_position, 6);
    }

    #[test]
    fn test_logcat_state_entry_count() {
        let mut state = LogcatState::new();
        state.entries.push(LogEntry::parse("line 1"));
        state.entries.push(LogEntry::parse("line 2"));
        state.filtered_indices.push(0);

        assert_eq!(state.entry_count(), 1);
        assert_eq!(state.total_count(), 2);
    }

    #[test]
    fn test_logcat_state_debug_impl() {
        let state = LogcatState::new();
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("LogcatState"));
        assert!(debug_str.contains("entries_len"));
        assert!(debug_str.contains("viewport_height"));
        assert!(debug_str.contains("trimmed_total"));
    }

    #[test]
    fn test_logcat_state_poll_with_no_receiver() {
        let mut state = LogcatState::new();
        // Should be a no-op, not panic
        state.poll_new_entries();
        assert_eq!(state.total_received, 0);
    }

    #[test]
    fn test_logcat_state_poll_drains_channel() {
        let mut state = LogcatState::new();
        let (tx, rx) = mpsc::channel();
        state.receiver = Some(rx);
        state.is_streaming = true;

        tx.send("01-15 12:34:56.789  1234  5678 I TestTag: hello world".to_string())
            .unwrap();
        tx.send("01-15 12:34:57.000  1234  5678 W TestTag: warning".to_string())
            .unwrap();
        drop(tx);

        state.poll_new_entries();

        assert_eq!(state.entries.len(), 2);
        assert_eq!(state.total_received, 2);
        assert_eq!(state.filtered_indices.len(), 2);
    }

    #[test]
    fn test_logcat_state_stop_streaming() {
        let mut state = LogcatState::new();
        let (_tx, rx) = mpsc::channel::<String>();
        state.receiver = Some(rx);
        state.is_streaming = true;

        state.stop_streaming();
        assert!(!state.is_streaming);
        assert!(state.receiver.is_none());
    }
}
