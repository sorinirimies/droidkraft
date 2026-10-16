#!/usr/bin/env nu
# Tests for scripts/ci/prune_releases.nu and syntax of the packaging scripts.
#
# Run with: nu scripts/tests/test_prune_releases.nu

use std/assert
use ./runner.nu *
use ../ci/prune_releases.nu [doomed]

def releases [] {
    [
        { tagName: "v1.0.0", assets: [{ name: "a.dmg" }, { name: "b.zip" }] }
        { tagName: "v1.1.0", assets: [{ name: "c.dmg" }] }
        { tagName: "v1.2.0", assets: [{ name: "d.dmg" }] }
    ]
}

def "test prune keeps the latest release" [] {
    let d = (doomed (releases) "v1.2.0")
    assert equal ($d | length) 3
    assert (($d | where tag == "v1.2.0" | is-empty))
}

def "test prune lists tag and asset names" [] {
    let d = (doomed (releases) "v1.2.0")
    assert ($d | any {|x| $x.tag == "v1.0.0" and $x.asset == "b.zip" })
}

def "test prune refuses unknown keep tag" [] {
    let r = (try { doomed (releases) "v9.9.9"; "no error" } catch {|e| $e.msg })
    assert str contains $r "refusing to prune"
}

def "test prune handles release without assets" [] {
    let d = (doomed [{ tagName: "v1", assets: [] } { tagName: "v2" }] "v2")
    assert equal ($d | length) 0
}

def "test packaging scripts parse" [] {
    for f in (glob ($env.FILE_PWD | path join ".." "ci" "*.nu")) {
        assert (nu-check $f) $"($f) has syntax errors"
    }
}

def main [] {
    print $"(ansi cyan)═══ test_prune_releases.nu ═══(ansi reset)"
    run-tests
}
