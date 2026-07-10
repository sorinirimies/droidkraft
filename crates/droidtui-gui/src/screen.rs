//! Background screen-mirroring backend.
//!
//! When started, a dedicated thread repeatedly captures the device screen as
//! PNG (via [`AdbManager::capture_frame_png`]) and writes it to a rotating pair
//! of temp files so the GUI can render the newest frame with `img(path)`
//! without hitting gpui's path-keyed image cache.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use droidtui_core::AdbManager;

/// Target capture interval (~5 fps). `screencap` is relatively expensive, so a
/// modest rate keeps the device responsive.
const FRAME_INTERVAL: Duration = Duration::from_millis(200);

/// Shared, cheaply-cloneable snapshot of the stream state read by the UI.
#[derive(Debug, Default, Clone)]
pub struct ScreenState {
    /// Path to the most recently written frame, if any.
    pub latest_path: Option<PathBuf>,
    /// Total frames captured.
    pub frame_count: u64,
    /// Rolling frames-per-second estimate.
    pub fps: f32,
    /// Last error, if capture failed.
    pub error: Option<String>,
}

/// A controllable background screen-capture stream.
pub struct ScreenStream {
    running: Arc<AtomicBool>,
    state: Arc<Mutex<ScreenState>>,
}

impl Default for ScreenStream {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenStream {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(Mutex::new(ScreenState::default())),
        }
    }

    /// Whether the capture thread is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// A snapshot of the current stream state.
    pub fn state(&self) -> ScreenState {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Start capturing frames from the given device serial.
    pub fn start(&self, serial: String) {
        if self.running.swap(true, Ordering::SeqCst) {
            return; // already running
        }
        // Reset state.
        if let Ok(mut s) = self.state.lock() {
            *s = ScreenState::default();
        }

        let running = self.running.clone();
        let state = self.state.clone();
        std::thread::spawn(move || capture_loop(serial, running, state));
    }

    /// Stop capturing.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn capture_loop(serial: String, running: Arc<AtomicBool>, state: Arc<Mutex<ScreenState>>) {
    let mut adb = AdbManager::new();
    adb.select_device(serial);

    let dir = std::env::temp_dir();
    let paths = [
        dir.join("droidtui_frame_a.png"),
        dir.join("droidtui_frame_b.png"),
    ];
    let mut toggle = 0usize;

    let mut frames_since = 0u32;
    let mut window_start = Instant::now();

    while running.load(Ordering::Relaxed) {
        let started = Instant::now();

        match adb.capture_frame_png() {
            Ok(bytes) if is_png(&bytes) => {
                let path = &paths[toggle % 2];
                toggle = toggle.wrapping_add(1);
                if let Err(e) = std::fs::write(path, &bytes) {
                    set_error(&state, format!("write frame: {e}"));
                } else if let Ok(mut s) = state.lock() {
                    s.latest_path = Some(path.clone());
                    s.frame_count += 1;
                    s.error = None;
                    frames_since += 1;
                }
            }
            Ok(_) => set_error(&state, "capture returned non-PNG data".into()),
            Err(e) => set_error(&state, e.to_string()),
        }

        // Update fps roughly once per second.
        if window_start.elapsed() >= Duration::from_secs(1) {
            let secs = window_start.elapsed().as_secs_f32().max(0.001);
            if let Ok(mut s) = state.lock() {
                s.fps = frames_since as f32 / secs;
            }
            frames_since = 0;
            window_start = Instant::now();
        }

        // Pace the loop.
        if let Some(remaining) = FRAME_INTERVAL.checked_sub(started.elapsed()) {
            std::thread::sleep(remaining);
        }
    }
}

fn set_error(state: &Arc<Mutex<ScreenState>>, msg: String) {
    if let Ok(mut s) = state.lock() {
        s.error = Some(msg);
    }
}

/// PNG magic-number check to guard against shell/CRLF corruption.
fn is_png(bytes: &[u8]) -> bool {
    bytes.len() > 8 && bytes[..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_png_magic() {
        let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert!(is_png(&png));
        assert!(!is_png(b"not a png"));
        assert!(!is_png(&[]));
    }

    #[test]
    fn stream_starts_not_running() {
        let s = ScreenStream::new();
        assert!(!s.is_running());
        assert_eq!(s.state().frame_count, 0);
    }
}
