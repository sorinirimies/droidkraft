#!/usr/bin/env bash
set -euo pipefail
# Usage: packaging/macos/create_dmg.sh <version>
# Builds a universal (arm64 + x86_64) DroidKraft.app + DMG from release builds
# in target/{aarch64,x86_64}-apple-darwin/release. Requires macOS (lipo, hdiutil).

VERSION="${1#v}"
DIST="dist"
STAGE="${DIST}/dmg"
APP="${STAGE}/DroidKraft.app"
rm -rf "$DIST"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

for BIN in droidkraft droidkraft-tui; do
  lipo -create \
    "target/aarch64-apple-darwin/release/${BIN}" \
    "target/x86_64-apple-darwin/release/${BIN}" \
    -output "${DIST}/${BIN}"
done

cp "${DIST}/droidkraft" "$APP/Contents/MacOS/droidkraft"
cp assets/icons/droidkraft.icns "$APP/Contents/Resources/droidkraft.icns"
chmod +x "$APP/Contents/MacOS/droidkraft"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>droidkraft</string>
  <key>CFBundleIdentifier</key><string>com.sorinirimies.droidkraft</string>
  <key>CFBundleName</key><string>DroidKraft</string>
  <key>CFBundleDisplayName</key><string>DroidKraft</string>
  <key>CFBundleIconFile</key><string>droidkraft</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF

# Ad-hoc sign so Apple Silicon will run the unsigned bundle.
codesign --force --deep --sign - "$APP"

cp "${DIST}/droidkraft-tui" "$STAGE/droidkraft-tui"
ln -s /Applications "$STAGE/Applications"

hdiutil create -volname "DroidKraft ${VERSION}" -srcfolder "$STAGE" \
  -ov -format UDZO "${DIST}/droidkraft-v${VERSION}-macos-universal.dmg"

# Plain universal CLI/GUI tarball too.
tar -C "$DIST" -czf "${DIST}/droidkraft-v${VERSION}-macos-universal.tar.gz" droidkraft droidkraft-tui
ls -lh "$DIST"
