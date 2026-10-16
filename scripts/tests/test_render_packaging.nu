#!/usr/bin/env nu
# Tests for scripts/ci/render_packaging.nu and package_tarball.nu
#
# Run with: nu scripts/tests/test_render_packaging.nu

use std/assert
use ./runner.nu *
use ../ci/render_packaging.nu [render, sha, outputs]

def "test render replaces every placeholder" [] {
    let out = (render "v@VERSION@ @SHA@ @VERSION@" { VERSION: "1.2.3", SHA: "abc" })
    assert equal $out "v1.2.3 abc 1.2.3"
}

def "test sha is SKIP for missing tarball" [] {
    assert equal (sha "/nonexistent" "1.0.0" "x86_64-unknown-linux-gnu") "SKIP"
}

def "test sha hashes an existing tarball" [] {
    let d = (mktemp -d)
    "hello" | save -f $"($d)/droidkraft-tui-1.0.0-t.tar.gz"
    let h = (sha $d "1.0.0" "t")
    rm -rf $d
    assert equal $h ("hello" | hash sha256)
}

def "test templates only use known placeholders" [] {
    let known = [VERSION SHA_X86_64 SHA_AARCH64 SHA_GUI_X86_64 SHA_MACOS_ARM SHA_MACOS_X86 SHA_WIN_ZIP]
    for f in (outputs | get tpl) {
        let used = (open --raw $f | parse --regex '@(?P<k>[A-Z0-9_]+)@' | get k | uniq)
        for k in $used { assert ($k in $known) $"($f): unknown placeholder @($k)@" }
    }
}

def "test tarball has the binary at the archive root" [] {
    let d = (mktemp -d)
    let here = $env.PWD
    mkdir $"($d)/target/release"
    "bin" | save -f $"($d)/target/release/droidkraft-tui"
    cp LICENSE README.md $d
    cd $d
    nu ($here | path join scripts/ci/package_tarball.nu) 9.9.9 x86_64-unknown-linux-gnu
    let listing = (^tar -tzf dist/droidkraft-tui-9.9.9-x86_64-unknown-linux-gnu.tar.gz | lines | str trim --left --char '.' | str trim --left --char '/')
    cd $here
    rm -rf $d
    assert ($listing | any {|l| $l == "droidkraft-tui" })
}

def main [] {
    print $"(ansi cyan)═══ test_render_packaging.nu ═══(ansi reset)"
    run-tests
}
