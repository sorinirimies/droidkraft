#!/usr/bin/env nu
# ── DroidKraft · test_ci_scripts.nu ───────────────────────────────────
# The decisions the CI scripts make, without network, git or cargo.

use std/assert
use runner.nu *
use ../ci/deps_update.nu [present commit_message lock_versions downgrades pins pins_changed next_patch upgrade_message]
use ../ci/publish_crate.nu [already_published package_args]
use ../ci/apt.nu [commands]
use ../ci/check_static.nu [is_static]
use ../ci/prune_releases.nu [from_gh from_gitea doomed]
use ../ci/gitea_release.nu [release_payload]
use ../ci/actions_update.nu [uses major_of bump PINNED]

def "test deps_update: commit message is a dated chore deps" [] {
    assert equal (commit_message "2026-01-02") "chore(deps): nightly cargo update 2026-01-02"
}

def "test deps_update: only existing lockfiles are touched" [] {
    let root = ($env.CURRENT_FILE | path dirname | path join ".." "..")
    cd $root
    assert equal (present ["Cargo.lock" "no/such/Cargo.lock"]) ["Cargo.lock"]
}

def "test deps_update: a raised requirement is a release, formatting is not" [] {
    let before = "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\na = \"1.0\"\nb = { version = \"0.3\", optional = true }\n" | from toml
    let reordered = "[package]\nversion = \"0.1.0\"\nname = \"x\"\n[dependencies]\nb = { optional = true, version = \"0.3\" }\na = \"1.0\"\n" | from toml
    assert not (pins_changed $before $reordered)
    assert (pins_changed $before ($before | upsert dependencies.a "1.0.5"))
    assert (pins_changed $before ($before | upsert dependencies.b.version "0.3.2"))
}

def "test deps_update: dev and target requirements count" [] {
    let base = {package: {name: x}, dev-dependencies: {c: "2"}, target: {"cfg(unix)": {dependencies: {d: "0.1"}}}}
    assert equal (pins $base) {"dev-dependencies c": "2", "cfg(unix) dependencies d": "0.1"}
    assert (pins_changed $base ($base | upsert target."cfg(unix)".dependencies.d "0.2"))
    assert not (pins_changed $base $base)
}

def "test deps_update: the next patch version" [] {
    assert equal (next_patch "0.4.7") "0.4.8"
    assert equal (next_patch "1.2.9") "1.2.10"
    assert equal (next_patch "1.0.0-rc.1") "1.0.0"
    assert equal (upgrade_message "2026-01-02") "chore(deps): upgrade dependencies 2026-01-02"
}

def "test deps_update: workspace requirements count" [] {
    let ws = {workspace: {dependencies: {a: "1", b: {version: "0.2"}}}}
    assert equal (pins $ws) {"workspace a": "1", "workspace b": "0.2"}
    assert (pins_changed $ws ($ws | upsert workspace.dependencies.b.version "0.3"))
}

def "test publish_crate: a re-release counts as success" [] {
    assert (already_published "error: crate version `0.1.0` is already uploaded")
    assert (already_published "error: crate kv@0.1.0 already exists on crates.io index")
    assert not (already_published "error: failed to verify package tarball")
}

def "test apt: nothing to install runs nothing" [] {
    assert equal (commands [] false) []
}

def "test apt: update then install, with sudo unless root" [] {
    let c = commands [libxkbcommon-dev] false
    assert equal ($c | length) 2
    assert equal ($c.0 | first 2) [sudo apt-get]
    assert ($c.0 | any {|a| $a == update })
    assert equal ($c.1 | last) libxkbcommon-dev
    assert equal ((commands [x] true).0.0) apt-get
}

def "test publish_crate: one -p per crate, in order" [] {
    assert equal (package_args [core app]) [-p core -p app]
    assert equal (package_args []) []
}

def "test check_static: both ldd wordings are static" [] {
    assert (is_static "\tnot a dynamic executable")
    assert (is_static "\tstatically linked")
    assert not (is_static "\tlibc.so.6 => /lib/x86_64-linux-gnu/libc.so.6")
}

const GITEA_RELEASES = [
    { id: 3, tag_name: "v1.2.0", assets: [{ id: 30, name: "app-x86_64.tar.gz" }] }
    { id: 2, tag_name: "v1.1.0", assets: [{ id: 20, name: "app-x86_64.tar.gz" } { id: 21, name: "app.deb" }] }
    { id: 1, tag_name: "v1.0.0", assets: [] }
]

def "test prune_releases: gitea deletes every asset but the kept release" [] {
    let d = doomed (from_gitea $GITEA_RELEASES) "v1.2.0"
    assert equal ($d | select release id) [{ release: 2, id: 20 } { release: 2, id: 21 }]
}

def "test prune_releases: github assets are addressed by tag and name" [] {
    let gh = [{ tagName: "v2", assets: [{ name: "a.zip" }] } { tagName: "v1", assets: [{ name: "a.zip" }] }]
    assert equal (doomed (from_gh $gh) "v2" | select release name) [{ release: "v1", name: "a.zip" }]
}

def "test prune_releases: refuses an unknown kept tag" [] {
    assert error { doomed (from_gitea $GITEA_RELEASES) "v9.9.9" }
}

def "test prune_releases: refuses when the kept release has no assets" [] {
    assert error { doomed (from_gitea $GITEA_RELEASES) "v1.0.0" }
}

def "test gitea_release: payload carries tag, title and notes" [] {
    assert equal (release_payload "v1.2.3" "KV" "notes") { tag_name: "v1.2.3", name: "KV v1.2.3", body: "notes" }
}

def "test actions_update: finds every action and major" [] {
    let text = "steps:\n  - uses: actions/checkout@v4\n  - uses: Swatinem/rust-cache@v2\n  - uses: hustcer/setup-nu@v3\n"
    assert equal (uses $text | get repo) ["actions/checkout" "Swatinem/rust-cache" "hustcer/setup-nu"]
    assert equal (uses $text | get major) [4 2 3]
}

def "test actions_update: major of a release tag" [] {
    assert equal (major_of "v5.2.0") 5
    assert equal (major_of "v12") 12
    assert equal (major_of "release-1") null
}

def "test actions_update: bumps older majors only" [] {
    let text = "- uses: Swatinem/rust-cache@v2\n- uses: hustcer/setup-nu@v3\n- uses: foo/bar@v9\n"
    let out = bump $text { "Swatinem/rust-cache": 3, "hustcer/setup-nu": 3, "foo/bar": 1 }
    assert ($out | str contains "Swatinem/rust-cache@v3")
    assert ($out | str contains "hustcer/setup-nu@v3")
    assert ($out | str contains "foo/bar@v9")
}

def "test actions_update: pinned actions never move" [] {
    let text = "- uses: actions/checkout@v4\n"
    assert equal (bump $text { "actions/checkout": 6 }) $text
    assert ("actions/upload-artifact" in $PINNED)
}

def "test actions_update: v1 does not touch v10" [] {
    let out = bump "- uses: a/b@v1\n- uses: a/b@v10\n" { "a/b": 2 }
    assert equal $out "- uses: a/b@v2\n- uses: a/b@v10\n"
}

const LOCK = '[[package]]
name = "syn"
version = "1.0.109"

[[package]]
name = "syn"
version = "2.0.100"

[[package]]
name = "wasi"
version = "0.11.0+wasi-snapshot-preview1"
'

def "test deps_update: lock_versions keeps the highest version and drops build metadata" [] {
    assert equal (lock_versions $LOCK) {syn: "2.0.100", wasi: "0.11.0"}
}

def "test deps_update: downgrades lists lowered packages only" [] {
    let before = {syn: "2.0.100", serde: "1.0.200", gone: "1.0.0"}
    let after = {syn: "2.0.101", serde: "1.0.199", new: "0.1.0"}
    assert equal (downgrades $before $after) ["serde 1.0.200 -> 1.0.199"]
}

def "test deps_update: a crate kept at two versions is no downgrade" [] {
    let lock = lock_versions $LOCK
    assert equal (downgrades $lock $lock) []
}

def main [] { run-tests }
