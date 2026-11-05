#!/usr/bin/env nu
# Flatpak bundle from the prebuilt release binary (build it first: scripts/ci/build_release.nu).
# usage: nu scripts/ci/package_flatpak.nu <version> [--tui] [--target x86_64-unknown-linux-gnu]
#   GUI -> dist/droidkraft-<version>.flatpak      TUI -> dist/droidkraft-tui-<version>.flatpak

# App id and binary of the GUI or the TUI bundle.
export def bundle [tui: bool]: nothing -> record<id: string, bin: string> {
    if $tui {
        { id: "io.github.sorinirimies.DroidKraftTui", bin: "droidkraft-tui" }
    } else {
        { id: "io.github.sorinirimies.DroidKraft", bin: "droidkraft" }
    }
}

# The metainfo with this release added: the committed files carry none, since the
# .deb and .rpm install them as they are.
export def with_release [xml: string, version: string, date: string]: nothing -> string {
    $xml | str replace "</component>" $"  <releases><release version=\"($version)\" date=\"($date)\"/></releases>\n</component>"
}

def main [version: string, --tui, --target: string = "x86_64-unknown-linux-gnu"] {
    let b = bundle $tui
    mkdir dist
    let dist = ($env.PWD | path join dist)
    rm -rf fp
    mkdir fp/stage
    cp $"target/($target)/release/($b.bin)" assets/icons/droidkraft-256.png fp/stage/
    open --raw $"packaging/linux/($b.id).desktop"
    | str replace --regex '(?m)^Icon=.*' $"Icon=($b.id)"
    | save -f $"fp/stage/($b.id).desktop"
    with_release (open --raw $"packaging/linux/($b.id).metainfo.xml") $version (date now | format date "%Y-%m-%d")
    | save -f $"fp/stage/($b.id).metainfo.xml"
    cp $"packaging/flatpak/($b.id).yml" fp/
    cd fp
    ^flatpak-builder --force-clean --user --disable-rofiles-fuse --repo=repo build $"($b.id).yml"
    ^flatpak build-bundle repo $"($dist)/($b.bin)-($version).flatpak" $b.id
}
