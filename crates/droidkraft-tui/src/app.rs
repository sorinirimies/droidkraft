use crate::{
    adb::AdbManager,
    event::{AppEvent, Event, EventHandler},
    message::Message,
    model::{AppState, Model},
    update,
};
use ratatui::{crossterm::event::KeyCode, DefaultTerminal};
use std::time::Duration;
use tokio::sync::mpsc;

/// Main application following Elm architecture
/// This is a thin wrapper that connects the event loop to the Model-Update-View cycle
pub struct App {
    /// Application model (all state)
    pub model: Model,

    /// Event handler
    pub events: EventHandler,
}

impl App {
    /// Create a new application
    pub fn new() -> Self {
        let events = EventHandler::new();

        // Spawn the background device watcher that detects connect/disconnect events.
        let sender = events.sender();
        tokio::spawn(device_watcher(sender));

        Self {
            model: Model::new(),
            events,
        }
    }

    /// Main application loop following Elm architecture:
    /// 1. Wait for events
    /// 2. Convert events to messages
    /// 3. Update model with message
    /// 4. Render view from model
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while !self.model.should_quit() {
            // View: Render current model state
            terminal.draw(|frame| {
                crate::view::render(&mut self.model, frame.area(), frame.buffer_mut())
            })?;

            // Event: Wait for next event
            let event = self.events.next().await?;

            // Update: Convert event to message and update model
            let message = self.event_to_message(event)?;
            if let Some(msg) = message {
                update::update(&mut self.model, msg).await;
            }
        }

        Ok(())
    }

    /// Convert events to messages (Elm architecture pattern)
    fn event_to_message(&self, event: Event) -> color_eyre::Result<Option<Message>> {
        match event {
            Event::Tick => Ok(Some(Message::Tick)),

            Event::Crossterm(event) => {
                if let crossterm::event::Event::Key(key_event) = event {
                    Ok(self.key_to_message(key_event.code))
                } else {
                    Ok(None)
                }
            }

            Event::App(app_event) => Ok(Some(match app_event {
                AppEvent::MenuUp => Message::MenuUp,
                AppEvent::MenuDown => Message::MenuDown,
                AppEvent::Execute => {
                    let command = self.model.get_selected_command();
                    Message::ExecuteCommand(command)
                }
                AppEvent::EnterChild => Message::EnterChild,
                AppEvent::ExitChild => Message::ExitChild,
                AppEvent::Quit => Message::Quit,
                AppEvent::DeviceStatusUpdate(status) => Message::DeviceStatusUpdate(status),
            })),
        }
    }

    /// Map keyboard input to messages based on current state
    fn key_to_message(&self, key: KeyCode) -> Option<Message> {
        // ── Global: theme selector ──────────────────────────────────────
        if self.model.theme_selector.open {
            return match key {
                KeyCode::Esc | KeyCode::Char('q') => Some(Message::ToggleThemeSelector),
                KeyCode::Up | KeyCode::Char('k') => Some(Message::ThemePrev),
                KeyCode::Down | KeyCode::Char('j') => Some(Message::ThemeNext),
                KeyCode::Char('t') => Some(Message::ThemeNext),
                KeyCode::Enter => Some(Message::ThemeApply),
                _ => None,
            };
        }
        // Shift+T opens theme selector from any state
        if key == KeyCode::Char('T') {
            return Some(Message::ToggleThemeSelector);
        }

        match self.model.state {
            AppState::Startup => Some(Message::SkipStartup),

            AppState::Menu => match key {
                KeyCode::Esc | KeyCode::Char('q') => Some(Message::Quit),
                KeyCode::Up | KeyCode::Char('k') => Some(Message::MenuUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Message::MenuDown),
                KeyCode::Tab => Some(Message::SectionNext),
                KeyCode::BackTab => Some(Message::SectionPrev),
                KeyCode::Char('t') => Some(Message::ThemeNext),
                KeyCode::Char('r') => Some(Message::RefreshDeviceInfo),
                KeyCode::Char('d') => Some(Message::NextDevice),
                KeyCode::Char('L') => Some(Message::OpenLogcat),
                KeyCode::Char('D') => Some(Message::OpenDevMode),
                KeyCode::Enter => {
                    let command = self.model.get_selected_command();
                    Some(Message::ExecuteCommand(command))
                }
                _ => None,
            },

            AppState::Loading => match key {
                KeyCode::Esc | KeyCode::Char('q') => Some(Message::ReturnToMenu),
                _ => None,
            },

            AppState::ShowResult => match key {
                KeyCode::Up | KeyCode::Char('k') => Some(Message::ScrollUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Message::ScrollDown),
                KeyCode::PageUp => Some(Message::ScrollPageUp),
                KeyCode::PageDown => Some(Message::ScrollPageDown),
                KeyCode::Home => Some(Message::ScrollToTop),
                KeyCode::End => Some(Message::ScrollToBottom),
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter | KeyCode::Backspace => {
                    Some(Message::ReturnToMenu)
                }
                _ => Some(Message::ReturnToMenu),
            },

            AppState::DevMode => {
                use crate::devtools::DevFocus;

                // Editor picker takes priority when open
                if self.model.devtools.editor_picker_open {
                    return match key {
                        KeyCode::Up | KeyCode::Char('k') => Some(Message::DevEditorUp),
                        KeyCode::Down | KeyCode::Char('j') => Some(Message::DevEditorDown),
                        KeyCode::Enter => Some(Message::DevEditorConfirm),
                        KeyCode::Esc => Some(Message::DevToggleEditorPicker),
                        _ => None,
                    };
                }

                // Variant picker
                if self.model.devtools.variant_picker_open {
                    return match key {
                        KeyCode::Up | KeyCode::Char('k') => Some(Message::DevPrevVariant),
                        KeyCode::Down | KeyCode::Char('j') => Some(Message::DevNextVariant),
                        KeyCode::Enter => Some(Message::DevToggleVariantPicker),
                        KeyCode::Esc => Some(Message::DevToggleVariantPicker),
                        _ => None,
                    };
                }

                // Global dev mode keys (regardless of focus)
                match key {
                    KeyCode::Esc | KeyCode::Char('q') => Some(Message::CloseDevMode),
                    KeyCode::Char('t') => Some(Message::ThemeNext),
                    KeyCode::Char('b') => Some(Message::DevBuild),
                    KeyCode::Char('i') => Some(Message::DevBuildAndInstall),
                    KeyCode::Char('R') => Some(Message::DevRun),
                    KeyCode::Char('E') => Some(Message::DevToggleEditorPicker),
                    KeyCode::Char('v') => Some(Message::DevToggleVariantPicker),
                    KeyCode::Tab => Some(Message::DevCycleFocus),
                    KeyCode::Char('L') => Some(Message::OpenLogcat),
                    KeyCode::Char('e') => Some(Message::DevOpenFile),
                    // Focus-specific keys
                    _ => {
                        match self.model.devtools.focus {
                            DevFocus::FileBrowser => {
                                let key_event = crossterm::event::KeyEvent::new(
                                    key,
                                    crossterm::event::KeyModifiers::NONE,
                                );
                                Some(Message::DevFileExplorerKey(key_event))
                            }
                            DevFocus::BuildOutput => {
                                // Scroll build output
                                match key {
                                    KeyCode::Up | KeyCode::Char('k') => None, // TODO: scroll build
                                    KeyCode::Down | KeyCode::Char('j') => None,
                                    _ => None,
                                }
                            }
                            DevFocus::Toolbar => match key {
                                KeyCode::Left => Some(Message::DevPrevVariant),
                                KeyCode::Right => Some(Message::DevNextVariant),
                                _ => None,
                            },
                        }
                    }
                }
            }

            AppState::RomFlash => match key {
                KeyCode::Esc | KeyCode::Char('q') => Some(Message::CloseRomFlash),
                KeyCode::Char('d') => Some(Message::RomDetect),
                KeyCode::Up | KeyCode::Char('k') => Some(Message::RomSelectUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Message::RomSelectDown),
                KeyCode::Enter => Some(Message::RomDownload),
                KeyCode::Char('f') => Some(Message::RomRunStep),
                KeyCode::Char('F') => Some(Message::RomConfirmStep),
                _ => None,
            },

            AppState::Logcat => {
                use crate::logcat::FilterField;
                use crate::model::LogcatSaveMode;

                // ── Save dialog active ──────────────────────────────────────
                if self.model.logcat_save_active {
                    // ── File browser sub-mode ───────────────────────────────
                    if self.model.logcat_save_mode == LogcatSaveMode::FileBrowser {
                        // Forward the raw KeyEvent to the file explorer
                        let key_event = crossterm::event::KeyEvent::new(
                            key,
                            crossterm::event::KeyModifiers::NONE,
                        );
                        return Some(Message::LogcatFileExplorerKey(key_event));
                    }

                    // ── Path-input sub-mode ─────────────────────────────────
                    return match key {
                        KeyCode::Esc => Some(Message::LogcatCancelSave),
                        KeyCode::Enter => {
                            let path = self.model.logcat_save_path.clone();
                            if path.trim().is_empty() {
                                Some(Message::LogcatCancelSave)
                            } else {
                                Some(Message::LogcatFileSaved(path))
                            }
                        }
                        KeyCode::Char('S') => Some(Message::LogcatSaveAs),
                        KeyCode::Backspace => Some(Message::LogcatSearchBackspace),
                        KeyCode::Left => Some(Message::LogcatCursorLeft),
                        KeyCode::Right => Some(Message::LogcatCursorRight),
                        KeyCode::Tab => Some(Message::LogcatSaveFilteredOnly),
                        KeyCode::Char(c) => Some(Message::LogcatSearchInput(c)),
                        _ => None,
                    };
                }

                // ── Detail popup active ─────────────────────────────────
                if self.model.logcat.detail_open {
                    return match key {
                        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                            Some(Message::LogcatToggleDetail)
                        }
                        KeyCode::Up | KeyCode::Char('k') => Some(Message::LogcatSelectUp),
                        KeyCode::Down | KeyCode::Char('j') => Some(Message::LogcatSelectDown),
                        KeyCode::Char('y') => Some(Message::LogcatCopyLine),
                        KeyCode::Char('m') => Some(Message::LogcatBookmarkToggle),
                        _ => None,
                    };
                }

                // ── Filter text input active ────────────────────────────────
                let editing = self.model.logcat.filter.active_field != FilterField::None;

                if editing {
                    match key {
                        KeyCode::Esc => Some(Message::LogcatExitFilter),
                        KeyCode::Enter => Some(Message::LogcatExitFilter),
                        KeyCode::Backspace => Some(Message::LogcatSearchBackspace),
                        KeyCode::Delete => Some(Message::LogcatSearchDelete),
                        KeyCode::Left => Some(Message::LogcatCursorLeft),
                        KeyCode::Right => Some(Message::LogcatCursorRight),
                        KeyCode::Char(c) => Some(Message::LogcatSearchInput(c)),
                        _ => None,
                    }
                } else {
                    // ── Normal logcat navigation mode ────────────────────────
                    match key {
                        KeyCode::Esc | KeyCode::Char('q') => Some(Message::CloseLogcat),
                        KeyCode::Up | KeyCode::Char('k') => Some(Message::LogcatScrollUp),
                        KeyCode::Down | KeyCode::Char('j') => Some(Message::LogcatScrollDown),
                        KeyCode::PageUp => Some(Message::LogcatScrollPageUp),
                        KeyCode::PageDown => Some(Message::LogcatScrollPageDown),
                        KeyCode::Home => Some(Message::LogcatScrollToTop),
                        KeyCode::End => Some(Message::LogcatScrollToBottom),
                        KeyCode::Char('G') => Some(Message::LogcatScrollToBottom),
                        KeyCode::Char('g') => Some(Message::LogcatScrollToTop),
                        KeyCode::Char(' ') => Some(Message::LogcatTogglePause),
                        KeyCode::Char('c') => Some(Message::LogcatClear),
                        KeyCode::Char('l') => Some(Message::LogcatCycleLevel),
                        KeyCode::Char('w') => Some(Message::LogcatToggleWordWrap),
                        KeyCode::Char('f') => Some(Message::LogcatToggleSearch),
                        KeyCode::Char('t') => Some(Message::LogcatToggleTagFilter),
                        KeyCode::Char('p') => Some(Message::LogcatTogglePackageFilter),
                        KeyCode::Char('s') => Some(Message::LogcatSave),
                        KeyCode::Char('S') => Some(Message::LogcatSaveAs),
                        // New feature keys
                        KeyCode::Char('r') => Some(Message::LogcatToggleRegex),
                        KeyCode::Char('e') => Some(Message::LogcatToggleExclude),
                        KeyCode::Char('x') => Some(Message::LogcatToggleCompact),
                        KeyCode::Enter => Some(Message::LogcatToggleDetail),
                        KeyCode::Char('m') => Some(Message::LogcatBookmarkToggle),
                        KeyCode::Char('[') => Some(Message::LogcatBookmarkPrev),
                        KeyCode::Char(']') => Some(Message::LogcatBookmarkNext),
                        KeyCode::Left => Some(Message::LogcatHScrollLeft),
                        KeyCode::Right => Some(Message::LogcatHScrollRight),
                        KeyCode::Char('0') => Some(Message::LogcatHScrollReset),
                        KeyCode::Char('y') => Some(Message::LogcatCopyLine),
                        KeyCode::Char('F') => Some(Message::LogcatToggleFold),
                        _ => None,
                    }
                }
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Background task: polls `adb devices` every 2 seconds.
///
/// When the set of connected device serials changes (a device connects or disconnects),
/// a full `DeviceStatus` snapshot is fetched off the async thread via `spawn_blocking`
/// and pushed into the event queue so the UI reflects the change immediately.
async fn device_watcher(sender: mpsc::UnboundedSender<Event>) {
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    // The first tick fires immediately — skip it so we don't duplicate the
    // initial load that `needs_device_refresh` already triggers on startup.
    interval.tick().await;

    let mut last_serials: Vec<String> = Vec::new();

    loop {
        interval.tick().await;

        // Stop the watcher once the app has shut down and dropped the receiver.
        if sender.is_closed() {
            break;
        }

        // Run the (blocking) ADB calls on the thread-pool so the async runtime
        // is never stalled.
        let status = tokio::task::spawn_blocking(|| {
            let mut mgr = AdbManager::new();
            mgr.fetch_device_status()
        })
        .await
        .unwrap_or_default();

        let new_serials: Vec<String> = status.devices.iter().map(|d| d.serial.clone()).collect();

        // Only push an event when the device list actually changed.
        if new_serials != last_serials {
            last_serials = new_serials;
            let _ = sender.send(Event::App(AppEvent::DeviceStatusUpdate(status)));
        }
    }
}
