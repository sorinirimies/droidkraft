# droidtui-core

Shared, framework-free core logic for [DroidTUI](https://github.com/sorinirimies/droidtui).

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
- **`features::shell`** — the typed `AdbCommand` enum

## Example

```rust,no_run
use droidtui_core::AdbManager;

let mut adb = AdbManager::new();
let status = adb.fetch_device_status();
if status.is_connected() {
    println!("Model: {}  Android {}", status.model, status.android_version);
}
```

## License

MIT
