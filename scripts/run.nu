#!/usr/bin/env nu
# DroidKraft - Android Development TUI Launcher
# Builds and runs the DroidKraft application.
#
# Usage: nu scripts/run.nu

def main [] {
    let blue   = (ansi blue)
    let green  = (ansi green)
    let yellow = (ansi yellow)
    let red    = (ansi red)
    let reset  = (ansi reset)

    print $"($blue)🤖 DroidKraft - Android Development TUI($reset)"
    print $"($blue)======================================($reset)"
    print ""

    # ── Check Rust/Cargo ──────────────────────────────────────────────────────
    if (which cargo | is-empty) {
        print $"($red)❌ Error: Rust/Cargo is not installed($reset)"
        print $"($yellow)Please install Rust from: https://rustup.rs/($reset)"
        exit 1
    }

    # ── Check ADB server ──────────────────────────────────────────────────────
    print $"($yellow)ℹ️  DroidKraft connects directly to the ADB server on port 5037.($reset)"
    print $"($yellow)   Make sure the ADB server is running: adb start-server($reset)"
    print ""

    # ── Build ─────────────────────────────────────────────────────────────────
    print $"($blue)🔨 Building DroidKraft...($reset)"
    let build = (do { run-external "cargo" "build" "--release" } | complete)
    if $build.exit_code != 0 {
        print $"($red)❌ Build failed!($reset)"
        exit 1
    }
    print $"($green)✅ Build successful!($reset)"
    print ""

    # ── Launch ────────────────────────────────────────────────────────────────
    print $"($blue)🚀 Starting DroidKraft...($reset)"
    print $"($yellow)   Press Ctrl+C or 'q' to exit($reset)"
    print ""

    run-external "./target/release/droidkraft"
}
