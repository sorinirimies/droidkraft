#!/usr/bin/env nu
# usage: nu scripts/ci/package_macos.nu <version> <arch-label>  -> dist/DroidKraft-<v>-macos-<arch>.dmg
def main [version: string, arch: string] {
    mkdir dist
    let app = "dist/DroidKraft.app"
    rm -rf $app
    mkdir $"($app)/Contents/MacOS" $"($app)/Contents/Resources"
    cp target/release/droidkraft $"($app)/Contents/MacOS/DroidKraft"
    cp target/release/droidkraft-tui $"($app)/Contents/Resources/"
    cp assets/icons/droidkraft.icns $"($app)/Contents/Resources/"
    $"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\"><dict>
<key>CFBundleName</key><string>DroidKraft</string>
<key>CFBundleDisplayName</key><string>DroidKraft</string>
<key>CFBundleIdentifier</key><string>io.github.sorinirimies.droidkraft</string>
<key>CFBundleExecutable</key><string>DroidKraft</string>
<key>CFBundleIconFile</key><string>droidkraft</string>
<key>CFBundleShortVersionString</key><string>($version)</string>
<key>CFBundleVersion</key><string>($version)</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
" | save -f $"($app)/Contents/Info.plist"
    # ad-hoc sign: avoids the "damaged" dialog on Apple silicon
    do --ignore-errors { ^codesign --force --deep -s - $app }
    let stage = (mktemp -d)
    cp -r $app $stage
    ^ln -s /Applications $"($stage)/Applications"
    ^hdiutil create -volname DroidKraft -srcfolder $stage -ov -format UDZO $"dist/DroidKraft-($version)-macos-($arch).dmg"
}
