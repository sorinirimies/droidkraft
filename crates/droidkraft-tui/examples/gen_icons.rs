//! Generates `assets/icons/droidkraft.{png,ico,icns}`.
//! Run from the repo root: `cargo run -p droidkraft-tui --example gen_icons`

use image::{imageops::FilterType, ImageEncoder, Rgba, RgbaImage};

const BG: Rgba<u8> = Rgba([17, 20, 28, 255]);
const GREEN: Rgba<u8> = Rgba([61, 220, 132, 255]);

/// Is (x, y) inside a rounded rect (all in 0..1 units)?
fn in_rrect(x: f32, y: f32, x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> bool {
    if x < x0 || x > x1 || y < y0 || y > y1 {
        return false;
    }
    let cx = x.max(x0 + r).min(x1 - r);
    let cy = y.max(y0 + r).min(y1 - r);
    (x - cx).powi(2) + (y - cy).powi(2) <= r * r
}

fn draw(n: u32) -> RgbaImage {
    let mut im = RgbaImage::new(n, n);
    let bars = [(0.36, 0.12), (0.435, 0.22), (0.51, 0.17), (0.585, 0.30)];
    for py in 0..n {
        for px in 0..n {
            let (x, y) = (px as f32 / n as f32, py as f32 / n as f32);
            let mut c = Rgba([0, 0, 0, 0]);
            if in_rrect(x, y, 0.0, 0.0, 1.0, 1.0, 0.2) {
                c = BG;
                let body = in_rrect(x, y, 0.30, 0.15, 0.70, 0.85, 0.07);
                let inner = in_rrect(x, y, 0.34, 0.19, 0.66, 0.81, 0.04);
                let speaker = in_rrect(x, y, 0.45, 0.20, 0.55, 0.22, 0.01);
                let bar = bars
                    .iter()
                    .any(|&(bx, h)| in_rrect(x, y, bx, 0.70 - h, bx + 0.05, 0.70, 0.012));
                if (body && !inner) || speaker || bar {
                    c = GREEN;
                }
            }
            im.put_pixel(px, py, c);
        }
    }
    im
}

fn png(im: &RgbaImage) -> Vec<u8> {
    let mut b = Vec::new();
    image::codecs::png::PngEncoder::new(&mut b)
        .write_image(im, im.width(), im.height(), image::ExtendedColorType::Rgba8)
        .expect("png encode");
    b
}

fn main() -> std::io::Result<()> {
    let big = draw(2048); // supersample, then downscale for smooth edges
    let at = |s: u32| image::imageops::resize(&big, s, s, FilterType::Lanczos3);
    std::fs::create_dir_all("assets/icons")?;
    std::fs::write("assets/icons/droidkraft.png", png(&at(256)))?;

    // .ico: PNG-compressed entries
    let sizes = [16u32, 32, 48, 64, 128, 256];
    let imgs: Vec<Vec<u8>> = sizes.iter().map(|&s| png(&at(s))).collect();
    let mut ico = vec![0, 0, 1, 0];
    ico.extend((sizes.len() as u16).to_le_bytes());
    let mut off = 6 + 16 * sizes.len() as u32;
    for (s, d) in sizes.iter().zip(&imgs) {
        let dim = if *s >= 256 { 0 } else { *s as u8 };
        ico.extend([dim, dim, 0, 0, 1, 0, 32, 0]);
        ico.extend((d.len() as u32).to_le_bytes());
        ico.extend(off.to_le_bytes());
        off += d.len() as u32;
    }
    imgs.iter().for_each(|d| ico.extend(d));
    std::fs::write("assets/icons/droidkraft.ico", ico)?;

    // .icns: PNG-compressed entries, big-endian lengths
    let mut body = Vec::new();
    for (tag, s) in [
        ("icp4", 16u32),
        ("icp5", 32),
        ("icp6", 64),
        ("ic07", 128),
        ("ic08", 256),
        ("ic09", 512),
        ("ic10", 1024),
    ] {
        let d = png(&at(s));
        body.extend(tag.as_bytes());
        body.extend((d.len() as u32 + 8).to_be_bytes());
        body.extend(d);
    }
    let mut icns = b"icns".to_vec();
    icns.extend((body.len() as u32 + 8).to_be_bytes());
    icns.extend(body);
    std::fs::write("assets/icons/droidkraft.icns", icns)?;
    println!("wrote assets/icons/droidkraft.{{png,ico,icns}}");
    Ok(())
}
