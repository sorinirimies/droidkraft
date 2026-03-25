use crate::{
    event::{AppEvent, Event, EventHandler},
    message::Message,
    model::{AppState, Model},
    update,
};
use ratatui::{crossterm::event::KeyCode, DefaultTerminal};

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
        Self {
            model: Model::new(),
            events: EventHandler::new(),
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
            })),
        }
    }

    /// Map keyboard input to messages based on current state
    fn key_to_message(&self, key: KeyCode) -> Option<Message> {
        match self.model.state {
            AppState::Startup => Some(Message::SkipStartup),

            AppState::Menu => match key {
                KeyCode::Esc | KeyCode::Char('q') => Some(Message::Quit),
                KeyCode::Up | KeyCode::Char('k') => Some(Message::MenuUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Message::MenuDown),
                KeyCode::Tab => Some(Message::SectionNext),
                KeyCode::BackTab => Some(Message::SectionPrev),
                KeyCode::Char('r') => Some(Message::RefreshDeviceInfo),
                KeyCode::Char('d') => Some(Message::NextDevice),
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
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
