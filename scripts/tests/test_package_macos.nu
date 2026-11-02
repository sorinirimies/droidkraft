#!/usr/bin/env nu
# Tests for scripts/ci/package_macos.nu
use std/assert
use ./runner.nu *
use ../ci/package_macos.nu [plist, stem]

def "test plist embeds name, exe, id and version" [] {
    let p = (plist "App" "app" "com.x.app" "1.2.3" "app")
    assert ($p | str contains "<key>CFBundleName</key><string>App</string>")
    assert ($p | str contains "<key>CFBundleExecutable</key><string>app</string>")
    assert ($p | str contains "<string>com.x.app</string>")
    assert ($p | str contains "<key>CFBundleShortVersionString</key><string>1.2.3</string>")
}

def "test plist has icon key only when icon given" [] {
    assert ((plist "A" "a" "i" "1" "a") | str contains "CFBundleIconFile")
    assert (not ((plist "A" "a" "i" "1" "") | str contains "CFBundleIconFile"))
}

def "test stem strips a leading v" [] {
    assert equal (stem "app" "v1.2.3") "app-v1.2.3-macos-universal"
    assert equal (stem "app" "1.2.3") "app-v1.2.3-macos-universal"
}

def main [] { run-tests }
