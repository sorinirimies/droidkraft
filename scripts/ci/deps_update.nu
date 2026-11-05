#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# DroidKraft — Nightly dependency update helpers (used by deps-update.yml)
# ──────────────────────────────────────────────────────────────────────────────
#   nu scripts/ci/deps_update.nu sync               reset to origin/main (scheduled runs start stale)
#   nu scripts/ci/deps_update.nu update             cargo update here and in fuzz/; fails on a downgrade
#   nu scripts/ci/deps_update.nu changed            prints changed=true|false (→ $GITHUB_OUTPUT)
#   nu scripts/ci/deps_update.nu commit <bot> <mail> commit the lockfiles, push to main
#   nu scripts/ci/deps_update.nu upgrade            cargo upgrade (compatible pins) + update
#   nu scripts/ci/deps_update.nu release <bot> <mail> commit, bump the patch, push, release
#
# With AUTO_RELEASE (scripts/project.nu) the GitHub nightly upgrades the
# requirements and ships a changed one as a patch release; a lockfile-only change
# is committed and nothing is released. A library's users only see its
# requirements, never its lockfile, so a library sets it.
# ──────────────────────────────────────────────────────────────────────────────

use ../bump_version.nu [compare_versions]
use ../version.nu [cargo_version]
use ../project.nu [AUTO_RELEASE]

export const LOCKFILES = ["Cargo.lock" "fuzz/Cargo.lock"]

# Each package's highest locked version (a lockfile can hold several), without
# build metadata (`+wasi-…`), which does not order versions.
export def lock_versions [lock: string]: nothing -> record {
    $lock | from toml | get package | group-by name | transpose name pkgs
    | reduce -f {} {|p, acc|
        let top = $p.pkgs.version | each {|v| $v | split row "+" | first }
            | reduce {|v, m| if (compare_versions $v $m) > 0 { $v } else { $m } }
        $acc | insert $p.name $top
    }
}

# Packages whose highest locked version went down: a stale registry cache or a
# yanked release can do that, and the build still passes.
export def downgrades [before: record, after: record]: nothing -> list<string> {
    $after | transpose name version
    | where {|p| ($before | get -o $p.name) != null and (compare_versions $p.version ($before | get $p.name)) < 0 }
    | each {|p| $"($p.name) ($before | get $p.name) -> ($p.version)" }
}

# The lockfiles that exist in this checkout (fuzz/ is optional).
export def present [files: list<string>] {
    $files | where {|f| $f | path exists }
}

export def commit_message [date: string] {
    $"chore\(deps\): nightly cargo update ($date)"
}

const KINDS = ["dependencies" "dev-dependencies" "build-dependencies"]

# Every dependency requirement of a parsed Cargo.toml, as `kind name` → version
# (`target.<cfg>.<kind>` and `workspace.dependencies` included). Formatting and
# key order don't matter.
export def pins [cargo: record]: nothing -> record {
    let tables = ($KINDS | each {|k| {prefix: $k, deps: ($cargo | get -o $k | default {})} })
        | append {prefix: "workspace", deps: ($cargo | get -o workspace.dependencies | default {})}
        | append ($cargo | get -o target | default {} | transpose cfg t | each {|row|
            $KINDS | each {|k| {prefix: $"($row.cfg) ($k)", deps: ($row.t | get -o $k | default {})} }
        } | flatten)
    $tables | each {|t|
        $t.deps | transpose name spec | each {|d|
            let v = if ($d.spec | describe) == "string" { $d.spec } else { $d.spec | get -o version | default "" }
            {key: $"($t.prefix) ($d.name)", value: $v}
        }
    } | flatten | reduce --fold {} {|it, acc| $acc | insert $it.key $it.value }
}

# Did a dependency requirement change between two parsed Cargo.toml files?
export def pins_changed [before: record, after: record]: nothing -> bool {
    (pins $before) != (pins $after)
}

# The next patch version: 0.4.7 → 0.4.8 (a pre-release ships as its release: 1.0.0-rc.1 → 1.0.0).
export def next_patch [version: string]: nothing -> string {
    let core = ($version | split row "-" | first | split row "." | each { into int })
    if ($version | str contains "-") { $core | str join "." } else { [$core.0 $core.1 ($core.2 + 1)] | str join "." }
}

export def upgrade_message [date: string] {
    $"chore\(deps\): upgrade dependencies ($date)"
}

def today [] { date now | date to-timezone UTC | format date "%Y-%m-%d" }

# A manifest as committed, parsed.
def committed [manifest: string] { ^git show $"HEAD:($manifest)" | from toml }

# The workspace's manifests, relative: the root and every member.
def manifests []: nothing -> list<string> {
    let m = ^cargo metadata --no-deps --format-version 1 | from json
    $m.packages.manifest_path | each {|p| $p | path relative-to $m.workspace_root } | prepend Cargo.toml | uniq
}

def "main sync" [] {
    ^git fetch origin main
    ^git checkout -B main origin/main
    ^git reset --hard origin/main
    ^git log -1 --oneline
}

# cargo update in `dir`, failing if it lowered any package's version.
def update_in [dir: string] {
    let lock = $dir | path join "Cargo.lock"
    let before = if ($lock | path exists) { lock_versions (open --raw $lock | decode utf-8) } else { {} }
    do { cd $dir; ^cargo update }
    let down = downgrades $before (lock_versions (open --raw $lock | decode utf-8))
    if ($down | is-not-empty) { error make {msg: $"cargo update downgraded: ($down | str join ', ')"} }
}

def "main update" [] {
    update_in "."
    if ("fuzz/Cargo.toml" | path exists) { update_in "fuzz" }
}

def "main changed" [] {
    let files = present $LOCKFILES
    let r = do { ^git diff --quiet -- ...$files } | complete
    let release = $AUTO_RELEASE and (manifests | any {|m| pins_changed (committed $m) (open $m) })
    let changed = $release or $r.exit_code != 0
    # stdout is $GITHUB_OUTPUT: key=value lines only; the human part goes to stderr.
    if $changed { print -e (^git diff --stat) }
    print $"changed=($changed)"
    print $"release=($release)"
}

def "main commit" [bot: string, email: string] {
    ^git config user.name $bot
    ^git config user.email $email
    ^git add ...(present $LOCKFILES)
    ^git commit -m (commit_message (date now | date to-timezone UTC | format date "%Y-%m-%d"))
    ^git push origin HEAD:main
}

# Raise the compatible requirements in the manifests (cargo-edit), then the lockfiles.
def "main upgrade" [] {
    ^cargo upgrade
    main update
}

# Commit the upgraded requirements, bump the patch version (changelog, tag) and push.
# The tag push starts release.yml only when it was made with a PAT (GH_PAT, so
# HAS_PAT=true); a GITHUB_TOKEN push starts no workflow, so then it is dispatched.
def "main release" [bot: string, email: string] {
    ^git config user.name $bot
    ^git config user.email $email
    ^git add ...(manifests) ...(present $LOCKFILES)
    ^git commit -m (upgrade_message (today))
    let next = next_patch (cargo_version (open Cargo.toml))
    ^nu scripts/bump_version.nu $next --yes --skip-checks
    ^git push --follow-tags origin HEAD:main
    if ($env.HAS_PAT? | default "false") != "true" {
        ^gh workflow run release.yml --ref main -f $"tag=v($next)"
    }
    print $"released=v($next)"
}

def main [] { print "usage: nu scripts/ci/deps_update.nu (sync | update | upgrade | changed | commit <bot> <email> | release <bot> <email>)" }
