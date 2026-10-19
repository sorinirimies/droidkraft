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
    cargo publish --dry-run -p droidkraft-mirror
    cargo publish --dry-run -p droidkraft-fastboot
    @echo "ℹ️  core/tui/gui dry-runs need their dependencies on crates.io first"

# Publish to crates.io
publish: release-check publish-mirror publish-fastboot publish-core publish-tui publish-gui
    @echo "✅ All crates published."

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

# Fail fast (5 s) instead of hanging when a remote host is unreachable.
export GIT_SSH_COMMAND := "ssh -o ConnectTimeout=5 -o BatchMode=yes"

# Unreachable remotes warn and are skipped; real rejections still fail the recipe.
PUSHFN := 'gpush() { out=$(git push "$@" 2>&1); rc=$?; if [ $rc -ne 0 ] && printf "%s" "$out" | grep -Eqi "timed out|could not resolve|connection refused|no route to host|network is unreachable"; then echo "⚠️  $r unreachable - NOT pushed (skipped)"; return 0; fi; [ -n "$out" ] && printf "%s\n" "$out"; return $rc; }'

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

# Sample memory of a running process: just soak droidkraft-tui 120 (start it first; open the Screen tab)
soak name="droidkraft" secs="60":
    nu scripts/soak.nu {{name}} {{secs}}

# macOS: report leaked allocations in a running process (leaks <name>). just leaks droidkraft-tui
leaks name="droidkraft-tui":
    leaks {{name}} | tail -40

# macOS: heap summary of a running process. just heap droidkraft
heap name="droidkraft":
    heap {{name}} | head -40

# ── Per-crate build / run / test / doc ────────────────────────────────────────

# Build core only
build-core:
    cargo build -p droidkraft-core

# Build the TUI only
build-tui:
    cargo build -p droidkraft-tui

# Run the TUI
run-tui:
    cargo run -p droidkraft-tui

# Test the TUI
test-tui:
    cargo test -p droidkraft-tui --all-features

# Test the GUI
test-gui:
    cargo test -p droidkraft --all-features

# Open TUI docs
doc-tui:
    cargo doc --no-deps -p droidkraft-tui --open

# Open GUI docs
doc-gui:
    cargo doc --no-deps -p droidkraft --open

# ── Coverage / benchmarks ─────────────────────────────────────────────────────

# HTML coverage report (needs `cargo install cargo-llvm-cov`)
coverage:
    cargo llvm-cov --workspace --html
    @echo "→ target/llvm-cov/html/index.html"

# Run benchmarks, if any (cargo bench)
bench:
    cargo bench --workspace

# ── Publishing (order: mirror/fastboot → core → tui → gui) ────────────────────

# Check publish readiness
check-publish: check-nu
    nu scripts/check_publish.nu

publish-mirror:
    @echo "📦 Publishing droidkraft-mirror…"
    cargo publish -p droidkraft-mirror

publish-fastboot:
    @echo "📦 Publishing droidkraft-fastboot…"
    cargo publish -p droidkraft-fastboot
    @echo "⏳ Waiting 30 s for the index to propagate…"
    sleep 30

publish-core:
    @echo "📦 Publishing droidkraft-core…"
    cargo publish -p droidkraft-core
    @echo "⏳ Waiting 30 s for the index to propagate…"
    sleep 30

publish-tui:
    @echo "📦 Publishing droidkraft-tui…"
    cargo publish -p droidkraft-tui

publish-gui:
    @echo "📦 Publishing the GUI (package droidkraft)…"
    cargo publish -p droidkraft

# ── Release helpers ───────────────────────────────────────────────────────────

# Show the latest changelog section
changelog-latest: check-git-cliff
    @git-cliff --latest

# Preview what the next release would contain
release-preview: check-git-cliff
    @echo "Current version: $(just version)"
    @echo ""
    @echo "Unreleased commits:"
    @git-cliff --unreleased
    @echo ""
    @echo "Published crates: droidkraft-mirror • droidkraft-fastboot • droidkraft-core • droidkraft-tui • droidkraft (GUI)"

# Release quality gate: fmt + clippy + test + release build
check-release: check-all build-release
    @echo "✅ Release quality gate passed!"

# Release to GitHub only: bump, tag, push to origin
release-github version: (bump version)
    git push --follow-tags origin "$(git branch --show-current)"
    @echo "✅ Release v{{version}} pushed to GitHub."

# Re-run the Release workflow for an existing tag
release-retrigger version:
    @command -v gh >/dev/null 2>&1 || { echo "❌ GitHub CLI (gh) not found. Install from https://cli.github.com"; exit 1; }
    gh workflow run release.yml --field tag=v{{version}}
    @echo "✅ Dispatched — see GitHub Actions."

# Push latest commit + tags to every remote (after the quality gate)
push-release-all: check-all
    #!/usr/bin/env sh
    {{PUSHFN}}
    failed=""; skipped=""
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r"; skipped="$skipped $r"; continue; }
        gpush --follow-tags "$r" "$(git branch --show-current)" || failed="$failed $r"
    done
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    [ -n "$failed" ] && { echo "⚠️  Failed to push to:$failed"; exit 1; }
    echo "✅ Latest commit + tags pushed to: {{REMOTES}}"

# Force-push the current branch to every remote
push-all-force:
    #!/usr/bin/env sh
    {{PUSHFN}}
    failed=""; skipped=""
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r"; skipped="$skipped $r"; continue; }
        gpush --force "$r" "$(git branch --show-current)" || failed="$failed $r"
    done
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    [ -n "$failed" ] && { echo "⚠️  Failed to force-push to:$failed"; exit 1; }
    echo "✅ Force-pushed to: {{REMOTES}}"

# ── Housekeeping ──────────────────────────────────────────────────────────────

# Update deps, run the gate, commit Cargo.lock and push
update-deps:
    @echo "⬆️  Updating dependencies…"
    cargo update
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace --locked
    git add Cargo.lock
    git diff --cached --quiet || git commit -m "chore: update dependencies"
    git push origin "$(git branch --show-current)"
    @echo "✅ Dependency updates pushed."

# Trigger the dependency-update workflow on GitHub
deps-update-now:
    gh workflow run deps-update.yml

# List available VHS tapes
vhs-list:
    @ls docs/tapes/*.tape | sed 's|.*/||; s|\.tape||' | sed 's/^/  /'

# Add any missing remotes
setup-remotes:
    #!/usr/bin/env sh
    git remote get-url origin >/dev/null 2>&1 || git remote add origin git@github.com:sorinirimies/droidkraft.git
    git remote get-url gitea-starscream >/dev/null 2>&1 || git remote add gitea-starscream gitea@192.168.1.44:sorin/droidkraft.git
    git remote -v

# Force-push the current branch to GitHub (origin). No lease!
push-force:
    git push --force origin "$(git branch --show-current)"

# Force-push the current branch to Gitea Starscream. No lease!
push-gitea-starscream-force:
    git push --force gitea-starscream "$(git branch --show-current)"

# Force-push all tags to GitHub (moves existing tags)
push-tags-force:
    git push --force origin --tags

# Force-push all tags to every remote (continues on failure)
push-tags-all-force:
    #!/usr/bin/env sh
    {{PUSHFN}}
    failed=""
    for r in {{REMOTES}}; do
        git remote get-url "$r" >/dev/null 2>&1 || { echo "↷ skipping $r (not configured)"; continue; }
        gpush --force "$r" --tags || failed="$failed $r"
    done
    if [ -n "$failed" ]; then echo "⚠️  Failed to force-push tags to:$failed"; exit 1; fi
    echo "✅ Tags force-pushed to every configured remote"
