#!/usr/bin/env nu
# Stage a snapcraft project from the prebuilt release binary (build it first:
# scripts/ci/build_release.nu), then build it with snapcore/action-build.
# usage: nu scripts/ci/package_snap.nu <version> [--tui] [--target x86_64-unknown-linux-gnu]
#   -> snapbuild-gui/ or snapbuild-tui/  {snap/snapcraft.yaml, stage/*}

# The snapcraft.yaml for this version.
export def snapcraft [template: string, version: string]: nothing -> string {
    $template | str replace --all "@VERSION@" $version
}

# The GUI's desktop entry, its icon pointing into the snap. `$$` is a literal `$` in a
# regex replacement; a bare `${SNAP}` would be read as a capture group and vanish.
export def snap_desktop [entry: string]: nothing -> string {
    $entry | str replace --regex '(?m)^Icon=.*' 'Icon=$${SNAP}/usr/share/icons/hicolor/256x256/apps/droidkraft.png'
}

def main [version: string, --tui, --target: string = "x86_64-unknown-linux-gnu"] {
    let kind = if $tui { "tui" } else { "gui" }
    let bin = if $tui { "droidkraft-tui" } else { "droidkraft" }
    let dir = $"snapbuild-($kind)"
    rm -rf $dir
    mkdir $"($dir)/snap" $"($dir)/stage"
    cp $"target/($target)/release/($bin)" assets/icons/droidkraft-256.png $"($dir)/stage/"
    if not $tui {
        snap_desktop (open --raw packaging/linux/io.github.sorinirimies.DroidKraft.desktop) | save -f $"($dir)/stage/droidkraft.desktop"
    }
    snapcraft (open --raw $"packaging/snap/snapcraft-($kind).yaml.in") $version | save -f $"($dir)/snap/snapcraft.yaml"
}
