# Preview recordings

- `tui-*.tape` are [VHS](https://github.com/charmbracelet/vhs) scripts for the terminal UI. They need a connected device and a release build (`cargo build --release -p droidkraft-tui`); run `just previews` or `just vhs tui-overview`.
- `gui-overview.gif` is a screen capture of the GPUI app (`cargo run --release -p droidkraft`, window ~1100x793), cropped and converted with `ffmpeg`.
- The Screen tab is intentionally not part of the previews for now (it would show the phone's screen).
