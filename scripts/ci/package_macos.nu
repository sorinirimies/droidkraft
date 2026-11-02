#!/usr/bin/env nu
# Universal (arm64 + x86_64) DroidKraft.app + DMG + tarball from the two
# `cargo build --release --target <arch>-apple-darwin` outputs. macOS only (lipo, hdiutil).
# usage: nu scripts/ci/package_macos.nu <version> [--build | --from-artifacts <dir>]
#   --build              cargo build --release for both targets first
#   --from-artifacts DIR copy per-target binaries named <slug>-gui-<target> / <slug>-tui-<target>
#   -> dist/droidkraft-v<v>-macos-universal.{dmg,tar.gz}

const APP = "DroidKraft"
const GUI = "droidkraft"
const TUI = "droidkraft-tui"
const SLUG = "droidkraft"
const BUNDLE_ID = "io.github.sorinirimies.droidkraft"
const PKGS = ["droidkraft" "droidkraft-tui"]  # cargo packages (GUI, TUI/CLI)
const TARGETS = ["aarch64-apple-darwin" "x86_64-apple-darwin"]
const ICNS = "assets/icons/droidkraft.icns"

# Info.plist for the .app bundle. `icon` is the icon file stem, or "" for none.
export def plist [name: string, exe: string, id: string, version: string, icon: string] {
    let icon_key = if $icon == "" { "" } else { $"<key>CFBundleIconFile</key><string>($icon)</string>" }
    $"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\"><dict>
<key>CFBundleName</key><string>($name)</string>
<key>CFBundleDisplayName</key><string>($name)</string>
<key>CFBundleIdentifier</key><string>($id)</string>
<key>CFBundleExecutable</key><string>($exe)</string>
($icon_key)
<key>CFBundleShortVersionString</key><string>($version)</string>
<key>CFBundleVersion</key><string>($version)</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
"
}

# Output file stem, e.g. droidkraft-v1.2.3-macos-universal
export def stem [slug: string, version: string] {
    $"($slug)-v($version | str replace --regex '^v' '')-macos-universal"
}

def main [version: string, --build, --from-artifacts: string = ""] {
    if $build {
        for t in $TARGETS { ^cargo build --release --target $t ...($PKGS | each {|p| ["-p" $p]} | flatten) }
    }
    if $from_artifacts != "" {
        for t in $TARGETS {
            mkdir $"target/($t)/release"
            cp $"($from_artifacts)/($SLUG)-gui-($t)" $"target/($t)/release/($GUI)"
            cp $"($from_artifacts)/($SLUG)-tui-($t)" $"target/($t)/release/($TUI)"
        }
    }
    let v = ($version | str replace --regex '^v' '')
    let dist = "dist"
    rm -rf $dist
    mkdir $dist
    for bin in [$GUI $TUI] {
        ^lipo -create $"target/aarch64-apple-darwin/release/($bin)" $"target/x86_64-apple-darwin/release/($bin)" -output $"($dist)/($bin)"
    }
    let stage = $"($dist)/dmg"
    let app = $"($stage)/($APP).app"
    mkdir $"($app)/Contents/MacOS" $"($app)/Contents/Resources"
    cp $"($dist)/($GUI)" $"($app)/Contents/MacOS/($GUI)"
    let has_icon = ($ICNS | path exists)
    if $has_icon { cp $ICNS $"($app)/Contents/Resources/($GUI).icns" }
    plist $APP $GUI $BUNDLE_ID $v (if $has_icon { $GUI } else { "" }) | save -f $"($app)/Contents/Info.plist"
    # ad-hoc sign: avoids the "damaged" dialog on Apple silicon
    ^codesign --force --deep -s - $app
    cp $"($dist)/($TUI)" $"($stage)/($TUI)"
    ^ln -s /Applications $"($stage)/Applications"
    let s = (stem $SLUG $v)
    ^hdiutil create -volname $"($APP) ($v)" -srcfolder $stage -ov -format UDZO $"($dist)/($s).dmg"
    ^tar -C $dist -czf $"($dist)/($s).tar.gz" $GUI $TUI
}
