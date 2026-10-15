//! Messages (input to `update`) and commands (side effects requested by it).

use crossterm::event::{KeyEvent, MouseEvent};

use crate::worker::{Req, Res};

pub enum Message {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    Worker(Res),
}

/// Side effects the app loop executes after `update`.
pub enum Cmd {
    Worker(Req),
    Quit,
    /// Reveal/open a file or folder with the OS handler.
    Open(std::path::PathBuf),
    /// Spawn the external GUI mirror window.
    Mirror {
        serial: Option<String>,
        size: Option<(u32, u32)>,
    },
}
