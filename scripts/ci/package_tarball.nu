#!/usr/bin/env nu
# Prebuilt archives consumed by cargo-binstall, Homebrew and the AUR -bin package.
# usage: nu scripts/ci/package_tarball.nu <version> <target-triple> [--gui]
#   -> dist/droidkraft-tui-<v>-<target>.tar.gz   (.zip on windows), binary at archive root
#      dist/droidkraft-<v>-<target>.tar.gz       with --gui
def main [version: string, target: string, --gui] {
    mkdir dist
    let windows = ($target | str contains "windows")
    let ext = if $windows { ".exe" } else { "" }
    let bins = if $gui { [["droidkraft" "droidkraft"]] } else { [["droidkraft-tui" "droidkraft-tui"]] }
    for b in $bins {
        let name = $b.0
        let stage = $"dist/($name)-($version)-($target)"
        rm -rf $stage
        mkdir $stage
        let triple_dir = $"target/($target)/release"
        let dir = if ($triple_dir | path exists) { $triple_dir } else { "target/release" }
        cp $"($dir)/($name)($ext)" LICENSE README.md $stage
        let out = $"($name)-($version)-($target)"
        cd $stage
        if $windows {
            ^zip -qr ("../" + $out + ".zip") .
        } else {
            ^tar -czf $"../($out).tar.gz" .
        }
        cd ../..
        rm -rf $stage
    }
}
