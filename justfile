# DroidKraft — task runner
#
# Install just:     cargo install just
# Install nushell:  https://www.nushell.sh
# Usage:            just <task>   |   just --list

# Default task — show available commands
default:
    @just --list

# ── Prerequisites ─────────────────────────────────────────────────────────────

# Install required tools (just, git-cliff). Nu must be installed manually.
install-tools:
    @echo "Installing required tools..."
    @command -v just >/dev/null 2>&1 || cargo install just
    @command -v git-cliff >/dev/null 2>&1 || cargo install git-cliff
    @command -v nu >/dev/null 2>&1 && echo "✅ nu found" || echo "⚠ nu (nushell) not found. Install: https://www.nushell.sh"
    @echo "✅ Done!"

# Check git-cliff is available
check-git-cliff:
    @command -v git-cliff >/dev/null 2>&1 || { echo "❌ git-cliff not found. Run: just install-tools"; exit 1; }

# Check nu (nushell) is available
check-nu:
    @command -v nu >/dev/null 2>&1 || { echo "❌ nu (nushell) not found. Install: https://www.nushell.sh"; exit 1; }

# ── Build ─────────────────────────────────────────────────────────────────────

# Build (debug) — core + TUI (default members)
build:
    cargo build

# Build (release) — core + TUI
build-release:
    cargo build --release

# Build the GPUI GUI (opt-in; needs the full Xcode/Metal toolchain on macOS)
build-gui:
    cargo build -p droidkraft

# Build everything including the GUI
build-all:
    cargo build --workspace

# ── Code quality ──────────────────────────────────────────────────────────────

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Check formatting (CI-safe, no writes)
fmt-check:
    cargo fmt --check

# Run clippy
clippy:
    cargo clippy -- -D warnings

# Run all quality checks (fmt, clippy, test)
check-all: fmt-check clippy test
    @echo "✅ All checks passed!"

# ── Tests ─────────────────────────────────────────────────────────────────────

# Run tests — core + TUI
test:
    cargo test

# Run only the core library tests
test-core:
    cargo test -p droidkraft-core

# Run tests with all features
test-all:
    cargo test --all-features --all-targets

# Run Nu script tests
test-nu: check-nu
    nu scripts/tests/run_all.nu

# Run both Rust and Nu tests
test-all-nu: test-all test-nu
    @echo "✅ All Rust and Nu tests passed!"

# ── Documentation ─────────────────────────────────────────────────────────────

# Build and open docs
doc:
    cargo doc --no-deps --open

# Build docs (no open, for CI)
doc-build:
    cargo doc --no-deps --all-features

# ── Changelog ─────────────────────────────────────────────────────────────────

# Generate full changelog from all tags
changelog: check-git-cliff
    @echo "Generating full changelog..."
    git-cliff -o CHANGELOG.md
    @echo "✅ Changelog generated!"

# Preview unreleased changes (no file write)
changelog-preview: check-git-cliff
    @git-cliff --unreleased

# Prepend unreleased commits to existing changelog
changelog-unreleased: check-git-cliff
    @echo "Prepending unreleased commits..."
    git-cliff --unreleased --prepend CHANGELOG.md
    @echo "✅ Done!"

# Generate changelog for a specific version tag
changelog-version version: check-git-cliff
    @echo "Generating changelog for version {{version}}..."
    git-cliff --tag v{{version}} -o CHANGELOG.md
    @echo "✅ Changelog generated for v{{version}}!"

# Regenerate full changelog from all history
changelog-update: check-git-cliff
    @echo "Regenerating complete changelog..."
    git-cliff --output CHANGELOG.md
    @echo "✅ Changelog updated!"

# ── Versioning & Release ──────────────────────────────────────────────────────

# Show current version
version: check-nu
    @nu scripts/version.nu

# Bump version interactively — prompts for confirmation, runs checks, commits and tags.
# Use `just release <version>` for fully automated non-interactive release.
bump version: check-all check-git-cliff check-nu
    nu scripts/bump_version.nu {{version}}

# Run pre-publish readiness checks
release-check: check-nu
    nu scripts/check_publish.nu

# Publish to crates.io (dry run — no side-effects)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io
publish: release-check
    cargo publish

# ── Full release workflows ────────────────────────────────────────────────────

# Full automated release to GitHub — bumps version, commits, tags, and pushes.
# This is the single command you run to cut a release:
#   just release 0.4.0
release version: check-all check-git-cliff check-nu
    nu scripts/bump_version.nu --yes {{version}}
    @echo "Pushing branch and tag to GitHub..."
    git push origin "$(git branch --show-current)"
    git push origin v{{version}}
    @echo "✅ Release v{{version}} pushed — GitHub Actions will handle the rest."

# Full automated release to Gitea only.
release-gitea-starscream version: check-all check-git-cliff check-nu
    nu scripts/bump_version.nu --yes {{version}}
    @echo "Pushing branch and tag to Gitea..."
    git push gitea-starscream "$(git branch --show-current)"
    git push gitea-starscream v{{version}}
    @echo "✅ Release v{{version}} pushed to Gitea."

# Full automated release to both GitHub and Gitea.
release-all version: check-all check-git-cliff check-nu
    nu scripts/bump_version.nu --yes {{version}}
    @echo "Pushing branch and tag to GitHub and Gitea..."
    git push origin "$(git branch --show-current)"
    git push gitea-starscream "$(git branch --show-current)"
    git push origin v{{version}}
    git push gitea-starscream v{{version}}
    @echo "✅ Release v{{version}} pushed to both remotes."

# ── Application ───────────────────────────────────────────────────────────────

# Run the TUI (cargo run, debug)
run:
    cargo run -p droidkraft-tui

# Run the GPUI GUI (opt-in; needs the full Xcode/Metal toolchain on macOS)
run-gui:
    cargo run -p droidkraft

# Build release binary and launch it
run-release: build-release
    nu scripts/run.nu

# Run interactive demo
demo: build-release
    nu scripts/demo.nu

# Run long output test (scrolling)
test-long-output:
    nu scripts/test_long_output.nu

# Run wide output test (wrapping)
test-wide-output:
    nu scripts/test_wide_output.nu

# ── VHS / Demo GIFs ──────────────────────────────────────────────────────────

# Re-record one preview GIF (usage: just vhs tui-overview; needs vhs + a device + release build)
vhs name:
    cargo build --release -p droidkraft-tui
    vhs docs/tapes/{{name}}.tape
    @echo "✅ Generated docs/previews/{{name}}.gif"

# Re-record all TUI preview GIFs (the GUI GIFs are screen captures, see docs/tapes/README.md)
previews:
    cargo build --release -p droidkraft-tui
    for t in docs/tapes/tui-*.tape; do vhs "$t"; done

# Check if VHS is installed
vhs-check:
    @command -v vhs >/dev/null 2>&1 || { echo "❌ VHS not installed. Install with: brew install vhs"; exit 1; }
    @echo "✅ VHS is installed"

# ── Git helpers ───────────────────────────────────────────────────────────────

# Show current git remotes
REMOTES := "origin gitea-starscream"

remotes:
    @echo "Configured git remotes:"
    @git remote -v

# Add a Gitea remote (usage: just setup-gitea https://gitea.example.com/user/repo.git)
setup-gitea url:
    git remote add gitea {{url}}
    @echo "✅ Gitea remote added! Test with: git push gitea-starscream main"

# Commit all staged changes
commit message:
    git add .
    git commit -m "{{message}}"

# Pull from GitHub
pull:
    git pull origin "$(git branch --show-current)"

# Pull from Gitea
pull-gitea-starscream:
    git pull gitea-starscream "$(git branch --show-current)"

# Pull from both remotes
pull-all:
    #!/usr/bin/env sh
    failed=""; ok_=""; skipped=""
    for r in {{REMOTES}}; do
        if ! git remote get-url "$r" >/dev/null 2>&1; then
            echo "↷ skipping $r (not configured here; add with: git remote add $r <url>)"
            skipped="$skipped $r"; continue
        fi
        echo "→ pulling from $r"
        if git pull "$r" "$(git branch --show-current)"; then ok_="$ok_ $r"; else failed="$failed $r"; fi
    done
    echo ""
    [ -n "$ok_" ] && echo "✅ Pulled from:$ok_"
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    if [ -n "$failed" ]; then echo "⚠️  Failed:$failed"; exit 1; fi

# Push to GitHub
push:
    git push origin "$(git branch --show-current)"

# Push to Gitea
push-gitea-starscream:
    git push gitea-starscream "$(git branch --show-current)"

# Push to both GitHub and Gitea
push-all:
    #!/usr/bin/env sh
    failed=""; ok_=""; skipped=""
    for r in {{REMOTES}}; do
        if ! git remote get-url "$r" >/dev/null 2>&1; then
            echo "↷ skipping $r (not configured here; add with: git remote add $r <url>)"
            skipped="$skipped $r"; continue
        fi
        echo "→ pushing to $r"
        if git push "$r" "$(git branch --show-current)"; then ok_="$ok_ $r"; else failed="$failed $r"; fi
    done
    echo ""
    [ -n "$ok_" ] && echo "✅ Pushed to:$ok_"
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    if [ -n "$failed" ]; then echo "⚠️  Failed:$failed"; exit 1; fi

# Push tags to GitHub
push-tags:
    git push origin --tags

# Push tags to both remotes
push-tags-all:
    #!/usr/bin/env sh
    failed=""; ok_=""; skipped=""
    for r in {{REMOTES}}; do
        if ! git remote get-url "$r" >/dev/null 2>&1; then
            echo "↷ skipping $r (not configured here; add with: git remote add $r <url>)"
            skipped="$skipped $r"; continue
        fi
        echo "→ pushing tags to $r"
        if git push "$r" --tags; then ok_="$ok_ $r"; else failed="$failed $r"; fi
    done
    echo ""
    [ -n "$ok_" ] && echo "✅ Tags pushed to:$ok_"
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    if [ -n "$failed" ]; then echo "⚠️  Failed:$failed"; exit 1; fi

# Force-sync Gitea from GitHub
sync-gitea-starscream:
    git push gitea-starscream "$(git branch --show-current)" --force
    git push gitea-starscream --tags --force
    @echo "✅ Gitea synced!"

# ── Misc ─────────────────────────────────────────────────────────────────────

# Update dependencies
update:
    cargo update

# Show outdated dependencies
outdated:
    cargo outdated

# Clean build artifacts
clean:
    cargo clean

# Install the application locally
install:
    cargo install --path .

# Watch and auto-run on file changes (requires cargo-watch)
watch:
    cargo watch -x run

# Build all examples
examples:
    cargo build --examples

# Run example: main menu
example-menu:
    cargo run --example main_menu

# Show project info
info:
    @echo "Project:  DroidKraft"
    @echo "Version:  $(just version)"
    @echo "Author:   Sorin Albu-Irimies"
    @echo "License:  MIT"
    @echo "Crate:    https://crates.io/crates/droidkraft"

# View changelog
view-changelog:
    @cat CHANGELOG.md

# ── Security ──────────────────────────────────────────────────────────────────

# ── Security ──────────────────────────────────────────────────────────────────

# Run cargo-deny checks (advisories, licenses, bans, sources)
deny:
    cargo deny check

# Run cargo-audit (RustSec advisories against Cargo.lock)
audit:
    cargo audit

# Full security gate: audit + deny
security: audit deny

# ── Fuzzing (needs nightly + `cargo install cargo-fuzz`) ──────────────────────

# List fuzz targets
fuzz-list:
    cargo +nightly fuzz list

# Fuzz one target: just fuzz log_entry [secs]
fuzz target="log_entry" secs="60":
    cargo +nightly fuzz run {{target}} -- -max_total_time={{secs}}

# Short fuzz run of every target (CI smoke)
fuzz-smoke secs="10":
    #!/usr/bin/env bash
    set -euo pipefail
    for t in $(cargo +nightly fuzz list); do
        echo "== $t"
        cargo +nightly fuzz run "$t" -- -max_total_time={{secs}}
    done

# Check the fuzz crate compiles (stable, no fuzzing)
fuzz-check:
    cd fuzz && cargo check --bins

# ── Memory / leak checks ──────────────────────────────────────────────────────

# TUI soak: drive the Screen tab in a pty and sample RSS (needs a device). just soak-tui 120
soak-tui secs="60":
    cargo build --release -p droidkraft-tui
    python3 scripts/soak_tui.py {{secs}}

# GUI soak: sample RSS of the running GUI every 10s. just soak-gui 120
soak-gui secs="60":
    cargo build -p droidkraft
    bash scripts/soak_gui.sh {{secs}}

# macOS: report leaked allocations in a running process (leaks <name>). just leaks droidkraft-tui
leaks name="droidkraft-tui":
    leaks {{name}} | tail -40

# macOS: heap summary of a running process. just heap droidkraft
heap name="droidkraft":
    heap {{name}} | head -40

