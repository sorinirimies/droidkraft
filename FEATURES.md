# DroidTUI Features Documentation 🚀

A comprehensive guide to all features in DroidTUI v0.5.

## 📺 Live Logcat Viewer

The flagship feature — a full-screen, real-time logcat viewer with Android Studio-level tooling, powered by the pure-Rust `adb_client` crate.

### Streaming Engine
- **Native ADB protocol** — uses `ADBServerDevice::get_logs()`, no `adb` binary shelling
- **Bounded channel** — `sync_channel(10,000)` with backpressure prevents OOM during logcat bursts
- **Ring buffer** — 50,000 entry cap with in-place index trimming (O(filtered_len), not O(50k))
- **500 lines/tick drain** — 15,000 lines/sec throughput at 30fps

### Log Parsing
- **Threadtime format** — `MM-DD HH:MM:SS.mmm PID TID LEVEL TAG: message`
- **Brief format** — `LEVEL/TAG(PID): message`
- **Fallback** — unrecognized lines stored as-is with Unknown level
- **Stack trace detection** — lines starting with `at `, `Caused by:`, `... N more` flagged as continuations

### Filters
| Filter | Key | Description |
|--------|-----|-------------|
| **Find** | `f` | Case-insensitive substring search with yellow match highlighting |
| **Exclude** | `e` | Negative match — hides lines containing the query |
| **Tag** | `t` | Filter by tag substring |
| **PID** | `p` | Filter by PID substring |
| **Level** | `l` | Cycle minimum level: V → D → I → W → E → F |
| **Regex** | `r` | Toggle regex mode for Find and Exclude (uses `regex` crate) |

All filters stack — a line must pass ALL active filters to be shown. Filtered count vs total shown in the status bar.

### Visual Features
| Feature | Key | Description |
|---------|-----|-------------|
| **Per-tag colors** | — | Each tag gets a deterministic color via djb2 hash over a 16-color palette |
| **Stack trace folding** | `F` | Collapse multi-line stack traces into a single foldable line |
| **Line detail popup** | `Enter` | Full-screen popup with level, timestamp, tag, PID, TID, and word-wrapped message |
| **JSON formatting** | — | Auto-detects JSON in messages, pretty-prints with syntax coloring in detail popup |
| **Soft wrap** | `w` | Wrap long messages across multiple rows with aligned indentation |
| **Compact mode** | `x` | Hide timestamp and PID columns for more message space |
| **Horizontal scroll** | `←`/`→` | Scroll long lines when wrap is off; `0` to reset |
| **Bookmarks** | `m` | Toggle bookmark on current line; yellow `●` marker |
| **Bookmark nav** | `[`/`]` | Jump to previous/next bookmark (wraps around) |
| **Selected line** | — | Subtle highlight on the focused line |
| **Auto-scroll indicator** | — | Green `↓ AUTO` badge when tailing |
| **Scrollbar** | — | Proportional thumb on the right edge |

### Actions
| Action | Key | Description |
|--------|-----|-------------|
| **Copy line** | `y` | Copy selected line to system clipboard (`pbcopy`/`xclip`/`xsel`) |
| **Pause** | `Space` | Freeze the view; stream continues but entries are buffered |
| **Clear** | `c` | Clear all entries, counters, bookmarks, and folds |
| **Save** | `s` | Save to file (TXT or JSON, all or filtered) |
| **Save As** | `S` | Open file browser to choose save location |
| **Save Here** | `Shift+S` | In file browser: save into current directory with timestamped name |
| **Close** | `q`/`Esc` | Stop streaming and return to main menu |

### Save Dialog
- **Tab** cycles through: `TXT all → TXT filtered → JSON all → JSON filtered`
- **Enter** saves to the typed path
- **S** opens the `tui-file-explorer` file browser
- **Esc** cancels
- Title shows count, scope, and format: `💾 Save 9940 entries (all) [JSON]`

### JSON Export (JSONL)
Each entry is serialized as a JSON object on its own line:
```json
{"raw":"...","timestamp":"03-25 12:00:00.000","pid":"1234","tid":"5678","level":"Info","tag":"MyApp","message":"Hello world","is_stack_continuation":false}
```
Designed for piping into Nushell, jq, or any JSON-aware tool.

### Live Stats
- **Lines/sec rate** — rolling average shown in status bar
- **Per-level counters** — tracked in `LogStats.counts[7]`
- **Total received** — includes lines consumed while paused

## 🖥️ CLI Query Mode

Non-TUI mode for scripting and piping:

```bash
droidtui --query                    # Stream live logcat as JSONL
droidtui --query --last 500         # Dump last 500 lines and exit
droidtui --query --level E          # Filter: errors only
droidtui --query --tag MyApp        # Filter: tag contains "MyApp"
droidtui --query --grep "timeout"   # Filter: message contains "timeout"
droidtui --help                     # Show all options
```

Combine with Nushell:
```bash
droidtui --query | nu -c 'lines | each { from json } | where level == "Error" | group-by tag'
```

## 📂 Nushell Recipe Scripts

Pre-built analysis scripts in `scripts/logcat/`:

| Script | Usage |
|--------|-------|
| `top_tags.nu` | `nu scripts/logcat/top_tags.nu logcat.jsonl` |
| `error_summary.nu` | `nu scripts/logcat/error_summary.nu logcat.jsonl` |
| `timeline.nu` | `nu scripts/logcat/timeline.nu logcat.jsonl` |
| `find_crashes.nu` | `nu scripts/logcat/find_crashes.nu logcat.jsonl` |
| `filter_json.nu` | `nu scripts/logcat/filter_json.nu logcat.jsonl` |

## 🎨 Theme System

Press `Shift+T` from **any screen** to open the theme selector.

### Available Presets (12)
Default, Dracula, Nord, Gruvbox Dark, Catppuccin Mocha, Tokyo Night, Solarized Dark, Moonfly, Oxocarbon, Forest, Neon, Mono

### Theme Fields
`brand`, `accent`, `success`, `dim`, `fg`, `sel_bg`, `warn`, `error`, `surface`, `border`, `key_hint`

### Selector Controls
| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Browse presets |
| `t` | Next preset |
| `Enter` | Apply selected theme |
| `Esc` | Close selector |

## 📱 ADB Commands

### Device Management
- List devices, device model, Android version, ADB version

### Package Management
- All packages, user packages, system packages, packages with paths

### System Monitoring
- Battery status, memory usage, CPU info, running processes
- System log, error log, system services, device properties

### Network
- Network status, WiFi info, IP configuration

### Root Toolkit
- Root status, SELinux status/toggle, Magisk status, bootloader state

### Bootloader & Flash
- Reboot to recovery/bootloader
- Fastboot: device info, OEM unlock/lock, wipe data

### Actions
- Take screenshot, screen resolution, clear logs, reboot device

## 📊 Device Dashboard

The right panel in the main menu shows live device stats:
- **Device selector** — multi-device support with `d` to cycle
- **Model & version** — device name and Android version
- **Battery** — percentage with color-coded progress bar
- **RAM** — used/total with utilization bar
- **CPU** — 1-min load average with bar

## 🔧 Technical Details

### Performance
- **Bounded channel** — `sync_channel(10k)` prevents memory overflow
- **In-place trim** — O(filtered_len) index adjustment instead of O(50k) rebuild
- **500 lines/tick** — 15k lines/sec throughput at 30fps
- **Incremental filter** — new entries checked individually, full rebuild only on filter change

### Architecture
Elm-like: Model → Message → Update → View cycle at 30fps.

### Testing
209 tests covering:
- Logcat parsing (threadtime, brief, unknown formats)
- Filter logic (level, search, tag, PID, exclude, regex)
- Scroll mechanics (auto-scroll transition, viewport tracking)
- Bookmarks (toggle, navigation, wrap-around)
- Stack trace detection and folding
- Channel writer (line splitting, flush, CR/LF stripping)
- JSON export and formatting
- Theme system (presets, selector, builder)
- Menu navigation (sections, wrapping)
- ADB command creation and error handling