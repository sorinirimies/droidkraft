#!/usr/bin/env nu
# ── DroidKraft · test_release_notes.nu ────────────────────────────────

use std/assert
use runner.nu *
use ../ci/release_notes.nu [notes]
use ../project.nu [PUBLISH bin_crates]

def "test notes: heading install and changes" [] {
    let n = (notes "0.2.0" "### ✨ Features\n\n- A thing")
    assert ($n | str starts-with "# DroidKraft 0.2.0")
    for c in (bin_crates | where {|c| $c in $PUBLISH }) {
        assert ($n | str contains $"cargo install ($c)")
    }
    assert ($n | str contains "## Changes")
    assert ($n | str contains "- A thing")
}

def main [] { run-tests }
