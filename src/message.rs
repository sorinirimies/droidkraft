use crate::menu::MenuCommand;
use ratatui::crossterm::event::KeyEvent;

/// Messages represent all possible actions/events in the application
/// This follows the Elm architecture pattern for clear state transitions
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation messages
    MenuUp,
    MenuDown,
    EnterChild,
    ExitChild,
    /// Jump to the first item of the next section (Tab).
    SectionNext,
    /// Jump to the first item of the previous section (Shift+Tab).
    SectionPrev,
    /// Refresh the live device status bar (r key).
    RefreshDeviceInfo,
    /// Cycle to the next connected device (d key).
    NextDevice,

    // Command execution
    ExecuteCommand(MenuCommand),
    CommandStarted,
    CommandCompleted(CommandResult),

    // Scroll messages for result view
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    ScrollToTop,
    ScrollToBottom,

    // Logcat messages
    OpenLogcat,
    LogcatScrollUp,
    LogcatScrollDown,
    LogcatScrollPageUp,
    LogcatScrollPageDown,
    LogcatScrollToTop,
    LogcatScrollToBottom,
    LogcatTogglePause,
    LogcatClear,
    LogcatCycleLevel,
    LogcatToggleSearch,
    LogcatToggleTagFilter,
    LogcatTogglePackageFilter,
    LogcatSearchInput(char),
    LogcatSearchBackspace,
    LogcatSearchDelete,
    LogcatCursorLeft,
    LogcatCursorRight,
    LogcatExitFilter,
    LogcatSave,
    LogcatSaveFilteredOnly,
    LogcatToggleWordWrap,
    LogcatFileSaved(String),
    LogcatFileExplorerKey(KeyEvent),
    LogcatCancelSave,
    LogcatSaveAs,
    CloseLogcat,

    // Application lifecycle
    Tick,
    Quit,
    ReturnToMenu,

    // UI state
    SkipStartup,
}

/// Result of command execution
#[derive(Debug, Clone)]
pub enum CommandResult {
    Success(String),
    Error(String),
}

impl Message {
    /// Check if this message should trigger a state transition
    pub fn is_state_changing(&self) -> bool {
        matches!(
            self,
            Message::ExecuteCommand(_)
                | Message::CommandStarted
                | Message::CommandCompleted(_)
                | Message::Quit
                | Message::ReturnToMenu
                | Message::EnterChild
                | Message::ExitChild
                | Message::SkipStartup
                | Message::OpenLogcat
                | Message::CloseLogcat
                | Message::LogcatSave
                | Message::LogcatCancelSave
        )
    }
}
