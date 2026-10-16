#!/usr/bin/env nu
# Stage a snapcraft project from the prebuilt binary (build with snapcore/action-build).
# usage: nu scripts/ci/package_snap.nu <version> [--tui]
#   -> snapbuild-gui/ or snapbuild-tui/  {snap/snapcraft.yaml, stage/*}
def main [version: string, --tui] {
    let kind = if $tui { "tui" } else { "gui" }
    let bin = if $tui { "droidkraft-tui" } else { "droidkraft" }
    let dir = $"snapbuild-($kind)"
    rm -rf $dir
    mkdir $"($dir)/snap" $"($dir)/stage"
    cp $"target/release/($bin)" assets/icons/droidkraft.png $"($dir)/stage/"
    if not $tui {
        open --raw packaging/linux/droidkraft.desktop
        | str replace --regex '(?m)^Icon=.*' 'Icon=${SNAP}/usr/share/icons/hicolor/256x256/apps/droidkraft.png'
        | save -f $"($dir)/stage/droidkraft.desktop"
    }
    open --raw $"packaging/snap/snapcraft-($kind).yaml.in" | str replace --all "@VERSION@" $version | save -f $"($dir)/snap/snapcraft.yaml"
}
