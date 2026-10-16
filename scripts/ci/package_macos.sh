#!/usr/bin/env bash
# usage: package_macos.sh <version> <arch-label>  -> dist/DroidKraft-<v>-macos-<arch>.dmg
set -euo pipefail
V="$1"; A="$2"; mkdir -p dist
APP="dist/DroidKraft.app"; rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/droidkraft "$APP/Contents/MacOS/DroidKraft"
cp target/release/droidkraft-tui "$APP/Contents/Resources/"
cp assets/icons/droidkraft.icns "$APP/Contents/Resources/"
cat > "$APP/Contents/Info.plist" <<PL
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>DroidKraft</string>
<key>CFBundleDisplayName</key><string>DroidKraft</string>
<key>CFBundleIdentifier</key><string>io.github.sorinirimies.droidkraft</string>
<key>CFBundleExecutable</key><string>DroidKraft</string>
<key>CFBundleIconFile</key><string>droidkraft</string>
<key>CFBundleShortVersionString</key><string>$V</string>
<key>CFBundleVersion</key><string>$V</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PL
codesign --force --deep -s - "$APP" || true   # ad-hoc: avoids "damaged" on Apple silicon
STAGE=$(mktemp -d); cp -R "$APP" "$STAGE/"; ln -s /Applications "$STAGE/Applications"
hdiutil create -volname DroidKraft -srcfolder "$STAGE" -ov -format UDZO "dist/DroidKraft-$V-macos-$A.dmg"
