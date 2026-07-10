#!/usr/bin/env nu
# Pre-publish readiness check for droidkraft
# Usage: nu scripts/check_publish.nu
# Run this before 'cargo publish' to catch problems early.

def main [] {
    let green  = (ansi green)
    let red    = (ansi red)
    let cyan   = (ansi cyan)
    let reset  = (ansi reset)

    print "Checking droidkraft for publish readiness..."
    print ""

    mut errors = 0

    # ── 1. Formatting ─────────────────────────────────────────────────────────
    print -n "Checking code formatting... "
    let fmt = (do { cargo fmt -- --check } | complete)
    if $fmt.exit_code == 0 {
        print $"($green)✓($reset)"
    } else {
        print $"($red)✗  (run: cargo fmt)($reset)"
        $errors = $errors + 1
    }

    # ── 2. Clippy ─────────────────────────────────────────────────────────────
    print -n "Checking clippy... "
    let clippy = (do { cargo clippy --lib -- -D warnings } | complete)
    if $clippy.exit_code == 0 {
        print $"($green)✓($reset)"
    } else {
        print $"($red)✗  (run: cargo clippy -- -D warnings)($reset)"
        $errors = $errors + 1
    }

    # ── 3. Tests ──────────────────────────────────────────────────────────────
    print -n "Running tests... "
    let tests = (do { cargo test --all-features } | complete)
    if $tests.exit_code == 0 {
        print $"($green)✓($reset)"
    } else {
        print $"($red)✗  (run: cargo test --all-features)($reset)"
        $errors = $errors + 1
    }

    # ── 4. Documentation ──────────────────────────────────────────────────────
    print -n "Building documentation... "
    let docs = (do { cargo doc --no-deps } | complete)
    if $docs.exit_code == 0 {
        print $"($green)✓($reset)"
    } else {
        print $"($red)✗  (run: cargo doc --no-deps)($reset)"
        $errors = $errors + 1
    }

    # ── 5. Required files ─────────────────────────────────────────────────────
    print -n "Checking required files... "
    let required = ["README.md", "LICENSE", "Cargo.toml", "CHANGELOG.md"]
    let missing = ($required | where { |f| not ($f | path exists) })
    if ($missing | is-empty) {
        print $"($green)✓($reset)"
    } else {
        for f in $missing {
            print $"($red)Missing: ($f)($reset)"
        }
        $errors = $errors + 1
    }

    # ── 6. Dry run ────────────────────────────────────────────────────────────
    # In a workspace, publish is per-crate. droidkraft-core is self-contained so
    # it can be dry-run directly; droidkraft-tui depends on it and can only be
    # dry-run once core is published, so we validate core here.
    print -n "Cargo publish dry-run (droidkraft-core)... "
    let dry_run = (do { cargo publish -p droidkraft-core --dry-run } | complete)
    if $dry_run.exit_code == 0 {
        print $"($green)✓($reset)"
    } else {
        print $"($red)✗  (run: cargo publish -p droidkraft-core --dry-run for details)($reset)"
        $errors = $errors + 1
    }

    # ── Summary ───────────────────────────────────────────────────────────────
    print ""
    if $errors == 0 {
        print $"($green)✓ All checks passed! Ready to publish.($reset)"
        print ""
        print "Run: cargo publish -p droidkraft-core, then cargo publish -p droidkraft-tui"
    } else {
        let plural = if $errors == 1 { "check" } else { "checks" }
        print $"($red)✗ ($errors) ($plural) failed. Please fix before publishing.($reset)"
        exit 1
    }
}
