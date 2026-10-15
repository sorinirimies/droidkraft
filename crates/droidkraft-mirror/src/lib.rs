//! Low-latency screen mirroring + remote input.
//!
//! Pipeline (scrcpy-style, no on-device server needed):
//!
//! ```text
//! adb exec-out screenrecord (H.264) ─▶ ffmpeg (decode, low-delay, synthetic pts) ─▶ MJPEG
//!        ─▶ this thread (latest-frame-wins decode) ─▶ `RgbaImage`
//! ```
//!
//! All device traffic uses the [`adb_client`] crate (talking to the local adb
//! server over TCP) — no `adb` binary is spawned. Input (tap / live drag /
//! keys / text) goes through one persistent raw shell we feed `input …` lines.
//!
//! Frontend-neutral: the GUI converts frames to its own image type, the TUI
//! renders them with half-block characters.

pub mod record;
pub use record::{
    default_output_dir, timestamped_name, RecordOptions, RecordStatus, ScreenRecorder,
};

use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use adb_client::{ADBDeviceExt, ADBServer, ADBServerDevice};
use image::{ImageFormat, RgbaImage};

/// Longest edge (px) of the mirrored video; larger is sharper but heavier.
const MAX_HEIGHT: u32 = 1280;
/// H.264 bitrate requested from `screenrecord`.
const BIT_RATE: &str = "16M";
/// `screenrecord` hard-stops after 180 s; we restart seamlessly.
const SESSION_SECS: &str = "180";

/// One decoded video frame, ready for display.
#[derive(Clone)]
pub struct VideoFrame {
    /// RGBA pixels.
    pub image: Arc<RgbaImage>,
    pub width: u32,
    pub height: u32,
    pub seq: u64,
}

/// Shared, cheaply-cloneable snapshot of the stream state read by the UI.
#[derive(Clone, Default)]
pub struct ScreenState {
    pub frame: Option<VideoFrame>,
    pub frame_count: u64,
    pub fps: f32,
    pub error: Option<String>,
    /// Device display size reported by `wm size` (portrait-natural).
    pub device_size: Option<(u32, u32)>,
    /// Smoothed JPEG decode time per frame (ms).
    pub decode_ms: f32,
    /// Complete frames skipped because newer ones had already arrived.
    pub dropped: u64,
    /// Video data rate from ffmpeg (MB/s).
    pub mbps: f32,
    /// Whether the displayed frame is a `screencap` still (idle screen).
    pub is_still: bool,
}

/// A controllable background screen stream with input injection.
pub struct ScreenStream {
    running: Arc<AtomicBool>,
    generation: Arc<AtomicU64>,
    state: Arc<Mutex<ScreenState>>,
    input: Arc<Mutex<Option<Sender<String>>>>,
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
            generation: Arc::new(AtomicU64::new(0)),
            state: Arc::new(Mutex::new(ScreenState::default())),
            input: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn state(&self) -> ScreenState {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Sequence number of the newest frame (cheap change check).
    pub fn latest_seq(&self) -> u64 {
        self.state.lock().map(|s| s.frame_count).unwrap_or(0)
    }

    pub fn start(&self, serial: String) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Ok(mut s) = self.state.lock() {
            *s = ScreenState::default();
        }
        let gen = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let ctx = Ctx {
            running: self.running.clone(),
            generation: self.generation.clone(),
            gen,
            state: self.state.clone(),
            epoch: Instant::now(),
            last_video_ms: Arc::new(AtomicU64::new(0)),
            video_seq: Arc::new(AtomicU64::new(0)),
        };
        let (tx, rx) = channel::<String>();
        if let Ok(mut i) = self.input.lock() {
            *i = Some(tx);
        }
        let s2 = serial.clone();
        let c2 = ctx.clone();
        std::thread::spawn(move || input_loop(s2, c2, rx));
        std::thread::spawn(move || stream_loop(serial, ctx));
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Ok(mut i) = self.input.lock() {
            *i = None;
        }
    }

    // ── Input ────────────────────────────────────────────────────────────

    fn send(&self, line: String) {
        if let Ok(i) = self.input.lock() {
            if let Some(tx) = i.as_ref() {
                let _ = tx.send(line);
            }
        }
    }

    pub fn tap(&self, x: u32, y: u32) {
        self.send(format!("input tap {x} {y}"));
    }

    pub fn swipe(&self, x1: u32, y1: u32, x2: u32, y2: u32, ms: u32) {
        self.send(format!("input swipe {x1} {y1} {x2} {y2} {ms}"));
    }

    /// Live touch: `action` is `DOWN`, `MOVE` or `UP` (needs Android 10+).
    pub fn touch(&self, action: &str, x: u32, y: u32) {
        self.send(format!("input motionevent {action} {x} {y}"));
    }

    /// Send an Android keycode name or number, e.g. `KEYCODE_BACK`.
    pub fn key(&self, keycode: &str) {
        self.send(format!("input keyevent {keycode}"));
    }

    /// Type text (ASCII only; other chars are dropped).
    pub fn text(&self, text: &str) {
        if let Some(line) = text_command(text) {
            self.send(line);
        }
    }
}

#[derive(Clone)]
struct Ctx {
    running: Arc<AtomicBool>,
    generation: Arc<AtomicU64>,
    gen: u64,
    state: Arc<Mutex<ScreenState>>,
    epoch: Instant,
    /// Millis since `epoch` of the last *video* frame.
    last_video_ms: Arc<AtomicU64>,
    /// Number of video frames published.
    video_seq: Arc<AtomicU64>,
}

impl Ctx {
    fn alive(&self) -> bool {
        self.running.load(Ordering::Relaxed) && self.generation.load(Ordering::Relaxed) == self.gen
    }

    fn set_error(&self, msg: impl Into<String>) {
        if !self.alive() {
            return;
        }
        if let Ok(mut s) = self.state.lock() {
            s.error = Some(msg.into());
        }
    }
}

// ── Tool discovery ─────────────────────────────────────────────────────────

/// Connect to the local adb server and select `serial`.
fn open_device(serial: &str) -> Result<ADBServerDevice, String> {
    let mut server = ADBServer::new(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037));
    server
        .get_device_by_name(serial)
        .map_err(|e| format!("adb: {e} (is the adb server running? try `adb start-server`)"))
}

/// Run a one-shot shell command and return its raw stdout.
fn shell_bytes(serial: &str, cmd: &str) -> Result<Vec<u8>, String> {
    let mut dev = open_device(serial)?;
    let mut out = Vec::new();
    dev.shell_command(&[cmd], &mut out)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

fn find_ffmpeg() -> Option<String> {
    // GUI apps launched from Finder have a minimal PATH, so probe common spots.
    let mut cands: Vec<String> = vec!["ffmpeg".into()];
    for d in [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/usr/bin",
        "/snap/bin",
    ] {
        cands.push(format!("{d}/ffmpeg"));
    }
    cands.into_iter().find(|c| {
        Command::new(c)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

fn wm_size(serial: &str) -> Option<(u32, u32)> {
    let out = shell_bytes(serial, "wm size").ok()?;
    parse_wm_size(&String::from_utf8_lossy(&out))
}

/// Parse `wm size` output; an "Override size" line wins over "Physical size".
pub fn parse_wm_size(s: &str) -> Option<(u32, u32)> {
    let mut found = None;
    for line in s.lines() {
        if let Some((_, dims)) = line.split_once(':') {
            if let Some((w, h)) = dims.trim().split_once('x') {
                if let (Ok(w), Ok(h)) = (w.trim().parse(), h.trim().parse()) {
                    found = Some((w, h));
                }
            }
        }
    }
    found
}

// ── Video ──────────────────────────────────────────────────────────────────

fn stream_loop(serial: String, ctx: Ctx) {
    let Some(ffmpeg) = find_ffmpeg() else {
        ctx.set_error(
            "ffmpeg not found. Install it (e.g. `brew install ffmpeg`) to enable mirroring.",
        );
        ctx.running.store(false, Ordering::SeqCst);
        return;
    };

    // First picture ASAP: the still thread grabs a screencap while the video
    // pipeline (ffmpeg + screenrecord) is still spinning up.
    {
        let (serial, ctx) = (serial.clone(), ctx.clone());
        std::thread::spawn(move || still_loop(&serial, &ctx));
    }
    let mut size_job = Some({
        let (serial, ctx) = (serial.clone(), ctx.clone());
        std::thread::spawn(move || {
            let size = wm_size(&serial);
            if let Ok(mut s) = ctx.state.lock() {
                s.device_size = size;
            }
            size
        })
    });
    let mut size: Option<(u32, u32)> = None;

    let mut fallback = false;
    let mut win = StatWindow::new();

    while ctx.alive() {
        let started = Instant::now();
        let frames = run_session(
            &ffmpeg,
            &serial,
            if fallback {
                // Only the fallback path needs the size synchronously.
                if let Some(h) = size_job.take() {
                    size = h.join().ok().flatten();
                }
                size
            } else {
                None
            },
            &ctx,
            &mut win,
        );
        if std::env::var_os("DROIDKRAFT_DEBUG").is_some() {
            eprintln!("session ended after {:?}: {frames:?}", started.elapsed());
        }
        if !ctx.alive() {
            break;
        }
        match frames {
            Ok(0) if started.elapsed() < Duration::from_secs(5) => {
                if fallback {
                    ctx.set_error("No video from device (screenrecord failed).");
                    std::thread::sleep(Duration::from_secs(2));
                }
                fallback = true; // retry once with an explicit --size
            }
            Ok(_) => {}
            Err(e) => {
                ctx.set_error(e);
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

/// `screenrecord` only emits data when the screen changes, and the decoder
/// holds the newest frame until another arrives. Whenever video has been idle
/// for a moment, grab one `screencap` so a static screen is always up to date.
fn still_loop(serial: &str, ctx: &Ctx) {
    const IDLE_MS: u64 = 350;
    let mut done_seq = u64::MAX;
    while ctx.alive() {
        std::thread::sleep(Duration::from_millis(100));
        let seq = ctx.video_seq.load(Ordering::Relaxed);
        let idle = (ctx.epoch.elapsed().as_millis() as u64)
            .saturating_sub(ctx.last_video_ms.load(Ordering::Relaxed));
        if seq == done_seq || idle < IDLE_MS {
            continue;
        }
        done_seq = seq;
        let Ok(out) = shell_bytes(serial, "screencap -p") else {
            continue;
        };
        // Skip if a newer video frame arrived while capturing.
        if ctx.video_seq.load(Ordering::Relaxed) != seq || !ctx.alive() {
            done_seq = u64::MAX;
            continue;
        }
        if let Some((img, w, h)) = decode_image(&out, ImageFormat::Png) {
            if let Ok(mut s) = ctx.state.lock() {
                s.frame_count += 1;
                s.error = None;
                s.is_still = true;
                s.fps = 0.0;
                s.frame = Some(VideoFrame {
                    image: img,
                    width: w,
                    height: h,
                    seq: s.frame_count,
                });
            }
        }
    }
}

/// Writer that forwards screenrecord bytes into ffmpeg's stdin and aborts the
/// adb stream (by erroring) once the session is no longer wanted.
struct FfmpegSink {
    stdin: std::process::ChildStdin,
    ctx: Ctx,
    done: Arc<AtomicBool>,
}

impl Write for FfmpegSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if !self.ctx.alive() || self.done.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "session ended"));
        }
        self.stdin.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdin.flush()
    }
}

fn run_session(
    ffmpeg: &str,
    serial: &str,
    force_size: Option<(u32, u32)>,
    ctx: &Ctx,
    win: &mut StatWindow,
) -> Result<u64, String> {
    let mut rec = format!(
        "screenrecord --output-format=h264 --bit-rate {BIT_RATE} --time-limit {SESSION_SECS}"
    );
    if let Some((w, h)) = force_size {
        let (w, h) = fit_even(w, h, 1280);
        rec.push_str(&format!(" --size {w}x{h}"));
    }
    rec.push_str(" -");

    let mut dev = open_device(serial)?;

    let scale = format!("setpts=N/TB/1000,scale=-2:min({MAX_HEIGHT}\\,ih)");
    let mut dec = Command::new(ffmpeg);
    dec.args([
        "-loglevel",
        if std::env::var_os("DROIDKRAFT_DEBUG").is_some() {
            "info"
        } else {
            "error"
        },
        "-fflags",
        "nobuffer",
        "-flags",
        "low_delay",
    ])
    .args([
        "-probesize",
        "32",
        "-analyzeduration",
        "0",
        "-framerate",
        "1000",
        "-f",
        "h264",
        "-i",
        "pipe:0",
    ])
    .args([
        "-vf", &scale, "-pix_fmt", "yuvj420p", "-c:v", "mjpeg", "-q:v", "4",
    ])
    .args(["-fps_mode", "passthrough", "-f", "mjpeg", "pipe:1"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(if std::env::var_os("DROIDKRAFT_DEBUG").is_some() {
        Stdio::inherit()
    } else {
        Stdio::null()
    });
    let mut dec = dec.spawn().map_err(|e| format!("spawn ffmpeg: {e}"))?;
    let stdin = dec.stdin.take().ok_or("ffmpeg stdin")?;
    let mut out = dec.stdout.take().ok_or("ffmpeg stdout")?;

    let done = Arc::new(AtomicBool::new(false));
    let dec = Arc::new(Mutex::new(dec));

    // Feeder: screenrecord (over adb_client) -> ffmpeg stdin.
    {
        let sink = FfmpegSink {
            stdin,
            ctx: ctx.clone(),
            done: done.clone(),
        };
        let mut sink = sink;
        let (done, dec) = (done.clone(), dec.clone());
        std::thread::spawn(move || {
            let r = dev.shell_command(&[rec.as_str()], &mut sink);
            if std::env::var_os("DROIDKRAFT_DEBUG").is_some() {
                eprintln!("screenrecord ended: {r:?}");
            }
            drop(sink); // close ffmpeg stdin -> EOF
            done.store(true, Ordering::Relaxed);
            // Give ffmpeg a moment to flush, then make sure it's gone.
            std::thread::sleep(Duration::from_millis(300));
            if let Ok(mut c) = dec.lock() {
                let _ = c.kill();
            }
        });
    }
    // Watcher: on stop, interrupt screenrecord on the device and kill ffmpeg.
    {
        let (done, dec, ctx, serial) = (done.clone(), dec.clone(), ctx.clone(), serial.to_string());
        std::thread::spawn(move || {
            while ctx.alive() && !done.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(100));
            }
            if !ctx.alive() {
                let _ = shell_bytes(&serial, "pkill -2 -f '[s]creenrecord --output-format'");
            }
            if let Ok(mut c) = dec.lock() {
                let _ = c.kill();
            }
        });
    }

    let mut buf: Vec<u8> = Vec::with_capacity(1 << 20);
    let mut chunk = vec![0u8; 256 * 1024];
    let mut scanned = 0usize;
    let mut frames = 0u64;

    while ctx.alive() {
        let n = match out.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        buf.extend_from_slice(&chunk[..n]);
        win.bytes += n as u64;

        // Find the newest complete JPEG; older ones are skipped (latency win).
        let mut last_end = None;
        let mut complete = 0u64;
        while let Some(end) = find_eoi(&buf, scanned) {
            last_end = Some(end);
            scanned = end;
            complete += 1;
        }
        win.dropped += complete.saturating_sub(1);
        if let Some(end) = last_end {
            let start = last_soi(&buf[..end]).unwrap_or(0);
            let t = Instant::now();
            if let Some(frame) = decode_jpeg(&buf[start..end]) {
                frames += 1;
                win.frames += 1;
                win.decode_ms += t.elapsed().as_secs_f32() * 1000.0;
                publish(ctx, frame, win);
            }
            buf.drain(..end);
            scanned = 0;
        } else {
            scanned = buf.len().saturating_sub(1);
        }
    }

    done.store(true, Ordering::Relaxed);
    if let Ok(mut c) = dec.lock() {
        let _ = c.kill();
        let _ = c.wait();
    }
    Ok(frames)
}

/// Rolling one-second statistics window for the video thread.
struct StatWindow {
    start: Instant,
    frames: u32,
    bytes: u64,
    decode_ms: f32,
    dropped: u64,
}

impl StatWindow {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            frames: 0,
            bytes: 0,
            decode_ms: 0.0,
            dropped: 0,
        }
    }
}

fn publish(ctx: &Ctx, (img, w, h): (Arc<RgbaImage>, u32, u32), win: &mut StatWindow) {
    if !ctx.alive() {
        return;
    }
    ctx.last_video_ms
        .store(ctx.epoch.elapsed().as_millis() as u64, Ordering::Relaxed);
    ctx.video_seq.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut s) = ctx.state.lock() {
        s.frame_count += 1;
        s.error = None;
        s.is_still = false;
        s.frame = Some(VideoFrame {
            image: img,
            width: w,
            height: h,
            seq: s.frame_count,
        });
        let el = win.start.elapsed();
        if el >= Duration::from_secs(1) {
            let secs = el.as_secs_f32();
            s.fps = win.frames as f32 / secs;
            s.mbps = win.bytes as f32 / secs / 1.0e6;
            s.decode_ms = win.decode_ms / win.frames.max(1) as f32;
            s.dropped += win.dropped;
            *win = StatWindow::new();
        }
    }
}

fn decode_jpeg(bytes: &[u8]) -> Option<(Arc<RgbaImage>, u32, u32)> {
    decode_image(bytes, ImageFormat::Jpeg)
}

fn decode_image(bytes: &[u8], fmt: ImageFormat) -> Option<(Arc<RgbaImage>, u32, u32)> {
    let img = image::load_from_memory_with_format(bytes, fmt).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some((Arc::new(rgba), w, h))
}

/// Index just past the next JPEG EOI marker (`FF D9`) at/after `from`.
fn find_eoi(buf: &[u8], from: usize) -> Option<usize> {
    let from = from.min(buf.len());
    buf[from..]
        .windows(2)
        .position(|w| w == [0xFF, 0xD9])
        .map(|p| from + p + 2)
}

/// Start of the JPEG that ends at the end of `buf` (last SOI `FF D8 FF`).
fn last_soi(buf: &[u8]) -> Option<usize> {
    // Skip the trailing EOI so we find the SOI of *this* image.
    buf.windows(3).rposition(|w| w == [0xFF, 0xD8, 0xFF])
}

fn fit_even(w: u32, h: u32, max_long: u32) -> (u32, u32) {
    let long = w.max(h).max(1);
    let k = (max_long as f32 / long as f32).min(1.0);
    let ev = |v: u32| (((v as f32 * k) as u32) & !1).max(2);
    (ev(w), ev(h))
}

// ── Input ──────────────────────────────────────────────────────────────────

/// `Read` adapter over the input channel: blocks for commands, coalescing
/// bursts of MOVE events, and reports EOF once the stream is stopped.
struct InputReader {
    rx: std::sync::mpsc::Receiver<String>,
    ctx: Ctx,
    pending: Vec<u8>,
}

impl Read for InputReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        while self.pending.is_empty() {
            if !self.ctx.alive() {
                return Ok(0);
            }
            match self.rx.recv_timeout(Duration::from_millis(200)) {
                Ok(first) => {
                    // Each `input` call costs ~30 ms on the device; collapse
                    // runs of MOVE events so a fast drag never backs up.
                    let mut batch = vec![first];
                    while let Ok(l) = self.rx.try_recv() {
                        batch.push(l);
                    }
                    let mut script = coalesce_moves(batch).join("\n");
                    script.push('\n');
                    self.pending = script.into_bytes();
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => return Ok(0),
            }
        }
        let n = out.len().min(self.pending.len());
        out[..n].copy_from_slice(&self.pending[..n]);
        self.pending.drain(..n);
        Ok(n)
    }
}

fn input_loop(serial: String, ctx: Ctx, rx: std::sync::mpsc::Receiver<String>) {
    let mut dev = match open_device(&serial) {
        Ok(d) => d,
        Err(e) => {
            ctx.set_error(e);
            return;
        }
    };
    let mut reader = InputReader {
        rx,
        ctx: ctx.clone(),
        pending: Vec::new(),
    };
    let r = dev.shell(&mut reader, Box::new(io::sink()));
    if std::env::var_os("DROIDKRAFT_DEBUG").is_some() {
        eprintln!("input shell ended: {r:?}");
    }
    if let Err(e) = r {
        ctx.set_error(format!("input shell: {e}"));
    }
}

const MOVE_PREFIX: &str = "input motionevent MOVE";

/// Drop every `MOVE` that is immediately followed by another `MOVE`.
fn coalesce_moves(lines: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    for l in lines {
        if l.starts_with(MOVE_PREFIX) && out.last().is_some_and(|p| p.starts_with(MOVE_PREFIX)) {
            out.pop();
        }
        out.push(l);
    }
    out
}

/// Build an `input text` line (quoted, spaces → `%s`); `None` if nothing typable.
pub fn text_command(text: &str) -> Option<String> {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            ' ' => out.push_str("%s"),
            '\'' => out.push_str("'\\''"),
            '%' => out.push_str("\\%"),
            c if c.is_ascii_graphic() => {
                if "\\\"()<>|;&*~`$#{}[]!?".contains(c) {
                    out.push('\\');
                }
                out.push(c);
            }
            _ => {}
        }
    }
    (!out.is_empty()).then(|| format!("input text '{out}'"))
}

// ── Coordinate mapping ─────────────────────────────────────────────────────

/// Letterboxed placement of a `fw`×`fh` frame inside a view: `(x, y, w, h)`.
pub fn fit_rect(view_w: f32, view_h: f32, fw: u32, fh: u32) -> (f32, f32, f32, f32) {
    let (fw, fh) = (fw.max(1) as f32, fh.max(1) as f32);
    let s = (view_w / fw).min(view_h / fh).max(0.0);
    let (w, h) = (fw * s, fh * s);
    ((view_w - w) / 2.0, (view_h - h) / 2.0, w, h)
}

/// Zoom mode of the mirror viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Zoom {
    /// Fit whole screen inside the viewport (letterboxed).
    Fit,
    /// Device pixels × factor (`1.0` = 1:1 device pixel per logical pixel).
    Scale(f32),
}

/// Device display size in the frame's current orientation. `wm size` is
/// portrait-natural, so swap when the frame orientation differs (rotation).
pub fn oriented_device(frame: (u32, u32), device: Option<(u32, u32)>) -> (u32, u32) {
    match device {
        Some((a, b)) if (frame.0 > frame.1) == (a > b) => (a, b),
        Some((a, b)) => (b, a),
        None => frame,
    }
}

/// Where the frame is drawn inside the view: `(x, y, w, h)` relative to view.
/// `pan` offsets the image when it is larger than the view (clamped).
pub fn layout(
    view: (f32, f32),
    frame: (u32, u32),
    device: (u32, u32),
    zoom: Zoom,
    pan: (f32, f32),
) -> (f32, f32, f32, f32) {
    match zoom {
        Zoom::Fit => fit_rect(view.0, view.1, frame.0, frame.1),
        Zoom::Scale(z) => {
            let (w, h) = (device.0 as f32 * z, device.1 as f32 * z);
            let axis = |v: f32, d: f32, p: f32| {
                if d <= v {
                    (v - d) / 2.0
                } else {
                    ((v - d) / 2.0 + p).clamp(v - d, 0.0)
                }
            };
            (axis(view.0, w, pan.0), axis(view.1, h, pan.1), w, h)
        }
    }
}

/// Map a point relative to the view's origin to device pixels, or `None` if it
/// falls outside the drawn frame rectangle `rect`.
pub fn map_to_device(
    rel: (f32, f32),
    rect: (f32, f32, f32, f32),
    frame: (u32, u32),
    device: Option<(u32, u32)>,
) -> Option<(u32, u32)> {
    let (x0, y0, w, h) = rect;
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let (nx, ny) = ((rel.0 - x0) / w, (rel.1 - y0) / h);
    if !(0.0..=1.0).contains(&nx) || !(0.0..=1.0).contains(&ny) {
        return None;
    }
    let (dw, dh) = oriented_device(frame, device);
    Some((
        ((nx * dw as f32) as u32).min(dw.saturating_sub(1)),
        ((ny * dh as f32) as u32).min(dh.saturating_sub(1)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eoi_and_soi() {
        let b = [
            0xFF, 0xD8, 0xFF, 0xE0, 1, 2, 0xFF, 0xD9, 0xFF, 0xD8, 0xFF, 0xDB, 0xFF, 0xD9,
        ];
        assert_eq!(find_eoi(&b, 0), Some(8));
        assert_eq!(find_eoi(&b, 8), Some(14));
        assert_eq!(last_soi(&b[..14]), Some(8));
    }

    #[test]
    fn wm_size_override_wins() {
        let s = "Physical size: 1080x2400\nOverride size: 720x1600\n";
        assert_eq!(parse_wm_size(s), Some((720, 1600)));
        assert_eq!(
            parse_wm_size("Physical size: 1080x2400"),
            Some((1080, 2400))
        );
    }

    #[test]
    fn text_escaping() {
        assert_eq!(text_command("a b").unwrap(), "input text 'a%sb'");
        assert_eq!(text_command("it's").unwrap(), "input text 'it'\\''s'");
        assert!(text_command("é").is_none());
    }

    #[test]
    fn maps_with_letterbox() {
        let r = fit_rect(200.0, 100.0, 50, 100); // centered 50 wide at x=75
        assert_eq!(map_to_device((10.0, 50.0), r, (50, 100), None), None);
        assert_eq!(
            map_to_device((100.0, 50.0), r, (50, 100), Some((1080, 2400))),
            Some((540, 1200))
        );
        // Rotated landscape frame vs portrait wm size swaps device dims.
        let r = fit_rect(200.0, 100.0, 200, 100);
        assert_eq!(
            map_to_device((100.0, 50.0), r, (200, 100), Some((1080, 2400))),
            Some((1200, 540))
        );
    }

    #[test]
    fn coalesces_moves() {
        let v = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let out = coalesce_moves(v(&[
            "input motionevent DOWN 1 1",
            "input motionevent MOVE 1 2",
            "input motionevent MOVE 1 3",
            "input motionevent UP 1 3",
        ]));
        assert_eq!(
            out,
            v(&[
                "input motionevent DOWN 1 1",
                "input motionevent MOVE 1 3",
                "input motionevent UP 1 3"
            ])
        );
    }

    #[test]
    fn zoom_layout() {
        // 1:1 smaller than view -> centered.
        assert_eq!(
            layout(
                (400.0, 300.0),
                (50, 100),
                (100, 200),
                Zoom::Scale(1.0),
                (0.0, 0.0)
            ),
            (150.0, 50.0, 100.0, 200.0)
        );
        // Larger than view -> pan clamped to edges.
        let (x, y, ..) = layout(
            (100.0, 100.0),
            (50, 100),
            (200, 400),
            Zoom::Scale(1.0),
            (9999.0, -9999.0),
        );
        assert_eq!((x, y), (0.0, -300.0));
    }

    #[test]
    fn stream_starts_not_running() {
        let s = ScreenStream::new();
        assert!(!s.is_running());
        assert_eq!(s.state().frame_count, 0);
    }
}
