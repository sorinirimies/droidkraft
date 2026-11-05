#!/usr/bin/env nu
# Tests for scripts/ci/package_flatpak.nu and package_snap.nu
#
# Run with: nu scripts/tests/test_bundles.nu

use std/assert
use ./runner.nu *
use ../ci/package_flatpak.nu [bundle with_release]
use ../ci/package_snap.nu [snapcraft snap_desktop]

const ROOT = path self ../..

def "test flatpak bundles have a manifest desktop entry and metainfo each" [] {
    for b in [(bundle false) (bundle true)] {
        for f in [$"packaging/flatpak/($b.id).yml" $"packaging/linux/($b.id).desktop" $"packaging/linux/($b.id).metainfo.xml"] {
            assert ($ROOT | path join $f | path exists) $"missing ($f)"
        }
    }
}

def "test with_release adds one release before the end of the component" [] {
    let xml = "<component>\n  <id>x</id>\n</component>\n"
    let out = with_release $xml "1.2.3" "2026-01-02"
    assert equal $out "<component>\n  <id>x</id>\n  <releases><release version=\"1.2.3\" date=\"2026-01-02\"/></releases>\n</component>\n"
}

def "test committed metainfo has no release line" [] {
    for f in (glob ($ROOT | path join packaging/linux/*.metainfo.xml)) {
        assert not (open --raw $f | str contains "<release") $"($f) has a <release> line; package_flatpak.nu adds it"
    }
}

def "test snapcraft fills every version placeholder" [] {
    for k in [gui tui] {
        let y = snapcraft (open --raw ($ROOT | path join $"packaging/snap/snapcraft-($k).yaml.in")) "1.2.3"
        assert not ($y | str contains "@VERSION@")
        assert ($y | str contains "version: \"1.2.3\"")
    }
}

def "test snap desktop entry points its icon into the snap" [] {
    let e = snap_desktop "[Desktop Entry]\nIcon=io.github.sorinirimies.DroidKraft\nExec=droidkraft\n"
    assert equal $e "[Desktop Entry]\nIcon=${SNAP}/usr/share/icons/hicolor/256x256/apps/droidkraft.png\nExec=droidkraft\n"
}

def main [] { run-tests }
