# Changelog

All notable changes to this project are documented here. It is generated from
[Conventional Commits](https://www.conventionalcommits.org) by
[git-cliff](https://git-cliff.org) — do not edit it by hand.

## [2.3.0](https://github.com/sorinirimies/droidkraft/compare/v2.2.3...v2.3.0) — 2026-11-07

### ✨ Features

- New app icon for macOS, Windows and Linux ([`86c53fd`](https://github.com/sorinirimies/droidkraft/commit/86c53fdbd710ef2408fc4aa19741e8f57eba8721))

### 📦 Dependencies

- Nightly cargo update 2026-11-07 ([`6599be7`](https://github.com/sorinirimies/droidkraft/commit/6599be7fdcf2cc875f76cd46282f70cda5ef68af))
- Nightly cargo update 2026-11-06 ([`acc9418`](https://github.com/sorinirimies/droidkraft/commit/acc9418d20e1f39fe281c9265d444a560afe0980))

### 🔄 CI and release

- **gitea:** Check the MSRV on Gitea too, with the same command as GitHub ([`fcbe2a5`](https://github.com/sorinirimies/droidkraft/commit/fcbe2a5220a513226dd58533145f1466abb97ade))
- **release:** Each install line in the release notes says which binary it is (GUI, TUI, web) ([`6b64879`](https://github.com/sorinirimies/droidkraft/commit/6b64879cb45b8b513e80583f4828b81bfcda2cf4))
- **packages:** .deb and .rpm package names are valid Debian names (no underscores) ([`2f702be`](https://github.com/sorinirimies/droidkraft/commit/2f702be0203485021307c5624e285f5b0cca68bd))
- **packages:** AppImages use the downloaded appimagetool; which also matched the script's own command ([`1090bf3`](https://github.com/sorinirimies/droidkraft/commit/1090bf36959d6025cb0518def83cc51e761f74bc))
- **release:** Pruning skips a re-run of an older release ([`ce49e53`](https://github.com/sorinirimies/droidkraft/commit/ce49e5377378ac7d60fb0fbb5c21d98ea3b101d1))
- **packages:** Alien gets an absolute .deb path, so the .rpm build finds it ([`ef03258`](https://github.com/sorinirimies/droidkraft/commit/ef03258f59216ef5cc966d49e8a7cc7275de7a33))

## [2.2.3](https://github.com/sorinirimies/droidkraft/compare/v2.2.2...v2.2.3) — 2026-11-05

### 🐛 Bug fixes

- **build:** Pin libc 0.2.189 on Linux; xattr 0.2.3 (via gpui) needs ENOATTR, gone in libc 0.2.190 ([`be191c7`](https://github.com/sorinirimies/droidkraft/commit/be191c72ac5f5f9d052c513025e340f3c18c4a21))
- **ci:** The nightly downgrade guard compares within a semver series ([`87c060b`](https://github.com/sorinirimies/droidkraft/commit/87c060b225ee723a29925abece116d7cf063a293))
- **gui:** Draw the window frame when the compositor does not (GNOME on Wayland); app_id and StartupWMClass are the desktop id ([`3fb2edb`](https://github.com/sorinirimies/droidkraft/commit/3fb2edb4aaa529b45dbc338b39ec85dd6d04da7a))

### ♻️ Refactor

- Edition 2024, MSRV 1.91 and the shared lints; Debug on every public type, no unwraps on locks, seven unused dependencies removed ([`1d77796`](https://github.com/sorinirimies/droidkraft/commit/1d777968dd43e12f8d7703da657c3cc3778dfeae))

### 📚 Documentation

- Fix four broken intra-doc links ([`62913ca`](https://github.com/sorinirimies/droidkraft/commit/62913ca5c1a98946b966ce5650a5498719390512))

### 🔄 CI and release

- **wayland:** Start the desktop app on headless GNOME (mutter) - weston has no seat, which gpui needs - inside one session bus, as nobody when root ([`70065e4`](https://github.com/sorinirimies/droidkraft/commit/70065e4c1d87f8dda9e246ac88ea670e1a44896e))
- Download archives to a file so curl retries a dropped transfer, instead of piping into tar ([`3ced50d`](https://github.com/sorinirimies/droidkraft/commit/3ced50d33ef6855299847a01e3951294bb15f326))
- Start the desktop app on a headless Wayland compositor; StartupWMClass follows the desktop id ([`d078dcb`](https://github.com/sorinirimies/droidkraft/commit/d078dcba4685d9760c2ebce3931bceb7b560e842))
- Port to the shared release template: Gitea releases with binaries, old release assets pruned on both hosts; Flatpak, Snap, musl, Arch and Alpine packages kept ([`ec0e419`](https://github.com/sorinirimies/droidkraft/commit/ec0e419b907b0357258ac8208c40cd2cbf36cbe9))

### 🔧 Chores

- **deny:** List pin-utils (RUSTSEC-2026-0441) in deny.toml; no upgrade path ([`1831cdd`](https://github.com/sorinirimies/droidkraft/commit/1831cddd735836f04791ce143ac86f2304cfada3))

## [2.2.2](https://github.com/sorinirimies/droidkraft/compare/v2.2.1...v2.2.2) — 2026-11-02

### ♻️ Refactor

- **ci:** Package macOS DMG and Windows installer with tested Nushell scripts ([`0d48830`](https://github.com/sorinirimies/droidkraft/commit/0d48830d9fad66ec2d79b885429b2f293d458318))

### 🔄 CI and release

- **release:** Windows NSIS setup.exe installer ([`eaf23d0`](https://github.com/sorinirimies/droidkraft/commit/eaf23d03cdda066b05b550807e7d9870ef9180ba))
- **release:** Ship macOS universal DMG + Windows TUI zipdroidkraft ([`93bda89`](https://github.com/sorinirimies/droidkraft/commit/93bda89852c9f02c4995f9f7f47d0ad8615e9468))
- **release:** Build universal macOS GUI+TUI, publish .app DMG and tarball ([`4781f9e`](https://github.com/sorinirimies/droidkraft/commit/4781f9e608cc9d7bb395a0aa3dc84f4a7c7f21bf))
- **gitea:** Create release with softprops action (upload_url was empty) ([`a72ae7a`](https://github.com/sorinirimies/droidkraft/commit/a72ae7a8d9683a2c40717ad4d4e46b9f8cbe05b8))

## [2.2.1](https://github.com/sorinirimies/droidkraft/compare/v2.2.0...v2.2.1) — 2026-11-02

### ✨ Features

- **tui:** Enable tui-nuances theme feature on widgets, nuances 0.1.9 ([`d99b7eb`](https://github.com/sorinirimies/droidkraft/commit/d99b7ebcbdc00e2c79cbe1a1abfcb12ad6fc21f8))
- **gui:** Redesigned buttons — shared style_btn (pill, tinted border, shadow, hover lift, pressed state) ([`bc36ccb`](https://github.com/sorinirimies/droidkraft/commit/bc36ccbb33d069043c1d0d722982549bb77f4321))

### 🐛 Bug fixes

- **scripts:** Bump_version.nu rejects downgrades (semver compare) and tags already on origin ([`f449792`](https://github.com/sorinirimies/droidkraft/commit/f449792f9ec7d6323b2ed1f058745a509b9fee70))

### 📦 Dependencies

- Cargo update; prune stale deny.toml advisory ignores ([`2f7eef9`](https://github.com/sorinirimies/droidkraft/commit/2f7eef96f49318a14577d2929eda22eca24e6a36))
- Bump rustls-webpki 0.103.16, webpki-roots 1.0.11 (RUSTSEC-2026-0381/0382) ([`0aed807`](https://github.com/sorinirimies/droidkraft/commit/0aed807719dc6cb878244f91e77c286226e1af77))
- Tui-file-explorer 4.0 (tui-nuances-only themes) ([`eda121e`](https://github.com/sorinirimies/droidkraft/commit/eda121e7c73f360d0dabd6d76b34f4843f2b1b40))
- Bump tui-nuances to 0.1.10 ([`9825a62`](https://github.com/sorinirimies/droidkraft/commit/9825a628f50e2aa78917a4c393f39d26c436e14d))

### 🔄 CI and release

- Nightly merges lockfile-only updates; bump + release only when a Cargo.toml requirement changes ([`b729988`](https://github.com/sorinirimies/droidkraft/commit/b7299888fad982cc176dae5416c89971ce045b1f))
- Nightly merges lockfile-only updates; bump + release only when a Cargo.toml requirement changes ([`5d4cf83`](https://github.com/sorinirimies/droidkraft/commit/5d4cf83555a97e1c29c0cef32556095c7631a3c6))
- Nightly dependency update pushes lockfile only, no version bump or release ([`37b9cd7`](https://github.com/sorinirimies/droidkraft/commit/37b9cd7440a1d9697851ad0e9ef9a527d7e510ea))
- **packages:** Drop windows-gui job (gpui shaders need fxc.exe on a Windows host; cannot cross-build) ([`d7a30ab`](https://github.com/sorinirimies/droidkraft/commit/d7a30ab165d68733fa291baab178c70fe0eecbb5))
- **packages:** Retry release creation (tag-push race with Release workflow) ([`e12bba0`](https://github.com/sorinirimies/droidkraft/commit/e12bba041b670daa93c1bee95fc302ba729a72da))

### 🔧 Chores

- **deny:** Ignore unmaintained zvariant/zbus_names advisories (transitive) ([`533666f`](https://github.com/sorinirimies/droidkraft/commit/533666f1caf48983601aec482faa4b4d15072049))
- Bump tui-nuances to 0.1.7 ([`cc95a96`](https://github.com/sorinirimies/droidkraft/commit/cc95a96612c75987e9a2de484a6a860000044685))

## [2.2.0](https://github.com/sorinirimies/droidkraft/compare/v2.1.9...v2.2.0) — 2026-10-17

### 📦 Dependencies

- Zip 2.4 -> 9 (RUSTSEC-2026-0367/0368/0369) ([`f6a9cd7`](https://github.com/sorinirimies/droidkraft/commit/f6a9cd70a5c0afd77fbccdc47ccea81638787d56))

## [2.1.9](https://github.com/sorinirimies/droidkraft/compare/v2.1.8...v2.1.9) — 2026-10-17

### ✨ Features

- Screen is now the 2nd tab (GUI + TUI), docs/help updated ([`3f0d658`](https://github.com/sorinirimies/droidkraft/commit/3f0d6584883076873bbe709d437dd2247ffc05fd))
- **gui:** MacOS shortcuts — Cmd+/-/0 UI zoom, Cmd+Q quit, Cmd+W close, menu bar; bind keys in main app (were mirror-only) ([`2582741`](https://github.com/sorinirimies/droidkraft/commit/258274160a23e7a62800bf3a7d39e06c979af0e7))
- **gui:** Fastboot slot A/B + reboot bootloader/fastbootd/recovery buttons (TUI parity) ([`88cad04`](https://github.com/sorinirimies/droidkraft/commit/88cad043443a1a8ffae8533bb7299ca11335b446))
- **gui:** Log pause/save; connection kind in TUI header via core; tests ([`f266777`](https://github.com/sorinirimies/droidkraft/commit/f266777220596f670c11ac059745463fc10e60ff))

### 🐛 Bug fixes

- **gui:** Zoom in detached/standalone windows, rem-based tab padding, quick actions + charts on short windows (scrolling); tui screen clippy ([`e075d3d`](https://github.com/sorinirimies/droidkraft/commit/e075d3d744662f5a16df41515c8e27ef1e6a2117))
- **tui:** Flash/root/rom short-pane layouts, log truncation+paging, rec panel overflow, wrapped msgs, banner cap ([`84ffae5`](https://github.com/sorinirimies/droidkraft/commit/84ffae59ad3f5eeae61dc60e750d374ca63f6d4b))
- **gui:** Zoom-consistent sizing (rems, scaled nav/cards), clamp zoom to window, scrollable wrapping Settings, flash rows wrap, button min height; tui PgUp ([`16d23d5`](https://github.com/sorinirimies/droidkraft/commit/16d23d573d127ca3f50c2c71998ffc01dac7caec))
- **tui:** Modal clip guard via draw_modal result, logs degrade + filter tail, footer keeps ? q, overview max-scroll/page, nav follows active, logcat scroll clamp persisted ([`7f70a4c`](https://github.com/sorinirimies/droidkraft/commit/7f70a4c96a2a926f8049d7c54ac81e05b22319a6))
- **gui:** Responsive hardening — scrollable mirror side panel, container-aware stacking, viewport floor, wrapping capture bars, ellipsis for chips/kv/cards, log row flex, scrollable tabs, bigger hit targets ([`8a49b6e`](https://github.com/sorinirimies/droidkraft/commit/8a49b6ee0e8157c41dbdfa267fc0c8b10a8e26ab))
- **tui:** Responsive hardening — clear stale hit rects, cancel invisible modals, help swallows mouse, flash list scroll, cell-width aware truncation, auto-size modals, prompt tail; matrix tests ([`734c343`](https://github.com/sorinirimies/droidkraft/commit/734c34359a4347c1a0281fc6e44581bdd8511bed))
- **responsive:** TUI tiny-size fallback, help scroll, devices guard; GUI mirror side panel stacks when narrow, commands output max height ([`f0a8c37`](https://github.com/sorinirimies/droidkraft/commit/f0a8c3790b88bc8b98d5d7cdcce28066b7dcf54a))
- **gui:** Bootloader chip flicker — keep last flash_locked, stable chip order ([`d88164b`](https://github.com/sorinirimies/droidkraft/commit/d88164ba0efea32a8c9fe4a103e0eb9661751792))

### ⚡ Performance

- **core:** Fast details poll (static fields cached, full refresh every 5m), suppress transient poll failures; shared DetailsCollector for GUI+TUI ([`8156edc`](https://github.com/sorinirimies/droidkraft/commit/8156edc2d15030d66a982ea7a53efd1fa091e16c))

### 🔧 Chores

- **just:** Fast-fail ssh, setup-remotes, force variants ([`5f95c8a`](https://github.com/sorinirimies/droidkraft/commit/5f95c8a84917e7f61955a418fdd15988839f57da))

## [2.1.8](https://github.com/sorinirimies/droidkraft/compare/v2.1.7...v2.1.8) — 2026-10-17

### ✨ Features

- **gui:** Record/screenshot controls in detached + standalone mirror windows; wire --mirror; best-effort Windows GUI cross-build ([`d7a445e`](https://github.com/sorinirimies/droidkraft/commit/d7a445e3ac1c395a375a9d5285792a83d568764e))
- **packaging:** MacOS TUI, Windows installer, Scoop/winget/Chocolatey, Arch .pkg.tar.zst, Alpine .apk, static musl tarballs ([`d3eab5d`](https://github.com/sorinirimies/droidkraft/commit/d3eab5d9df27b8fd3d3f839354dd52cbb3b081ae))

### 📚 Documentation

- Platform support section, cargo install for Linux/Windows in release notes ([`d83e1f1`](https://github.com/sorinirimies/droidkraft/commit/d83e1f1b86fb850cd6b859b51e99f8a25974e0f4))

### 🔄 CI and release

- **packages:** Alpine .apk for aarch64 via qemu ([`1db1e43`](https://github.com/sorinirimies/droidkraft/commit/1db1e436eb7e865e27b85dfa3c56ce5f1bb785cc))

### 💄 Style

- Fmt; test: TUI detach args parse to mirror mode ([`d80bc46`](https://github.com/sorinirimies/droidkraft/commit/d80bc46f691a54d40a429b72575d072bc7b7111f))

## [2.1.7](https://github.com/sorinirimies/droidkraft/compare/v2.1.6...v2.1.7) — 2026-10-16

### 🐛 Bug fixes

- **snap:** Base was indented into description in GUI template ([`2aa672a`](https://github.com/sorinirimies/droidkraft/commit/2aa672aa7460ffa9745d03948529d0ae4a3c467d))

## [2.1.6](https://github.com/sorinirimies/droidkraft/compare/v2.1.5...v2.1.6) — 2026-10-16

### 🔄 CI and release

- **packages:** GUI rpm path is crates/droidkraft-gui ([`8360782`](https://github.com/sorinirimies/droidkraft/commit/8360782cdb340349f9d65b38b414493f1df4bba8))

## [2.1.5](https://github.com/sorinirimies/droidkraft/compare/v2.1.4...v2.1.5) — 2026-10-16

### 🐛 Bug fixes

- **flatpak:** Drop invalid --socket=pty from TUI manifest ([`74e5cb9`](https://github.com/sorinirimies/droidkraft/commit/74e5cb9a4b60ec809356b28f4be4167bee7a6513))

## [2.1.4](https://github.com/sorinirimies/droidkraft/compare/v2.1.3...v2.1.4) — 2026-10-16

### 📚 Documentation

- **readme:** GUI crate badge points at droidkraft ([`ae0fb2e`](https://github.com/sorinirimies/droidkraft/commit/ae0fb2e960b114e895dee5677503dc02fbf8936f))

### 🔄 CI and release

- **packages:** X86_64 TUI native, aarch64 via zigbuild (zigbuild breaks libusb cc build on same-arch target) ([`a085699`](https://github.com/sorinirimies/droidkraft/commit/a085699367dd9d064c738195b9aaedff2146fa23))

## [2.1.3](https://github.com/sorinirimies/droidkraft/compare/v2.1.2...v2.1.3) — 2026-10-16

### 🔄 CI and release

- **packages:** Zig 0.15.2 (0.14.1 rejects the .2.17 glibc suffix in cc-rs --target) ([`f6d90d9`](https://github.com/sorinirimies/droidkraft/commit/f6d90d92fe3aa344f9c966b0a42669bee7a801be))

## [2.1.2](https://github.com/sorinirimies/droidkraft/compare/v2.1.1...v2.1.2) — 2026-10-16

### 🔄 CI and release

- **packages:** Direct-download fallback if zvm fails ([`0348416`](https://github.com/sorinirimies/droidkraft/commit/0348416d397fb1d50ff557d12dee488baade3512))
- **packages:** Install zig via zvm ([`34d4c3c`](https://github.com/sorinirimies/droidkraft/commit/34d4c3cee0fa71f78f1a8dacfec4819846997a4b))
- **packages:** Zvm fallback for zig install ([`fe8aefb`](https://github.com/sorinirimies/droidkraft/commit/fe8aefbc895f60b76f3d07d2369f738410ccfa13))
- **packages:** Install zig straight from ziglang.org ([`fa4473a`](https://github.com/sorinirimies/droidkraft/commit/fa4473a1e4496f8de856c49de8716abfed2ff07e))
- **packages:** Setup-zig v2 (v1 URL scheme 404s for zig 0.14.1) ([`ca7666a`](https://github.com/sorinirimies/droidkraft/commit/ca7666a4b0ed4ff74bc9e5810084f823e9050e08))

## [2.1.1](https://github.com/sorinirimies/droidkraft/compare/v2.1.0...v2.1.1) — 2026-10-16

### ✨ Features

- **packaging:** TUI and GUI each get their own Flatpak, Snap, AUR, Homebrew, binstall and Nix packages ([`14fdbcc`](https://github.com/sorinirimies/droidkraft/commit/14fdbcc9522d89495178f523405b5ea797664f0c))
- **packaging:** Snap package ([`8e5aa03`](https://github.com/sorinirimies/droidkraft/commit/8e5aa03f5aea8a654f24ee8fcbfb703b7e43e01f))

### 🐛 Bug fixes

- **flatpak:** Disable appstream-compose (missing in CI sandbox) ([`08f31d4`](https://github.com/sorinirimies/droidkraft/commit/08f31d43029ac2eb54d855376f7e130dbca3aaf6))

### 📚 Documentation

- **release:** GUI+TUI release notes with per-format downloads table; cliff packaging group; README downloads section ([`39c6357`](https://github.com/sorinirimies/droidkraft/commit/39c635779cdbee4c8f3b1127f80b46bad0fb170b))

### 🧪 Tests

- Assert flatpak appstream-compose off ([`ce47960`](https://github.com/sorinirimies/droidkraft/commit/ce4796055d7a0d440721ee711b5815a8eb17faa8))

### 🔄 CI and release

- **packages:** Zigbuild portable TUI (glibc 2.17, x86_64+aarch64) replaces arm runner ([`a61cada`](https://github.com/sorinirimies/droidkraft/commit/a61cadaba72f10ea598c71ae57de15c5ae7e8196))
- **packages:** Linux-only runners — drop macOS/Windows jobs, cross-compile Windows TUI with cargo-xwin ([`fa592b2`](https://github.com/sorinirimies/droidkraft/commit/fa592b2b7b4177b9f5c004061623716c3f821a56))

## [2.1.0](https://github.com/sorinirimies/droidkraft/compare/v2.0.0...v2.1.0) — 2026-10-16

### 📦 Dependencies

- Pin libc 0.2.189 — 0.2.190 drops ENOATTR and breaks xattr 0.2.3 (gpui) on Linux ([`fd403ac`](https://github.com/sorinirimies/droidkraft/commit/fd403ac0aca23cd2fd241384a42d999629e368af))

## [2.0.0](https://github.com/sorinirimies/droidkraft/compare/v1.2.0...v2.0.0) — 2026-10-16

### ✨ Features

- **packaging:** Tarballs + binstall, deb, rpm, AUR PKGBUILD, Homebrew formula, Nix flake, aarch64 TUI ([`82b1ec5`](https://github.com/sorinirimies/droidkraft/commit/82b1ec5472151cd6d2c1ff28835da281b3a182ab))
- **logs:** GUI search box, fuzzy search mode in core/TUI/GUI, regex/fuzzy/text cycling ([`022bb91`](https://github.com/sorinirimies/droidkraft/commit/022bb91df4afed54e7da016033ab0877b7f93628))
- Thorough responsive design for TUI (wrapping footer, fitted header/tabs/banner/help) and GUI (adaptive tab bar), with tests ([`b10bcdf`](https://github.com/sorinirimies/droidkraft/commit/b10bcdf026787a1d7f4b75a9e0e84a32ad57ce26))
- Adjustable refresh rate for stats/graphs (core RefreshRate, TUI </>, GUI Refresh −/+ control) ([`1ecb702`](https://github.com/sorinirimies/droidkraft/commit/1ecb70233a1cb7b6cc86fbc2d9ae0c61191cdae4))
- **fastboot:** Ergonomic API (flash/erase/getvar/variables/reboot*/flash_factory), docs + README; polish(gui): stable sparkline bars ([`464e26a`](https://github.com/sorinirimies/droidkraft/commit/464e26a6c0d2268d4d7ee8d9669f451d472af53c))
- **gui:** Device-centric redesign, responsive layouts, recording UI, per-device mirror windows ([`1768802`](https://github.com/sorinirimies/droidkraft/commit/1768802bd35fc12b5bb0946f7cfa9be42af87f8c))
- **tui:** Device-centric redesign, responsive layouts, direct Kitty graphics, braille pie chart ([`57836fe`](https://github.com/sorinirimies/droidkraft/commit/57836fe46f56ac589b79d514a23a53f7513b6d91))
- Droidkraft-mirror crate (screen mirror + screen recorder shared by TUI/GUI) ([`a5a5f26`](https://github.com/sorinirimies/droidkraft/commit/a5a5f2630a02242e1f3037c50b0f65f0862447ef))
- Pure-Rust fastboot crate, device details API, native ADB sideload ([`61bbcae`](https://github.com/sorinirimies/droidkraft/commit/61bbcaef1e6a94b5eade173fd17dfa0177382240))

### 🐛 Bug fixes

- Fmt; make screen no-frame test deterministic without adb ([`ca74259`](https://github.com/sorinirimies/droidkraft/commit/ca74259ad16d82f91fe72dffeb578ae608c55046))
- Packaging test + script refs after nushell port ([`e462ee3`](https://github.com/sorinirimies/droidkraft/commit/e462ee32904751fd5a0ef03e5fd25ed5d05b3443))
- **core:** Threadtime log parser panicked on multibyte chars (found by fuzz CI) ([`c72fcb3`](https://github.com/sorinirimies/droidkraft/commit/c72fcb35f095574601605b451ccc18e2f7bf25e0))
- **gui:** Robust nav rail (icon-only when narrow, device icons, text budgets, scrolling list) ([`3874574`](https://github.com/sorinirimies/droidkraft/commit/38745749e3f29a035b9f811baa511a4ffb07eedf))

### 📚 Documentation

- Add TUI/GUI install, audit and fuzz badges ([`f054990`](https://github.com/sorinirimies/droidkraft/commit/f05499065f2dfb15f25ae00388fe746f8cba5d88))
- Rewrite README, new VHS tapes and preview GIFs, drop stale docs ([`0c3f94b`](https://github.com/sorinirimies/droidkraft/commit/0c3f94b5b1e2655c83d16bc37a0a4cde9710a149))

### 🔄 CI and release

- **release:** Publish mirror + fastboot crates before core/tui/gui ([`8370629`](https://github.com/sorinirimies/droidkraft/commit/8370629140b7e495f0f8b6b7211792189e78ef52))
- Package workflow (AppImage, Flatpak, Windows, macOS), icons, prune old releases ([`e6330b5`](https://github.com/sorinirimies/droidkraft/commit/e6330b501f8ea2fe12e9c7f5a93f9c250a8fb0cb))
- Prebuilt Nushell, cargo-deny + cargo-audit security workflow ([`6b407ed`](https://github.com/sorinirimies/droidkraft/commit/6b407ed50a6f39b3a310120a7249c933e31f7260))
- **security:** Add cargo-audit workflow and just audit recipe ([`e70c29f`](https://github.com/sorinirimies/droidkraft/commit/e70c29f94af060d24c2d4bc72521b9dbee219b92))
- Trigger Gitea Actions (retest after adding runners) ([`a0a5055`](https://github.com/sorinirimies/droidkraft/commit/a0a5055edc0f8d38558dbe5db8077355edcecdaa))

### 🔧 Chores

- Packaging scripts in nushell, packaging asset tests, prune tests ([`a8fc6f7`](https://github.com/sorinirimies/droidkraft/commit/a8fc6f71b230c4579683ac9a226092b1cdfe3917))
- Drop python; icon generator as Rust example, soak as nushell ([`cc123d9`](https://github.com/sorinirimies/droidkraft/commit/cc123d9cb715c5a1ecd70da95c1272efbaaefd39))
- **just:** Port tokenburn recipes (publish-*, release-*, coverage, update-deps, ...), add deps-update workflow ([`b8f5795`](https://github.com/sorinirimies/droidkraft/commit/b8f5795be7e1ebb990c5fcbffa1502b82ca187b4))
- Fuzz targets + fix UTF-8 panic in brief log parser, security/fuzz/soak recipes and workflows, gitea-starscream remote, bounded mirror input queue, cargo update ([`f666ea9`](https://github.com/sorinirimies/droidkraft/commit/f666ea9e135830cb19d8a90bdbe5d776da696aea))
- **justfile:** Push/pull current branch instead of hardcoded main ([`379e9ef`](https://github.com/sorinirimies/droidkraft/commit/379e9ef6668b55bef3a1d4d1b855342d5fa4eed8))
- **justfile:** Skip unconfigured remotes in *-all recipes ([`d2e70a2`](https://github.com/sorinirimies/droidkraft/commit/d2e70a2dcec5935ed9edb337caa64f6311957321))

## [1.2.0](https://github.com/sorinirimies/droidkraft/compare/v1.1.0...v1.2.0) — 2026-07-19

### ✨ Features

- **rom:** Big catalog + GrapheneOS + multi-source resolution ([`7fdcf85`](https://github.com/sorinirimies/droidkraft/commit/7fdcf85253b0d55cb368c9eab17a2655e1af15e8))

## [1.1.0](https://github.com/sorinirimies/droidkraft/compare/v1.0.1...v1.1.0) — 2026-07-17

### ✨ Features

- **tui:** One-stop custom-ROM flasher screen ([`9de0730`](https://github.com/sorinirimies/droidkraft/commit/9de0730276e46013f673b1fa8b78dbaf3322572e))
- **gui:** One-stop custom-ROM flashing in Flash & Root panel ([`640568e`](https://github.com/sorinirimies/droidkraft/commit/640568ef8f78ca4292843dbe88432b6faad1c153))
- **core:** Custom-ROM flashing engine (features::rom) ([`c3f6dbb`](https://github.com/sorinirimies/droidkraft/commit/c3f6dbb9cb69c257453d7b194a6efb967d9f52ed))

### 📚 Documentation

- Document one-stop custom-ROM flasher across READMEs ([`a18519c`](https://github.com/sorinirimies/droidkraft/commit/a18519c366bf490efe142920023cfa47a333da5f))

## [1.0.1](https://github.com/sorinirimies/droidkraft/compare/v1.0.0...v1.0.1) — 2026-07-16

### ✨ Features

- **gui:** Make GUI publishable to crates.io as `droidkraft` ([`e40cba4`](https://github.com/sorinirimies/droidkraft/commit/e40cba406af1886a6409d1bacb0187ff64bd1285))

## [1.0.0](https://github.com/sorinirimies/droidkraft/compare/v0.6.1...v1.0.0) — 2026-07-13

### 🐛 Bug fixes

- **ci:** Make nu test runner portable across Nushell versions ([`451638c`](https://github.com/sorinirimies/droidkraft/commit/451638c8f67ed3867af5900148fe16b88ec20d8e))

### ♻️ Refactor

- Deep scan — remove dead code, share logic via core public API ([`9cc4616`](https://github.com/sorinirimies/droidkraft/commit/9cc4616204eb86fe833839030beb1934b3c28415))
- Prune dead macro, DRY GUI error mapping via str_err trait ([`7ad55cf`](https://github.com/sorinirimies/droidkraft/commit/7ad55cf6fc8f969c943dfe08559bccf3c818084e))

### 📚 Documentation

- Regenerate VHS previews (Git LFS), refresh all READMEs ([`881b94c`](https://github.com/sorinirimies/droidkraft/commit/881b94cc9cfa54525c707509771bb20ee8809aa5))

### 🔄 CI and release

- Pin Nushell to 0.111.0 in CI and release workflows ([`2486bbb`](https://github.com/sorinirimies/droidkraft/commit/2486bbba0e414944983dcb3f692ac1d8f291ec00))

## [0.6.1](https://github.com/sorinirimies/droidkraft/compare/v0.6.0...v0.6.1) — 2026-07-10

### ⚠️ Breaking changes

- Rename project droidtui → droidkraft ([`92ded36`](https://github.com/sorinirimies/droidkraft/commit/92ded3654d78c58f1ebbafe69a6ad95a21e500a6))

### 🐛 Bug fixes

- **clippy:** Remove redundant references in format args (rust 1.97) ([`6aa5ea7`](https://github.com/sorinirimies/droidkraft/commit/6aa5ea778f28ee4ec957333f20b0206a143016b0))
- Clean up rename fallout (BSD sed \b misses) + workspace-aware scripts ([`b74e85e`](https://github.com/sorinirimies/droidkraft/commit/b74e85ea25fdb2cb97d30541f744754165deacd8))

### ♻️ Refactor

- Rename project droidtui → droidkraft ([`92ded36`](https://github.com/sorinirimies/droidkraft/commit/92ded3654d78c58f1ebbafe69a6ad95a21e500a6))

## [0.6.0](https://github.com/sorinirimies/droidkraft/compare/v0.5.3...v0.6.0) — 2026-07-10

### ✨ Features

- **gui:** New droidtui-gui — Zed GPUI device monitor & toolkit ([`7c5f3bf`](https://github.com/sorinirimies/droidkraft/commit/7c5f3bf0ae5ca1593609ea7bf02fd3cb43075dd3))

### 🐛 Bug fixes

- **gui:** Compile against gpui — stateful scroll ids, AppContext prelude ([`3c5ce20`](https://github.com/sorinirimies/droidkraft/commit/3c5ce20b9368c8580709153605c01e9d14df7768))

### ♻️ Refactor

- **tui:** Consume droidtui-core logcat engine, drop duplicated domain ([`6add84c`](https://github.com/sorinirimies/droidkraft/commit/6add84c5c85a4a95c0b9610168f6b3463197276f))
- Convert to Cargo workspace, extract droidtui-core library ([`48a3ba2`](https://github.com/sorinirimies/droidkraft/commit/48a3ba2153b1b20fadcb9a9fb2f7a1cd0258d857))

### 🧪 Tests

- Workspace tooling, core API integration tests, macros polish ([`c0bcff8`](https://github.com/sorinirimies/droidkraft/commit/c0bcff81ca161fe5012ad48c55310aab43c61d2f))

### 🔧 Chores

- Workspace-aware release tooling + publishable TUI crate ([`d2a34cd`](https://github.com/sorinirimies/droidkraft/commit/d2a34cd15e2941c9e38f3f6a121929fb9b1ac09c))

## [0.5.2](https://github.com/sorinirimies/droidkraft/compare/v0.5.1...v0.5.2) — 2026-03-27

### ✨ Features

- Theme applies to entire UI + 27 presets matching tui-file-explorer ([`49aedd4`](https://github.com/sorinirimies/droidkraft/commit/49aedd4e4028276ed68b7718fa1b284a43c1d5df))
- Dev Mode — build, run, edit Android projects from the terminal ([`e0b1e3e`](https://github.com/sorinirimies/droidkraft/commit/e0b1e3e5edf440b2c4ca2977bc31c511a0555ebd))
- JSON export, soft wrap, CLI query mode, Nu recipes, formatted JSON ([`0f1172f`](https://github.com/sorinirimies/droidkraft/commit/0f1172fc16007deb723877dfa91fcf4991b1d1ad))

### 🐛 Bug fixes

- Multi-strategy Gradle resolution (wrapper → PATH → SDKMAN → Homebrew) ([`ff2893f`](https://github.com/sorinirimies/droidkraft/commit/ff2893f1c26f0d0a64401984958667d4e0187413))
- T cycles theme immediately from Menu and DevMode ([`128a5e0`](https://github.com/sorinirimies/droidkraft/commit/128a5e0cb0bbb619f1a0e6267c310d1ffd0c3fdb))
- UTF-8 panic in theme description truncation ([`1c0e858`](https://github.com/sorinirimies/droidkraft/commit/1c0e8586f873bb180e4982877d0222f7c70fb139))
- Theme applies immediately on ↑/↓/t — live preview while browsing ([`c74a57e`](https://github.com/sorinirimies/droidkraft/commit/c74a57ed41e65641f0391688c4d82e88fbac01be))
- Skip startup animation, go straight to menu ([`5013241`](https://github.com/sorinirimies/droidkraft/commit/5013241a33125a66b87af284840b9f7deedd749b))
- Walk up directory tree to find gradlew (like Gradle itself) ([`0008144`](https://github.com/sorinirimies/droidkraft/commit/0008144663c56cf13d5f20d866a9dc6afa12431b))
- Remove 'S save as' from logcat footer — only shown in save dialog ([`93697c9`](https://github.com/sorinirimies/droidkraft/commit/93697c90b435c4fef57a6b68dc5b28ab3b01a451))

### 📚 Documentation

- Rewrite README.md and FEATURES.md for v0.5 ([`1d94eb9`](https://github.com/sorinirimies/droidkraft/commit/1d94eb922e6ad4ff08df6224fdfd157bb7a65fa4))

## [0.5.1](https://github.com/sorinirimies/droidkraft/compare/v0.4.1...v0.5.1) — 2026-03-25

### 🐛 Bug fixes

- Resolve clippy field_reassign_with_default in test modules ([`d0388bf`](https://github.com/sorinirimies/droidkraft/commit/d0388bff79923810ed7f79265121707cb7ee9b5e))
- Remap S to Save As (file browser), update all hints ([`cf21586`](https://github.com/sorinirimies/droidkraft/commit/cf21586a920cc1ba617845669fa3a95dda7276a8))
- Remap find to f, fold to F, update filter bar labels ([`62dc409`](https://github.com/sorinirimies/droidkraft/commit/62dc40995720d83e1f4cc73e838f21ce482d060b))

## [0.4.1](https://github.com/sorinirimies/droidkraft/compare/v0.4.0...v0.4.1) — 2026-03-25

### ✨ Features

- Theme system + redesigned logcat shortcuts & footer ([`df0f8f8`](https://github.com/sorinirimies/droidkraft/commit/df0f8f82c0c966e449f98954fd0137d00f49d490))

## [0.4.0](https://github.com/sorinirimies/droidkraft/compare/v0.3.2...v0.4.0) — 2026-03-25

### ✨ Features

- Live Logcat Viewer with 10 pro features ([`1131f48`](https://github.com/sorinirimies/droidkraft/commit/1131f48fb13ec2f47285b1cdeb6eb2196148fcb7))

### 📚 Documentation

- Update README and CHANGELOG for v0.3.2 ([`1447d5d`](https://github.com/sorinirimies/droidkraft/commit/1447d5d5860fe15eb79b2038ea9076b44e91a8b3))

## [0.3.0](https://github.com/sorinirimies/droidkraft/compare/v0.2.9...v0.3.0) — 2025-10-19

### 📚 Documentation

- Update README and CHANGELOG for v0.2.9 ([`af93523`](https://github.com/sorinirimies/droidkraft/commit/af93523c5fcbdc6135b258e70d2bc677bfb338d8))

## [0.2.9](https://github.com/sorinirimies/droidkraft/compare/v0.2.8...v0.2.9) — 2025-10-15

### 📚 Documentation

- Update README and CHANGELOG for v0.2.8 ([`ec292a2`](https://github.com/sorinirimies/droidkraft/commit/ec292a2ff803b44f353a9b47e5bf6d07b6d86a7a))

## [0.2.7](https://github.com/sorinirimies/droidkraft/compare/v0.2.6...v0.2.7) — 2025-10-09

### ✨ Features

- Add impressive multi-layered loading animations ([`c26b707`](https://github.com/sorinirimies/droidkraft/commit/c26b7072806e5e161617e2fe70d466d687af8dc0))
- Add slide-in/slide-out animations with TachyonFX ([`4a5822f`](https://github.com/sorinirimies/droidkraft/commit/4a5822fb0e545b50fc6e46696931f9014c322ce0))

### 📚 Documentation

- Update README and CHANGELOG for v0.2.6 ([`6d84b45`](https://github.com/sorinirimies/droidkraft/commit/6d84b45687380a45ce563181468bd2fd3fb4c23b))

## [0.2.6](https://github.com/sorinirimies/droidkraft/compare/v0.2.5...v0.2.6) — 2025-10-02

### ✨ Features

- Add automatic changelog generation with git-cliff ([`1473c46`](https://github.com/sorinirimies/droidkraft/commit/1473c46258a15a70c9b657f89e8144587eee8696))

### 🐛 Bug fixes

- Remove syntax error from bump_version.sh script ([`da7a7e6`](https://github.com/sorinirimies/droidkraft/commit/da7a7e6258313a04ca5977f7fd527b948e5fb31d))

### 📚 Documentation

- Add quick release guide ([`5c6a863`](https://github.com/sorinirimies/droidkraft/commit/5c6a86339808bf5b360ce27868f9bb17a419e541))

## [0.2.5](https://github.com/sorinirimies/droidkraft/compare/v0.2.4...v0.2.5) — 2025-10-02

### 📚 Documentation

- Add release automation section to README ([`048f27c`](https://github.com/sorinirimies/droidkraft/commit/048f27cd75639210dab398a434bc077297f14399))
- Add comprehensive release process documentation ([`97045a8`](https://github.com/sorinirimies/droidkraft/commit/97045a860be445746a4733041be485842510e9f6))

### 🔧 Chores

- Add version automation tools ([`e46d8c2`](https://github.com/sorinirimies/droidkraft/commit/e46d8c241e6a6d412d780b0ce0896489b2864c06))

## [0.2.4](https://github.com/sorinirimies/droidkraft/compare/v0.2.3...v0.2.4) — 2025-10-02

### 🐛 Bug fixes

- Add spacing between layout sections to fix broken borders ([`f7c429f`](https://github.com/sorinirimies/droidkraft/commit/f7c429fd5c49aeac64775fc66f22206296983532))

### ♻️ Refactor

- Migrate to Elm architecture for better code organization ([`ea57627`](https://github.com/sorinirimies/droidkraft/commit/ea576276332a4ad80d487fc51f002a73d45489de))

## [0.2.3](https://github.com/sorinirimies/droidkraft/compare/v0.2.2...v0.2.3) — 2025-10-01

### ✨ Features

- Require passing tests before release ([`5aafccf`](https://github.com/sorinirimies/droidkraft/commit/5aafccf26171d4300a87b146d68690428daf1e45))

### 🐛 Bug fixes

- Add rustfmt and clippy components to test job ([`7895679`](https://github.com/sorinirimies/droidkraft/commit/7895679bc73afb06b755d353e743607b6d65f30f))

### 📚 Documentation

- Update version badge to 0.2.3 ([`99ff558`](https://github.com/sorinirimies/droidkraft/commit/99ff558004a36f7dae7992f0dcbce72de68c032e))

## [0.2.2](https://github.com/sorinirimies/droidkraft/compare/v0.2.1...v0.2.2) — 2025-10-01

### ✨ Features

- Improve crates.io publishing with conditional execution ([`025be9c`](https://github.com/sorinirimies/droidkraft/commit/025be9cd9618c2e7c4c50231ad5b10e9c4124019))

### 📚 Documentation

- Add release badges and crates.io installation instructions ([`65fc9f1`](https://github.com/sorinirimies/droidkraft/commit/65fc9f106d7b44188fa8a3413b95653f9368b877))

### 🔄 CI and release

- Test on both Linux and Windows ([`0eaad62`](https://github.com/sorinirimies/droidkraft/commit/0eaad6299a0a94b530f9ec0abdd70c194d5aba14))
- Remove macOS from test matrix ([`f836e16`](https://github.com/sorinirimies/droidkraft/commit/f836e160e81eadffaff4916fda45b83e73b4f71e))

## [0.2.1](https://github.com/sorinirimies/droidkraft/compare/v0.2.0...v0.2.1) — 2025-10-01

### 🐛 Bug fixes

- Use Python for cross-platform version updates ([`fb5013e`](https://github.com/sorinirimies/droidkraft/commit/fb5013e9e79036054db01a0260ee43ddb3d665dc))
- Make sed commands cross-platform compatible ([`ed7c2b2`](https://github.com/sorinirimies/droidkraft/commit/ed7c2b2e32e369dc911a0ffcd63ea8c88475be0b))

### ♻️ Refactor

- Simplify release workflow to publish single crate ([`522bb0f`](https://github.com/sorinirimies/droidkraft/commit/522bb0f1409ba32a11e431cf83aaca0360e2b939))

## [0.2.0](https://github.com/sorinirimies/droidkraft/compare/v0.1.0...v0.2.0) — 2025-09-27

### ✨ Features

- Add scrollable command results and fix GitHub Actions ([`b99c7e1`](https://github.com/sorinirimies/droidkraft/commit/b99c7e1085d9e7d67a18d5b5ffb123a86594c3af))

## [0.1.0](https://github.com/sorinirimies/droidkraft/releases/tag/v0.1.0) — 2025-09-27


