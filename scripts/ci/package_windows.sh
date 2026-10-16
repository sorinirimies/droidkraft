#!/usr/bin/env bash
# usage: package_windows.sh <version>  (run in git-bash) -> portable zip + NSIS installer
set -euo pipefail
V="$1"; mkdir -p dist
S="dist/DroidKraft-$V-windows-x86_64"; rm -rf "$S"; mkdir -p "$S"
cp target/release/droidkraft.exe target/release/droidkraft-tui.exe LICENSE README.md "$S/"
cp assets/icons/droidkraft.ico "$S/"
(cd dist && 7z a -tzip "DroidKraft-$V-windows-x86_64.zip" "DroidKraft-$V-windows-x86_64" >/dev/null)
if command -v makensis >/dev/null; then
  makensis -DVERSION="$V" -DSRC="$PWD/$S" -DOUT="$PWD/dist/DroidKraft-$V-windows-x86_64-setup.exe" packaging/windows/installer.nsi
fi
