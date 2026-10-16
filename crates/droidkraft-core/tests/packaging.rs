//! Sanity checks for release packaging assets (icons, desktop entries,
//! Flatpak manifest) so a broken file fails `cargo test`, not the release.

use std::path::PathBuf;

const APP_ID: &str = "io.github.sorinirimies.DroidKraft";

fn root(p: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(p)
}

fn read(p: &str) -> Vec<u8> {
    std::fs::read(root(p)).unwrap_or_else(|e| panic!("{p}: {e}"))
}

fn text(p: &str) -> String {
    String::from_utf8(read(p)).unwrap()
}

#[test]
fn png_icon_is_256_square() {
    let b = read("assets/icons/droidkraft.png");
    assert_eq!(&b[..8], b"\x89PNG\r\n\x1a\n");
    let w = u32::from_be_bytes(b[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(b[20..24].try_into().unwrap());
    assert_eq!((w, h), (256, 256));
}

#[test]
fn ico_has_directory_and_valid_offsets() {
    let b = read("assets/icons/droidkraft.ico");
    assert_eq!(&b[..4], [0, 0, 1, 0]);
    let n = u16::from_le_bytes([b[4], b[5]]) as usize;
    assert!(n >= 4);
    for i in 0..n {
        let e = &b[6 + 16 * i..6 + 16 * (i + 1)];
        let len = u32::from_le_bytes(e[8..12].try_into().unwrap()) as usize;
        let off = u32::from_le_bytes(e[12..16].try_into().unwrap()) as usize;
        assert!(off + len <= b.len(), "entry {i} out of bounds");
        assert_eq!(&b[off..off + 4], b"\x89PNG");
    }
}

#[test]
fn icns_length_header_matches_file() {
    let b = read("assets/icons/droidkraft.icns");
    assert_eq!(&b[..4], b"icns");
    assert_eq!(u32::from_be_bytes(b[4..8].try_into().unwrap()) as usize, b.len());
}

#[test]
fn desktop_entries_point_at_real_binaries() {
    let gui = text("packaging/linux/droidkraft.desktop");
    assert!(gui.contains("Exec=droidkraft\n"));
    assert!(gui.contains("StartupWMClass=droidkraft"));
    let tui = text("packaging/linux/droidkraft-tui.desktop");
    assert!(tui.contains("Exec=droidkraft-tui\n") && tui.contains("Terminal=true"));
}

#[test]
fn flatpak_manifest_matches_metainfo() {
    let m = text(&format!("packaging/flatpak/{APP_ID}.yml"));
    assert!(m.contains(&format!("app-id: {APP_ID}")));
    assert!(m.contains("command: droidkraft"));
    let info = text(&format!("packaging/linux/{APP_ID}.metainfo.xml"));
    assert!(info.contains(&format!("<id>{APP_ID}</id>")));
    assert!(info.contains("VERSION"), "placeholder replaced by package_flatpak.sh");
}

#[test]
fn release_workflow_builds_every_package_type() {
    let w = text(".github/workflows/packages.yml");
    for needle in ["AppImage", "flatpak", "windows-latest", "macos-14", "prune_releases.sh"] {
        assert!(w.contains(needle), "packages.yml missing {needle}");
    }
}
