use crate::adb::{AdbManager, DeviceStatus};
use crate::effects::EffectsManager;
use crate::fastboot::FastbootManager;
use crate::menu::{Menu, MenuCommand};

use std::time::Instant;

/// Application state following Elm architecture
/// All mutable state is contained within this model
#[derive(Debug)]
pub struct Model {
    /// Current application state
    pub state: AppState,

    /// Menu state and navigation
    pub menu: Menu,

    /// Visual effects manager
    pub effects: EffectsManager,

    /// Last tick time for animations
    pub last_tick: Instant,

    /// Command execution result (success)
    pub command_result: Option<String>,

    /// Command execution error
    pub command_error: Option<String>,

    /// Loading animation counter
    pub loading_counter: u64,

    /// Scroll position for result view
    pub scroll_position: usize,

    /// Original result lines (before wrapping)
    pub result_lines: Vec<String>,

    /// Wrapped lines for display (handles wide content)
    pub wrapped_lines: Vec<String>,

    /// Reveal animation counter for result display
    pub reveal_counter: u64,

    /// Whether the application should continue running
    pub running: bool,

    /// ADB client manager
    pub adb_manager: AdbManager,

    /// Fastboot manager (shells out to the `fastboot` binary)
    pub fastboot_manager: FastbootManager,

    /// Label of the last executed command (shown in the result title)
    pub last_command_label: Option<String>,

    /// Live device status shown in the header status bar.
    pub device_status: DeviceStatus,

    /// When true the next tick will fetch fresh device info.
    pub needs_device_refresh: bool,
}

/// Application states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Initial startup animation
    Startup,

    /// Main menu navigation
    Menu,

    /// Loading animation during command execution
    Loading,

    /// Showing command results
    ShowResult,
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    /// Create a new model with initial state
    pub fn new() -> Self {
        Self {
            state: AppState::Startup,
            menu: Menu::new(),
            effects: EffectsManager::new(),
            last_tick: Instant::now(),
            command_result: None,
            command_error: None,
            loading_counter: 0,
            scroll_position: 0,
            result_lines: Vec::new(),
            wrapped_lines: Vec::new(),
            reveal_counter: 0,
            running: true,
            adb_manager: AdbManager::new(),
            fastboot_manager: FastbootManager::new(),
            last_command_label: None,
            device_status: DeviceStatus::default(),
            needs_device_refresh: true,
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        !self.running
    }

    /// Check if we're in the result view
    pub fn is_showing_result(&self) -> bool {
        self.state == AppState::ShowResult
    }

    /// Check if we're in the menu
    pub fn is_in_menu(&self) -> bool {
        self.state == AppState::Menu
    }

    /// Check if startup animation is complete
    pub fn is_startup_complete(&self) -> bool {
        self.state != AppState::Startup || self.effects.is_startup_complete()
    }

    /// Get the currently selected command
    pub fn get_selected_command(&self) -> MenuCommand {
        self.menu.get_selected_command()
    }

    /// Clear result state
    pub fn clear_results(&mut self) {
        self.command_result = None;
        self.command_error = None;
        self.scroll_position = 0;
        self.result_lines.clear();
        self.wrapped_lines.clear();
    }

    /// Set command result (success)
    pub fn set_result(&mut self, output: String) {
        self.command_result = Some(output.clone());
        self.result_lines = output.lines().map(|s| s.to_string()).collect();
        self.scroll_position = 0;
        self.reveal_counter = 0;
    }

    /// Set command error
    pub fn set_error(&mut self, error: String) {
        self.command_error = Some(error.clone());
        self.result_lines = error.lines().map(|s| s.to_string()).collect();
        self.scroll_position = 0;
        self.reveal_counter = 0;
    }

    /// Get total number of lines in current result
    pub fn total_result_lines(&self) -> usize {
        self.wrapped_lines.len()
    }

    /// Check if scrolling is available
    pub fn can_scroll(&self) -> bool {
        self.wrapped_lines.len() > 1
    }

    /// Update wrapped lines for current terminal width
    pub fn update_wrapped_lines(&mut self, max_width: usize) {
        if self.result_lines.is_empty() {
            self.wrapped_lines = vec!["No output".to_string()];
            return;
        }

        self.wrapped_lines = self
            .result_lines
            .iter()
            .flat_map(|line| wrap_line(line, max_width))
            .collect();
    }
}

/// Helper function to wrap a single line at word boundaries.
/// Tabs are expanded to 4 spaces before processing so ratatui renders them correctly.
fn wrap_line(line: &str, max_width: usize) -> Vec<String> {
    // Expand tabs — ratatui treats \t as zero-width, causing text to overlap.
    let line = line.replace('\t', "    ");
    let line = line.as_str();

    // Guard: if max_width is too small to be useful, return the line as-is.
    if max_width < 4 {
        return vec![line.to_string()];
    }

    if line.len() <= max_width {
        return vec![line.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut current_length = 0;

    for word in line.split_whitespace() {
        if current_length + word.len() < max_width {
            if !current_chunk.is_empty() {
                current_chunk.push(' ');
                current_length += 1;
            }
            current_chunk.push_str(word);
            current_length += word.len();
        } else {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk);
            }
            if word.len() > max_width {
                // Word is too long, break it
                for chunk in word.chars().collect::<Vec<char>>().chunks(max_width) {
                    chunks.push(chunk.iter().collect::<String>());
                }
                current_chunk = String::new();
                current_length = 0;
            } else {
                current_chunk = word.to_string();
                current_length = word.len();
            }
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    if chunks.is_empty() {
        vec![line.to_string()]
    } else {
        chunks
    }
}
