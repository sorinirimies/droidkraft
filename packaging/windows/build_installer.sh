#!/usr/bin/env bash
set -euo pipefail
# Usage (Git Bash on a Windows runner, NSIS installed):
#   packaging/windows/build_installer.sh <version> <bindir> [gui_exe]
# Produces dist/droidkraft-v<version>-windows-x86_64-setup.exe
VERSION="${1#v}"; BINDIR="$2"; GUI="${3:-}"
export MSYS2_ARG_CONV_EXCL="*"
mkdir -p dist
ROOT="$(pwd -W | tr '/' '\\')"
ARGS=(-DAPP_NAME="DroidKraft" -DSLUG="droidkraft" -DVERSION="$VERSION" -DROOT="$ROOT"
      -DBINDIR="${BINDIR//\//\\}" -DCLI_EXE="droidkraft-tui.exe")
[ -n "$GUI" ] && ARGS+=(-DGUI_EXE="$GUI")
'/c/Program Files (x86)/NSIS/makensis.exe' "${ARGS[@]}" packaging/windows/installer.nsi
ls -l dist/*-setup.exe
