//! Logcat viewer state for the TUI.
//!
//! The framework-free domain (log levels, parsing, filtering, stats, saving,
//! and the streaming engine) lives in [`droidtui_core::features::logcat`]; this
//! module keeps only the TUI-specific view state ([`LogcatState`]) and the
//! Ratatui colour mapping.

use ratatui::style::Color;
use std::collections::{BTreeSet, HashSet};
use std::fmt;
use std::io::Write;

// Re-export the shared domain types so existing `crate::logcat::…` paths keep
// working across the TUI.
pub use droidtui_core::features::logcat::{
    try_format_json, wrap_entry_message, ChannelWriter, FilterField, LogEntry, LogLevel, LogStats,
    LogcatFilter, SaveFormat,
};
use droidtui_core::features::logcat::{DrainStatus, LogcatStream};
use droidtui_core::utils::copy_to_clipboard;

// ---------------------------------------------------------------------------
// Ratatui colour mapping (presentation — TUI only)
// ---------------------------------------------------------------------------

/// Colour used for a log level's message text.
pub fn level_color(level: LogLevel) -> Color {
    match level {
        LogLevel::Verbose => Color::DarkGray,
        LogLevel::Debug => Color::Cyan,
        LogLevel::Info => Color::Green,
        LogLevel::Warn => Color::Yellow,
        LogLevel::Error => Color::Red,
        LogLevel::Fatal => Color::LightRed,
        LogLevel::Unknown => Color::Gray,
    }
}

/// Brighter colour used for a log level's badge / label.
pub fn level_label_color(level: LogLevel) -> Color {
    match level {
        LogLevel::Verbose => Color::Gray,
        LogLevel::Debug => Color::LightCyan,
        LogLevel::Info => Color::LightGreen,
        LogLevel::Warn => Color::LightYellow,
        LogLevel::Error => Color::LightRed,
        LogLevel::Fatal => Color::Magenta,
        LogLevel::Unknown => Color::White,
    }
}

/// Hash a tag string to a stable, visually distinct colour.
pub fn tag_color(tag: &str) -> Color {
    let mut hash: u32 = 5381;
    for b in tag.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    const PALETTE: &[Color] = &[
        Color::Rgb(86, 156, 214),  // blue
        Color::Rgb(78, 201, 176),  // teal
        Color::Rgb(220, 220, 170), // light yellow
        Color::Rgb(206, 145, 120), // salmon
        Color::Rgb(181, 206, 168), // light green
        Color::Rgb(200, 130, 200), // purple
        Color::Rgb(100, 200, 220), // cyan
        Color::Rgb(220, 180, 100), // gold
        Color::Rgb(130, 180, 220), // periwinkle
        Color::Rgb(180, 140, 180), // mauve
        Color::Rgb(150, 220, 150), // mint
        Color::Rgb(220, 150, 150), // rose
        Color::Rgb(170, 200, 130), // olive
        Color::Rgb(140, 180, 200), // steel blue
        Color::Rgb(200, 170, 140), // tan
        Color::Rgb(160, 200, 200), // powder blue
    ];
    PALETTE[(hash as usize) % PALETTE.len()]
}

// ---------------------------------------------------------------------------
// LogcatState
// ---------------------------------------------------------------------------

/// Maximum number of log entries kept in memory.
const MAX_ENTRIES: usize = 50_000;

/// Maximum number of lines drained from the stream per tick to keep the UI
/// responsive.
const MAX_DRAIN_PER_TICK: usize = 500;

/// Full state of the logcat viewer.
pub struct LogcatState {
    /// All received log entries (ring buffer capped at `MAX_ENTRIES`).
    pub entries: Vec<LogEntry>,
    /// Indices into `entries` that match the current filter.
    pub filtered_indices: Vec<usize>,
    /// Current filter settings.
    pub filter: LogcatFilter,
    /// Index of the first visible line within the filtered view.
    pub scroll_position: usize,
    /// Whether the view automatically sticks to the bottom.
    pub auto_scroll: bool,
    /// Whether ingestion of new entries is paused.
    pub paused: bool,
    /// Background streaming engine (shared core type).
    stream: LogcatStream,
    /// Number of entries trimmed from the front of `entries` so far.
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
    pub viewport_height: usize,
    /// Whether the line-detail popup is open.
    pub detail_open: bool,
    /// Index into `filtered_indices` for the currently selected line.
    pub selected_line: usize,
    /// Entry indices of fold-start heads for stack traces.
    pub folded_groups: HashSet<usize>,
    /// Aggregate statistics for the log stream.
    pub stats: LogStats,
    /// Horizontal scroll offset (in characters).
    pub h_scroll: usize,
    /// Whether compact display mode is enabled.
    pub compact: bool,
    /// Bookmarked entry indices (stable across filter changes).
    pub bookmarks: BTreeSet<usize>,
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
            stream: LogcatStream::new(),
            trimmed_total: 0,
            is_streaming: false,
            total_received: 0,
            status_message: None,
            word_wrap: false,
            viewport_height: 30,
            detail_open: false,
            selected_line: 0,
            folded_groups: HashSet::new(),
            stats: LogStats::new(),
            h_scroll: 0,
            compact: false,
            bookmarks: BTreeSet::new(),
        }
    }

    /// Start streaming logcat from the device identified by `serial`.
    pub fn start_streaming(&mut self, serial: String) {
        self.stream.start(serial.clone());
        self.is_streaming = true;
        self.status_message = Some(format!("Streaming logcat from {}…", serial));
    }

    /// Stop the current logcat stream.
    pub fn stop_streaming(&mut self) {
        self.stream.stop();
        self.is_streaming = false;
        self.status_message = Some("Logcat streaming stopped.".to_string());
    }

    /// Poll the stream for new log lines. Should be called on each UI tick.
    pub fn poll_new_entries(&mut self) {
        if !self.stream.is_running() {
            return;
        }

        let mut lines: Vec<String> = Vec::new();
        let (_drained, status) = self.stream.drain_into(&mut lines, MAX_DRAIN_PER_TICK);

        let mut new_count: usize = 0;
        for line in lines {
            self.total_received += 1;
            if self.paused {
                // Still consumed to avoid backpressure, but not stored.
                continue;
            }
            let entry = LogEntry::parse(&line);
            let idx = self.entries.len();
            self.stats.record(&entry.level);
            self.entries.push(entry);
            new_count += 1;
            if self.filter.matches(&self.entries[idx]) {
                self.filtered_indices.push(idx);
            }
        }

        if status == DrainStatus::Disconnected {
            self.is_streaming = false;
            self.status_message = Some("Logcat stream ended (thread disconnected).".to_string());
        }

        // Trim oldest entries if we exceed the cap.
        if self.entries.len() > MAX_ENTRIES {
            let excess = self.entries.len() - MAX_ENTRIES;
            self.entries.drain(0..excess);
            self.trimmed_total += excess;

            let mut write = 0;
            for read in 0..self.filtered_indices.len() {
                let idx = self.filtered_indices[read];
                if idx >= excess {
                    self.filtered_indices[write] = idx - excess;
                    write += 1;
                }
            }
            self.filtered_indices.truncate(write);

            let len = self.filtered_indices.len();
            if self.scroll_position >= len && len > 0 {
                self.scroll_position = len.saturating_sub(self.viewport_height);
            } else if len == 0 {
                self.scroll_position = 0;
            }
        }

        if new_count > 0 && self.auto_scroll {
            self.scroll_position = self.filtered_indices.len();
            self.selected_line = self.filtered_indices.len().saturating_sub(1);
        }

        self.stats.update_rate(self.total_received);
    }

    /// Recompute `filtered_indices` from scratch, respecting fold state.
    pub fn rebuild_filtered(&mut self) {
        self.filtered_indices.clear();
        let mut skip_continuations = false;
        for (i, entry) in self.entries.iter().enumerate() {
            if !entry.is_stack_continuation {
                skip_continuations = self.folded_groups.contains(&i);
            }
            if skip_continuations && entry.is_stack_continuation {
                continue;
            }
            if self.filter.matches(entry) {
                self.filtered_indices.push(i);
            }
        }

        let len = self.filtered_indices.len();
        if len == 0 {
            self.scroll_position = 0;
            self.selected_line = 0;
        } else {
            if self.scroll_position >= len {
                self.scroll_position = len.saturating_sub(self.viewport_height);
            }
            if self.selected_line >= len {
                self.selected_line = len.saturating_sub(1);
            }
        }
    }

    /// Return the currently visible window of `filtered_indices`.
    pub fn visible_entries(&mut self, height: usize) -> Vec<usize> {
        self.viewport_height = height;
        if self.filtered_indices.is_empty() || height == 0 {
            return vec![];
        }
        let total = self.filtered_indices.len();
        let start = if self.auto_scroll {
            let s = total.saturating_sub(height);
            self.scroll_position = s;
            s
        } else {
            self.scroll_position.min(total.saturating_sub(height))
        };
        let end = (start + height).min(total);
        self.filtered_indices[start..end].to_vec()
    }

    /// Scroll up by `n` lines. Disables auto-scroll.
    pub fn scroll_up(&mut self, n: usize) {
        if self.auto_scroll {
            let total = self.filtered_indices.len();
            self.scroll_position = total.saturating_sub(self.viewport_height);
        }
        self.auto_scroll = false;
        self.scroll_position = self.scroll_position.saturating_sub(n);
        self.selected_line = self.scroll_position;
    }

    /// Scroll down by `n` lines. Disables auto-scroll.
    pub fn scroll_down(&mut self, n: usize) {
        if self.auto_scroll {
            return;
        }
        let max_scroll = self
            .filtered_indices
            .len()
            .saturating_sub(self.viewport_height);
        self.scroll_position = (self.scroll_position + n).min(max_scroll);
        let max_line = self.filtered_indices.len().saturating_sub(1);
        self.selected_line =
            (self.scroll_position + self.viewport_height.saturating_sub(1)).min(max_line);
    }

    /// Jump to the bottom and re-enable auto-scroll.
    pub fn scroll_to_bottom(&mut self) {
        self.auto_scroll = true;
        let total = self.filtered_indices.len();
        self.scroll_position = total.saturating_sub(self.viewport_height);
        self.selected_line = self.filtered_indices.len().saturating_sub(1);
    }

    /// Jump to the top of the log. Disables auto-scroll.
    pub fn scroll_to_top(&mut self) {
        self.auto_scroll = false;
        self.scroll_position = 0;
        self.selected_line = 0;
    }

    /// Clear all entries and filtered indices.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.filtered_indices.clear();
        self.scroll_position = 0;
        self.total_received = 0;
        self.trimmed_total = 0;
        self.auto_scroll = true;
        self.stats.reset();
        self.bookmarks.clear();
        self.folded_groups.clear();
        self.detail_open = false;
    }

    /// Directory used for "Save Here".
    pub fn default_save_dir() -> std::path::PathBuf {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    }

    /// Build a timestamped default filename for saving logs.
    pub fn default_save_filename(format: SaveFormat) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("logcat_{}.{}", now, format.extension())
    }

    /// Toggle the paused state.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.status_message = Some(if self.paused {
            "Logcat paused.".to_string()
        } else {
            "Logcat resumed.".to_string()
        });
    }

    /// Number of entries matching the current filter.
    pub fn entry_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Total number of entries stored (before filtering).
    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    /// Save all entries to a plain-text file.
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<usize> {
        self.write_entries(path, self.entries.iter(), false)
    }

    /// Save only filtered entries to a plain-text file.
    pub fn save_filtered_to_file(&self, path: &std::path::Path) -> std::io::Result<usize> {
        let it = self.filtered_indices.iter().filter_map(|&i| self.entries.get(i));
        self.write_entries(path, it, false)
    }

    /// Save all entries to a JSONL file.
    pub fn save_to_json_file(&self, path: &std::path::Path) -> std::io::Result<usize> {
        self.write_entries(path, self.entries.iter(), true)
    }

    /// Save filtered entries to a JSONL file.
    pub fn save_filtered_to_json_file(&self, path: &std::path::Path) -> std::io::Result<usize> {
        let it = self.filtered_indices.iter().filter_map(|&i| self.entries.get(i));
        self.write_entries(path, it, true)
    }

    /// Shared writer for the four save variants.
    fn write_entries<'a>(
        &self,
        path: &std::path::Path,
        entries: impl Iterator<Item = &'a LogEntry>,
        json: bool,
    ) -> std::io::Result<usize> {
        use std::io::BufWriter;
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        let mut count = 0;
        for entry in entries {
            if json {
                serde_json::to_writer(&mut writer, entry).map_err(std::io::Error::other)?;
                writeln!(writer)?;
            } else {
                writeln!(writer, "{}", entry.raw)?;
            }
            count += 1;
        }
        writer.flush()?;
        Ok(count)
    }

    /// Toggle word wrapping for long lines.
    pub fn toggle_word_wrap(&mut self) {
        self.word_wrap = !self.word_wrap;
    }

    /// Toggle the detail popup open/closed.
    pub fn toggle_detail(&mut self) {
        self.detail_open = !self.detail_open;
    }

    /// Reference to the currently selected log entry (if any).
    pub fn selected_entry(&self) -> Option<&LogEntry> {
        self.filtered_indices
            .get(self.selected_line)
            .and_then(|&idx| self.entries.get(idx))
    }

    /// Move the selection up by one line.
    pub fn select_up(&mut self) {
        if self.selected_line > 0 {
            self.selected_line -= 1;
        }
        if self.selected_line < self.scroll_position {
            self.scroll_position = self.selected_line;
        }
        self.auto_scroll = false;
    }

    /// Move the selection down by one line.
    pub fn select_down(&mut self) {
        let max = self.filtered_indices.len().saturating_sub(1);
        if self.selected_line < max {
            self.selected_line += 1;
        }
        if self.selected_line >= self.scroll_position + self.viewport_height {
            self.scroll_position = self.selected_line.saturating_sub(self.viewport_height - 1);
        }
    }

    /// Toggle fold state at the currently selected line.
    pub fn toggle_fold_at_selected(&mut self) {
        if let Some(&entry_idx) = self.filtered_indices.get(self.selected_line) {
            let head = if self.entries[entry_idx].is_stack_continuation {
                (0..entry_idx)
                    .rev()
                    .find(|&i| !self.entries[i].is_stack_continuation)
                    .unwrap_or(entry_idx)
            } else {
                entry_idx
            };
            if !self.folded_groups.remove(&head) {
                self.folded_groups.insert(head);
            }
            self.rebuild_filtered();
        }
    }

    /// Scroll left by `n` characters.
    pub fn h_scroll_left(&mut self, n: usize) {
        self.h_scroll = self.h_scroll.saturating_sub(n);
    }

    /// Scroll right by `n` characters.
    pub fn h_scroll_right(&mut self, n: usize) {
        self.h_scroll += n;
    }

    /// Reset horizontal scroll to the beginning.
    pub fn h_scroll_reset(&mut self) {
        self.h_scroll = 0;
    }

    /// Toggle compact display mode.
    pub fn toggle_compact(&mut self) {
        self.compact = !self.compact;
    }

    /// Copy the currently selected log line to the system clipboard.
    pub fn copy_selected_to_clipboard(&self) -> Result<(), String> {
        match self.selected_entry() {
            Some(entry) => copy_to_clipboard(&entry.raw),
            None => Err("No line selected".into()),
        }
    }

    /// Toggle a bookmark on the currently selected line.
    pub fn toggle_bookmark(&mut self) {
        if let Some(&entry_idx) = self.filtered_indices.get(self.selected_line) {
            if !self.bookmarks.remove(&entry_idx) {
                self.bookmarks.insert(entry_idx);
            }
        }
    }

    /// Whether the given entry index is bookmarked.
    pub fn is_bookmarked(&self, entry_idx: usize) -> bool {
        self.bookmarks.contains(&entry_idx)
    }

    /// Jump to the next bookmark (wrapping around).
    pub fn next_bookmark(&mut self) {
        self.jump_bookmark(true);
    }

    /// Jump to the previous bookmark (wrapping around).
    pub fn prev_bookmark(&mut self) {
        self.jump_bookmark(false);
    }

    /// Shared bookmark navigation for [`next_bookmark`](Self::next_bookmark)
    /// and [`prev_bookmark`](Self::prev_bookmark).
    fn jump_bookmark(&mut self, forward: bool) {
        if self.bookmarks.is_empty() {
            return;
        }
        let current = self
            .filtered_indices
            .get(self.selected_line)
            .copied()
            .unwrap_or(0);

        let target = if forward {
            self.bookmarks
                .range((current + 1)..)
                .next()
                .copied()
                .or_else(|| self.bookmarks.iter().next().copied())
        } else {
            self.bookmarks
                .range(..current)
                .next_back()
                .copied()
                .or_else(|| self.bookmarks.iter().next_back().copied())
        };

        if let Some(entry_idx) = target {
            if let Some(pos) = self.filtered_indices.iter().position(|&i| i == entry_idx) {
                self.selected_line = pos;
                self.auto_scroll = false;
                if self.selected_line < self.scroll_position
                    || self.selected_line >= self.scroll_position + self.viewport_height
                {
                    self.scroll_position =
                        self.selected_line.saturating_sub(self.viewport_height / 2);
                }
            }
        }
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
            .field("is_streaming", &self.is_streaming)
            .field("total_received", &self.total_received)
            .field("trimmed_total", &self.trimmed_total)
            .field("status_message", &self.status_message)
            .field("word_wrap", &self.word_wrap)
            .field("viewport_height", &self.viewport_height)
            .field("detail_open", &self.detail_open)
            .field("selected_line", &self.selected_line)
            .field("folded_groups_len", &self.folded_groups.len())
            .field("stats", &self.stats)
            .field("h_scroll", &self.h_scroll)
            .field("compact", &self.compact)
            .field("bookmarks_len", &self.bookmarks.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_is_empty() {
        let s = LogcatState::new();
        assert_eq!(s.entry_count(), 0);
        assert_eq!(s.total_count(), 0);
        assert!(s.auto_scroll);
    }

    #[test]
    fn filter_rebuild_selects_matching() {
        let mut s = LogcatState::new();
        s.entries.push(LogEntry::parse(
            "01-15 12:34:56.789  1  2 I Tag: hello",
        ));
        s.entries.push(LogEntry::parse(
            "01-15 12:34:56.789  1  2 E Tag: boom",
        ));
        s.filter.min_level = LogLevel::Error;
        s.rebuild_filtered();
        assert_eq!(s.filtered_indices, vec![1]);
    }

    #[test]
    fn bookmark_toggle() {
        let mut s = LogcatState::new();
        s.entries.push(LogEntry::parse("x"));
        s.filtered_indices.push(0);
        s.selected_line = 0;
        s.toggle_bookmark();
        assert!(s.is_bookmarked(0));
        s.toggle_bookmark();
        assert!(!s.is_bookmarked(0));
    }

    #[test]
    fn level_colors_distinct() {
        assert_ne!(level_color(LogLevel::Error), level_color(LogLevel::Info));
    }

    #[test]
    fn tag_color_is_stable() {
        assert_eq!(tag_color("ActivityManager"), tag_color("ActivityManager"));
    }
}
