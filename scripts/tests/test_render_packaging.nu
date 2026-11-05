#!/usr/bin/env nu
# Tests for scripts/ci/render_packaging.nu
#
# Run with: nu scripts/tests/test_render_packaging.nu

use std/assert
use ./runner.nu *
use ../ci/render_packaging.nu [render, sha, outputs, MUSL]
use ../ci/package.nu [asset_name]
use ../project.nu [LINUX release_matrix bins_for]

def "test render replaces every placeholder" [] {
    let out = (render "v@VERSION@ @SHA@ @VERSION@" { VERSION: "1.2.3", SHA: "abc" })
    assert equal $out "v1.2.3 abc 1.2.3"
}

def "test sha is SKIP for a missing archive" [] {
    assert equal (sha "/nonexistent" "x86_64-unknown-linux-gnu") "SKIP"
}

def "test sha hashes the archive package nu writes" [] {
    let d = (mktemp -d)
    "hello" | save -f ($d | path join (asset_name "x86_64-unknown-linux-gnu" "droidkraft-tui"))
    let h = (sha $d "x86_64-unknown-linux-gnu")
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

# Every download URL in a rendered manifest names an archive the release uploads
# (scripts/ci/package.nu for the release matrix and MUSL).
def "test every manifest url names a released archive" [] {
    let targets = (release_matrix $LINUX | get target) ++ $MUSL
    let built = $targets | each {|t| bins_for $t | each {|b| asset_name $t $b.bin } } | flatten
    for o in (outputs) {
        let text = render (open --raw $o.tpl) { VERSION: "1.2.3" }
        let name = $text | parse --regex '(?m)^_name=(?P<n>\S+)' | get -o n.0 | default ""
        let urls = $text | str replace --all '$_name' $name
            | parse --regex 'releases/download/v[^/\s]+/(?P<f>[^\s"'')]+)' | get f
        let files = $urls | each {|u| if ($u | str contains '$CARCH') { [x86_64 aarch64] | each {|a| $u | str replace '$CARCH' $a } } else { [$u] } } | flatten
        for f in $files { assert ($f in $built) $"($o.tpl): ($f) is not a release archive" }
    }
}

def main [] { run-tests }
