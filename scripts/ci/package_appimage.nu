#!/usr/bin/env nu
# usage: nu scripts/ci/package_appimage.nu <version>
# Expects target/release/{droidkraft,droidkraft-tui}; writes dist/<app>-<v>-<arch>.AppImage
def main [version: string] {
    let arch = (uname | get machine)
    mkdir dist
    for app in [droidkraft droidkraft-tui] {
        let d = $"dist/AppDir-($app)"
        rm -rf $d
        mkdir $"($d)/usr/bin" $"($d)/usr/share/applications" $"($d)/usr/share/icons/hicolor/256x256/apps"
        cp $"target/release/($app)" $"($d)/usr/bin/"
        let desktop = (open --raw $"packaging/linux/($app).desktop" | str replace --regex '(?m)^Icon=.*' 'Icon=droidkraft')
        $desktop | save -f $"($d)/($app).desktop"
        $desktop | save -f $"($d)/usr/share/applications/($app).desktop"
        cp assets/icons/droidkraft.png $"($d)/droidkraft.png"
        cp assets/icons/droidkraft.png $"($d)/usr/share/icons/hicolor/256x256/apps/"
        $"#!/bin/sh\nexec \"$\(dirname \"$\(readlink -f \"$0\"\)\"\)/usr/bin/($app)\" \"$@\"\n" | save -f $"($d)/AppRun"
        chmod +x $"($d)/AppRun"
        with-env { ARCH: $arch, APPIMAGE_EXTRACT_AND_RUN: "1" } {
            ^appimagetool $d $"dist/($app)-($version)-($arch).AppImage"
        }
    }
}
