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
    git push origin main
    git push origin v{{version}}
    @echo "✅ Release v{{version}} pushed — GitHub Actions will handle the rest."

# Full automated release to Gitea only.
release-gitea version: check-all check-git-cliff check-nu
    nu scripts/bump_version.nu --yes {{version}}
    @echo "Pushing branch and tag to Gitea..."
    git push gitea main
    git push gitea v{{version}}
    @echo "✅ Release v{{version}} pushed to Gitea."

# Full automated release to both GitHub and Gitea.
release-all version: check-all check-git-cliff check-nu
    nu scripts/bump_version.nu --yes {{version}}
    @echo "Pushing branch and tag to GitHub and Gitea..."
    git push origin main
    git push gitea main
    git push origin v{{version}}
    git push gitea v{{version}}
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

# Generate quickstart VHS demo (requires vhs: brew install vhs)
vhs-quickstart:
    vhs examples/vhs/quickstart.tape
    @echo "✅ Generated examples/vhs/quickstart.gif"

# Generate main menu VHS demo
vhs-menu:
    vhs examples/vhs/main_menu.tape
    @echo "✅ Generated examples/vhs/main_menu.gif"

# Generate full demo VHS
vhs-full:
    vhs examples/vhs/full_demo.tape
    @echo "✅ Generated examples/vhs/full_demo.gif"

# Generate all VHS demos
vhs-all:
    ./examples/vhs/generate_all.sh

# Check if VHS is installed
vhs-check:
    @command -v vhs >/dev/null 2>&1 || { echo "❌ VHS not installed. Install with: brew install vhs"; exit 1; }
    @echo "✅ VHS is installed"

# ── Git helpers ───────────────────────────────────────────────────────────────

# Show current git remotes
remotes:
    @echo "Configured git remotes:"
    @git remote -v

# Add a Gitea remote (usage: just setup-gitea https://gitea.example.com/user/repo.git)
setup-gitea url:
    git remote add gitea {{url}}
    @echo "✅ Gitea remote added! Test with: git push gitea main"

# Commit all staged changes
commit message:
    git add .
    git commit -m "{{message}}"

# Pull from GitHub
pull:
    git pull origin main

# Pull from Gitea
pull-gitea:
    git pull gitea main

# Pull from both remotes
pull-all:
    git pull origin main
    git pull gitea main
    @echo "✅ Pulled from both!"

# Push to GitHub
push:
    git push origin main

# Push to Gitea
push-gitea:
    git push gitea main

# Push to both GitHub and Gitea
push-all:
    git push origin main
    git push gitea main
    @echo "✅ Pushed to both!"

# Push tags to GitHub
push-tags:
    git push origin --tags

# Push tags to both remotes
push-tags-all:
    git push origin --tags
    git push gitea --tags
    @echo "✅ Tags pushed to both!"

# Force-sync Gitea from GitHub
sync-gitea:
    git push gitea main --force
    git push gitea --tags --force
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
