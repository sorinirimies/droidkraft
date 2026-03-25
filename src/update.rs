use crate::adb::AdbCommand;
use crate::fastboot::FastbootManager;
use crate::menu::MenuCommand;
use crate::message::{CommandResult, Message};
use crate::model::{AppState, Model};

/// Update function - the heart of Elm architecture
/// Takes the current model and a message, returns updated model
/// This is a pure function that handles all state transitions
pub async fn update(model: &mut Model, message: Message) {
    match message {
        // Navigation messages
        Message::MenuUp => {
            model.menu.previous();
        }

        Message::MenuDown => {
            model.menu.next();
        }

        Message::SectionNext => {
            model.menu.next_section();
        }

        Message::SectionPrev => {
            model.menu.previous_section();
        }

        Message::RefreshDeviceInfo => {
            let status = model.adb_manager.fetch_device_status();
            model.device_status = status;
        }

        Message::NextDevice => {
            model.device_status.cycle_next();
            let status = model.adb_manager.fetch_device_status();
            model.device_status = status;
        }

        Message::EnterChild => {
            // Flat menu has no child mode — trigger visual effects only
            model.effects.start_fade_in();
            model.effects.start_slide_in();
        }

        Message::ExitChild => {
            // No-op: flat menu has no child mode
        }

        // Command execution
        Message::ExecuteCommand(command) => {
            model.last_command_label = Some(model.menu.get_selected_label().to_string());
            model.state = AppState::Loading;
            model.clear_results();
            model.loading_counter = 0;
            model.effects.reset_slide();

            let result = execute_adb_command(model, command).await;

            // Handle result directly to avoid recursion
            match result {
                CommandResult::Success(output) => {
                    model.set_result(output);
                }
                CommandResult::Error(error) => {
                    model.set_error(error);
                }
            }
            model.state = AppState::ShowResult;
            model.effects.start_slide_in();
        }

        Message::CommandStarted => {
            model.state = AppState::Loading;
            model.loading_counter = 0;
        }

        Message::CommandCompleted(result) => {
            match result {
                CommandResult::Success(output) => {
                    model.set_result(output);
                }
                CommandResult::Error(error) => {
                    model.set_error(error);
                }
            }
            model.state = AppState::ShowResult;
            model.effects.start_slide_in();
        }

        // Scroll messages
        Message::ScrollUp => {
            if model.scroll_position > 0 {
                model.scroll_position -= 1;
            }
        }

        Message::ScrollDown => {
            if model.scroll_position + 1 < model.total_result_lines() {
                model.scroll_position += 1;
            }
        }

        Message::ScrollPageUp => {
            model.scroll_position = model.scroll_position.saturating_sub(10);
        }

        Message::ScrollPageDown => {
            let max_scroll = model.total_result_lines().saturating_sub(1);
            model.scroll_position = (model.scroll_position + 10).min(max_scroll);
        }

        Message::ScrollToTop => {
            model.scroll_position = 0;
        }

        Message::ScrollToBottom => {
            model.scroll_position = model.total_result_lines().saturating_sub(1);
        }

        // Screen streaming messages
        // Application lifecycle
        Message::Tick => {
            tick(model).await;
        }

        Message::Quit => {
            model.running = false;
        }

        Message::ReturnToMenu => {
            model.state = AppState::Menu;
            model.clear_results();
            // Refresh device stats after every command so the dashboard stays current.
            model.needs_device_refresh = true;
        }

        Message::SkipStartup => {
            model.state = AppState::Menu;
            model.effects.start_slide_in();
            model.needs_device_refresh = true;
        }
    }
}

/// Handle tick updates (animations, timers, etc.)
async fn tick(model: &mut Model) {
    let now = std::time::Instant::now();
    let elapsed = now.duration_since(model.last_tick);
    model.last_tick = now;

    // Update effects
    model.effects.tick(elapsed);

    // Update menu animations
    model.menu.tick();

    // Update loading animation
    if model.state == AppState::Loading {
        model.loading_counter += 1;
    }

    // Update reveal animation for results
    if model.state == AppState::ShowResult {
        model.reveal_counter += 1;
    }

    // Check if startup is complete
    if model.state == AppState::Startup && model.effects.is_startup_complete() {
        model.state = AppState::Menu;
        model.needs_device_refresh = true;
    }

    // Auto-fetch device status once after transitioning to the Menu
    if model.needs_device_refresh && model.state == AppState::Menu {
        model.needs_device_refresh = false;
        let status = model.adb_manager.fetch_device_status();
        model.device_status = status;
    }
}

/// Execute a command dispatched from the menu (ADB or fastboot).
async fn execute_adb_command(model: &mut Model, command: MenuCommand) -> CommandResult {
    // Dispatch fastboot commands to FastbootManager without ADB
    let adb_command: AdbCommand = match command {
        MenuCommand::Fastboot(cmd) => {
            return match FastbootManager::new().execute(cmd) {
                Ok(output) => CommandResult::Success(output),
                Err(e) => CommandResult::Error(format!("{}", e)),
            };
        }
        MenuCommand::Adb(cmd) => cmd,
    };

    // Execute the ADB command using the ADB manager
    match model.adb_manager.execute(adb_command) {
        Ok(output) => CommandResult::Success(output),
        Err(e) => {
            let error_msg = format!("{}", e);

            // Add helpful context for common errors
            let enhanced_error = if error_msg.contains("Connection")
                || error_msg.contains("connection")
            {
                format!(
                    "{}\n\nTroubleshooting:\n• Make sure ADB server is running (adb start-server)\n• Check that device is connected (adb devices)\n• Verify USB debugging is enabled on device",
                    error_msg
                )
            } else if error_msg.contains("No device selected") {
                format!(
                    "{}\n\nPlease:\n• Connect an Android device via USB\n• Enable USB debugging on the device\n• Run 'List Devices' to detect your device",
                    error_msg
                )
            } else {
                error_msg
            };

            CommandResult::Error(enhanced_error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_menu_navigation() {
        let mut model = Model::new();
        model.state = AppState::Menu;

        let initial = model.menu.selected;
        update(&mut model, Message::MenuDown).await;
        assert!(model.menu.selected > initial);
        assert!(model.menu.entries[model.menu.selected].is_selectable());

        update(&mut model, Message::MenuUp).await;
        assert_eq!(model.menu.selected, initial);
    }

    #[tokio::test]
    async fn test_quit() {
        let mut model = Model::new();
        assert!(model.running);

        update(&mut model, Message::Quit).await;
        assert!(!model.running);
    }

    #[tokio::test]
    async fn test_scroll_boundaries() {
        let mut model = Model::new();
        model.wrapped_lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        model.scroll_position = 0;

        // Can't scroll up from position 0
        update(&mut model, Message::ScrollUp).await;
        assert_eq!(model.scroll_position, 0);

        // Can scroll down
        update(&mut model, Message::ScrollDown).await;
        assert_eq!(model.scroll_position, 1);

        // Can't scroll past end
        update(&mut model, Message::ScrollDown).await;
        assert_eq!(model.scroll_position, 1);
    }

    #[tokio::test]
    async fn test_enter_exit_child_mode() {
        let mut model = Model::new();
        model.state = AppState::Menu;
        // EnterChild/ExitChild are no-ops in the flat menu — verify no panic
        // and that selection still points at a valid item afterwards.
        update(&mut model, Message::EnterChild).await;
        update(&mut model, Message::ExitChild).await;
        assert!(model.menu.entries[model.menu.selected].is_selectable());
    }

    #[tokio::test]
    async fn test_clear_results() {
        let mut model = Model::new();
        model.set_result("Test output".to_string());
        assert!(model.command_result.is_some());
        assert!(!model.result_lines.is_empty());

        update(&mut model, Message::ReturnToMenu).await;
        assert!(model.command_result.is_none());
        assert!(model.result_lines.is_empty());
        assert_eq!(model.state, AppState::Menu);
    }
}
