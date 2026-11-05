#!/usr/bin/env nu
# Tests for scripts/ci/distro_packages.nu
#
# Run with: nu scripts/tests/test_distro_packages.nu

use std/assert
use ./runner.nu *
use ../ci/distro_packages.nu [ARCH_PKGS ALPINE arch_args alpine_args apk_name]
use ../ci/render_packaging.nu [outputs]

def "test every arch package has a rendered pkgbuild" [] {
    let rendered = outputs | get out
    for p in $ARCH_PKGS { assert ($"PKGBUILD-($p)" in $rendered) $"render_packaging.nu does not write PKGBUILD-($p)" }
    assert ("APKBUILD" in $rendered)
}

def "test arch build mounts the work dir and runs makepkg as a user" [] {
    let a = arch_args "/w/x"
    assert equal ($a | take 4) [run --rm -v "/w/x:/pkg"]
    assert ($a | last | str contains "su b -c 'cd /pkg && makepkg")
}

def "test alpine build runs on the requested platform and copies that arch" [] {
    let a = alpine_args "/w/a" arm64 aarch64
    assert equal ($a | take 6) [run --rm --platform linux/arm64 -v "/w/a:/w"]
    assert ($a | last | str ends-with "cp /home/b/packages/w/aarch64/*.apk /w/")
}

def "test alpine builds both architectures" [] {
    assert equal ($ALPINE | get arch) [x86_64 aarch64]
}

def "test apk name gets the architecture" [] {
    assert equal (apk_name "droidkraft-tui-2.3.0-r0.apk" aarch64) "droidkraft-tui-2.3.0-r0-aarch64.apk"
}

def main [] { run-tests }
