# droidkraft-core

Shared, framework-free core logic for [DroidKraft](https://github.com/sorinirimies/droidkraft).

This crate offers a clean, reusable API for communicating with Android devices
over the embedded Android Debug Bridge (ADB) and fastboot. It has **no** GUI or
TUI dependencies and is consumed by both the Ratatui TUI and the GPUI GUI
frontends.

## Features

- **`client`** — [`AdbManager`]: ADB server connection + shell execution
- **`features::device`** — device enumeration and live status snapshots
- **`features::packages`** — list / install / uninstall / clear packages
- **`features::system`** — battery, memory, CPU, properties, network
- **`features::screen`** — screenshots, resolution, PNG frame capture (for streaming)
- **`features::logcat`** — log parsing, filtering, stats, and a background streaming engine
- **`features::fastboot`** — bootloader / fastboot operations
- **`features::flash`** — reboot targets and root-detection toolkit
- **`features::rom`** — custom-ROM catalog (7 projects incl. **GrapheneOS**),
  device-compatibility filtering, live build resolution (LineageOS + GrapheneOS
  APIs, OTA JSON), verified downloads, and a consent-gated flash orchestrator
  with both **sideload** and fastboot **factory-image** install paths
- **`features::shell`** — the typed `AdbCommand` enum
- **`color`** — framework-neutral `Rgb`, stable tag-colour hashing, and log-level
  colours shared by both frontends
- **`utils`** — text wrapping, POSIX shell single-quoting, clipboard, `/proc` parsing

## Example

```rust,no_run
use droidkraft_core::AdbManager;

let mut adb = AdbManager::new();
let status = adb.fetch_device_status();
if status.is_connected() {
    println!("Model: {}  Android {}", status.model, status.android_version);
}
```

## License

MIT
