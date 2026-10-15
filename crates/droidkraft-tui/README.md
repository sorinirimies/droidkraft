# droidkraft-tui

Terminal UI for Android development over ADB. Mirrors the DroidKraft GUI: a nav rail plus five panels, themeable, mouse-aware.

## Features

Everything is organised **per device**.

- **Devices home** (`d`): fleet summary + one card per device (battery, RAM/CPU/storage gauges, sparkline). Click a card to open its workspace.
- **Device workspace** with tabs `Overview | Logs | Commands | Flash & Root | Screen`, all scoped to that device and with state kept per device:
  - **Overview**: battery, memory/CPU history charts, storage pie, thermal, display, network, system info.
  - **Logs**: live logcat with level/tag/text filters, pause, clear, scroll.
  - **Commands**: ADB / dev-tool commands with scrollable output.
  - **Flash & Root**: fastboot, root detection, reboot targets, ROM flashing with progress.
  - **Screen**: live half-block mirror with touch/keys/text, **screen recording** (`R`, REC badge + timer, bit-rate / max height / touches options, finished-file list), screenshots (`S`), external window.
- Themes (`t`), help (`?`), clickable nav rail listing every device.

## Keys

Global: `d` devices home · `1`–`9` open nth device · `[` / `]` previous/next device · `Tab` / `Shift-Tab` cycle tabs · `o l c f s` jump to tab · `t` theme · `r` refresh · `>` / `<` graphs faster / slower (0.5 s–10 s) · `?` help · `q` / `Ctrl-C` quit.

| Tab | Keys |
|---|---|
| Logs / Commands / Flash | see the status bar hints |
| Screen | `R` record · `S` screenshot · `B`/`H`/`T` record options · `O` open folder · `b` Back · `h` Home · `r` Recents · `s` start/stop mirror · `p` Power · `+`/`-` volume · `w` external window · `Ctrl-G` input capture |

Screen mouse: left press/drag/release = touch · right click = Back · middle click = Home · wheel = swipe.

## CLI

`droidkraft-tui [devices | info [--serial S] [--json] | screenshot [PATH] | mirror | --help | --version] [--theme NAME] [--graphics MODE]`

## Mirror modes

1. **In-terminal** (Screen panel): real image rendering (Kitty / iTerm2 / Sixel graphics protocols ; Kitty uses a built-in direct backend (PNG chunks placed in place with a fixed image id, no unicode placeholders, deleted on exit); iTerm2/Sixel use `ratatui-image`), auto-detected at startup; falls back to half-blocks (`▀`, 2 px per cell, pixelated) when none is available. The active protocol is shown in the panel title (`Screen · kitty`). Frames are resized/encoded on a background thread and only re-encoded when the frame changes. Override with `--graphics auto|kitty|iterm2|sixel|halfblocks` or `DROIDKRAFT_GRAPHICS` (flag wins). Supported terminals: Kitty, WezTerm, iTerm2, Ghostty, foot, and other Sixel terminals (e.g. xterm -ti vt340, mlterm, Windows Terminal ≥1.22).
2. **External window** (scrcpy style): press `w`, or run `droidkraft-tui mirror [--serial S] [--size WxH]` / `droidkraft-tui --mirror`. Spawns the GUI binary `droidkraft --mirror --serial S [--size WxH]` detached. It is looked up next to the running executable, then on `PATH`. If missing: `cargo build -p droidkraft --release` or `cargo install --path crates/droidkraft-gui`.

## CLI

```
droidkraft-tui                          launch the TUI
droidkraft-tui devices                  list serial / state / model
droidkraft-tui screenshot [PATH] [--serial S]   save PNG (default droidkraft-screenshot.png)
droidkraft-tui mirror [--serial S] [--size WxH] external mirror window
droidkraft-tui --theme NAME             start with a theme
droidkraft-tui --graphics MODE          mirror renderer: auto|kitty|iterm2|sixel|halfblocks
droidkraft-tui --query [--last N] [--level L] [--tag T] [--grep P]   logcat as JSON lines
droidkraft-tui --help | --version
```

## Requirements

- A running ADB server (`adb start-server`, port 5037) and a device with USB debugging enabled.
- `ffmpeg` on `PATH` for the Screen stream (`brew install ffmpeg`, `sudo apt install ffmpeg`).
- Live touch (`DOWN/MOVE/UP`) needs Android 10+.
- External window needs the `droidkraft` GUI binary.

## Libraries

ratatui 0.30 · crossterm 0.29 · tokio 1 · color-eyre 0.6 · adb_client 2.1 · tui-nuances 0.1 (themes) · tui-chart 0.1 · tui-piechart 1.1 · tui-spinner 0.5 · tui-slider 0.4 · tui-checkbox 0.5 · tui-file-explorer 3 · image 0.25 · regex 1 · serde / serde_json · workspace crates `droidkraft-core`, `droidkraft-mirror`, `droidkraft-fastboot`.

## Develop

```
cargo clippy -p droidkraft-tui --all-targets
cargo test -p droidkraft-tui
```
