//! Real-image rendering of the screen mirror (Kitty / iTerm2 / Sixel) via `ratatui-image`.
//!
//! Pure logic (mode parsing, protocol choice, rect fitting) is separated from the
//! stateful [`Gfx`] which owns the picker, the background resize+encode worker and
//! the "clear stale image" bookkeeping.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};
use std::sync::Arc;

use image::imageops::FilterType;
use image::{DynamicImage, RgbaImage};
use ratatui::layout::Rect;
use ratatui::Frame;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::thread::{ResizeResponse, ThreadProtocol};
use ratatui_image::{Resize, StatefulImage};

/// User-selectable graphics mode (`--graphics` / `DROIDKRAFT_GRAPHICS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphicsMode {
    #[default]
    Auto,
    Kitty,
    Iterm2,
    Sixel,
    Halfblocks,
}

impl GraphicsMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" | "" => Some(Self::Auto),
            "kitty" => Some(Self::Kitty),
            "iterm2" | "iterm" => Some(Self::Iterm2),
            "sixel" => Some(Self::Sixel),
            "halfblocks" | "halfblock" | "half" => Some(Self::Halfblocks),
            _ => None,
        }
    }
}

/// CLI value wins over env; an invalid env value is ignored (→ auto).
pub fn resolve_mode(cli: Option<GraphicsMode>, env: Option<&str>) -> GraphicsMode {
    cli.or_else(|| env.and_then(GraphicsMode::parse))
        .unwrap_or_default()
}

/// Final protocol: forced modes win; `Auto` uses detection, else half-blocks.
pub fn choose(mode: GraphicsMode, detected: Option<ProtocolType>) -> ProtocolType {
    match mode {
        GraphicsMode::Auto => detected.unwrap_or(ProtocolType::Halfblocks),
        GraphicsMode::Kitty => ProtocolType::Kitty,
        GraphicsMode::Iterm2 => ProtocolType::Iterm2,
        GraphicsMode::Sixel => ProtocolType::Sixel,
        GraphicsMode::Halfblocks => ProtocolType::Halfblocks,
    }
}

pub fn protocol_name(p: ProtocolType) -> &'static str {
    match p {
        ProtocolType::Halfblocks => "halfblocks",
        ProtocolType::Sixel => "sixel",
        ProtocolType::Kitty => "kitty",
        ProtocolType::Iterm2 => "iterm2",
    }
}

/// Fit a `fw`×`fh` frame into `area` (cells) preserving aspect, for a cell
/// of `font` = (width, height) pixels. Returns the centred cell rect.
pub fn fit_cells_font(area: Rect, fw: u32, fh: u32, font: (u16, u16)) -> Rect {
    let (cw, ch) = (font.0.max(1) as f32, font.1.max(1) as f32);
    if area.width == 0 || area.height == 0 || fw == 0 || fh == 0 {
        return Rect::new(area.x, area.y, 0, 0);
    }
    let (aw, ah) = (area.width as f32 * cw, area.height as f32 * ch);
    let s = (aw / fw as f32).min(ah / fh as f32);
    let w = ((fw as f32 * s / cw).round() as u16).clamp(1, area.width);
    let h = ((fh as f32 * s / ch).round() as u16).clamp(1, area.height);
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

/// Stateful image renderer. `proto == Halfblocks` means "use the half-block painter".
pub struct Gfx {
    pub proto: ProtocolType,
    pub font: (u16, u16),
    picker: Option<Picker>,
    thread: Option<ThreadProtocol>,
    rx: Option<Receiver<ResizeResponse>>,
    shown_seq: Option<u64>,
    drawn: Option<Rect>,
    drawn_now: Option<Rect>,
    /// A full terminal clear is needed (stale Kitty/Sixel pixels).
    pub clear_pending: bool,
    kitty: Option<KittyState>,
}

impl Default for Gfx {
    fn default() -> Self {
        Self::halfblocks()
    }
}

impl Gfx {
    pub fn halfblocks() -> Self {
        Self {
            proto: ProtocolType::Halfblocks,
            font: (1, 2),
            picker: None,
            thread: None,
            rx: None,
            shown_seq: None,
            drawn: None,
            drawn_now: None,
            clear_pending: false,
            kitty: None,
        }
    }

    /// Protocol guess from well-known terminal env vars (query replies can be missed).
    pub fn env_protocol() -> Option<ProtocolType> {
        let var = |k: &str| std::env::var(k).unwrap_or_default().to_lowercase();
        let (term, prog) = (var("TERM"), var("TERM_PROGRAM"));
        if term.contains("kitty")
            || term.contains("ghostty")
            || prog.contains("ghostty")
            || prog.contains("wezterm")
            || std::env::var_os("KITTY_WINDOW_ID").is_some()
            || std::env::var_os("WEZTERM_EXECUTABLE").is_some()
        {
            Some(ProtocolType::Kitty)
        } else if prog.contains("iterm") || std::env::var_os("ITERM_SESSION_ID").is_some() {
            Some(ProtocolType::Iterm2)
        } else if term.contains("foot") || term.contains("mlterm") {
            Some(ProtocolType::Sixel)
        } else {
            None
        }
    }

    /// Detect (query the terminal) and start the worker. Call before entering the alt screen.
    pub fn detect(mode: GraphicsMode) -> Self {
        if mode == GraphicsMode::Halfblocks {
            return Self::halfblocks();
        }
        let queried = Picker::from_query_stdio().ok();
        let detected = Self::env_protocol().or_else(|| queried.as_ref().map(|p| p.protocol_type()));
        let mut picker = queried.unwrap_or_else(Picker::halfblocks);
        let proto = choose(mode, detected);
        if proto == ProtocolType::Halfblocks {
            return Self::halfblocks();
        }
        // Prefer the real pixel size of a cell (window pixels / cells): a wrong cell size makes
        // the terminal upscale a too-small image, which looks pixelated on HiDPI screens.
        if let Ok(ws) = crossterm::terminal::window_size() {
            if ws.width > 0 && ws.height > 0 && ws.columns > 0 && ws.rows > 0 {
                let fw = (ws.width / ws.columns).max(1);
                let fh = (ws.height / ws.rows).max(1);
                #[allow(deprecated)]
                {
                    picker = Picker::from_fontsize((fw, fh).into());
                }
            }
        }
        picker.set_protocol_type(proto);
        if proto == ProtocolType::Kitty {
            let fs = picker.font_size();
            return Self {
                proto,
                font: (fs.width.max(1), fs.height.max(1)),
                picker: Some(picker),
                kitty: Some(KittyState::spawn()),
                ..Self::halfblocks()
            };
        }
        Self::with_picker(picker)
    }

    fn with_picker(picker: Picker) -> Self {
        let fs = picker.font_size();
        let (req_tx, req_rx) = channel::<ratatui_image::thread::ResizeRequest>();
        let (res_tx, res_rx) = channel();
        std::thread::spawn(move || {
            while let Ok(req) = req_rx.recv() {
                if let Ok(done) = req.resize_encode() {
                    if res_tx.send(done).is_err() {
                        break;
                    }
                }
            }
        });
        Self {
            proto: picker.protocol_type(),
            font: (fs.width.max(1), fs.height.max(1)),
            thread: Some(ThreadProtocol::new(req_tx, None)),
            picker: Some(picker),
            rx: Some(res_rx),
            ..Self::halfblocks()
        }
    }

    /// Real image protocol in use (not the half-block fallback).
    pub fn active(&self) -> bool {
        self.picker.is_some() && self.proto != ProtocolType::Halfblocks
    }

    pub fn name(&self) -> &'static str {
        protocol_name(self.proto)
    }

    /// Cell height / width ratio for coordinate mapping.
    pub fn cell_ratio(&self) -> f32 {
        if self.active() {
            self.font.1 as f32 / self.font.0 as f32
        } else {
            2.0
        }
    }

    /// Apply finished background encodes. Returns true when a redraw is needed.
    pub fn poll(&mut self) -> bool {
        if let Some(k) = &mut self.kitty {
            let mut any = false;
            while let Ok(done) = k.rx.try_recv() {
                if Some(done.rect) == self.drawn_now.or(self.drawn) {
                    k.ready = Some(done);
                    any = true;
                }
            }
            return any;
        }
        let (Some(rx), Some(t)) = (&self.rx, &mut self.thread) else {
            return false;
        };
        let mut any = false;
        while let Ok(done) = rx.try_recv() {
            t.update_resized_protocol(done);
            any = true;
        }
        any
    }

    /// Start of a UI frame.
    pub fn begin_frame(&mut self) {
        self.drawn_now = None;
    }

    /// End of a UI frame: flag a clear when the image vanished or moved.
    pub fn end_frame(&mut self) {
        if let Some(k) = &mut self.kitty {
            if self.drawn_now != self.drawn {
                k.ready = None;
                k.sent_seq = None;
                if k.placed {
                    k.delete_pending = true;
                }
            }
            self.drawn = self.drawn_now;
            // The image is placed by escape codes; cells are blank, so no full clear is needed.
            return;
        }
        if self.drawn.is_some() && self.drawn != self.drawn_now {
            self.clear_pending = true;
            self.shown_seq = None; // re-encode for the new place
            if let Some(t) = &mut self.thread {
                t.empty_protocol();
            }
        }
        self.drawn = self.drawn_now;
    }

    /// Draw the frame image into `rect`. Updates the protocol only when `seq` changed
    /// and no encode is in flight (throttle); encoding runs on the worker thread.
    pub fn render(&mut self, f: &mut Frame, rect: Rect, seq: u64, img: &RgbaImage) {
        if let Some(k) = &mut self.kitty {
            // Keep cells under the image blank so ratatui never rewrites them.
            let buf = f.buffer_mut();
            for y in rect.top()..rect.bottom() {
                for x in rect.left()..rect.right() {
                    buf[(x, y)].set_symbol(" ");
                }
            }
            if k.sent_seq != Some(seq) && k.submit(rect, seq, img, self.font) {
                k.sent_seq = Some(seq);
            }
            self.drawn_now = Some(rect);
            return;
        }
        let (Some(picker), Some(t)) = (&self.picker, &mut self.thread) else {
            return;
        };
        let in_flight = t
            .size_for(Resize::Scale(Some(FilterType::CatmullRom)), rect.as_size())
            .is_none()
            && self.shown_seq.is_some();
        if self.shown_seq != Some(seq) && !in_flight {
            t.replace_protocol(picker.new_resize_protocol(DynamicImage::ImageRgba8(img.clone())));
            self.shown_seq = Some(seq);
        }
        f.render_stateful_widget(
            StatefulImage::default().resize(Resize::Scale(Some(FilterType::CatmullRom))),
            rect,
            t,
        );
        if t.size_for(Resize::Scale(Some(FilterType::CatmullRom)), rect.as_size())
            .is_none()
        {
            // Encode in flight: keep the previous image on screen instead of blanking it.
            let buf = f.buffer_mut();
            for y in rect.top()..rect.bottom() {
                for x in rect.left()..rect.right() {
                    buf[(x, y)].set_diff_option(ratatui::buffer::CellDiffOption::Skip);
                }
            }
        }
        self.drawn_now = Some(rect);
    }

    /// Frame (`seq`) currently shown or queued.
    pub fn shown_seq(&self) -> Option<u64> {
        self.kitty.as_ref().map_or(self.shown_seq, |k| k.sent_seq)
    }

    /// Write pending Kitty output (image update / delete) to `out`. Call after each draw.
    pub fn flush_kitty<W: Write>(&mut self, out: &mut W) -> std::io::Result<()> {
        let Some(k) = &mut self.kitty else {
            return Ok(());
        };
        let mut buf = Vec::new();
        if std::mem::take(&mut k.delete_pending) {
            buf.extend_from_slice(kitty_delete().as_bytes());
            k.placed = false;
        }
        if let Some(done) = k.ready.take() {
            buf.extend_from_slice(&kitty_place(done.rect, &done.png));
            k.placed = true;
            KITTY_USED.store(true, Ordering::SeqCst);
        }
        if buf.is_empty() {
            return Ok(());
        }
        let mut all = Vec::with_capacity(buf.len() + 16);
        all.extend_from_slice(b"\x1b[?2026h");
        all.extend_from_slice(&buf);
        all.extend_from_slice(b"\x1b[?2026l");
        out.write_all(&all)?;
        out.flush()
    }

    /// Remove the Kitty image (app exit).
    pub fn shutdown_kitty<W: Write>(&mut self, out: &mut W) {
        if self.kitty.as_ref().is_some_and(|k| k.placed) {
            let _ = out.write_all(kitty_delete().as_bytes());
            let _ = out.flush();
            if let Some(k) = &mut self.kitty {
                k.placed = false;
            }
        }
    }
}

/// Fixed Kitty image id / placement id used for the mirror.
pub const KITTY_ID: u32 = 7001;
/// Set once a Kitty image was placed; lets panic/exit paths emit a delete.
pub static KITTY_USED: AtomicBool = AtomicBool::new(false);

/// Delete the mirror image (and its placement).
pub fn kitty_delete() -> String {
    format!("\x1b_Ga=d,d=I,i={KITTY_ID},q=2\x1b\\")
}

/// Pixel size of a cell rect for `font` = (w, h).
pub fn rect_px(rect: Rect, font: (u16, u16)) -> (u32, u32) {
    (
        rect.width as u32 * font.0.max(1) as u32,
        rect.height as u32 * font.1.max(1) as u32,
    )
}

pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16
            | (*c.get(1).unwrap_or(&0) as u32) << 8
            | *c.get(2).unwrap_or(&0) as u32;
        o.push(T[(n >> 18) as usize & 63] as char);
        o.push(T[(n >> 12) as usize & 63] as char);
        o.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        o.push(if c.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    o
}

/// Kitty APC chunks (each ≤4096 base64 bytes) transmitting+placing a PNG into `rect`.
pub fn kitty_chunks(png: &[u8], cols: u16, rows: u16) -> Vec<String> {
    let b64 = base64(png);
    let n = b64.len().div_ceil(4096).max(1);
    (0..n)
        .map(|i| {
            let part = &b64[i * 4096..((i + 1) * 4096).min(b64.len())];
            let m = u8::from(i + 1 < n);
            if i == 0 {
                format!(
                    "\x1b_Ga=T,f=100,i={KITTY_ID},p=1,c={cols},r={rows},C=1,q=2,m={m};{part}\x1b\\"
                )
            } else {
                format!("\x1b_Gm={m};{part}\x1b\\")
            }
        })
        .collect()
}

/// Full output for placing the image: save cursor, move, chunks, restore cursor.
pub fn kitty_place(rect: Rect, png: &[u8]) -> Vec<u8> {
    let mut s = format!("\x1b7\x1b[{};{}H", rect.y + 1, rect.x + 1);
    for c in kitty_chunks(png, rect.width, rect.height) {
        s.push_str(&c);
    }
    s.push_str("\x1b8");
    s.into_bytes()
}

/// Scale `img` to `w`×`h` and PNG-encode it (fast compression).
pub fn encode_png(img: &RgbaImage, w: u32, h: u32) -> Option<Vec<u8>> {
    use image::codecs::png::{CompressionType, FilterType as PF, PngEncoder};
    use image::ImageEncoder;
    let rgb = DynamicImage::ImageRgba8(img.clone()).to_rgb8();
    let scaled = image::imageops::resize(&rgb, w.max(1), h.max(1), FilterType::Triangle);
    let mut out = Vec::new();
    PngEncoder::new_with_quality(&mut out, CompressionType::Fast, PF::Sub)
        .write_image(
            scaled.as_raw(),
            scaled.width(),
            scaled.height(),
            image::ExtendedColorType::Rgb8,
        )
        .ok()?;
    Some(out)
}

struct KittyJob {
    rect: Rect,
    img: RgbaImage,
    px: (u32, u32),
}

struct KittyDone {
    rect: Rect,
    png: Vec<u8>,
}

struct KittyState {
    tx: SyncSender<KittyJob>,
    rx: Receiver<KittyDone>,
    busy: Arc<AtomicBool>,
    ready: Option<KittyDone>,
    sent_seq: Option<u64>,
    placed: bool,
    delete_pending: bool,
}

impl KittyState {
    fn spawn() -> Self {
        let (tx, job_rx) = sync_channel::<KittyJob>(1);
        let (res_tx, rx) = channel();
        let busy = Arc::new(AtomicBool::new(false));
        let b = busy.clone();
        std::thread::spawn(move || {
            while let Ok(job) = job_rx.recv() {
                if let Some(png) = encode_png(&job.img, job.px.0, job.px.1) {
                    if res_tx
                        .send(KittyDone {
                            rect: job.rect,
                            png,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                b.store(false, Ordering::SeqCst);
            }
        });
        Self {
            tx,
            rx,
            busy,
            ready: None,
            sent_seq: None,
            placed: false,
            delete_pending: false,
        }
    }

    /// Queue an encode unless the worker is busy (frame dropped).
    fn submit(&mut self, rect: Rect, _seq: u64, img: &RgbaImage, font: (u16, u16)) -> bool {
        if self.busy.load(Ordering::SeqCst) {
            return false;
        }
        self.busy.store(true, Ordering::SeqCst);
        let job = KittyJob {
            rect,
            img: img.clone(),
            px: rect_px(rect, font),
        };
        if self.tx.try_send(job).is_err() {
            self.busy.store(false, Ordering::SeqCst);
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_known() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn chunking_and_sequences() {
        let data = vec![7u8; 9_000];
        let ch = kitty_chunks(&data, 30, 12);
        assert_eq!(ch.len(), 3);
        assert!(ch[0].starts_with("\x1b_Ga=T,f=100,i=7001,p=1,c=30,r=12,C=1,q=2,m=1;"));
        assert!(ch[1].starts_with("\x1b_Gm=1;") && ch[2].starts_with("\x1b_Gm=0;"));
        assert!(ch.iter().all(|c| c.ends_with("\x1b\\")));
        let small = kitty_chunks(b"x", 1, 1);
        assert_eq!(small.len(), 1);
        assert!(small[0].contains("m=0;"));
        let p = String::from_utf8(kitty_place(Rect::new(4, 2, 3, 3), b"x")).unwrap();
        assert!(p.starts_with("\x1b7\x1b[3;5H") && p.ends_with("\x1b8"));
        assert_eq!(kitty_delete(), "\x1b_Ga=d,d=I,i=7001,q=2\x1b\\");
    }

    #[test]
    fn rect_to_px_and_png() {
        assert_eq!(rect_px(Rect::new(0, 0, 5, 3), (20, 40)), (100, 120));
        let img = RgbaImage::from_pixel(10, 20, image::Rgba([1, 2, 3, 255]));
        let png = encode_png(&img, 40, 80).unwrap();
        assert_eq!(&png[1..4], b"PNG");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 40);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 80);
    }

    #[test]
    fn parse_modes() {
        assert_eq!(GraphicsMode::parse("Kitty"), Some(GraphicsMode::Kitty));
        assert_eq!(GraphicsMode::parse("iterm2"), Some(GraphicsMode::Iterm2));
        assert_eq!(GraphicsMode::parse("sixel"), Some(GraphicsMode::Sixel));
        assert_eq!(
            GraphicsMode::parse("halfblocks"),
            Some(GraphicsMode::Halfblocks)
        );
        assert_eq!(GraphicsMode::parse("auto"), Some(GraphicsMode::Auto));
        assert_eq!(GraphicsMode::parse("nope"), None);
    }

    #[test]
    fn cli_beats_env() {
        assert_eq!(
            resolve_mode(Some(GraphicsMode::Sixel), Some("kitty")),
            GraphicsMode::Sixel
        );
        assert_eq!(resolve_mode(None, Some("kitty")), GraphicsMode::Kitty);
        assert_eq!(resolve_mode(None, Some("garbage")), GraphicsMode::Auto);
        assert_eq!(resolve_mode(None, None), GraphicsMode::Auto);
    }

    #[test]
    fn choose_and_fallback() {
        assert_eq!(choose(GraphicsMode::Auto, None), ProtocolType::Halfblocks);
        assert_eq!(
            choose(GraphicsMode::Auto, Some(ProtocolType::Kitty)),
            ProtocolType::Kitty
        );
        assert_eq!(
            choose(GraphicsMode::Halfblocks, Some(ProtocolType::Kitty)),
            ProtocolType::Halfblocks
        );
        assert_eq!(choose(GraphicsMode::Sixel, None), ProtocolType::Sixel);
    }

    #[test]
    fn fit_preserves_aspect_with_font() {
        // 8x16 px cells, 80x24 area = 640x384 px; portrait 1080x2400 → height-bound.
        let r = fit_cells_font(Rect::new(2, 3, 80, 24), 1080, 2400, (8, 16));
        assert_eq!(r.height, 24);
        let (pw, ph) = (r.width as f32 * 8.0, r.height as f32 * 16.0);
        assert!((pw / ph - 1080.0 / 2400.0).abs() < 0.03, "{pw}x{ph}");
        assert_eq!(r.x, 2 + (80 - r.width) / 2);
        // landscape, width-bound
        let r = fit_cells_font(Rect::new(0, 0, 40, 40), 1600, 800, (10, 20));
        assert_eq!(r.width, 40);
        assert_eq!(r.height, 10);
        assert_eq!(
            fit_cells_font(Rect::new(0, 0, 0, 5), 10, 10, (8, 16)).width,
            0
        );
    }

    #[test]
    fn halfblocks_inactive() {
        let g = Gfx::halfblocks();
        assert!(!g.active());
        assert_eq!(g.cell_ratio(), 2.0);
        assert_eq!(g.name(), "halfblocks");
    }
}
