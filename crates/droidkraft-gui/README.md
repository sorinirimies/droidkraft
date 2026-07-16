# droidkraft

The **DroidKraft** desktop GUI — an Android device monitor built with
[Zed GPUI](https://www.gpui.rs/), on top of the shared
[`droidkraft-core`](https://crates.io/crates/droidkraft-core) library.

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

## Install

```sh
cargo install droidkraft
```

> **Build prerequisites.** GPUI renders with the GPU, so building requires the
> platform graphics toolchain:
> - **macOS** — the full **Xcode** toolchain with the Metal compiler
>   (`xcodebuild -downloadComponent MetalToolchain`), not just the Command Line
>   Tools.
> - **Linux** — the usual GPUI system libraries (Wayland/X11, `libxkbcommon`,
>   Vulkan, `fontconfig`, …).
>
> The GUI is an **opt-in** workspace member, so the repo's default
> `cargo build` / `cargo test` (core + TUI) don't require this toolchain.

## Run

```sh
droidkraft
```

## Architecture

| Module | Responsibility |
|--------|----------------|
| `app` | The root gpui `Render` view, panels, and per-tick data pump |
| `worker` | Background thread owning an `AdbManager`; request/response channels |
| `screen` | Background screen-capture backend (framework-free, unit-tested) |
| `commands` | Framework-free catalogue of button commands (unit-tested) |
| `theme` | Colour palette (log-level / tag colours shared via `droidkraft-core`) |

`commands` and `screen` have no gpui dependency and are covered by unit tests;
`worker` depends only on `droidkraft-core`.

## License

MIT
