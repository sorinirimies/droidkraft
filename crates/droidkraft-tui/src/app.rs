//! Event loop: crossterm events + tick + worker results.

use std::time::{Duration, Instant};

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::execute;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::message::{Cmd, Message};
use crate::model::Model;
use crate::update::update;
use crate::worker::Worker;

pub const TICK: Duration = Duration::from_millis(100);
/// Redraw cadence (~30 fps) while the Screen panel is open.
pub const FRAME: Duration = Duration::from_millis(33);

pub struct App {
    pub model: Model,
    pub worker: Worker,
    /// Graphics protocol selection (`--graphics` / `DROIDKRAFT_GRAPHICS`).
    pub graphics: crate::gfx::GraphicsMode,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            model: Model::default(),
            worker: Worker::spawn(),
            graphics: crate::gfx::GraphicsMode::Auto,
        }
    }

    /// Feed a message through `update` and execute resulting commands.
    pub fn dispatch(&mut self, msg: Message) {
        for c in update(&mut self.model, msg) {
            match c {
                Cmd::Worker(r) => self.worker.send(r),
                Cmd::Quit => self.model.quit = true,
                Cmd::Open(p) => match open_path(&p) {
                    Ok(()) => self.model.set_message(format!("Opened {}", p.display())),
                    Err(e) => self.model.set_message(format!("Open failed: {e}")),
                },
                Cmd::Mirror { serial, size } => {
                    match crate::mirror_cmd::spawn(serial.as_deref(), size) {
                        Ok(()) => self.model.set_message("Opened external mirror window"),
                        Err(e) => self.model.set_message(e),
                    }
                }
            }
        }
    }

    pub fn run(mut self) -> color_eyre::Result<()> {
        install_panic_hook();
        // Query the terminal BEFORE raw mode / alt screen (reads stdin replies).
        self.model.screen.gfx = crate::gfx::Gfx::detect(self.graphics);
        let mut out = std::io::stdout();
        crossterm::terminal::enable_raw_mode()?;
        execute!(
            out,
            crossterm::terminal::EnterAlternateScreen,
            EnableMouseCapture
        )?;
        let mut term = Terminal::new(CrosstermBackend::new(out))?;
        let res = self.event_loop(&mut term);
        self.model.screen.gfx.shutdown_kitty(&mut std::io::stdout());
        crate::ui::screen::shutdown(&mut self.model);
        self.model.shutdown_all();
        restore();
        res
    }

    fn event_loop<B: ratatui::backend::Backend>(
        &mut self,
        term: &mut Terminal<B>,
    ) -> color_eyre::Result<()>
    where
        B::Error: Send + Sync + 'static,
    {
        let mut last = Instant::now();
        let mut dirty = true;
        while !self.model.quit {
            if self.model.screen.gfx.poll() {
                dirty = true;
            }
            if dirty {
                if std::mem::take(&mut self.model.screen.gfx.clear_pending) {
                    term.clear()?;
                }
                term.draw(|f| crate::ui::draw(f, &mut self.model))?;
                let _ = self.model.screen.gfx.flush_kitty(&mut std::io::stdout());
                // Image vanished/moved: wipe stale Kitty/Sixel pixels and repaint.
                dirty = self.model.screen.gfx.clear_pending;
            }
            let on_screen =
                self.model.active.is_some() && self.model.panel == crate::model::Panel::Screen;
            let wait =
                if on_screen { FRAME } else { TICK }.min(TICK.saturating_sub(last.elapsed()));
            if on_screen && self.model.screen.seq_changed() {
                dirty = true;
            }
            if event::poll(wait)? {
                dirty = true;
                match event::read()? {
                    Event::Key(k) => self.dispatch(Message::Key(k)),
                    Event::Mouse(e) => self.dispatch(Message::Mouse(e)),
                    _ => {}
                }
            }
            for r in self.worker.drain() {
                dirty = true;
                self.dispatch(Message::Worker(r));
            }
            if last.elapsed() >= TICK {
                dirty = true;
                last = Instant::now();
                self.dispatch(Message::Tick);
            }
        }
        Ok(())
    }
}

/// OS opener program for the current platform.
pub fn opener() -> &'static str {
    if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    }
}

/// Open/reveal a file or folder with the OS handler (non-blocking).
pub fn open_path(p: &std::path::Path) -> Result<(), String> {
    std::process::Command::new(opener())
        .arg(p)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("{}: {e}", opener()))
}

fn restore() {
    if crate::gfx::KITTY_USED.load(std::sync::atomic::Ordering::SeqCst) {
        use std::io::Write;
        let _ = std::io::stdout().write_all(crate::gfx::kitty_delete().as_bytes());
    }
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = execute!(
        std::io::stdout(),
        DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen
    );
}

fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        prev(info);
    }));
}
