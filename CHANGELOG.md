# Changelog

All notable changes to this project will be documented in this file.

## [2.1.1] - 2026-10-16

### 📦 Packaging

- **flatpak:** Disable appstream-compose (missing in CI sandbox) - ([08f31d4](https://github.com/sorinirimies/droidkraft/commit/08f31d43029ac2eb54d855376f7e130dbca3aaf6))

- **packages:** Linux-only runners — drop macOS/Windows jobs, cross-compile Windows TUI with cargo-xwin - ([fa592b2](https://github.com/sorinirimies/droidkraft/commit/fa592b2b7b4177b9f5c004061623716c3f821a56))

- **packages:** Zigbuild portable TUI (glibc 2.17, x86_64+aarch64) replaces arm runner - ([a61cada](https://github.com/sorinirimies/droidkraft/commit/a61cadaba72f10ea598c71ae57de15c5ae7e8196))

- **packaging:** Snap package - ([8e5aa03](https://github.com/sorinirimies/droidkraft/commit/8e5aa03f5aea8a654f24ee8fcbfb703b7e43e01f))

- **release:** GUI+TUI release notes with per-format downloads table; cliff packaging group; README downloads section - ([39c6357](https://github.com/sorinirimies/droidkraft/commit/39c635779cdbee4c8f3b1127f80b46bad0fb170b))

- **packaging:** TUI and GUI each get their own Flatpak, Snap, AUR, Homebrew, binstall and Nix packages - ([14fdbcc](https://github.com/sorinirimies/droidkraft/commit/14fdbcc9522d89495178f523405b5ea797664f0c))


### 🧪 Testing

- Assert flatpak appstream-compose off - ([ce47960](https://github.com/sorinirimies/droidkraft/commit/ce4796055d7a0d440721ee711b5815a8eb17faa8))


## [2.1.0] - 2026-10-16

### ⚙️ Miscellaneous Tasks

- Bump version to 2.1.0 - ([082c51a](https://github.com/sorinirimies/droidkraft/commit/082c51ad630d7c9ee1cbfb9fe112a48a4a963318))


### 🐛 Bug Fixes

- **deps:** Pin libc 0.2.189 — 0.2.190 drops ENOATTR and breaks xattr 0.2.3 (gpui) on Linux - ([fd403ac](https://github.com/sorinirimies/droidkraft/commit/fd403ac0aca23cd2fd241384a42d999629e368af))


## [2.0.0] - 2026-10-16

### ⚙️ Miscellaneous Tasks

- Add gitea-nexus-lab remote recipes (push/pull/sync/release) - ([0f55897](https://github.com/sorinirimies/droidkraft/commit/0f55897528d802fd37ad4f404a3298b79be447b8))

- Rename gitea -> gitea-microlab; add full gitea-microlab/-starscream/-nexus-lab parity - ([9e9943a](https://github.com/sorinirimies/droidkraft/commit/9e9943a80a6f2f337cc23629e4f74e1b6413de89))

- Trigger Gitea Actions (retest after adding runners) - ([a0a5055](https://github.com/sorinirimies/droidkraft/commit/a0a5055edc0f8d38558dbe5db8077355edcecdaa))

- **security:** Add cargo-audit workflow and just audit recipe - ([e70c29f](https://github.com/sorinirimies/droidkraft/commit/e70c29f94af060d24c2d4bc72521b9dbee219b92))

- Prebuilt Nushell, cargo-deny + cargo-audit security workflow - ([6b407ed](https://github.com/sorinirimies/droidkraft/commit/6b407ed50a6f39b3a310120a7249c933e31f7260))

- **justfile:** Skip unconfigured remotes in *-all recipes - ([d2e70a2](https://github.com/sorinirimies/droidkraft/commit/d2e70a2dcec5935ed9edb337caa64f6311957321))

- **justfile:** Push/pull current branch instead of hardcoded main - ([379e9ef](https://github.com/sorinirimies/droidkraft/commit/379e9ef6668b55bef3a1d4d1b855342d5fa4eed8))

- Fuzz targets + fix UTF-8 panic in brief log parser, security/fuzz/soak recipes and workflows, gitea-starscream remote, bounded mirror input queue, cargo update - ([f666ea9](https://github.com/sorinirimies/droidkraft/commit/f666ea9e135830cb19d8a90bdbe5d776da696aea))

- **just:** Port tokenburn recipes (publish-*, release-*, coverage, update-deps, ...), add deps-update workflow - ([b8f5795](https://github.com/sorinirimies/droidkraft/commit/b8f5795be7e1ebb990c5fcbffa1502b82ca187b4))

- Package workflow (AppImage, Flatpak, Windows, macOS), icons, prune old releases - ([e6330b5](https://github.com/sorinirimies/droidkraft/commit/e6330b501f8ea2fe12e9c7f5a93f9c250a8fb0cb))

- Drop python; icon generator as Rust example, soak as nushell - ([cc123d9](https://github.com/sorinirimies/droidkraft/commit/cc123d9cb715c5a1ecd70da95c1272efbaaefd39))

- Packaging scripts in nushell, packaging asset tests, prune tests - ([a8fc6f7](https://github.com/sorinirimies/droidkraft/commit/a8fc6f71b230c4579683ac9a226092b1cdfe3917))

- Bump version to 2.0.0 - ([978acb5](https://github.com/sorinirimies/droidkraft/commit/978acb56f069cc47f8645f44c74010520620daba))


### 🐛 Bug Fixes

- **gui:** Robust nav rail (icon-only when narrow, device icons, text budgets, scrolling list) - ([3874574](https://github.com/sorinirimies/droidkraft/commit/38745749e3f29a035b9f811baa511a4ffb07eedf))

- **core:** Threadtime log parser panicked on multibyte chars (found by fuzz CI) - ([c72fcb3](https://github.com/sorinirimies/droidkraft/commit/c72fcb35f095574601605b451ccc18e2f7bf25e0))

- Packaging test + script refs after nushell port - ([e462ee3](https://github.com/sorinirimies/droidkraft/commit/e462ee32904751fd5a0ef03e5fd25ed5d05b3443))

- Fmt; make screen no-frame test deterministic without adb - ([ca74259](https://github.com/sorinirimies/droidkraft/commit/ca74259ad16d82f91fe72dffeb578ae608c55046))


### 📚 Documentation

- Rewrite README, new VHS tapes and preview GIFs, drop stale docs - ([0c3f94b](https://github.com/sorinirimies/droidkraft/commit/0c3f94b5b1e2655c83d16bc37a0a4cde9710a149))

- Add TUI/GUI install, audit and fuzz badges - ([f054990](https://github.com/sorinirimies/droidkraft/commit/f05499065f2dfb15f25ae00388fe746f8cba5d88))


### 📦 Packaging

- **release:** Publish mirror + fastboot crates before core/tui/gui - ([8370629](https://github.com/sorinirimies/droidkraft/commit/8370629140b7e495f0f8b6b7211792189e78ef52))

- **packaging:** Tarballs + binstall, deb, rpm, AUR PKGBUILD, Homebrew formula, Nix flake, aarch64 TUI - ([82b1ec5](https://github.com/sorinirimies/droidkraft/commit/82b1ec5472151cd6d2c1ff28835da281b3a182ab))


### 🚀 Features

- Pure-Rust fastboot crate, device details API, native ADB sideload - ([61bbcae](https://github.com/sorinirimies/droidkraft/commit/61bbcaef1e6a94b5eade173fd17dfa0177382240))

- Droidkraft-mirror crate (screen mirror + screen recorder shared by TUI/GUI) - ([a5a5f26](https://github.com/sorinirimies/droidkraft/commit/a5a5f2630a02242e1f3037c50b0f65f0862447ef))

- **tui:** Device-centric redesign, responsive layouts, direct Kitty graphics, braille pie chart - ([57836fe](https://github.com/sorinirimies/droidkraft/commit/57836fe46f56ac589b79d514a23a53f7513b6d91))

- **gui:** Device-centric redesign, responsive layouts, recording UI, per-device mirror windows - ([1768802](https://github.com/sorinirimies/droidkraft/commit/1768802bd35fc12b5bb0946f7cfa9be42af87f8c))

- **fastboot:** Ergonomic API (flash/erase/getvar/variables/reboot*/flash_factory), docs + README; polish(gui): stable sparkline bars - ([464e26a](https://github.com/sorinirimies/droidkraft/commit/464e26a6c0d2268d4d7ee8d9669f451d472af53c))

- Adjustable refresh rate for stats/graphs (core RefreshRate, TUI </>, GUI Refresh −/+ control) - ([1ecb702](https://github.com/sorinirimies/droidkraft/commit/1ecb70233a1cb7b6cc86fbc2d9ae0c61191cdae4))

- Thorough responsive design for TUI (wrapping footer, fitted header/tabs/banner/help) and GUI (adaptive tab bar), with tests - ([b10bcdf](https://github.com/sorinirimies/droidkraft/commit/b10bcdf026787a1d7f4b75a9e0e84a32ad57ce26))

- **logs:** GUI search box, fuzzy search mode in core/TUI/GUI, regex/fuzzy/text cycling - ([022bb91](https://github.com/sorinirimies/droidkraft/commit/022bb91df4afed54e7da016033ab0877b7f93628))


## [1.2.0] - 2026-07-19

### ⚙️ Miscellaneous Tasks

- Bump version to 1.2.0 - ([b94c900](https://github.com/sorinirimies/droidkraft/commit/b94c900c69e00d8d87faf5addd687379e6f9799e))


### 🚀 Features

- **rom:** Big catalog + GrapheneOS + multi-source resolution - ([7fdcf85](https://github.com/sorinirimies/droidkraft/commit/7fdcf85253b0d55cb368c9eab17a2655e1af15e8))


## [1.1.0] - 2026-07-17

### ⚙️ Miscellaneous Tasks

- Bump version to 1.1.0 - ([661e17d](https://github.com/sorinirimies/droidkraft/commit/661e17db0d998c43e2564c3cc73bf7aff09162af))


### 📚 Documentation

- Document one-stop custom-ROM flasher across READMEs - ([a18519c](https://github.com/sorinirimies/droidkraft/commit/a18519c366bf490efe142920023cfa47a333da5f))


### 🚀 Features

- **core:** Custom-ROM flashing engine (features::rom) - ([c3f6dbb](https://github.com/sorinirimies/droidkraft/commit/c3f6dbb9cb69c257453d7b194a6efb967d9f52ed))

- **gui:** One-stop custom-ROM flashing in Flash & Root panel - ([640568e](https://github.com/sorinirimies/droidkraft/commit/640568ef8f78ca4292843dbe88432b6faad1c153))

- **tui:** One-stop custom-ROM flasher screen - ([9de0730](https://github.com/sorinirimies/droidkraft/commit/9de0730276e46013f673b1fa8b78dbaf3322572e))


## [1.0.1] - 2026-07-16

### ⚙️ Miscellaneous Tasks

- Bump version to 1.0.1 - ([4f224ce](https://github.com/sorinirimies/droidkraft/commit/4f224ce70e9b100fd8b6a66a945da3768f3803f3))


### 🚀 Features

- **gui:** Make GUI publishable to crates.io as `droidkraft` - ([e40cba4](https://github.com/sorinirimies/droidkraft/commit/e40cba406af1886a6409d1bacb0187ff64bd1285))


## [1.0.0] - 2026-07-13

### ⚙️ Miscellaneous Tasks

- Pin Nushell to 0.111.0 in CI and release workflows - ([2486bbb](https://github.com/sorinirimies/droidkraft/commit/2486bbba0e414944983dcb3f692ac1d8f291ec00))

- Bump version to 1.0.0 - ([c576c77](https://github.com/sorinirimies/droidkraft/commit/c576c775a7966e8580c6b3eaba415177e40dc909))


### 🐛 Bug Fixes

- **ci:** Make nu test runner portable across Nushell versions - ([451638c](https://github.com/sorinirimies/droidkraft/commit/451638c8f67ed3867af5900148fe16b88ec20d8e))


### 📚 Documentation

- Regenerate VHS previews (Git LFS), refresh all READMEs - ([881b94c](https://github.com/sorinirimies/droidkraft/commit/881b94cc9cfa54525c707509771bb20ee8809aa5))


### 🚜 Refactor

- Prune dead macro, DRY GUI error mapping via str_err trait - ([7ad55cf](https://github.com/sorinirimies/droidkraft/commit/7ad55cf6fc8f969c943dfe08559bccf3c818084e))

- Deep scan — remove dead code, share logic via core public API - ([9cc4616](https://github.com/sorinirimies/droidkraft/commit/9cc4616204eb86fe833839030beb1934b3c28415))


## [0.6.1] - 2026-07-10

### ⚙️ Miscellaneous Tasks

- Bump version to 0.6.1 - ([6bb08ab](https://github.com/sorinirimies/droidkraft/commit/6bb08ab6a377bc8565ae5e91d82be43fcf5de3f3))


### 🐛 Bug Fixes

- Clean up rename fallout (BSD sed \b misses) + workspace-aware scripts - ([b74e85e](https://github.com/sorinirimies/droidkraft/commit/b74e85ea25fdb2cb97d30541f744754165deacd8))

- **clippy:** Remove redundant references in format args (rust 1.97) - ([6aa5ea7](https://github.com/sorinirimies/droidkraft/commit/6aa5ea778f28ee4ec957333f20b0206a143016b0))


### 🚜 Refactor

- Rename project droidtui → droidkraft - ([92ded36](https://github.com/sorinirimies/droidkraft/commit/92ded3654d78c58f1ebbafe69a6ad95a21e500a6))


## [0.6.0] - 2026-07-10

### ⚙️ Miscellaneous Tasks

- Workspace-aware release tooling + publishable TUI crate - ([d2a34cd](https://github.com/sorinirimies/droidkraft/commit/d2a34cd15e2941c9e38f3f6a121929fb9b1ac09c))

- Bump version to 0.6.0 - ([606f92e](https://github.com/sorinirimies/droidkraft/commit/606f92e8b6d7c99af79770cfef7e693e27540a30))


### 🐛 Bug Fixes

- **gui:** Compile against gpui — stateful scroll ids, AppContext prelude - ([3c5ce20](https://github.com/sorinirimies/droidkraft/commit/3c5ce20b9368c8580709153605c01e9d14df7768))


### 🚀 Features

- **gui:** New droidtui-gui — Zed GPUI device monitor & toolkit - ([7c5f3bf](https://github.com/sorinirimies/droidkraft/commit/7c5f3bf0ae5ca1593609ea7bf02fd3cb43075dd3))


### 🚜 Refactor

- Convert to Cargo workspace, extract droidtui-core library - ([48a3ba2](https://github.com/sorinirimies/droidkraft/commit/48a3ba2153b1b20fadcb9a9fb2f7a1cd0258d857))

- **tui:** Consume droidtui-core logcat engine, drop duplicated domain - ([6add84c](https://github.com/sorinirimies/droidkraft/commit/6add84c5c85a4a95c0b9610168f6b3463197276f))


### 🧪 Testing

- Workspace tooling, core API integration tests, macros polish - ([c0bcff8](https://github.com/sorinirimies/droidkraft/commit/c0bcff81ca161fe5012ad48c55310aab43c61d2f))


## [0.5.3] - 2026-04-06

### ⚙️ Miscellaneous Tasks

- Bump version to 0.5.3 - ([24244eb](https://github.com/sorinirimies/droidkraft/commit/24244eb2b5490d419d458e78c4686353345f2130))


## [0.5.2] - 2026-03-27

### ⚙️ Miscellaneous Tasks

- Bump version to 0.5.2 - ([70082fd](https://github.com/sorinirimies/droidkraft/commit/70082fdf24fef693fecf3ec8c0d18b52b1e2f40a))


### 🐛 Bug Fixes

- Remove 'S save as' from logcat footer — only shown in save dialog - ([93697c9](https://github.com/sorinirimies/droidkraft/commit/93697c90b435c4fef57a6b68dc5b28ab3b01a451))

- Walk up directory tree to find gradlew (like Gradle itself) - ([0008144](https://github.com/sorinirimies/droidkraft/commit/0008144663c56cf13d5f20d866a9dc6afa12431b))

- Skip startup animation, go straight to menu - ([5013241](https://github.com/sorinirimies/droidkraft/commit/5013241a33125a66b87af284840b9f7deedd749b))

- Theme applies immediately on ↑/↓/t — live preview while browsing - ([c74a57e](https://github.com/sorinirimies/droidkraft/commit/c74a57ed41e65641f0391688c4d82e88fbac01be))

- UTF-8 panic in theme description truncation - ([1c0e858](https://github.com/sorinirimies/droidkraft/commit/1c0e8586f873bb180e4982877d0222f7c70fb139))

- T cycles theme immediately from Menu and DevMode - ([128a5e0](https://github.com/sorinirimies/droidkraft/commit/128a5e0cb0bbb619f1a0e6267c310d1ffd0c3fdb))

- Multi-strategy Gradle resolution (wrapper → PATH → SDKMAN → Homebrew) - ([ff2893f](https://github.com/sorinirimies/droidkraft/commit/ff2893f1c26f0d0a64401984958667d4e0187413))


### 📚 Documentation

- Rewrite README.md and FEATURES.md for v0.5 - ([1d94eb9](https://github.com/sorinirimies/droidkraft/commit/1d94eb922e6ad4ff08df6224fdfd157bb7a65fa4))


### 🚀 Features

- JSON export, soft wrap, CLI query mode, Nu recipes, formatted JSON - ([0f1172f](https://github.com/sorinirimies/droidkraft/commit/0f1172fc16007deb723877dfa91fcf4991b1d1ad))

- Dev Mode — build, run, edit Android projects from the terminal - ([e0b1e3e](https://github.com/sorinirimies/droidkraft/commit/e0b1e3e5edf440b2c4ca2977bc31c511a0555ebd))

- Theme applies to entire UI + 27 presets matching tui-file-explorer - ([49aedd4](https://github.com/sorinirimies/droidkraft/commit/49aedd4e4028276ed68b7718fa1b284a43c1d5df))


## [0.5.1] - 2026-03-25

### ⚙️ Miscellaneous Tasks

- Bump version to 0.5.1 - ([865e62a](https://github.com/sorinirimies/droidkraft/commit/865e62a26c73d77f723348f86c95633d45209c79))


### 🐛 Bug Fixes

- Remap find to f, fold to F, update filter bar labels - ([62dc409](https://github.com/sorinirimies/droidkraft/commit/62dc40995720d83e1f4cc73e838f21ce482d060b))

- Remap S to Save As (file browser), update all hints - ([cf21586](https://github.com/sorinirimies/droidkraft/commit/cf21586a920cc1ba617845669fa3a95dda7276a8))

- Resolve clippy field_reassign_with_default in test modules - ([d0388bf](https://github.com/sorinirimies/droidkraft/commit/d0388bff79923810ed7f79265121707cb7ee9b5e))


## [0.4.1] - 2026-03-25

### 🚀 Features

- Theme system + redesigned logcat shortcuts & footer - ([df0f8f8](https://github.com/sorinirimies/droidkraft/commit/df0f8f82c0c966e449f98954fd0137d00f49d490))


## [0.4.0] - 2026-03-25

### 📚 Documentation

- Update README and CHANGELOG for v0.3.2 - ([1447d5d](https://github.com/sorinirimies/droidkraft/commit/1447d5d5860fe15eb79b2038ea9076b44e91a8b3))


### 🚀 Features

- Live Logcat Viewer with 10 pro features - ([1131f48](https://github.com/sorinirimies/droidkraft/commit/1131f48fb13ec2f47285b1cdeb6eb2196148fcb7))


## [0.3.2] - 2025-10-19

### ⚙️ Miscellaneous Tasks

- Bump version to 0.3.2 - ([d003d55](https://github.com/sorinirimies/droidkraft/commit/d003d5512c37b483b43ec5bb257f75c38f1655ef))


## [0.3.1] - 2025-10-19

### ⚙️ Miscellaneous Tasks

- Bump version to 0.3.1 - ([8e63c55](https://github.com/sorinirimies/droidkraft/commit/8e63c55cfda5f2ba9e4350b23c59d6ef05d7dd1b))


## [0.3.0] - 2025-10-19

### ⚙️ Miscellaneous Tasks

- Bump version to 0.3.0 - ([888e2f1](https://github.com/sorinirimies/droidkraft/commit/888e2f1bf296e063267b842b48c526182ffa3ed6))


### 📚 Documentation

- Update README and CHANGELOG for v0.2.9 - ([af93523](https://github.com/sorinirimies/droidkraft/commit/af93523c5fcbdc6135b258e70d2bc677bfb338d8))


## [0.2.9] - 2025-10-15

### ⚙️ Miscellaneous Tasks

- Bump version to 0.2.9 - ([90cf9dc](https://github.com/sorinirimies/droidkraft/commit/90cf9dcae5878fcb7cc3cf189240a945977a5faa))


### 📚 Documentation

- Update README and CHANGELOG for v0.2.8 - ([ec292a2](https://github.com/sorinirimies/droidkraft/commit/ec292a2ff803b44f353a9b47e5bf6d07b6d86a7a))


## [0.2.8] - 2025-10-09

### Refactor

- Remove redundant parentheses in text brightness calculation - ([8dc3600](https://github.com/sorinirimies/droidkraft/commit/8dc3600e1202f94a9c9fd9c87b55f1e707dab78b))


### ⚙️ Miscellaneous Tasks

- Bump version to 0.2.8 - ([ef1a08a](https://github.com/sorinirimies/droidkraft/commit/ef1a08a845f02770e6700eb66a2735155be22aa9))


## [0.2.7] - 2025-10-09

### ⚙️ Miscellaneous Tasks

- Bump version to 0.2.7 - ([846e3c3](https://github.com/sorinirimies/droidkraft/commit/846e3c34af2203058f4778852f595c58ec141ca7))


### 📚 Documentation

- Update README and CHANGELOG for v0.2.6 - ([6d84b45](https://github.com/sorinirimies/droidkraft/commit/6d84b45687380a45ce563181468bd2fd3fb4c23b))


### 🚀 Features

- Add slide-in/slide-out animations with TachyonFX - ([4a5822f](https://github.com/sorinirimies/droidkraft/commit/4a5822fb0e545b50fc6e46696931f9014c322ce0))

- Add impressive multi-layered loading animations - ([c26b707](https://github.com/sorinirimies/droidkraft/commit/c26b7072806e5e161617e2fe70d466d687af8dc0))


## [0.2.6] - 2025-10-02

### ⚙️ Miscellaneous Tasks

- Bump version to 0.2.6 - ([9f84dae](https://github.com/sorinirimies/droidkraft/commit/9f84dae09ea75344e6b6bc25c912b91ef64fe8a2))


### 🐛 Bug Fixes

- Remove syntax error from bump_version.sh script - ([da7a7e6](https://github.com/sorinirimies/droidkraft/commit/da7a7e6258313a04ca5977f7fd527b948e5fb31d))


### 📚 Documentation

- Add quick release guide - ([5c6a863](https://github.com/sorinirimies/droidkraft/commit/5c6a86339808bf5b360ce27868f9bb17a419e541))


### 🚀 Features

- Add automatic changelog generation with git-cliff - ([1473c46](https://github.com/sorinirimies/droidkraft/commit/1473c46258a15a70c9b657f89e8144587eee8696))


## [0.2.5] - 2025-10-02

### ⚙️ Miscellaneous Tasks

- Add version automation tools - ([e46d8c2](https://github.com/sorinirimies/droidkraft/commit/e46d8c241e6a6d412d780b0ce0896489b2864c06))

- Bump version to 0.2.5 - ([eabf786](https://github.com/sorinirimies/droidkraft/commit/eabf7864e9fb9e746c81616ef05dc1df8c9147c1))


### 📚 Documentation

- Add comprehensive release process documentation - ([97045a8](https://github.com/sorinirimies/droidkraft/commit/97045a860be445746a4733041be485842510e9f6))

- Add release automation section to README - ([048f27c](https://github.com/sorinirimies/droidkraft/commit/048f27cd75639210dab398a434bc077297f14399))


## [0.2.4] - 2025-10-02

### 🐛 Bug Fixes

- Add spacing between layout sections to fix broken borders - ([f7c429f](https://github.com/sorinirimies/droidkraft/commit/f7c429fd5c49aeac64775fc66f22206296983532))


### 🚜 Refactor

- Migrate to Elm architecture for better code organization - ([ea57627](https://github.com/sorinirimies/droidkraft/commit/ea576276332a4ad80d487fc51f002a73d45489de))


## [0.2.3] - 2025-10-01

### 🐛 Bug Fixes

- Add rustfmt and clippy components to test job - ([7895679](https://github.com/sorinirimies/droidkraft/commit/7895679bc73afb06b755d353e743607b6d65f30f))


### 📚 Documentation

- Update version badge to 0.2.3 - ([99ff558](https://github.com/sorinirimies/droidkraft/commit/99ff558004a36f7dae7992f0dcbce72de68c032e))


### 🚀 Features

- Require passing tests before release - ([5aafccf](https://github.com/sorinirimies/droidkraft/commit/5aafccf26171d4300a87b146d68690428daf1e45))


## [0.2.2] - 2025-10-01

### ⚙️ Miscellaneous Tasks

- Remove macOS from test matrix - ([f836e16](https://github.com/sorinirimies/droidkraft/commit/f836e160e81eadffaff4916fda45b83e73b4f71e))

- Test on both Linux and Windows - ([0eaad62](https://github.com/sorinirimies/droidkraft/commit/0eaad6299a0a94b530f9ec0abdd70c194d5aba14))


### 📚 Documentation

- Add release badges and crates.io installation instructions - ([65fc9f1](https://github.com/sorinirimies/droidkraft/commit/65fc9f106d7b44188fa8a3413b95653f9368b877))


### 🚀 Features

- Improve crates.io publishing with conditional execution - ([025be9c](https://github.com/sorinirimies/droidkraft/commit/025be9cd9618c2e7c4c50231ad5b10e9c4124019))


## [0.2.1] - 2025-10-01

### 🐛 Bug Fixes

- Make sed commands cross-platform compatible - ([ed7c2b2](https://github.com/sorinirimies/droidkraft/commit/ed7c2b2e32e369dc911a0ffcd63ea8c88475be0b))

- Use Python for cross-platform version updates - ([fb5013e](https://github.com/sorinirimies/droidkraft/commit/fb5013e9e79036054db01a0260ee43ddb3d665dc))


### 🚜 Refactor

- Simplify release workflow to publish single crate - ([522bb0f](https://github.com/sorinirimies/droidkraft/commit/522bb0f1409ba32a11e431cf83aaca0360e2b939))


## [0.2.0] - 2025-09-27

### 🚀 Features

- Add scrollable command results and fix GitHub Actions - ([b99c7e1](https://github.com/sorinirimies/droidkraft/commit/b99c7e1085d9e7d67a18d5b5ffb123a86594c3af))


## [0.1.0] - 2025-09-27

<!-- generated by git-cliff -->
