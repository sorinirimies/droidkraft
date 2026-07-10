//! Logcat filtering — level, text search, tag, package, and exclude filters
//! with optional regex support.

use regex::Regex;
use std::fmt;

use super::types::{LogEntry, LogLevel};
use crate::utils::char_to_byte_index;

/// Which input field is currently active for keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterField {
    Search,
    Tag,
    Package,
    Exclude,
    None,
}

/// Filter state for the logcat viewer.
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
    /// Negative match / exclude filter query.
    pub exclude_query: String,
    /// Cursor position within the exclude filter field.
    pub exclude_cursor: usize,
    /// Whether regex mode is enabled for search/exclude.
    pub use_regex: bool,
    compiled_regex: Option<Regex>,
    compiled_exclude: Option<Regex>,
}

impl fmt::Debug for LogcatFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogcatFilter")
            .field("search_query", &self.search_query)
            .field("min_level", &self.min_level)
            .field("tag_filter", &self.tag_filter)
            .field("package_filter", &self.package_filter)
            .field("active_field", &self.active_field)
            .field("exclude_query", &self.exclude_query)
            .field("use_regex", &self.use_regex)
            .finish()
    }
}

impl Clone for LogcatFilter {
    fn clone(&self) -> Self {
        let mut cloned = Self {
            search_query: self.search_query.clone(),
            search_cursor: self.search_cursor,
            min_level: self.min_level,
            tag_filter: self.tag_filter.clone(),
            tag_cursor: self.tag_cursor,
            package_filter: self.package_filter.clone(),
            package_cursor: self.package_cursor,
            active_field: self.active_field,
            exclude_query: self.exclude_query.clone(),
            exclude_cursor: self.exclude_cursor,
            use_regex: self.use_regex,
            compiled_regex: None,
            compiled_exclude: None,
        };
        cloned.recompile_regex();
        cloned
    }
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
            exclude_query: String::new(),
            exclude_cursor: 0,
            use_regex: false,
            compiled_regex: None,
            compiled_exclude: None,
        }
    }
}

impl LogcatFilter {
    /// Check whether a log entry passes all active filters.
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if entry.level.order() < self.min_level.order() {
            return false;
        }

        if !self.search_query.is_empty() {
            if self.use_regex {
                if let Some(ref re) = self.compiled_regex {
                    if !re.is_match(&entry.raw) {
                        return false;
                    }
                }
            } else if !entry
                .raw
                .to_lowercase()
                .contains(&self.search_query.to_lowercase())
            {
                return false;
            }
        }

        if !self.exclude_query.is_empty() {
            if self.use_regex {
                if let Some(ref re) = self.compiled_exclude {
                    if re.is_match(&entry.raw) {
                        return false;
                    }
                }
            } else if entry
                .raw
                .to_lowercase()
                .contains(&self.exclude_query.to_lowercase())
            {
                return false;
            }
        }

        if !self.tag_filter.is_empty() {
            let tag_lower = self.tag_filter.to_lowercase();
            match &entry.tag {
                Some(tag) if tag.to_lowercase().contains(&tag_lower) => {}
                _ => return false,
            }
        }

        if !self.package_filter.is_empty() {
            let pkg_lower = self.package_filter.to_lowercase();
            match &entry.pid {
                Some(pid) if pid.to_lowercase().contains(&pkg_lower) => {}
                _ => return false,
            }
        }

        true
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_cursor = 0;
    }

    pub fn clear_tag(&mut self) {
        self.tag_filter.clear();
        self.tag_cursor = 0;
    }

    pub fn clear_package(&mut self) {
        self.package_filter.clear();
        self.package_cursor = 0;
    }

    pub fn clear_exclude(&mut self) {
        self.exclude_query.clear();
        self.exclude_cursor = 0;
    }

    /// Recompile cached regexes from the current query strings.
    pub fn recompile_regex(&mut self) {
        self.compiled_regex = if self.use_regex && !self.search_query.is_empty() {
            Regex::new(&self.search_query).ok()
        } else {
            None
        };
        self.compiled_exclude = if self.use_regex && !self.exclude_query.is_empty() {
            Regex::new(&self.exclude_query).ok()
        } else {
            None
        };
    }

    /// Toggle regex mode on/off and recompile.
    pub fn toggle_regex(&mut self) {
        self.use_regex = !self.use_regex;
        self.recompile_regex();
    }

    /// Insert a character at the cursor of the active field.
    pub fn insert_char(&mut self, c: char) {
        {
            let (field, cursor) = self.active_field_mut();
            let byte_idx = char_to_byte_index(field, *cursor);
            field.insert(byte_idx, c);
            *cursor += 1;
        }
        if self.use_regex {
            self.recompile_regex();
        }
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_char(&mut self) {
        {
            let (field, cursor) = self.active_field_mut();
            if *cursor > 0 {
                *cursor -= 1;
                let byte_idx = char_to_byte_index(field, *cursor);
                field.remove(byte_idx);
            }
        }
        if self.use_regex {
            self.recompile_regex();
        }
    }

    /// Delete the character at the cursor (forward delete).
    pub fn delete_char_forward(&mut self) {
        {
            let (field, cursor) = self.active_field_mut();
            let char_count = field.chars().count();
            if *cursor < char_count {
                let byte_idx = char_to_byte_index(field, *cursor);
                field.remove(byte_idx);
            }
        }
        if self.use_regex {
            self.recompile_regex();
        }
    }

    pub fn move_cursor_left(&mut self) {
        let (_field, cursor) = self.active_field_mut();
        *cursor = cursor.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        let (field, cursor) = self.active_field_mut();
        let char_count = field.chars().count();
        if *cursor < char_count {
            *cursor += 1;
        }
    }

    /// Cycle the minimum log level V → D → I → W → E → F → V.
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

    fn active_field_mut(&mut self) -> (&mut String, &mut usize) {
        match self.active_field {
            FilterField::Search => (&mut self.search_query, &mut self.search_cursor),
            FilterField::Tag => (&mut self.tag_filter, &mut self.tag_cursor),
            FilterField::Package => (&mut self.package_filter, &mut self.package_cursor),
            FilterField::Exclude => (&mut self.exclude_query, &mut self.exclude_cursor),
            FilterField::None => (&mut self.search_query, &mut self.search_cursor),
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    fn entry(raw: &str) -> LogEntry {
        LogEntry::parse(raw)
    }

    #[test]
    fn level_filter_excludes_lower() {
        let mut f = LogcatFilter::default();
        f.min_level = LogLevel::Warn;
        let info = entry("01-15 12:34:56.789  1  2 I Tag: hi");
        let err = entry("01-15 12:34:56.789  1  2 E Tag: boom");
        assert!(!f.matches(&info));
        assert!(f.matches(&err));
    }

    #[test]
    fn search_substring_case_insensitive() {
        let mut f = LogcatFilter::default();
        f.search_query = "boom".into();
        assert!(f.matches(&entry("01-15 12:34:56.789  1  2 E Tag: BOOM happened")));
        assert!(!f.matches(&entry("01-15 12:34:56.789  1  2 E Tag: all good")));
    }

    #[test]
    fn exclude_filters_out() {
        let mut f = LogcatFilter::default();
        f.exclude_query = "spam".into();
        assert!(!f.matches(&entry("01-15 12:34:56.789  1  2 I Tag: spam spam")));
    }

    #[test]
    fn regex_search() {
        let mut f = LogcatFilter::default();
        f.use_regex = true;
        f.search_query = r"proc \d+".into();
        f.recompile_regex();
        assert!(f.matches(&entry("01-15 12:34:56.789  1  2 I Tag: proc 42")));
        assert!(!f.matches(&entry("01-15 12:34:56.789  1  2 I Tag: proc abc")));
    }

    #[test]
    fn insert_and_delete_char() {
        let mut f = LogcatFilter::default();
        f.active_field = FilterField::Search;
        f.insert_char('a');
        f.insert_char('b');
        assert_eq!(f.search_query, "ab");
        f.delete_char();
        assert_eq!(f.search_query, "a");
    }

    #[test]
    fn cycle_level_wraps() {
        let mut f = LogcatFilter::default();
        f.min_level = LogLevel::Fatal;
        f.cycle_level();
        assert_eq!(f.min_level, LogLevel::Verbose);
    }
}
