#!/usr/bin/env nu
# NSIS installer for the cross-compiled Windows TUI (needs `makensis`, apt package `nsis`).
# usage: nu scripts/ci/package_windows.nu <version>  -> dist/droidkraft-tui-<version>-x86_64-setup.exe
def main [version: string] {
    mkdir dist
    let pwd = $env.PWD
    let stage = $"($pwd)/dist/nsis-stage"
    rm -rf $stage
    mkdir $stage
    cp target/x86_64-pc-windows-msvc/release/droidkraft-tui.exe LICENSE $stage
    ^makensis $"-DVERSION=($version)" $"-DSRC=($stage)" $"-DOUT=($pwd)/dist/droidkraft-tui-($version)-x86_64-setup.exe" packaging/windows/installer.nsi
    rm -rf $stage
}
