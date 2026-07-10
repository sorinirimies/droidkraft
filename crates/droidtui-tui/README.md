# droidtui

A beautiful Terminal User Interface (TUI) for Android development — ADB
commands, a full-featured live logcat viewer, a device dashboard, and a
flash/root toolkit. Built with Rust, [Ratatui](https://ratatui.rs), and the
pure-Rust [`adb_client`](https://crates.io/crates/adb_client) crate (no Android
SDK required).

This is the terminal frontend of the [DroidTUI](https://github.com/sorinirimies/droidtui)
workspace; all device logic lives in the reusable
[`droidtui-core`](https://crates.io/crates/droidtui-core) library.

## Install

```sh
cargo install droidtui
```

## Run

```sh
droidtui            # launch the TUI
droidtui --help     # CLI options (incl. JSON logcat streaming)
```

## Highlights

- **Live logcat** — streaming with regex/find/exclude/tag/PID/level filters,
  per-tag colours, stack-trace folding, bookmarks, JSON pretty-printing,
  clipboard copy, and TXT/JSONL export.
- **Device dashboard** — model, Android version, battery, RAM and CPU stats with
  a live device selector.
- **Command menu** — 40+ typed ADB commands across device, packages, system,
  network, and a bootloader/flash & root toolkit.

See the [main README](https://github.com/sorinirimies/droidtui) for full
documentation and keybindings.

## License

MIT
