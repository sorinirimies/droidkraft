#!/usr/bin/env bash
# usage: package_appimage.sh <version>   (expects target/release/{droidkraft,droidkraft-tui})
set -euo pipefail
V="$1"; ARCH=$(uname -m); mkdir -p dist
for APP in droidkraft droidkraft-tui; do
  D="dist/AppDir-$APP"; rm -rf "$D"
  mkdir -p "$D/usr/bin" "$D/usr/share/applications" "$D/usr/share/icons/hicolor/256x256/apps"
  cp "target/release/$APP" "$D/usr/bin/"
  sed 's|^Icon=.*|Icon=droidkraft|' "packaging/linux/$APP.desktop" > "$D/$APP.desktop"
  cp "$D/$APP.desktop" "$D/usr/share/applications/"
  cp assets/icons/droidkraft.png "$D/droidkraft.png"
  cp assets/icons/droidkraft.png "$D/usr/share/icons/hicolor/256x256/apps/"
  printf '#!/bin/sh\nexec "$(dirname "$(readlink -f "$0")")/usr/bin/%s" "$@"\n' "$APP" > "$D/AppRun"
  chmod +x "$D/AppRun"
  ARCH="$ARCH" APPIMAGE_EXTRACT_AND_RUN=1 appimagetool "$D" "dist/$APP-$V-$ARCH.AppImage"
done
