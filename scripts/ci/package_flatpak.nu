#!/usr/bin/env nu
# usage: nu scripts/ci/package_flatpak.nu <version>  -> dist/droidkraft-<version>.flatpak
const APP_ID = "io.github.sorinirimies.DroidKraft"
def main [version: string] {
    mkdir dist
    let dist = ($env.PWD | path join dist)
    rm -rf fp
    mkdir fp/stage
    cp target/release/droidkraft target/release/droidkraft-tui assets/icons/droidkraft.png fp/stage/
    open --raw packaging/linux/droidkraft.desktop
    | str replace --regex '(?m)^Icon=.*' $"Icon=($APP_ID)"
    | save -f $"fp/stage/($APP_ID).desktop"
    open --raw $"packaging/linux/($APP_ID).metainfo.xml"
    | str replace "VERSION" $version
    | str replace "DATE" (date now | format date "%Y-%m-%d")
    | save -f $"fp/stage/($APP_ID).metainfo.xml"
    cp $"packaging/flatpak/($APP_ID).yml" fp/
    cd fp
    ^flatpak-builder --force-clean --user --disable-rofiles-fuse --repo=repo build $"($APP_ID).yml"
    ^flatpak build-bundle repo $"($dist)/droidkraft-($version).flatpak" $APP_ID
}
