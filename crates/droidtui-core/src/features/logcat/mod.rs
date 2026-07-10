//! Logcat feature — framework-free log parsing, filtering, statistics, saving,
//! and a background streaming engine.
//!
//! Presentation concerns (colors) live in the frontends; this module deals only
//! with domain data so it can be shared by both the TUI and GUI.

pub mod filter;
pub mod stream;
pub mod types;

pub use filter::{FilterField, LogcatFilter};
pub use stream::{ChannelWriter, DrainStatus, LogcatStream};
pub use types::{
    try_format_json, wrap_entry_message, LogEntry, LogLevel, LogStats, SaveFormat,
};
