use crate::adb::AdbCommand;

/// Messages represent all possible actions/events in the application
/// This follows the Elm architecture pattern for clear state transitions
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation messages
    MenuUp,
    MenuDown,
    EnterChild,
    ExitChild,

    // Command execution
    ExecuteCommand(AdbCommand),
    CommandStarted,
    CommandCompleted(CommandResult),

    // Scroll messages for result view
    ScrollUp,
    ScrollDown,
    ScrollPageUp,
    ScrollPageDown,
    ScrollToTop,
    ScrollToBottom,

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
        )
    }
}
