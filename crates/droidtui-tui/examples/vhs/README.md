# VHS Demos for DroidTUI

This directory contains [VHS](https://github.com/charmbracelet/vhs) tape files for generating animated demos of droidtui.

## What is VHS?

VHS is a tool for generating terminal GIFs. It allows you to script terminal sessions and produce high-quality animated demos automatically.

## Prerequisites

### Install VHS

**macOS:**
```bash
brew install vhs
```

**Linux:**
```bash
# Download the latest release from:
# https://github.com/charmbracelet/vhs/releases

# Or use go install:
go install github.com/charmbracelet/vhs@latest
```

**Windows:**
```bash
# Using scoop
scoop install vhs

# Or download from releases
```

### Install ttyd (Required by VHS)

VHS requires `ttyd` for terminal recording:

**macOS:**
```bash
brew install ttyd
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install ttyd

# Arch
sudo pacman -S ttyd
```

## Available Tapes

### 1. `quickstart.tape`

**Quick 30-second demo** showcasing basic features.

**Generate:**
```bash
vhs examples/vhs/quickstart.tape
```

**Output:** `quickstart.gif` (~1-2 MB)

**Shows:**
- Application launch with startup animation
- Basic menu navigation
- Entering and exiting submenus
- Simple workflow

**Use case:** Social media posts, README badges, quick previews

---

### 2. `main_menu.tape`

**Standalone example demo** featuring the `main_menu.rs` example.

**Generate:**
```bash
vhs examples/vhs/main_menu.tape
```

**Output:** `main_menu.gif` (~2-3 MB)

**Shows:**
- Vim-like navigation (j/k keys)
- Menu item selection and highlighting
- Command details panel
- Help screen toggle
- Full navigation workflow

**Use case:** Documentation, tutorials, example showcases

---

### 3. `full_demo.tape`

**Complete feature showcase** of the full droidtui application.

**Generate:**
```bash
vhs examples/vhs/full_demo.tape
```

**Output:** `full_demo.gif` (~4-6 MB)

**Shows:**
- TachyonFX startup animation
- All 14 ADB command categories
- Multi-level submenu navigation
- Various command options
- Complete application workflow
- Android green theming throughout

**Use case:** Project presentations, detailed walkthroughs, marketing

---

### 4. `streaming.tape`

**Screen streaming demo** featuring the streaming example with scrcpy integration.

**Generate:**
```bash
vhs examples/vhs/streaming.tape
```

**Output:** `streaming.gif` (~3-5 MB)

**Shows:**
- Multiple quality presets (480p to 1080p+)
- Gaming mode with 60fps support
- Recording capabilities
- Read-only viewing mode
- Various streaming options

**Use case:** Showcasing screen streaming features, scrcpy integration demos

---

### 5. `device_info.tape`

**Device information query demo** showing how to query various device details.

**Generate:**
```bash
vhs examples/vhs/device_info.tape
```

**Output:** `device_info.gif` (~3-4 MB)

**Shows:**
- Hardware information queries
- System properties display
- Network status information
- Battery and storage details
- Security patch information

**Use case:** Device information feature demos, ADB query examples

---

### 6. `navigation_showcase.tape`

**Navigation features demo** highlighting vim-like navigation and smooth animations.

**Generate:**
```bash
vhs examples/vhs/navigation_showcase.tape
```

**Output:** `navigation_showcase.gif` (~4-5 MB)

**Shows:**
- Vim-style j/k navigation
- Arrow key support
- Menu and submenu navigation
- Quick scrolling
- Smooth animations

**Use case:** UI/UX demonstrations, navigation tutorials

---

### 7. `package_manager.tape`

**Package management demo** featuring app installation, permissions, and management.

**Generate:**
```bash
vhs examples/vhs/package_manager.tape
```

**Output:** `package_manager.gif` (~3-5 MB)

**Shows:**
- Package listing and queries
- App installation/uninstallation
- Permission management
- Cache and data clearing
- Enable/disable apps

**Use case:** Package management feature demos, app management tutorials

---

### 8. `all_examples.tape`

**Comprehensive showcase** featuring all standalone examples in one demo.

**Generate:**
```bash
vhs examples/vhs/all_examples.tape
```

**Output:** `all_examples.gif` (~6-8 MB)

**Shows:**
- Quick tour of main_menu example
- Device information queries
- Package management operations
- Screen streaming options
- Brief full application demo
- All 4 standalone examples

**Use case:** Overview presentations, comprehensive feature showcase

---

### 9. `features_highlight.tape`

**Features showcase** highlighting the best and most impressive features of DroidTUI.

**Generate:**
```bash
vhs examples/vhs/features_highlight.tape
```

**Output:** `features_highlight.gif` (~5-7 MB)

**Shows:**
- TachyonFX startup animation
- Smooth vim-like navigation
- Multi-level menu system
- All 14 command categories
- Lightning fast workflow
- Beautiful theming throughout

**Use case:** Marketing material, feature presentations, social media highlights

---

## Quick Start

### Generate All Demos

Use the provided script:

```bash
./examples/vhs/generate_all.sh
```

This will generate all three GIF files automatically.

### Generate Individual Demos

From the project root:

```bash
# Quick demo
vhs examples/vhs/quickstart.tape

# Example demo
vhs examples/vhs/main_menu.tape

# Full demo
vhs examples/vhs/full_demo.tape
```

### View Generated GIFs

```bash
# macOS
open examples/vhs/quickstart.gif

# Linux
xdg-open examples/vhs/quickstart.gif

# Or use your preferred image viewer
```

## Customizing Tapes

### Basic Structure

A VHS tape file consists of commands that control the recording:

```tape
# Settings
Output output.gif
Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 800

# Actions
Type "droidtui"
Enter
Sleep 2s
Type "q"
```

### Common Commands

| Command | Description | Example |
|---------|-------------|---------|
| `Output` | Set output filename | `Output demo.gif` |
| `Set` | Configure settings | `Set FontSize 16` |
| `Type` | Type text | `Type "hello"` |
| `Enter` | Press Enter key | `Enter` |
| `Sleep` | Wait duration | `Sleep 1s` |
| `Up/Down` | Arrow keys | `Down 3` |
| `Space` | Press spacebar | `Space` |
| `Backspace` | Press backspace | `Backspace` |
| `Ctrl+C` | Control key combo | `Ctrl+C` |

### Settings

Common VHS settings used in our tapes:

```tape
Set Shell "bash"          # Shell to use
Set FontSize 14           # Terminal font size
Set Width 1200            # Terminal width in pixels
Set Height 800            # Terminal height in pixels
Set Padding 20            # Padding around terminal
Set Theme "Dracula"       # Color theme
Set TypingSpeed 50ms      # Speed of typing
Set PlaybackSpeed 1.0     # Playback speed multiplier
```

### Available Themes

VHS supports many themes:
- `Dracula` (default in our tapes)
- `Nord`
- `Monokai`
- `Solarized Dark`
- `Solarized Light`
- `Gruvbox Dark`
- And many more...

## Creating New Tapes

### Template

```tape
# Output settings
Output examples/vhs/my_demo.gif
Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 800
Set Padding 20
Set Theme "Dracula"

# Your demo script
Type "droidtui"
Enter
Sleep 2s

# ... your actions ...

Type "q"
Sleep 1s
```

### Best Practices

1. **Keep it focused** - Each tape should demonstrate one concept
2. **Use appropriate timing** - 500ms-1s for quick actions, 1.5-2s for viewing
3. **Add context** - Use `Type "# Comment"` to explain what's happening
4. **Test multiple times** - Run the tape several times to ensure consistency
5. **Optimize file size** - Shorter recordings = smaller GIFs
6. **Clear state** - Start with `clear` or fresh terminal state
7. **End cleanly** - Always quit the application properly

### Tips for Recording droidtui

```tape
# Wait for startup animation (important!)
Sleep 3s

# Press any key to continue past startup
Space
Sleep 1s

# Use navigation keys
Type "j"    # Move down
Type "k"    # Move up
Enter       # Select/Enter submenu
Backspace   # Go back to main menu
Type "q"    # Quit

# Add pauses to show content
Sleep 1.5s  # Let viewers read
```

## Troubleshooting

### VHS Command Not Found

Ensure VHS is in your PATH:
```bash
which vhs
# If not found, reinstall or check installation
```

### ttyd Not Found

Install ttyd (required by VHS):
```bash
# macOS
brew install ttyd

# Linux
sudo apt install ttyd  # Ubuntu/Debian
```

### GIF Not Generated

Check for errors in the VHS output:
```bash
vhs examples/vhs/quickstart.tape --verbose
```

### Recording Too Long

- Reduce sleep times
- Remove unnecessary navigation
- Focus on key features only

### Application Not Found

Make sure droidtui is built and in PATH:
```bash
cargo build --release
# Or install it
cargo install --path .
```

For the example demos:
```bash
# Make sure you can run the example
cargo run --example main_menu
```

## Output Files

Generated files (`.gif`, `.mp4`, `.webm`) are ignored by git (see `.gitignore`).

To share demos:
1. Generate the GIFs
2. Upload to GitHub releases or issues
3. Or host on image sharing services

## Resources

- [VHS Documentation](https://github.com/charmbracelet/vhs)
- [VHS Examples](https://github.com/charmbracelet/vhs/tree/main/examples)
- [Charm.sh Blog](https://charm.sh/blog/)

## Summary of Tapes

| Tape | Duration | Size | Purpose |
|------|----------|------|---------|
| `quickstart.tape` | ~30s | 1-2 MB | Quick social media demo |
| `main_menu.tape` | ~45s | 2-3 MB | Example showcase |
| `full_demo.tape` | ~60s | 4-6 MB | Complete app walkthrough |
| `streaming.tape` | ~45s | 3-5 MB | Streaming features |
| `device_info.tape` | ~50s | 3-4 MB | Device queries |
| `navigation_showcase.tape` | ~55s | 4-5 MB | Navigation demo |
| `package_manager.tape` | ~50s | 3-5 MB | Package management |
| `all_examples.tape` | ~90s | 6-8 MB | Comprehensive tour |
| `features_highlight.tape` | ~60s | 5-7 MB | Best features showcase |

## Contributing

When adding new VHS tapes:

1. Create the `.tape` file in this directory
2. Create a corresponding example in `examples/` (if needed)
3. Test it multiple times to ensure consistency
4. Update this README with tape details
5. Add the tape to `generate_all.sh` script
6. Document the use case and output
7. Consider file size and demo length (keep under 60 seconds if possible)

---

## Available Examples

DroidTUI provides several standalone examples that can be run independently:

### `main_menu.rs`
Basic menu navigation example with 13 ADB commands.
```bash
cargo run --example main_menu
```

### `streaming.rs`
Screen streaming options with scrcpy integration and quality presets.
```bash
cargo run --example streaming
```

### `device_info.rs`
Comprehensive device information queries (16 categories).
```bash
cargo run --example device_info
```

### `package_manager.rs`
Package management operations including install, uninstall, and permissions (20 actions).
```bash
cargo run --example package_manager
```

Each example demonstrates specific features and can be used as reference implementations.

---

**Made with ❤️ and VHS by Charm**