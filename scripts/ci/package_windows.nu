#!/usr/bin/env nu
# usage: nu scripts/ci/package_windows.nu <version>  -> portable zip (+ NSIS installer when makensis exists)
def main [version: string] {
    mkdir dist
    let name = $"DroidKraft-($version)-windows-x86_64"
    let stage = $"dist/($name)"
    rm -rf $stage
    mkdir $stage
    cp target/release/droidkraft.exe target/release/droidkraft-tui.exe LICENSE README.md assets/icons/droidkraft.ico $stage
    cd dist
    ^7z a -tzip $"($name).zip" $name | ignore
    cd ..
    if (which makensis | is-not-empty) {
        let pwd = $env.PWD
        ^makensis $"-DVERSION=($version)" $"-DSRC=($pwd)/($stage)" $"-DOUT=($pwd)/dist/($name)-setup.exe" packaging/windows/installer.nsi
    }
}
