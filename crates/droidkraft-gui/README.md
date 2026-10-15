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
- **Custom ROM flasher** — device-aware ROM catalog (LineageOS live API + seed
  catalog for others), verified downloads (SHA-256, progress), and a
  consent-gated flash orchestrator (unlock → wipe → flash recovery → sideload →
  reboot). Destructive steps require an explicit "⚠ Confirm & run" click.

All device I/O runs on background threads (a command worker + the logcat and
screen capture threads), so the UI thread never blocks.

## Responsive layout

Breakpoints come from the window size (`src/layout.rs`, unit-tested): compact (<760 px), medium (<1100 px), wide. The nav rail goes full → shorter → icon-only; device cards reflow 1..N columns and drop sparklines, gauges and quick actions as they shrink; workspace chips/tabs shorten; Overview panels reflow 3 → 2 → 1 columns and hide charts when small; the Screen tab's recording panel moves below the mirror on narrow windows. Minimum window size is 640×420.

## Detach / attach the mirror

In the **Screen** panel, **⇱ Detach** (or `⌘D` / `Ctrl+D`) moves the live mirror
into its own resizable window titled "DroidKraft — <model> (<serial>)". The same
stream keeps running (no restart, Stop/Start state preserved). The main panel
shows a placeholder with **Attach** and **Focus window** buttons. Re-embed with
**Attach**, the detached window's **⇲ Attach** button, `⌘D`/`Ctrl+D`, or by
closing the detached window (`⌘W`/`Ctrl+W`). Closing the main window closes the
detached one too. `⌘/Ctrl+C` etc. are never forwarded to the device.

## Mirror-only mode (scrcpy-like)

```sh
droidkraft --mirror [--serial SERIAL] [--size WxH] [--title TITLE]
droidkraft --help
```

Opens only the mirror window (viewport + thin control strip: zoom, Back/Home,
Start/Stop), auto-starts on `SERIAL` (or the first connected device) and quits
when the window closes.

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
