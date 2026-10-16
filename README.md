# DroidKraft 🤖

[![Crates.io](https://img.shields.io/crates/v/droidkraft-tui.svg)](https://crates.io/crates/droidkraft-tui)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Release](https://github.com/sorinirimies/droidkraft/actions/workflows/release.yml/badge.svg)](https://github.com/sorinirimies/droidkraft/actions/workflows/release.yml)
[![CI](https://github.com/sorinirimies/droidkraft/actions/workflows/ci.yml/badge.svg)](https://github.com/sorinirimies/droidkraft/actions/workflows/ci.yml)
[![TUI installs](https://img.shields.io/crates/d/droidkraft-tui?label=droidkraft-tui%20installs&logo=rust&color=orange)](https://crates.io/crates/droidkraft-tui)
[![GUI installs](https://img.shields.io/crates/d/droidkraft?label=droidkraft%20installs&logo=rust&color=orange)](https://crates.io/crates/droidkraft)
[![Audit](https://github.com/sorinirimies/droidkraft/actions/workflows/security.yml/badge.svg)](https://github.com/sorinirimies/droidkraft/actions/workflows/security.yml)
[![Fuzz](https://github.com/sorinirimies/droidkraft/actions/workflows/fuzz.yml/badge.svg)](https://github.com/sorinirimies/droidkraft/actions/workflows/fuzz.yml)

**Everything for your Android devices, in pure Rust.** A device-centric toolkit
with a **terminal UI**, a **GPU desktop app** and reusable **libraries**. No
Android SDK, no `scrcpy`, no Java: ADB is spoken natively through
[`adb_client`](https://crates.io/crates/adb_client), fastboot natively over USB.

- 📱 **Per-device workspaces**: every connected device gets its own Overview, Logs, Commands, Flash & Root and Screen tabs.
- 📊 **Live statistics**: battery, memory, CPU, storage, thermals, display, network and system info with history charts.
- 📺 **Screen mirroring**: low-latency H.264 mirror with touch, swipe, keyboard and scroll input; sharp image rendering in Kitty/Ghostty/WezTerm/iTerm2/Sixel terminals.
- 🎥 **Screen recording** to MP4 on your computer, plus screenshots.
- 📜 **Logcat** with level/tag/text filters, pause, search and export.
- ⚡ **Flash & Root toolkit**: native fastboot (unlock, wipe, sparse flashing, factory images), ADB sideload, custom-ROM catalog with verified downloads.
- 🪟 **Responsive** layouts in both frontends: they adapt from tiny terminals/windows to ultra-wide ones.

## Previews 🎬

> Preview GIFs are stored with **[Git LFS](https://git-lfs.com/)**. Run
> `git lfs install` before cloning (or `git lfs pull` afterwards) to fetch them.

### Terminal UI

| Devices home | Device overview |
|---|---|
| ![TUI devices](docs/previews/tui-devices.gif) | ![TUI overview](docs/previews/tui-overview.gif) |

| Live logcat |
|---|
| ![TUI logs](docs/previews/tui-logs.gif) |

### Desktop app (GPUI)

![GUI overview](docs/previews/gui-overview.gif)

## Installation 🔧

```bash
cargo install droidkraft-tui      # terminal UI
cargo install droidkraft          # desktop app (needs the GPU toolchain, see below)
```

From source:

```bash
git clone https://github.com/sorinirimies/droidkraft.git
cd droidkraft
cargo run --release -p droidkraft-tui     # TUI
cargo run --release -p droidkraft         # GUI
```

**Prerequisites**

- A running ADB server (`adb start-server`). The `adb` binary is only needed to start it; all traffic uses `adb_client`.
- A device with USB debugging enabled.
- `ffmpeg` on `PATH` for screen mirroring and recording (`brew install ffmpeg`).
- GUI only: Xcode's Metal toolchain on macOS, or Wayland/X11 + Vulkan libs on Linux.
- Fastboot over USB: on Linux add a udev rule for your device; on Windows install the Google USB driver.

## Platform support 🖥️

| | Linux x86_64 | Linux aarch64 | macOS | Windows x86_64 |
|---|:-:|:-:|:-:|:-:|
| **TUI** prebuilt | ✅ | ✅ | ✅ arm64 · x86_64 | ✅ `.zip` · installer |
| **TUI** `cargo install` | ✅ | ✅ | ✅ | ✅ |
| **GUI** prebuilt | ✅ | — | — | — |
| **GUI** `cargo install` | ✅ | ✅ | ✅ | ⚠️ builds on Windows only |

- **Linux:** AppImage, Flatpak, Snap, `.deb`, `.rpm`, `.tar.gz`, AUR, Nix, Homebrew.
- **macOS:** prebuilt TUI tarballs (arm64 + x86_64, cross-compiled from Linux with zig, best-effort) and a Homebrew formula. The GUI needs `cargo install` and Xcode's Metal toolchain.
- **Windows:** prebuilt TUI zip and installer (adds `droidkraft-tui` to PATH), cross-compiled with `cargo-xwin`; Scoop, winget and Chocolatey manifests are attached. The GUI can only be built natively on Windows (gpui compiles its shaders with `fxc.exe`).
- **Fastboot:** Linux needs a udev rule; Windows needs the Google USB driver.

## Downloads 📥

Every release attaches these, all built on Linux CI:

| Platform | Format | GUI | TUI |
|---|---|:-:|:-:|
| Linux | AppImage | ✅ | ✅ |
| Linux | Flatpak (one per app) | ✅ | ✅ |
| Linux | Snap (one per app) | ✅ | ✅ |
| Debian · Ubuntu · Mint | `.deb` | ✅ x86_64 | ✅ x86_64 · arm64 |
| Fedora · openSUSE · RHEL | `.rpm` | ✅ x86_64 | ✅ x86_64 · aarch64 |
| Arch | `.pkg.tar.zst` + `PKGBUILD` (`droidkraft-bin`, `droidkraft-tui-bin`) | ✅ x86_64 | ✅ x86_64 · aarch64 |
| Alpine | `.apk` (static musl) | — | ✅ x86_64 |
| Linux | static musl `.tar.gz` (any distro) | — | ✅ x86_64 · aarch64 |
| Linux | `.tar.gz` (TUI) | ✅ x86_64 | ✅ x86_64 · aarch64 |
| Windows | `.zip`, NSIS installer `-setup.exe` | — | ✅ x86_64 |
| Windows | Scoop · winget · Chocolatey manifests | — | ✅ x86_64 |
| macOS | `.tar.gz` (arm64, x86_64), Homebrew formula | — | ✅ |
| macOS | `cargo install` | ✅ | ✅ |

```sh
cargo binstall droidkraft-tui                  # prebuilt TUI, no compile
sudo apt install ./droidkraft-tui_*.deb         # Debian / Ubuntu
sudo dnf install ./droidkraft-tui-*.rpm          # Fedora / openSUSE
flatpak install --user droidkraft-*.flatpak     # any distro
sudo snap install --dangerous droidkraft_*.snap && sudo snap connect droidkraft:raw-usb
chmod +x droidkraft-*.AppImage && ./droidkraft-*.AppImage
makepkg -si                                     # Arch: rename PKGBUILD-droidkraft-bin (or -tui-bin) to PKGBUILD
nix run github:sorinirimies/droidkraft          # Nix flake (TUI; `#droidkraft` for the GUI)
brew install sorinirimies/tap/droidkraft-tui    # Linuxbrew formula (droidkraft-tui.rb on the release)
brew install sorinirimies/tap/droidkraft       # GUI formula (droidkraft.rb on the release)
```

## Usage 🎮

```bash
droidkraft-tui                       # open the TUI
droidkraft-tui devices               # list devices
droidkraft-tui info [--serial S] [--json]   # full device details
droidkraft-tui screenshot [PATH]     # save a PNG
droidkraft-tui mirror                # open the mirror window
droidkraft-tui --graphics kitty      # auto | kitty | iterm2 | sixel | halfblocks
droidkraft --mirror --serial S       # standalone GUI mirror window
```

Key bindings (TUI): `d` devices home · `1`-`9` open device · `[` `]` previous/next
device · `Tab` cycle tabs · `o l c f s` jump to tab · `R` record · `S` screenshot ·
`t` theme · `?` help. Everything is also clickable with the mouse. See the
[TUI README](crates/droidkraft-tui/README.md) and [GUI README](crates/droidkraft-gui/README.md).

## Workspace layout 🏗️

| Crate | Kind | Description |
|---|---|---|
| [`droidkraft-core`](crates/droidkraft-core) | library | Framework-free ADB API: device details & history, packages, system, logcat engine, ROM catalog/flasher, root toolkit. |
| [`droidkraft-fastboot`](crates/droidkraft-fastboot) | library | Pure-Rust fastboot: USB (`nusb`) and TCP transports, sparse images, factory-image flashing, binary fallback. |
| [`droidkraft-mirror`](crates/droidkraft-mirror) | library | Frontend-neutral screen mirror + recorder (H.264 → frames, input mapping, zoom/layout). |
| [`droidkraft-tui`](crates/droidkraft-tui) | binary | Ratatui terminal app. |
| [`droidkraft`](crates/droidkraft-gui) | binary | GPUI desktop app (opt-in workspace member). |

```
droidkraft-core ─┬─ droidkraft-fastboot
                 ├─ droidkraft-mirror ─┬─ droidkraft-tui
                 └─────────────────────┴─ droidkraft (GUI)
```

The GUI is not in `default-members` (it needs the platform GPU toolchain), so
`cargo build` / `cargo test` cover everything else; use `-p droidkraft` for the GUI.

### Using the libraries

```rust,no_run
use droidkraft_core::{collect_device_details, AdbManager};

let mut adb = AdbManager::new();
let status = adb.fetch_device_status();
for dev in &status.devices {
    let (d, _cpu) = collect_device_details(&dev.serial, None);
    println!("{} · Android {} · {}%", d.display_name(), d.system.android_release, d.battery.level);
}
```

## Development 🛠️

```bash
cargo test --workspace            # tests (GUI excluded from default members: add -p droidkraft)
cargo clippy --workspace --all-targets
just --list                       # helper recipes
just previews                     # re-record the preview GIFs (needs vhs + a device)
```

## License

MIT. See [LICENSE](LICENSE).
