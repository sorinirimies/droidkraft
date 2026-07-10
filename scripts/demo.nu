#!/usr/bin/env nu
# DroidKraft Demo Script
# Demonstrates the features of the DroidKraft application interactively.
#
# Usage: nu scripts/demo.nu

def print_header [title: string] {
    let cyan  = (ansi cyan)
    let reset = (ansi reset)
    print ""
    print $"($cyan)================================($reset)"
    print $"($cyan)($title)($reset)"
    print $"($cyan)================================($reset)"
    print ""
}

def print_feature [text: string] {
    let green = (ansi green)
    let reset = (ansi reset)
    print $"($green)✓($reset) ($text)"
}

def wait_for_user [] {
    let yellow = (ansi yellow)
    let reset  = (ansi reset)
    print $"($yellow)Press Enter to continue...($reset)"
    input ""
    null
}

def main [] {
    let green   = (ansi green)
    let blue    = (ansi blue)
    let yellow  = (ansi yellow)
    let red     = (ansi red)
    let cyan    = (ansi cyan)
    let magenta = (ansi magenta)
    let reset   = (ansi reset)

    # ASCII Art Header
    print $"($green)"
    print "     ____            _     _ _____ _   _ _____"
    print "    |  _ \\  _ __ ___  (_) __| |_   _| | | |_   _|"
    print "    | | | || '__/ _ \\ | |/ _` | | | | | | | | |"
    print "    | |_| || | | (_) || | (_| | | | | |_| | | |"
    print "    |____/ |_|  \\___/ |_|\\__,_| |_|  \\___/  |_|"
    print ""
    print "        Android Development TUI Demo"
    print $"($reset)"

    print_header "🤖 Welcome to DroidKraft Demo"

    print $"($blue)DroidKraft is a beautiful Terminal User Interface for Android development($reset)"
    print $"($blue)that provides an intuitive interface for ADB commands with visual effects.($reset)"
    print ""
    wait_for_user

    print_header "✨ Key Features"

    print_feature "Beautiful startup animation with Android logo and gradient background"
    print_feature "Interactive menu system with ADB command categories"
    print_feature "Real-time command execution with result display"
    print_feature "Keyboard navigation (vim-style j/k or arrow keys)"
    print_feature "Visual effects powered by TachyonFX library"
    print_feature "Clean, responsive terminal interface"
    print_feature "Error handling with user-friendly messages"
    print_feature "Self-contained ADB communication via adb_client (no adb CLI needed)"

    wait_for_user

    print_header "📱 Available ADB Commands"

    print $"($magenta)Device Management:($reset)"
    print_feature "List connected devices"
    print_feature "Get device information"
    print_feature "Reboot device"

    print ""
    print $"($magenta)App Management:($reset)"
    print_feature "Install APK files"
    print_feature "Uninstall applications"
    print_feature "List installed packages"

    print ""
    print $"($magenta)File Operations:($reset)"
    print_feature "Push files to device"
    print_feature "Pull files from device"
    print_feature "Access device shell"

    print ""
    print $"($magenta)Development Tools:($reset)"
    print_feature "Take screenshots"
    print_feature "Record screen"
    print_feature "View system logs (logcat)"

    wait_for_user

    print_header "⌨️  Navigation Controls"

    print $"($yellow)Menu Navigation:($reset)"
    print "  ↑/↓ or j/k    - Move up/down in menu"
    print "  Enter         - Execute selected command"
    print "  q/Esc         - Quit application"
    print "  Ctrl+C        - Force quit"
    print ""
    print $"($yellow)During Command Execution:($reset)"
    print "  Any key       - Return to menu after viewing results"
    print "  Esc           - Cancel execution (if supported)"

    wait_for_user

    print_header "🔧 Prerequisites Check"

    # Check if Rust/Cargo is installed
    if (which cargo | is-empty) {
        print $"($red)❌ Rust/Cargo not found. Please install from: https://rustup.rs/($reset)"
        exit 1
    }
    print_feature "Rust/Cargo is installed"

    # Check that the ADB server is reachable (droidkraft uses adb_client, not the CLI)
    print_feature "ADB communication via adb_client (no CLI required)"
    print $"  ($cyan)Connects directly to ADB server on 127.0.0.1:5037($reset)"
    print $"  ($cyan)Make sure the ADB server is running: adb start-server($reset)"

    print ""
    wait_for_user

    print_header "🎬 Starting DroidKraft"

    print $"($blue)Building application...($reset)"
    let build = (do { run-external "cargo" "build" "--release" "--quiet" } | complete)
    if $build.exit_code != 0 {
        print $"($red)❌ Build failed!($reset)"
        exit 1
    }
    print_feature "Build successful!"

    print ""
    print $"($green)🚀 Launching DroidKraft...($reset)"
    print $"($yellow)   Use Ctrl+C or 'q' to exit($reset)"
    print $"($yellow)   Explore the menu and try different commands!($reset)"
    print ""

    sleep 2sec

    run-external "./target/release/droidkraft-tui"

    print_header "🎉 Demo Complete"

    print $"($green)Thank you for trying DroidKraft!($reset)"
    print ""
    print $"($blue)What's next?($reset)"
    print "• Customize the menu items in src/menu.rs"
    print "• Add your own ADB commands and shortcuts"
    print "• Contribute to the project on GitHub"
    print "• Share feedback and suggestions"
    print ""
    print $"($cyan)Happy Android development! 🤖✨($reset)"
    print ""
}
