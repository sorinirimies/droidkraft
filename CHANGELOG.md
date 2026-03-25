# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2026-03-25

### 🚀 Features

- **Live Logcat Viewer** — Full-screen, real-time logcat streaming via the `adb_client` crate's native `ADBServerDevice::get_logs()` API
- **Regex search** — Toggle regex mode (`r`) for powerful pattern matching (e.g. `Error|Exception`, `OkHttp.*failed`)
- **Exclude filter** — Negative match filter (`e`) to hide noisy tags/messages (supports regex when enabled)
- **Line detail popup** — Press `Enter` to inspect any log line with full unwrapped message, tag, PID, TID, and timestamp
- **Per-tag color hashing** — Each tag gets a stable, visually distinct color via djb2 hash over a 16-color palette
- **Stack trace folding** — Detect Java/Kotlin stack traces and fold/unfold them with `f`
- **Live stats bar** — Real-time lines/sec rate and per-level counters in the status area
- **Horizontal scroll** — `←`/`→` arrow keys to scroll long messages when word-wrap is off, `0` to reset
- **Compact mode** — Toggle with `x` to hide timestamp and PID columns for more message space
- **Copy to clipboard** — Press `y` to copy the selected line to the system clipboard (macOS `pbcopy`, Linux `xclip`/`xsel`)
- **Bookmarks** — `m` to toggle bookmarks on log lines, `[`/`]` to jump between them (wraps around)
- **Search highlighting** — Search matches are highlighted with yellow background in the log output
- **Log level filter** — Cycle minimum level with `l` (V→D→I→W→E→F)
- **Tag / PID filters** — Dedicated filter fields for tag substring and PID matching
- **Auto-scroll** — Sticks to the bottom by default; manual scroll disables it; `G`/`End` re-enables
- **Pause / Resume** — `Space` to pause ingestion without losing the stream
- **Clear logs** — `c` to clear all entries and counters
- **Save logs** — `s` to save all entries, `S` to save filtered only, with path input dialog
- **Save As… file browser** — `F2` in save dialog opens `tui-file-explorer` for directory browsing
- **Save Here** — `S` in the file browser to save directly into the current directory with a timestamped filename
- **Word wrap toggle** — `w` to toggle line wrapping
- **Menu integration** — 📺 Live Logcat entry in the SYSTEM menu section, `Shift+L` shortcut from menu

### 🐛 Bug Fixes

- **Bounded channel** — Replaced unbounded `mpsc::channel` with `sync_channel(10_000)` to prevent OOM crashes during logcat bursts
- **Backpressure** — ADB streaming thread now blocks when the channel is full instead of flooding memory
- **Optimized trim** — Ring buffer trim now adjusts filtered indices in-place O(filtered_len) instead of full O(50k) rebuild
- **Scroll position crash** — Fixed `scroll_position` semantics so transitioning from auto-scroll doesn't jump to line 0
- **Drain rate** — Increased from 200 to 500 lines/tick for better throughput (15k lines/sec at 30fps)

### ⚙️ Miscellaneous Tasks

- Added `regex` crate dependency for regex search support
- Added `tui-file-explorer` crate (no-default-features) for file browser in save dialog
- Added 49 new unit tests covering all logcat features (177 total project tests)
- Bumped version to 0.4.0

## [0.3.2] - 2025-10-19

### ⚙️ Miscellaneous Tasks

- Bump version to 0.3.2 - ([d003d55](https://github.com/sorinirimies/droidtui/commit/d003d5512c37b483b43ec5bb257f75c38f1655ef))


## [0.3.1] - 2025-10-19

### ⚙️ Miscellaneous Tasks

- Bump version to 0.3.1 - ([8e63c55](https://github.com/sorinirimies/droidtui/commit/8e63c55cfda5f2ba9e4350b23c59d6ef05d7dd1b))


## [0.3.0] - 2025-10-19

### ⚙️ Miscellaneous Tasks

- Bump version to 0.3.0 - ([888e2f1](https://github.com/sorinirimies/droidtui/commit/888e2f1bf296e063267b842b48c526182ffa3ed6))


### 📚 Documentation

- Update README and CHANGELOG for v0.2.9 - ([af93523](https://github.com/sorinirimies/droidtui/commit/af93523c5fcbdc6135b258e70d2bc677bfb338d8))


## [0.2.9] - 2025-10-15

### ⚙️ Miscellaneous Tasks

- Bump version to 0.2.9 - ([90cf9dc](https://github.com/sorinirimies/droidtui/commit/90cf9dcae5878fcb7cc3cf189240a945977a5faa))


### 📚 Documentation

- Update README and CHANGELOG for v0.2.8 - ([ec292a2](https://github.com/sorinirimies/droidtui/commit/ec292a2ff803b44f353a9b47e5bf6d07b6d86a7a))


## [0.2.8] - 2025-10-09

### Refactor

- Remove redundant parentheses in text brightness calculation - ([8dc3600](https://github.com/sorinirimies/droidtui/commit/8dc3600e1202f94a9c9fd9c87b55f1e707dab78b))


### ⚙️ Miscellaneous Tasks

- Bump version to 0.2.8 - ([ef1a08a](https://github.com/sorinirimies/droidtui/commit/ef1a08a845f02770e6700eb66a2735155be22aa9))


## [0.2.7] - 2025-10-09

### ⚙️ Miscellaneous Tasks

- Bump version to 0.2.7 - ([846e3c3](https://github.com/sorinirimies/droidtui/commit/846e3c34af2203058f4778852f595c58ec141ca7))


### 📚 Documentation

- Update README and CHANGELOG for v0.2.6 - ([6d84b45](https://github.com/sorinirimies/droidtui/commit/6d84b45687380a45ce563181468bd2fd3fb4c23b))


### 🚀 Features

- Add slide-in/slide-out animations with TachyonFX - ([4a5822f](https://github.com/sorinirimies/droidtui/commit/4a5822fb0e545b50fc6e46696931f9014c322ce0))

- Add impressive multi-layered loading animations - ([c26b707](https://github.com/sorinirimies/droidtui/commit/c26b7072806e5e161617e2fe70d466d687af8dc0))


## [0.2.6] - 2025-10-02

### ⚙️ Miscellaneous Tasks

- Bump version to 0.2.6 - ([9f84dae](https://github.com/sorinirimies/droidtui/commit/9f84dae09ea75344e6b6bc25c912b91ef64fe8a2))


### 🐛 Bug Fixes

- Remove syntax error from bump_version.sh script - ([da7a7e6](https://github.com/sorinirimies/droidtui/commit/da7a7e6258313a04ca5977f7fd527b948e5fb31d))


### 📚 Documentation

- Add quick release guide - ([5c6a863](https://github.com/sorinirimies/droidtui/commit/5c6a86339808bf5b360ce27868f9bb17a419e541))


### 🚀 Features

- Add automatic changelog generation with git-cliff - ([1473c46](https://github.com/sorinirimies/droidtui/commit/1473c46258a15a70c9b657f89e8144587eee8696))


## [0.2.5] - 2025-10-02

### ⚙️ Miscellaneous Tasks

- Add version automation tools - ([e46d8c2](https://github.com/sorinirimies/droidtui/commit/e46d8c241e6a6d412d780b0ce0896489b2864c06))

- Bump version to 0.2.5 - ([eabf786](https://github.com/sorinirimies/droidtui/commit/eabf7864e9fb9e746c81616ef05dc1df8c9147c1))


### 📚 Documentation

- Add comprehensive release process documentation - ([97045a8](https://github.com/sorinirimies/droidtui/commit/97045a860be445746a4733041be485842510e9f6))

- Add release automation section to README - ([048f27c](https://github.com/sorinirimies/droidtui/commit/048f27cd75639210dab398a434bc077297f14399))


## [0.2.4] - 2025-10-02

### 🐛 Bug Fixes

- Add spacing between layout sections to fix broken borders - ([f7c429f](https://github.com/sorinirimies/droidtui/commit/f7c429fd5c49aeac64775fc66f22206296983532))


### 🚜 Refactor

- Migrate to Elm architecture for better code organization - ([ea57627](https://github.com/sorinirimies/droidtui/commit/ea576276332a4ad80d487fc51f002a73d45489de))


## [0.2.3] - 2025-10-01

### 🐛 Bug Fixes

- Add rustfmt and clippy components to test job - ([7895679](https://github.com/sorinirimies/droidtui/commit/7895679bc73afb06b755d353e743607b6d65f30f))


### 📚 Documentation

- Update version badge to 0.2.3 - ([99ff558](https://github.com/sorinirimies/droidtui/commit/99ff558004a36f7dae7992f0dcbce72de68c032e))


### 🚀 Features

- Require passing tests before release - ([5aafccf](https://github.com/sorinirimies/droidtui/commit/5aafccf26171d4300a87b146d68690428daf1e45))


## [0.2.2] - 2025-10-01

### ⚙️ Miscellaneous Tasks

- Remove macOS from test matrix - ([f836e16](https://github.com/sorinirimies/droidtui/commit/f836e160e81eadffaff4916fda45b83e73b4f71e))

- Test on both Linux and Windows - ([0eaad62](https://github.com/sorinirimies/droidtui/commit/0eaad6299a0a94b530f9ec0abdd70c194d5aba14))


### 📚 Documentation

- Add release badges and crates.io installation instructions - ([65fc9f1](https://github.com/sorinirimies/droidtui/commit/65fc9f106d7b44188fa8a3413b95653f9368b877))


### 🚀 Features

- Improve crates.io publishing with conditional execution - ([025be9c](https://github.com/sorinirimies/droidtui/commit/025be9cd9618c2e7c4c50231ad5b10e9c4124019))


## [0.2.1] - 2025-10-01

### 🐛 Bug Fixes

- Make sed commands cross-platform compatible - ([ed7c2b2](https://github.com/sorinirimies/droidtui/commit/ed7c2b2e32e369dc911a0ffcd63ea8c88475be0b))

- Use Python for cross-platform version updates - ([fb5013e](https://github.com/sorinirimies/droidtui/commit/fb5013e9e79036054db01a0260ee43ddb3d665dc))


### 🚜 Refactor

- Simplify release workflow to publish single crate - ([522bb0f](https://github.com/sorinirimies/droidtui/commit/522bb0f1409ba32a11e431cf83aaca0360e2b939))


## [0.2.0] - 2025-09-27

### 🚀 Features

- Add scrollable command results and fix GitHub Actions - ([b99c7e1](https://github.com/sorinirimies/droidtui/commit/b99c7e1085d9e7d67a18d5b5ffb123a86594c3af))


## [0.1.0] - 2025-09-27

<!-- generated by git-cliff -->
