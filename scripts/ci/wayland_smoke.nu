#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# DroidKraft — Start each desktop app on Wayland
# ──────────────────────────────────────────────────────────────────────────────
# Runs every gui binary in BINS (scripts/project.nu) on a headless Weston with
# DISPLAY unset, so the app can only talk Wayland, and fails unless the app is
# still running after --seconds. Needs weston and a Vulkan driver (lavapipe from
# mesa-vulkan-drivers on a GPU-less runner). Skipped off Linux.
#
# Usage: nu scripts/ci/wayland_smoke.nu [--build] [--seconds 8]
# ──────────────────────────────────────────────────────────────────────────────

use ../project.nu [BINS]

# The Wayland socket the headless compositor listens on.
export const SOCKET = "wayland-smoke"

# The gui rows of `bins` (a BINS table).
export def gui_bins [bins: table]: nothing -> table {
    $bins | where {|b| ($b.kind? | default "") == "gui" }
}

# The command line that runs `binary` for `seconds` with Wayland only: `env -u DISPLAY`
# hides X11, so a Wayland failure cannot fall back to XWayland.
export def app_command [binary: string, runtime: string, seconds: int]: nothing -> list<string> {
    [
        timeout $"($seconds)s" env -u DISPLAY
        $"WAYLAND_DISPLAY=($SOCKET)" $"XDG_RUNTIME_DIR=($runtime)" "WINIT_UNIX_BACKEND=wayland"
        $binary
    ]
}

# What `timeout`'s exit code says about the app: 124 means it ran until stopped.
export def verdict [code: int]: nothing -> string {
    match $code {
        124 => "ran"
        0 => "quit"
        _ => "crashed"
    }
}

# Wait until the compositor's socket exists, up to `tries` tenths of a second.
def wait_socket [path: string, tries: int] {
    for _ in 1..$tries {
        if ($path | path exists) { return }
        sleep 100ms
    }
    error make {msg: $"weston did not create ($path)"}
}

def main [--build, --seconds: int = 8] {
    let guis = gui_bins $BINS
    if $nu.os-info.name != "linux" or ($guis | is-empty) {
        print "wayland=skipped"
        return
    }
    if $build { ^cargo build --locked ...($guis | each {|g| [-p $g.crate] } | flatten) }
    let runtime = mktemp -d
    ^chmod 700 $runtime
    let log = $runtime | path join weston.log
    let limit = $seconds * ($guis | length) + 30
    let weston = job spawn {
        with-env {XDG_RUNTIME_DIR: $runtime} {
            ^timeout $"($limit)s" weston --backend=headless --socket=($SOCKET) --idle-time=0 out+err> $log
        }
    }
    wait_socket ($runtime | path join $SOCKET) 100
    mut failed = []
    for g in $guis {
        let cmd = app_command $"target/debug/($g.bin)" $runtime $seconds
        let r = run-external $cmd.0 ...($cmd | skip 1) | complete
        let v = verdict $r.exit_code
        print $"($g.bin): ($v) on Wayland \(exit ($r.exit_code)\)"
        if $v != "ran" {
            print ($r.stderr | lines | last 40 | str join "\n")
            $failed = $failed ++ [$g.bin]
        }
    }
    job kill $weston
    if ($failed | is-not-empty) {
        print $"weston log:\n(open --raw $log | lines | last 20 | str join "\n")"
        error make {msg: $"did not stay up on Wayland: ($failed | str join ', ')"}
    }
}
