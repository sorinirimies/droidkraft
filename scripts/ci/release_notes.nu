#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# DroidKraft — Release notes
# ──────────────────────────────────────────────────────────────────────────────
# Writes RELEASE_NOTES.md for a tag: install instructions plus this release's
# section of the changelog (git-cliff --latest; the workflow runs at the tag, so
# --unreleased would be empty).
#
# Usage:
#   nu scripts/ci/release_notes.nu v0.2.0
# ──────────────────────────────────────────────────────────────────────────────

use ../project.nu [PUBLISH bin_crates]

# `cargo install` lines for the published crates that ship a binary.
export def install_lines []: nothing -> list<string> {
    bin_crates | where {|c| $c in $PUBLISH } | each {|c| $"cargo install ($c)" }
}

# The notes text for `version` with the changelog section `changes`.
export def notes [version: string, changes: string]: nothing -> string {
    [
        $"# DroidKraft ($version)"
        ""
        "Download the archive for your platform below \(binary at the archive root\),"
        "or install from crates.io:"
        ""
        "```sh"
        ...(install_lines)
        "```"
        ""
        "## Changes"
        ""
        $changes
        ""
    ] | str join "\n"
}

def main [raw_tag: string] {
    let version = ($raw_tag | str replace --regex '^v' '')
    let changes = if (which git-cliff | is-not-empty) {
        let r = (do { ^git-cliff --latest --strip header } | complete)
        if $r.exit_code == 0 and ($r.stdout | str trim | is-not-empty) { $r.stdout | str trim } else { "- See the commit history." }
    } else {
        "- See the commit history."
    }
    notes $version $changes | save --force RELEASE_NOTES.md
    print $"(ansi green)✅ RELEASE_NOTES.md for ($version)(ansi reset)"
}
