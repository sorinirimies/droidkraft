#!/usr/bin/env nu
# ── DroidKraft · test_project.nu ──────────────────────────────────────
# scripts/project.nu must match the workspace: a stale list breaks a release.

use std/assert
use runner.nu *
use ../project.nu [BINS PUBLISH]

def packages [] {
    let root = ($env.CURRENT_FILE | path dirname | path join ".." "..")
    do { cd $root; ^cargo metadata --no-deps --offline --format-version 1 } | from json | get packages
}

def "test project: every binary is a bin target of its crate" [] {
    let pkgs = packages
    for b in $BINS {
        let p = $pkgs | where name == $b.crate
        assert equal ($p | length) 1 $"($b.crate) is a workspace package"
        let bins = $p.0.targets | where {|t| "bin" in $t.kind } | get name
        assert ($b.bin in $bins) $"($b.crate) builds the binary ($b.bin)"
    }
}

def "test project: every published crate is publishable and listed after its path dependencies" [] {
    let pkgs = packages
    for i in 0..<($PUBLISH | length) {
        let c = $PUBLISH | get $i
        let p = $pkgs | where name == $c
        assert equal ($p | length) 1 $"($c) is a workspace package"
        assert ($p.0.publish != []) $"($c) is not publish = false"
        let local = $p.0.dependencies | where {|d| $d.path? != null and $d.kind? != "dev" } | get name
        for d in $local {
            let at = $PUBLISH | enumerate | where item == $d | get -o index.0
            assert ($at != null and $at < $i) $"($d) is published before ($c)"
        }
    }
}

def "test project: the readme of every published crate exists" [] {
    let pkgs = packages
    for c in $PUBLISH {
        let p = $pkgs | where name == $c | first
        if $p.readme? != null {
            let path = $p.manifest_path | path dirname | path join $p.readme
            assert ($path | path exists) $"($c): readme ($p.readme) exists, or cargo publish fails"
        }
    }
}

def "test project: each gui window app_id is its desktop entry id" [] {
    let root = $env.CURRENT_FILE | path dirname | path join ".." ".."
    use ../project.nu [APP]
    use ../ci/package_desktop.nu [entry-id]
    for g in ($BINS | where {|b| ($b.kind? | default "") == "gui" }) {
        let id = entry-id $APP $g
        let src = glob ($root | path join "crates" "**" "src" "**" "*.rs") | each {|f| open --raw $f } | str join "\n"
        let ids = $src | parse --regex `pub const APP_ID: &str = "(?<id>[^"]+)"` | get id
        assert ($ids | any {|i| $i == $id }) $"($g.bin): set APP_ID = \"($id)\" as the Wayland app_id \(found ($ids)\)"
    }
}

def main [] { run-tests }
