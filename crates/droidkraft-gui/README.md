# droidkraft-gui

A [Zed GPUI](https://www.gpui.rs/) desktop frontend for DroidKraft, built on the
shared [`droidkraft-core`](../droidkraft-core) library.

## Features

- **Dashboard** — live device status: model, Android version, battery, RAM, CPU
  load, and a list of all attached devices with online/authorised indicators.
- **Live Logs** — realtime logcat streaming (via the core `LogcatStream`
  engine) with a log-level filter, start/stop, and clear.
- **Commands** — one-click device commands grouped by category (Device, System,
  Packages, Screen, Power); output is shown in a scrollable pane.
- **Flash & Root** — reboot targets (system / bootloader / recovery / sideload /
  fastbootd), root detection, remount, and fastboot operations (unlock / lock /
  wipe / reboot / getvar) with destructive-action styling.
- **Screen** — live screen mirroring: a background thread captures `screencap`
  PNG frames which are rendered in the window, with an fps / frame counter.

All device I/O runs on background threads (a command worker + the logcat and
screen capture threads), so the UI thread never blocks.

## Architecture

| Module | Responsibility |
|--------|----------------|
| `app` | The root gpui `Render` view, panels, and per-tick data pump |
| `worker` | Background thread owning an `AdbManager`; request/response channels |
| `screen` | Background screen-capture backend (framework-free, unit-tested) |
| `commands` | Framework-free catalogue of button commands (unit-tested) |
| `theme` | Colour palette + log-level / tag colours |

`commands` and `screen` have **no** gpui dependency and are covered by unit
tests; `worker` depends only on `droidkraft-core`.

## Building

GPUI is consumed as a **git dependency** pinned to the Zed repository
(`rev = "60314a7"`) — it is not published on crates.io. The GUI is an **opt-in**
workspace member, so the default `cargo build` / `cargo test` (core + TUI) do
**not** require the gpui toolchain. Build it explicitly:

```sh
cargo build -p droidkraft-gui
cargo run  -p droidkraft-gui
```

### macOS prerequisite

GPUI renders with Metal, so its build compiles `.metal` shaders using the
**full Xcode** toolchain (the Command Line Tools alone are not enough). One-time
setup:

```sh
# Point the toolchain at a full Xcode install and accept its licence:
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
sudo xcodebuild -license accept
```

Without this you will see `xcrun: error: unable to find utility "metal"` or an
Xcode-licence error during the `gpui_macos` build. (This is an environment
requirement of gpui, independent of DroidKraft.)
