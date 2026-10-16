#!/usr/bin/env nu
# usage: nu scripts/ci/package_flatpak.nu <version> [--tui]
#   GUI -> dist/droidkraft-<version>.flatpak      TUI -> dist/droidkraft-tui-<version>.flatpak
export def main [version: string, --tui] {
    let id = if $tui { "io.github.sorinirimies.DroidKraftTui" } else { "io.github.sorinirimies.DroidKraft" }
    let bin = if $tui { "droidkraft-tui" } else { "droidkraft" }
    mkdir dist
    let dist = ($env.PWD | path join dist)
    rm -rf fp
    mkdir fp/stage
    cp $"target/release/($bin)" assets/icons/droidkraft.png fp/stage/
    open --raw $"packaging/linux/($bin).desktop"
    | str replace --regex '(?m)^Icon=.*' $"Icon=($id)"
    | save -f $"fp/stage/($id).desktop"
    open --raw $"packaging/linux/($id).metainfo.xml"
    | str replace "VERSION" $version
    | str replace "DATE" (date now | format date "%Y-%m-%d")
    | save -f $"fp/stage/($id).metainfo.xml"
    cp $"packaging/flatpak/($id).yml" fp/
    cd fp
    ^flatpak-builder --force-clean --user --disable-rofiles-fuse --repo=repo build $"($id).yml"
    ^flatpak build-bundle repo $"($dist)/($bin)-($version).flatpak" $id
}
