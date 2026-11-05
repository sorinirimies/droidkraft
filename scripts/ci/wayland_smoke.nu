#!/usr/bin/env nu
# ──────────────────────────────────────────────────────────────────────────────
# DroidKraft — Start each desktop app on Wayland
# ──────────────────────────────────────────────────────────────────────────────
# Runs every gui binary in BINS (scripts/project.nu) on a headless GNOME
# compositor (mutter) with DISPLAY unset, so the app can only talk Wayland, and
# fails unless the app is still running after --seconds. GNOME is the strict
# case: it has a seat but no server-side decorations, so apps draw their own
# frame. As root (Gitea's job container) it all runs as `nobody`. Needs mutter,
# dbus and a Vulkan driver (lavapipe from mesa-vulkan-drivers on a GPU-less
# runner). Skipped off Linux.
#
# Usage: nu scripts/ci/wayland_smoke.nu [--build] [--seconds 8]
# ──────────────────────────────────────────────────────────────────────────────

use ../project.nu [BINS]

# The Wayland socket the headless compositor listens on.
export const SOCKET = "wayland-smoke"

# The uid and gid of `nobody`, which runs the smoke when the job runs as root.
const NOBODY = 65534

# The gui rows of `bins` (a BINS table).
export def gui_bins [bins: table]: nothing -> table {
    $bins | where {|b| ($b.kind? | default "") == "gui" }
}

# The headless GNOME compositor on SOCKET, stopped after `seconds`.
export def compositor_command [seconds: int]: nothing -> list<string> {
    [
        timeout $"($seconds)s" mutter --headless --wayland --no-x11
        --virtual-monitor 1280x800 --wayland-display $SOCKET
    ]
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

# `cmd` as run by `uid`: as root (Gitea's job container), as `nobody` with HOME at
# `home` instead: a desktop app may refuse an interactive start as root (FlashKraft does).
export def as_user [cmd: list<string>, uid: int, home: string]: nothing -> list<string> {
    if $uid != 0 { return $cmd }
    [setpriv $"--reuid=($NOBODY)" $"--regid=($NOBODY)" --clear-groups env $"HOME=($home)" ...$cmd]
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
def wait_socket [path: string, tries: int, log: string] {
    for _ in 1..$tries {
        if ($path | path exists) { return }
        sleep 100ms
    }
    print (open --raw $log)
    error make {msg: $"the compositor did not create ($path)"}
}

def main [--build, --seconds: int = 8, --in-session] {
    let guis = gui_bins $BINS
    if $nu.os-info.name != "linux" or ($guis | is-empty) {
        print "wayland=skipped"
        return
    }
    if $build { ^cargo build --locked ...($guis | each {|g| [-p $g.crate] } | flatten) }
    # mutter and the apps (portals, settings) share a session bus; as root the bus,
    # the compositor and the apps all run as `nobody`, which can not use root's bus.
    if not $in_session {
        let uid = ^id -u | into int
        let home = mktemp -d
        if $uid == 0 { ^chown $"($NOBODY):($NOBODY)" $home }
        let cmd = as_user [dbus-run-session -- nu $env.CURRENT_FILE --seconds $"($seconds)" --in-session] $uid $home
        exec $cmd.0 ...($cmd | skip 1)
    }
    let runtime = mktemp -d
    ^chmod 700 $runtime
    let log = $runtime | path join compositor.log
    let comp = compositor_command ($seconds * ($guis | length) + 60)
    let compositor = job spawn {
        with-env {XDG_RUNTIME_DIR: $runtime} {
            run-external $comp.0 ...($comp | skip 1) out+err> $log
        }
    }
    wait_socket ($runtime | path join $SOCKET) 300 $log
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
    job kill $compositor
    if ($failed | is-not-empty) {
        print $"compositor log:\n(open --raw $log | lines | last 20 | str join "\n")"
        error make {msg: $"did not stay up on Wayland: ($failed | str join ', ')"}
    }
}
