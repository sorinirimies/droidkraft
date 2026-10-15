//! Half-block (`▀`) terminal rendering of RGBA frames + input coordinate mapping.
//!
//! Each terminal cell shows two vertically stacked square-ish pixels: the upper
//! one as the foreground colour of `▀`, the lower one as the background.

use droidkraft_mirror::{fit_rect, map_to_device};
use image::RgbaImage;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

/// A downscaled frame, `w` × `2*cells_h` pixels, row-major RGB.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scaled {
    pub w: u16,
    pub cells_h: u16,
    pub px: Vec<[u8; 3]>,
}

/// Fit a `fw`×`fh` frame into `area` (cells), keeping aspect and compensating
/// for the 1:2 cell aspect. Returns the centred cell rectangle.
pub fn fit_cells(area: Rect, fw: u32, fh: u32) -> Rect {
    if area.width == 0 || area.height == 0 || fw == 0 || fh == 0 {
        return Rect::new(area.x, area.y, 0, 0);
    }
    // Pixel grid is `width` × `2*height` with ~square pixels.
    let (x, y, w, h) = fit_rect(area.width as f32, area.height as f32 * 2.0, fw, fh);
    let cw = (w.round() as u16).clamp(1, area.width);
    let ch = ((h / 2.0).round() as u16).clamp(1, area.height);
    let cx = area.x + ((area.width - cw) / 2).min((x.max(0.0)) as u16 + area.width);
    let cy = area.y + (area.height - ch) / 2;
    let _ = y;
    Rect::new(cx, cy, cw, ch)
}

/// Fast box-ish downscale: each destination pixel averages up to 4×4 samples
/// taken on a regular grid inside its source footprint.
pub fn downscale(img: &RgbaImage, dw: u32, dh: u32) -> Vec<[u8; 3]> {
    let (sw, sh) = img.dimensions();
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return Vec::new();
    }
    let raw = img.as_raw();
    let mut out = Vec::with_capacity((dw * dh) as usize);
    for dy in 0..dh {
        let y0 = (dy as u64 * sh as u64 / dh as u64) as u32;
        let y1 = (((dy as u64 + 1) * sh as u64 / dh as u64) as u32).clamp(y0 + 1, sh);
        for dx in 0..dw {
            let x0 = (dx as u64 * sw as u64 / dw as u64) as u32;
            let x1 = (((dx as u64 + 1) * sw as u64 / dw as u64) as u32).clamp(x0 + 1, sw);
            let (nx, ny) = ((x1 - x0).min(4), (y1 - y0).min(4));
            let (mut r, mut g, mut b, mut n) = (0u32, 0u32, 0u32, 0u32);
            for iy in 0..ny {
                let sy = y0 + iy * (y1 - y0) / ny;
                for ix in 0..nx {
                    let sx = x0 + ix * (x1 - x0) / nx;
                    let i = ((sy * sw + sx) * 4) as usize;
                    r += raw[i] as u32;
                    g += raw[i + 1] as u32;
                    b += raw[i + 2] as u32;
                    n += 1;
                }
            }
            out.push([(r / n) as u8, (g / n) as u8, (b / n) as u8]);
        }
    }
    out
}

/// Cache so we only rescale when the frame (`seq`) or target size changes.
#[derive(Debug, Default)]
pub struct ScaleCache {
    key: Option<(u64, u16, u16)>,
    pub scaled: Scaled,
    /// How many times a rescale actually ran (for tests / diagnostics).
    pub rescales: u64,
}

impl ScaleCache {
    pub fn get(&mut self, seq: u64, img: &RgbaImage, cells_w: u16, cells_h: u16) -> &Scaled {
        let key = (seq, cells_w, cells_h);
        if self.key != Some(key) {
            self.scaled = Scaled {
                w: cells_w,
                cells_h,
                px: downscale(img, cells_w as u32, cells_h as u32 * 2),
            };
            self.key = Some(key);
            self.rescales += 1;
        }
        &self.scaled
    }

    pub fn clear(&mut self) {
        self.key = None;
        self.scaled = Scaled::default();
    }
}

/// Paint a scaled frame into `buf` with its top-left cell at `at`.
pub fn paint(buf: &mut Buffer, at: Rect, s: &Scaled) {
    let (w, h) = (s.w.min(at.width), s.cells_h.min(at.height));
    for cy in 0..h {
        for cx in 0..w {
            let top = s.px.get((cy as usize * 2) * s.w as usize + cx as usize);
            let bot = s.px.get((cy as usize * 2 + 1) * s.w as usize + cx as usize);
            if let (Some(t), Some(b)) = (top, bot) {
                if let Some(cell) = buf.cell_mut((at.x + cx, at.y + cy)) {
                    cell.set_char('▀')
                        .set_fg(Color::Rgb(t[0], t[1], t[2]))
                        .set_bg(Color::Rgb(b[0], b[1], b[2]));
                }
            }
        }
    }
}

/// Map a terminal cell (`col`,`row`) to device pixels given the cell rect the
/// frame occupies. `None` when outside the frame.
pub fn cell_to_device(
    col: u16,
    row: u16,
    frame_rect: Rect,
    frame: (u32, u32),
    device: Option<(u32, u32)>,
) -> Option<(u32, u32)> {
    cell_to_device_aspect(col, row, frame_rect, frame, device, 2.0)
}

/// [`cell_to_device`] for a cell whose height is `ratio` × its width
/// (`2.0` for half-blocks; `font.h / font.w` for real image protocols).
pub fn cell_to_device_aspect(
    col: u16,
    row: u16,
    frame_rect: Rect,
    frame: (u32, u32),
    device: Option<(u32, u32)>,
    ratio: f32,
) -> Option<(u32, u32)> {
    if frame_rect.width == 0
        || frame_rect.height == 0
        || col < frame_rect.x
        || row < frame_rect.y
        || col >= frame_rect.x + frame_rect.width
        || row >= frame_rect.y + frame_rect.height
    {
        return None;
    }
    // Centre of the cell in the (w × 2h) half-block pixel grid.
    let rel = (
        (col - frame_rect.x) as f32 + 0.5,
        ((row - frame_rect.y) as f32 + 0.5) * ratio,
    );
    let view = (frame_rect.width as f32, frame_rect.height as f32 * ratio);
    map_to_device(rel, (0.0, 0.0, view.0, view.1), frame, device)
}

/// Like [`cell_to_device`] but clamps points outside the frame to its edge
/// (used while dragging so a gesture can end cleanly).
pub fn cell_to_device_clamped(
    col: u16,
    row: u16,
    frame_rect: Rect,
    frame: (u32, u32),
    device: Option<(u32, u32)>,
) -> Option<(u32, u32)> {
    cell_to_device_clamped_aspect(col, row, frame_rect, frame, device, 2.0)
}

/// Clamped variant of [`cell_to_device_aspect`].
pub fn cell_to_device_clamped_aspect(
    col: u16,
    row: u16,
    frame_rect: Rect,
    frame: (u32, u32),
    device: Option<(u32, u32)>,
    ratio: f32,
) -> Option<(u32, u32)> {
    if frame_rect.width == 0 || frame_rect.height == 0 {
        return None;
    }
    let c = col.clamp(frame_rect.x, frame_rect.x + frame_rect.width - 1);
    let r = row.clamp(frame_rect.y, frame_rect.y + frame_rect.height - 1);
    cell_to_device_aspect(c, r, frame_rect, frame, device, ratio)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid(w: u32, h: u32, c: [u8; 4]) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba(c))
    }

    #[test]
    fn downscale_solid_keeps_colour() {
        let px = downscale(&solid(100, 200, [10, 20, 30, 255]), 10, 20);
        assert_eq!(px.len(), 200);
        assert!(px.iter().all(|p| *p == [10, 20, 30]));
    }

    #[test]
    fn downscale_averages_halves() {
        let mut img = solid(4, 2, [0, 0, 0, 255]);
        for y in 0..2 {
            for x in 2..4 {
                img.put_pixel(x, y, Rgba([200, 100, 50, 255]));
            }
        }
        let px = downscale(&img, 2, 1);
        assert_eq!(px, vec![[0, 0, 0], [200, 100, 50]]);
    }

    #[test]
    fn downscale_upscale_nearest_and_empty() {
        let px = downscale(&solid(2, 2, [9, 9, 9, 255]), 4, 4);
        assert_eq!(px.len(), 16);
        assert!(downscale(&solid(2, 2, [0; 4]), 0, 4).is_empty());
    }

    #[test]
    fn fit_cells_letterboxes_portrait() {
        // 1080x2400 phone into 80x24 cells (80x48 px grid): height-limited.
        let r = fit_cells(Rect::new(0, 0, 80, 24), 1080, 2400);
        assert_eq!(r.height, 24);
        assert!(r.width < 80 && r.width >= 20);
        assert!(r.x > 0);
        // aspect roughly preserved
        let ratio = r.width as f32 / (r.height as f32 * 2.0);
        assert!((ratio - 0.45).abs() < 0.05, "{ratio}");
    }

    #[test]
    fn fit_cells_degenerate() {
        assert_eq!(fit_cells(Rect::new(0, 0, 0, 5), 10, 10).width, 0);
        assert_eq!(fit_cells(Rect::new(0, 0, 5, 5), 0, 10).height, 0);
    }

    #[test]
    fn cache_rescales_only_on_change() {
        let img = solid(10, 10, [1, 2, 3, 255]);
        let mut c = ScaleCache::default();
        c.get(1, &img, 5, 3);
        c.get(1, &img, 5, 3);
        assert_eq!(c.rescales, 1);
        c.get(2, &img, 5, 3);
        c.get(2, &img, 6, 3);
        assert_eq!(c.rescales, 3);
        c.clear();
        c.get(2, &img, 6, 3);
        assert_eq!(c.rescales, 4);
    }

    #[test]
    fn paint_writes_half_blocks() {
        let s = Scaled {
            w: 1,
            cells_h: 1,
            px: vec![[255, 0, 0], [0, 0, 255]],
        };
        let mut buf = Buffer::empty(Rect::new(0, 0, 2, 2));
        paint(&mut buf, Rect::new(1, 1, 1, 1), &s);
        let c = buf.cell((1, 1)).unwrap();
        assert_eq!(c.symbol(), "▀");
        assert_eq!(c.fg, Color::Rgb(255, 0, 0));
        assert_eq!(c.bg, Color::Rgb(0, 0, 255));
    }

    #[test]
    fn mapping_corners_and_outside() {
        let r = Rect::new(10, 5, 20, 20); // 20x40 px grid
        let frame = (1080, 2400);
        let dev = Some((1080, 2400));
        assert_eq!(cell_to_device(9, 5, r, frame, dev), None);
        assert_eq!(cell_to_device(30, 5, r, frame, dev), None);
        let (x, y) = cell_to_device(10, 5, r, frame, dev).unwrap();
        assert!(x < 60 && y <= 60);
        let (x, y) = cell_to_device(29, 24, r, frame, dev).unwrap();
        assert!(x > 1020 && y >= 2340);
        let (x, y) = cell_to_device(20, 15, r, frame, dev).unwrap();
        assert!((x as i32 - 570).abs() <= 60 && (y as i32 - 1200).abs() <= 60);
    }

    #[test]
    fn mapping_clamped_edge() {
        let r = Rect::new(10, 5, 20, 20);
        let p = cell_to_device_clamped(0, 0, r, (100, 200), None);
        assert!(p.is_some());
        let p = cell_to_device_clamped(200, 200, r, (100, 200), None).unwrap();
        assert!(p.0 >= 95 && p.1 >= 195);
    }

    #[test]
    fn mapping_rotated_device_size() {
        // Landscape frame, portrait-natural wm size → oriented to landscape.
        let r = Rect::new(0, 0, 40, 10);
        let p = cell_to_device(39, 9, r, (2400, 1080), Some((1080, 2400))).unwrap();
        assert!(p.0 > 2300 && p.1 > 1000);
    }
}
