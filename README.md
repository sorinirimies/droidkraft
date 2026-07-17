# DroidKraft 🤖

[![Crates.io](https://img.shields.io/crates/v/droidkraft-tui.svg)](https://crates.io/crates/droidkraft-tui)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Release](https://github.com/sorinirimies/droidkraft/actions/workflows/release.yml/badge.svg)](https://github.com/sorinirimies/droidkraft/actions/workflows/release.yml)
[![CI](https://github.com/sorinirimies/droidkraft/actions/workflows/ci.yml/badge.svg)](https://github.com/sorinirimies/droidkraft/actions/workflows/ci.yml)

A toolkit for Android development over the embedded ADB — a full-featured
**terminal UI**, a **GPUI desktop app**, and a reusable **core library**. Live
logcat, device dashboard, one-click ADB commands, a flash/root toolkit, and
live screen mirroring. Built with Rust and the pure-Rust `adb_client` crate
(no Android SDK required).

## Previews 🎬

> Preview GIFs are stored with **[Git LFS](https://git-lfs.com/)**. Run
> `git lfs install` before cloning (or `git lfs pull` afterwards) to fetch them.

![Quick demo](docs/previews/quickstart.gif)

| Main menu | Navigation |
|-----------|------------|
| ![Main menu](docs/previews/main_menu.gif) | ![Navigation](docs/previews/navigation_showcase.gif) |

<details><summary>More previews</summary>

![Full demo](docs/previews/full_demo.gif)

![Features](docs/previews/features_highlight.gif)

</details>

## Workspace layout 🏗️

DroidKraft is a Cargo workspace with a reusable core library and two frontends:

| Crate | Kind | Description |
|-------|------|-------------|
| [`droidkraft-core`](crates/droidkraft-core) | library (`droidkraft_core`) | Framework-free ADB & fastboot API: device info, packages, system, logcat parsing + streaming engine, flash/root toolkit, screen capture, and shared colour/text helpers. **No** GUI/TUI deps — publishable and reusable. |
| [`droidkraft-tui`](crates/droidkraft-tui) | binary (TUI) | The Ratatui terminal app, built on `droidkraft-core`. |
| [`droidkraft`](crates/droidkraft-gui) | binary (GUI) | A [Zed GPUI](https://www.gpui.rs/) desktop app: device monitor, realtime logs, one-click commands, flash/root toolkit, and live screen mirroring. |

```
droidkraft-core  ◄── droidkraft (TUI, Ratatui)
     ▲
     └────────── droidkraft (GUI, GPUI)
```

The GUI is an **opt-in** member (it needs the platform GPU toolchain — Xcode's
Metal on macOS, or Wayland/X11/Vulkan libs on Linux), so `cargo build` /
`cargo test` build only the core + TUI. Build the GUI explicitly with
`cargo build -p droidkraft` (see its [README](crates/droidkraft-gui/README.md)).

### Using the core library

```rust,no_run
use droidkraft_core::AdbManager;

let mut adb = AdbManager::new();
let status = adb.fetch_device_status();
if status.is_connected() {
    println!("{} — Android {}", status.model, status.android_version);
}
```

## Features ✨

### 📺 Live Logcat Viewer
Full-screen, real-time logcat streaming with professional-grade tooling:

- **Live streaming** via `adb_client`'s native API — no `adb` binary needed
- **Regex search** (`r` toggle) — powerful pattern matching (`Error|Exception`, `OkHttp.*failed`)
- **Find filter** (`f`) — case-insensitive substring search with match highlighting
- **Exclude filter** (`e`) — negative matching to hide noisy tags/messages
- **Tag & PID filters** (`t`, `p`) — dedicated filter fields
- **Log level filter** (`l`) — cycle minimum level V → D → I → W → E → F
- **Per-tag color hashing** — each tag gets a stable, visually distinct color
- **Stack trace folding** (`F`) — detect and fold/unfold Java/Kotlin stack traces
- **Line detail popup** (`Enter`) — inspect any line with full message, JSON formatting
- **Bookmarks** (`m`, `[`, `]`) — mark lines and jump between them
- **Copy to clipboard** (`y`) — copy selected line via `pbcopy`/`xclip`
- **Soft wrap** (`w`) — wrap long messages across multiple rows with aligned indentation
- **Compact mode** (`x`) — hide timestamp/PID columns for more message space
- **Horizontal scroll** (`←`/`→`) — scroll long messages when wrap is off
- **Auto-scroll** — sticks to bottom; manual scroll disables; `G`/`End` re-enables
- **Pause / Resume** (`Space`) — freeze the view without losing the stream
- **Live stats** — lines/sec rate and per-level counters in the status bar
- **Save logs** (`s`) — save to file with path input dialog
- **Save As…** (`S`) — browse with integrated file explorer, `Shift+S` to save in current folder
- **JSON export** — Tab in save dialog cycles TXT/JSON format (JSONL for Nushell/jq)
- **Bounded channel** — `sync_channel(10k)` with backpressure prevents OOM during bursts
- **JSON detection** — auto-detects and pretty-prints JSON in log messages with syntax coloring

### 📱 ADB Command Dashboard
- **Device panel** — live device selector with model, Android version, battery, RAM, CPU stats
- **40+ typed commands** — packages, system, network, root toolkit, bootloader & flash
- **Fastboot support** — OEM unlock/lock, wipe data, device info
- **Type-safe execution** — compile-time guarantees via `adb_client` crate

### 🚀 One-stop custom-ROM flasher (TUI + GUI)
A guided, consent-gated flow to install a custom ROM entirely from the app:
- **Device-aware catalog** — detects the connected device's codename and shows
  only compatible ROMs (LineageOS, /e/OS, crDroid, Pixel Experience, …)
- **Live build resolution** — real downloadable builds via the official
  **LineageOS download API**
- **Verified downloads** — streamed with progress and **SHA-256** integrity checks
- **Automated flash orchestration** — reboot to bootloader → (unlock) → wipe →
  flash recovery → reboot recovery → `adb sideload` the ROM → reboot, driven end
  to end from Rust
- **Safety first** — every destructive step is gated behind an explicit
  confirmation (a red "⚠ Confirm & run" button in the GUI / `Shift+F` in the TUI);
  requires an unlocked bootloader and the `fastboot`/`adb` platform-tools.
  ⚠ Flashing erases all data and can brick a device — use at your own risk.

> Shared engine: all of the above lives in
> [`droidkraft-core::features::rom`](crates/droidkraft-core) so the TUI and GUI
> behave identically.

### 🎨 Theme System
- **12 named presets** — Default, Dracula, Nord, Gruvbox Dark, Catppuccin Mocha, Tokyo Night, Solarized Dark, Moonfly, Oxocarbon, Forest, Neon, Mono
- **Global selector** (`Shift+T`) — works from any screen
- **11 colour fields** — brand, accent, success, dim, fg, sel_bg, warn, error, surface, border, key_hint

### 🖥️ CLI Query Mode
Stream logcat as JSON lines to stdout — designed for piping into Nushell, jq, or grep:

```bash
droidkraft-tui --query                          # Stream live logcat as JSONL
droidkraft-tui --query --last 500               # Dump last 500 lines
droidkraft-tui --query --level E                # Only errors
droidkraft-tui --query --tag MyApp              # Filter by tag
droidkraft-tui --query --grep "timeout"         # Filter by message
droidkraft-tui --query | nu -c 'lines | each { from json } | where level == "Error"'
```

### 📂 Nushell Recipe Scripts
Pre-built analysis scripts in `scripts/logcat/`:

| Script | What it does |
|--------|-------------|
| `top_tags.nu` | Rank tags by frequency (top 20) |
| `error_summary.nu` | Group Error/Fatal by tag with sample messages |
| `timeline.nu` | Log volume + error count per second |
| `find_crashes.nu` | Detect Fatal entries, ANRs, exceptions |
| `filter_json.nu` | Extract and pretty-print JSON payloads from messages |

```bash
droidkraft-tui --query --last 5000 > logcat.jsonl
nu scripts/logcat/top_tags.nu logcat.jsonl
```

## Installation 🔧

### From crates.io

```bash
cargo install droidkraft-tui
```

### From source

```bash
git clone https://github.com/sorinirimies/droidkraft.git
cd droidkraft
cargo install --path .
```

### Prerequisites

- **ADB server** running (`adb start-server`) — the `adb` binary is only needed to start the server; all commands use the pure-Rust `adb_client` crate
- A connected Android device with USB debugging enabled

## Usage 🎮

```bash
droidkraft-tui          # Launch the TUI
droidkraft-tui --query   # CLI mode — stream logcat as JSON
droidkraft-tui --help    # Show all options
```

### Key Bindings — Main Menu

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate menu |
| `Tab` / `Shift+Tab` | Jump between sections |
| `Enter` | Execute selected command |
| `L` | Open Live Logcat |
| `T` | Open Theme Selector |
| `d` | Cycle connected device |
| `r` | Refresh device info |
| `q` / `Esc` | Quit |

### Key Bindings — Logcat Viewer

#### Navigation
| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Scroll up/down |
| `PgUp` / `PgDn` | Scroll 20 lines |
| `g` / `G` or `Home`/`End` | Jump to top / bottom |
| `←` / `→` | Horizontal scroll (when wrap off) |
| `0` | Reset horizontal scroll |

#### Filters
| Key | Action |
|-----|--------|
| `f` | Find (search filter) |
| `e` | Exclude filter (negative match) |
| `t` | Tag filter |
| `p` | PID filter |
| `l` | Cycle log level (V→D→I→W→E→F) |
| `r` | Toggle regex mode |

#### Actions
| Key | Action |
|-----|--------|
| `Enter` | Line detail popup (with JSON formatting) |
| `y` | Copy line to clipboard |
| `m` | Toggle bookmark |
| `[` / `]` | Jump to prev/next bookmark |
| `F` | Fold/unfold stack trace |
| `w` | Toggle soft wrap |
| `x` | Toggle compact mode |
| `Space` | Pause / resume |
| `c` | Clear all entries |
| `s` | Save logs |
| `S` | Save As… (file browser) |
| `q` / `Esc` | Close logcat |

#### Save Dialog
| Key | Action |
|-----|--------|
| `Enter` | Save to typed path |
| `S` | Save As… (open file browser) |
| `Tab` | Cycle format: TXT all → TXT filtered → JSON all → JSON filtered |
| `Esc` | Cancel |

#### File Browser (Save As…)
| Key | Action |
|-----|--------|
| `↑`/`↓` | Navigate |
| `Enter` / `l` | Open directory / select file |
| `h` / `←` / `Backspace` | Go to parent |
| `Shift+S` | Save Here (into current directory) |
| `/` | Incremental search |
| `n` | Create new folder |
| `.` | Toggle hidden files |
| `s` | Cycle sort mode |
| `Esc` | Back to path input |

## Architecture 🏗️

DroidKraft follows an **Elm-like architecture** with clear separation of concerns:

```
┌── Model ──────────────────────┐
│ app state, menu, logcat,      │
│ device status, theme          │
├── Message ────────────────────┤
│ all possible state changes    │
├── Update ─────────────────────┤
│ message → state transitions   │
├── View ───────────────────────┤
│ model → terminal rendering    │
├── Event ──────────────────────┤
│ keyboard, tick → messages     │
└───────────────────────────────┘
```

### Project Structure

```
droidkraft/
├── src/
│   ├── main.rs       # Entry point + CLI query mode
│   ├── app.rs        # Event loop, key → message mapping
│   ├── model.rs      # All application state
│   ├── view.rs       # UI rendering (ratatui)
│   ├── update.rs     # State transitions
│   ├── message.rs    # Message enum
│   ├── event.rs      # Async event handling
│   ├── menu.rs       # Command menu widget
│   ├── adb.rs        # ADB client abstraction
│   ├── fastboot.rs   # Fastboot command support
│   ├── logcat.rs     # Logcat viewer (streaming, parsing, filters, stats)
│   ├── theme.rs      # Theme system (12 presets, selector)
│   ├── effects.rs    # Visual effects (TachyonFX)
│   └── lib.rs        # Library exports
├── scripts/
│   ├── logcat/       # Nushell recipe scripts
│   ├── bump_version.nu
│   └── release_prepare.nu
├── tests/
│   └── adb_integration_tests.rs
├── examples/
└── Cargo.toml
```

## Dependencies 📦

| Crate | Purpose |
|-------|---------|
| `ratatui` | Terminal UI framework |
| `crossterm` | Cross-platform terminal I/O |
| `adb_client` | Pure-Rust ADB protocol client |
| `tokio` | Async runtime |
| `tachyonfx` | Visual effects & animations |
| `regex` | Regex search in logcat |
| `serde` + `serde_json` | JSON serialization for logcat export |
| `tui-file-explorer` | File browser widget (save dialog) |
| `color-eyre` | Error handling |

## Development 🛠️

```bash
cargo test                    # Run all 209 tests
cargo clippy --all-targets --all-features -- -D warnings  # Lint
cargo fmt --check             # Format check
just release patch            # Bump, test, tag, push
```

### Adding New ADB Commands

1. Add a variant to `AdbCommand` in `src/adb.rs`
2. Implement the handler in `AdbManager::execute`
3. Add a menu entry in `src/menu.rs` via `build_entries()`
4. Tests in `tests/adb_integration_tests.rs`

## Contributing 🤝

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing`)
3. Run `cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo test`
4. Commit and push
5. Open a Pull Request

## License 📄

MIT — Copyright (c) Sorin Albu-Irimies

## Acknowledgments 🙏

- [Ratatui](https://ratatui.rs) — TUI framework
- [adb_client](https://github.com/nicoulaj/adb_client) — Pure-Rust ADB client
- [TachyonFX](https://github.com/junkdog/tachyonfx) — Visual effects
- [tui-file-explorer](https://github.com/sorinirimies/tui-file-explorer) — File browser widget

---

**Made with ❤️ and ☕ for Android developers** · *Powered by Rust 🦀*