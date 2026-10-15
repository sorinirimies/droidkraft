# droidkraft-fastboot

Fastboot for Rust: a native, pure-Rust USB (and TCP) implementation with an automatic fallback to the `fastboot` binary.

```toml
[dependencies]
droidkraft-fastboot = "0.1"
```

```rust
use droidkraft_fastboot::{FactoryOptions, FastbootManager};

let fb = FastbootManager::new();                 // first device in fastboot mode
println!("{:?}", fb.variables()?.get("product"));
fb.flash("boot", "boot.img")?;                   // sparse-splits big images
fb.reboot()?;

// Whole factory image (what flash-all.sh does), with progress:
fb.flash_factory("husky-ap1a.zip", &FactoryOptions::new().wipe(true), |p| {
    println!("[{}/{}] {}", p.step_index, p.total_steps, p.label);
})?;
```

## Features

- `getvar` / `getvar all` (parsed into a map), `flash`, `erase`, `reboot*`, `flashing lock|unlock`, A/B slot switch, raw `oem ...` commands.
- Large images: split to `max-download-size`, raw → sparse conversion.
- Factory images: bootloader, radio, A/B slots and logical partitions, optional wipe.
- Transports: USB (`nusb`) and TCP (`tcp:HOST[:PORT]`, default 5554 – Cuttlefish/emulators).
- Backend selection: `FastbootManager::with_backend` or `DROIDKRAFT_FASTBOOT_BACKEND=auto|native|binary`.
- Destructive commands are flagged by `FastbootCommand::is_destructive()`.

## Status

Protocol logic is covered by tests against fake transports. It has not been exercised on every device: please report problems.

## USB permissions

- **Linux:** add a udev rule for your device (or run as root).
- **Windows:** install the Google USB driver / WinUSB for the bootloader interface.
- **macOS:** works out of the box.

Licensed under MIT.
