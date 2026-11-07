# DroidKraft — task runner
# Recipes run in Nushell; anything longer than a line lives in scripts/*.nu (tested).
# Install: cargo install just nu git-cliff   — then `just install-tools`
# Usage:   just <task>

set shell := ["nu", "-c"]

# ── Default ───────────────────────────────────────────────────────────────────

default:
    @just --list

# ── Tool checks ───────────────────────────────────────────────────────────────

_check-git-cliff:
    @nu scripts/tools.nu check git-cliff

# Install the development tools (git-cliff, cargo-audit/deny/fuzz/llvm-cov/outdated/machete)
install-tools:
    nu scripts/tools.nu install

# ── Build ─────────────────────────────────────────────────────────────────────

# Check without building
check:
    cargo check --workspace

# Build (dev)
build:
    cargo build --workspace

# Release build of the shipped binaries (scripts/project.nu)
build-release:
    nu scripts/ci/build_release.nu

# Install the shipped binaries from this checkout into ~/.cargo/bin
install:
    nu scripts/ci/build_release.nu install

# ── Run ───────────────────────────────────────────────────────────────────────

# Run the main binary (e.g. just run --help)
run *args:
    cargo run -q -p droidkraft-tui -- {{ args }}

# ── Test & quality ────────────────────────────────────────────────────────────

# Rust tests
test:
    cargo test --workspace --all-features

# Nushell script tests
test-nu:
    nu scripts/tests/run_all.nu

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

# Clippy, warnings are errors
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Rust and Nu tests
test-all-nu: test test-nu
    @echo "✅ All Rust and Nu tests passed!"

# Line coverage (needs cargo-llvm-cov); writes target/llvm-cov/html
coverage:
    cargo llvm-cov --workspace --html
    @echo "→ target/llvm-cov/html/index.html"

# Docs for the workspace, warnings are errors (what docs.rs would reject)
doc:
    $env.RUSTDOCFLAGS = "-D warnings"; cargo doc --no-deps --workspace

# Secrets anywhere in the git history (brew install gitleaks); CI runs it on every push.
# A false positive goes in .gitleaksignore, by the fingerprint gitleaks prints.
secrets:
    gitleaks git --redact --no-banner .

# Everything CI runs (fmt check, clippy, tests, versions, nu tests)
check-all: fmt
    nu scripts/ci/quality_gate.nu

# Security: known vulnerabilities (cargo install cargo-audit)
audit:
    cargo audit

# Security: licenses, advisories, bans, sources (cargo install cargo-deny)
deny:
    cargo deny check

# Dependencies nothing uses (cargo install cargo-machete; the gate runs it too)
unused-deps:
    cargo machete

# Fuzz one target (e.g. just fuzz parse 60); needs nightly + cargo-fuzz
fuzz target secs="60":
    cargo +nightly fuzz run {{ target }} -- -max_total_time={{ secs }}

# Every fuzz target for a short while
fuzz-all secs="30":
    nu scripts/ci/fuzz_all.nu --secs {{ secs }}

# List the fuzz targets
fuzz-list:
    cargo +nightly fuzz list

# Security + fuzz locally, like the nightly workflows
nightly-local: audit deny fuzz-all

# ── Dependencies ──────────────────────────────────────────────────────────────

# Update Cargo.lock and fuzz/Cargo.lock (semver-compatible only)
update:
    nu scripts/ci/deps_update.nu update

# Show outdated dependencies (needs cargo-outdated)
outdated:
    cargo outdated --workspace

# What the nightly workflow does: update, full gate, commit and push if green
update-deps: update check-all
    nu scripts/ci/deps_update.nu commit (git config user.name) (git config user.email)

# ── Nightly workflows on demand ───────────────────────────────────────────────

# Run a GitHub workflow now (deps-update | security | fuzz | ci), needs gh
gh-run workflow="deps-update":
    gh workflow run {{ workflow }}.yml
    @echo "✅ Dispatched {{ workflow }} — gh run watch"

# Run a Gitea workflow now (deps-update | security | fuzz); needs GITEA_URL + GITEA_TOKEN
gitea-run workflow="deps-update":
    nu scripts/gitea_run.nu {{ workflow }}

# The nightly dependency update, now, on both hosts
deps-update-now: (gh-run "deps-update")
    -just gitea-run deps-update

# Security audit workflow, now, on both hosts
security-now: (gh-run "security")
    -just gitea-run security

# Fuzz workflow, now, on both hosts
fuzz-now: (gh-run "fuzz")
    -just gitea-run fuzz

# ── Changelog ─────────────────────────────────────────────────────────────────

# Regenerate CHANGELOG.md from all tags
changelog: _check-git-cliff
    git-cliff --output CHANGELOG.md

# Preview the next release's changelog
changelog-preview: _check-git-cliff
    @git-cliff --unreleased

# Prepend only the unreleased commits to CHANGELOG.md
changelog-unreleased: _check-git-cliff
    git-cliff --unreleased --prepend CHANGELOG.md

# Show the latest release's entry
changelog-latest: _check-git-cliff
    @git-cliff --latest

# ── Version & release ─────────────────────────────────────────────────────────

# Current version (Cargo.toml)
version:
    @nu scripts/version.nu

# Check Cargo.toml and EXTRA_MANIFESTS agree (and a tag, if given)
check-versions tag="":
    nu scripts/ci/check_versions.nu {{ tag }}

# Validate that a version gives a valid vX.Y.Z tag
validate-tag version:
    nu scripts/ci/validate_tag.nu "v{{ version }}"

# What would be released, without changing anything
release-preview: _check-git-cliff
    @print $"Current version: (nu scripts/version.nu)\n"
    @git-cliff --unreleased
    @nu scripts/project.nu

# Everything in check-all plus a release build and the docs
check-release: check-all build-release doc
    @echo "✅ Release gate passed"

# Dry-run the crates.io publish of every crate in scripts/project.nu
publish-dry:
    nu scripts/ci/publish_crate.nu --dry-run

# Bump Cargo.toml (+ EXTRA_MANIFESTS) + Cargo.lock, regenerate the changelog, commit and tag
bump version: _check-git-cliff check-all
    nu scripts/bump_version.nu --yes --skip-checks {{ version }}

# Bump, tag and push to GitHub — starts the Release workflow
release version: (bump version)
    git push --follow-tags origin (git branch --show-current)
    @print "✅ v{{ version }} pushed — the Release workflow builds the binaries."

# First release of the version already in Cargo.toml (no bump), pushed to GitHub
release-initial: _check-git-cliff check-all
    nu scripts/release_initial.nu
    git push --follow-tags origin (git branch --show-current)

# Bump, tag and push to every remote — starts the Release workflows on both hosts
release-all version: (bump version) push-all

# Bump, tag and push to Gitea only
release-gitea-starscream version: (bump version)
    git push --follow-tags gitea-starscream (git branch --show-current)

# First release, pushed to every remote
release-initial-all: _check-git-cliff check-all
    nu scripts/release_initial.nu
    nu scripts/remotes.nu push

# Re-run the GitHub Release workflow for an existing tag (e.g. just release-retrigger 0.1.0)
release-retrigger version:
    gh workflow run release.yml --field tag=v{{ version }}

# Package the release binaries locally (e.g. just package aarch64-apple-darwin)
package target:
    nu scripts/ci/build_release.nu {{ target }}
    nu scripts/ci/package.nu {{ target }} --out dist

# ── Git remotes (GitHub = origin, Gitea = gitea-starscream; no mirroring) ─────

# Push the current branch + tags to every configured remote (continues on failure)
push-all:
    nu scripts/remotes.nu push

# Show all configured remotes
remotes:
    @git remote -v

# Push the current branch to GitHub
push:
    git push origin (git branch --show-current)

# Push the current branch to Gitea Starscream
push-gitea-starscream:
    git push gitea-starscream (git branch --show-current)

# Push all tags to GitHub
push-tags:
    git push origin --tags

# Push all tags to every remote (continues on failure)
push-tags-all:
    nu scripts/remotes.nu push-tags

# Latest commit + tags to every remote, after the gate (no bump)
push-release-all: check-all push-all

# Force-push the current branch to every remote (no lease — be sure!)
push-all-force:
    nu scripts/remotes.nu push --force

# Pull the current branch from GitHub
pull:
    git pull origin (git branch --show-current)

# Pull the current branch from Gitea Starscream
pull-gitea-starscream:
    git pull gitea-starscream (git branch --show-current)

# Pull the current branch from every remote (continues on failure)
pull-all:
    nu scripts/remotes.nu pull

# Force-sync Gitea Starscream with this checkout (branch + tags) — the nightlies diverge
sync-gitea-starscream:
    git push gitea-starscream (git branch --show-current) --force
    git push gitea-starscream --tags --force

# Add the missing remotes
setup-remotes:
    nu scripts/remotes.nu setup

# Remove build artifacts
clean:
    cargo clean
    rm -rf dist

# ── Project extras ─────────────────────────────────────────────────────────────

# Run the GUI (needs the Xcode Metal toolchain on macOS)
run-gui:
    cargo run -p droidkraft

# Build the TUI in release mode and launch it
run-release:
    cargo build --release -p droidkraft-tui; nu scripts/run.nu

# Interactive feature demo
demo:
    cargo build --release -p droidkraft-tui; nu scripts/demo.nu

# Memory growth of a running droidkraft or droidkraft-tui (start it first)
soak name="droidkraft" secs="60":
    nu scripts/soak.nu {{name}} {{secs}}

# Regenerate every app icon (PNGs, .icns, .ico) from assets/icons/droidkraft.svg (needs resvg, magick; iconutil for the .icns)
icons:
    nu scripts/icons.nu

# Re-record one preview GIF, e.g. `just vhs tui-overview` (needs vhs and a device)
vhs name:
    cargo build --release -p droidkraft-tui; vhs docs/tapes/{{name}}.tape

# Re-record every TUI preview GIF (the GUI GIFs are screen captures)
previews:
    cargo build --release -p droidkraft-tui; glob docs/tapes/tui-*.tape | each {|t| ^vhs $t } | ignore
