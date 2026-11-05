//! Sanity checks for the packaging files (icons, desktop entries, Flatpak, Snap and
//! store manifests) so a broken file fails `cargo test`, not the release.

use std::path::PathBuf;

const APP_ID: &str = "io.github.sorinirimies.DroidKraft";
const TUI_ID: &str = "io.github.sorinirimies.DroidKraftTui";

fn root(p: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(p)
}

fn read(p: &str) -> Vec<u8> {
    std::fs::read(root(p)).unwrap_or_else(|e| panic!("{p}: {e}"))
}

fn text(p: &str) -> String {
    String::from_utf8(read(p)).unwrap_or_else(|e| panic!("{p}: {e}"))
}

#[test]
fn png_icon_is_256_square() {
    let b = read("assets/icons/droidkraft-256.png");
    assert_eq!(&b[..8], b"\x89PNG\r\n\x1a\n");
    let w = u32::from_be_bytes(b[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(b[20..24].try_into().unwrap());
    assert_eq!((w, h), (256, 256));
}

#[test]
fn ico_has_directory_and_valid_offsets() {
    let b = read("packaging/windows/droidkraft.ico");
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
    let b = read("packaging/macos/droidkraft.icns");
    assert_eq!(&b[..4], b"icns");
    assert_eq!(
        u32::from_be_bytes(b[4..8].try_into().unwrap()) as usize,
        b.len()
    );
}

#[test]
fn desktop_entries_point_at_real_binaries_and_the_app_icon() {
    let gui = text(&format!("packaging/linux/{APP_ID}.desktop"));
    assert!(gui.contains("Exec=droidkraft\n"));
    assert!(gui.contains(&format!("StartupWMClass={APP_ID}\n")));
    let tui = text(&format!("packaging/linux/{TUI_ID}.desktop"));
    assert!(tui.contains("Exec=droidkraft-tui\n") && tui.contains("Terminal=true"));
    // package_desktop.nu installs the icon under the APP.icon name in scripts/project.nu.
    for entry in [gui, tui] {
        assert!(entry.contains(&format!("Icon={APP_ID}\n")));
    }
}

#[test]
fn flatpak_manifests_match_their_metainfo() {
    for (id, command) in [(APP_ID, "droidkraft"), (TUI_ID, "droidkraft-tui")] {
        let m = text(&format!("packaging/flatpak/{id}.yml"));
        assert!(m.contains(&format!("app-id: {id}")));
        assert!(m.contains(&format!("command: {command}\n")));
        assert!(m.contains("install -Dm644 droidkraft-256.png"));
        let info = text(&format!("packaging/linux/{id}.metainfo.xml"));
        assert!(info.contains(&format!("<id>{id}</id>")));
        // The .deb and .rpm install it as it is; package_flatpak.nu adds the release.
        assert!(
            !info.contains("<release"),
            "{id}: no <release> in the committed metainfo"
        );
    }
}

#[test]
fn binstall_urls_use_the_release_archive_names() {
    // scripts/ci/package.nu writes <bin>-<target>.tar.gz (zip on Windows).
    for c in [
        "crates/droidkraft-tui/Cargo.toml",
        "crates/droidkraft-gui/Cargo.toml",
    ] {
        let t = text(c);
        assert!(t.contains("[package.metadata.binstall]"), "{c}");
        assert!(
            t.contains("/releases/download/v{ version }/{ name }-{ target }{ archive-suffix }"),
            "{c}"
        );
    }
}

#[test]
fn snap_templates_are_consistent() {
    for (f, name) in [
        ("packaging/snap/snapcraft-gui.yaml.in", "name: droidkraft\n"),
        (
            "packaging/snap/snapcraft-tui.yaml.in",
            "name: droidkraft-tui\n",
        ),
    ] {
        let s = text(f);
        assert!(
            s.lines().any(|l| l == "base: core22"),
            "{f} needs a top-level base"
        );
        for needle in ["@VERSION@", "raw-usb", name] {
            assert!(s.contains(needle), "{f} missing {needle}");
        }
    }
}

#[test]
fn package_manifests_exist() {
    for p in [
        "packaging/aur/PKGBUILD.in",
        "packaging/aur/PKGBUILD-gui.in",
        "packaging/alpine/APKBUILD.in",
        "packaging/homebrew/droidkraft.rb.in",
        "packaging/homebrew/droidkraft-tui.rb.in",
        "packaging/windows/droidkraft-tui.scoop.json.in",
        "packaging/windows/winget.installer.yaml.in",
        "packaging/windows/droidkraft-tui.nuspec.in",
    ] {
        assert!(root(p).exists(), "{p} missing");
    }
}
