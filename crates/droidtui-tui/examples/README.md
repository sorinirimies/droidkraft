# DroidTUI Examples

This directory contains standalone examples demonstrating various features of DroidTUI. Each example can be run independently and showcases specific functionality.

## Running Examples

All examples can be run using Cargo:

```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example main_menu
```

## Available Examples

### 1. Main Menu (`main_menu.rs`)

A comprehensive example showing the basic menu system with navigation and command execution.

**Features:**
- 13 ADB commands organized in a menu
- Vim-like navigation (j/k keys)
- Command descriptions and details panel
- Help screen with keyboard shortcuts
- Android green theming

**Run:**
```bash
cargo run --example main_menu
```

**Use Case:** Understanding the basic menu structure and navigation patterns used in DroidTUI.

---

### 2. Screen Streaming (`streaming.rs`)

Demonstrates screen streaming capabilities with scrcpy integration and multiple quality presets.

**Features:**
- 10 different streaming quality options
- Standard, High, Low Latency, Gaming modes
- Recording session support
- Read-only and borderless window modes
- Detailed command and quality information

**Run:**
```bash
cargo run --example streaming
```

**Prerequisites:** scrcpy must be installed on your system
- macOS: `brew install scrcpy`
- Linux: `sudo apt install scrcpy`
- Windows: `scoop install scrcpy`

**Use Case:** Understanding streaming options and scrcpy integration.

---

### 3. Device Information (`device_info.rs`)

Query and display comprehensive device information using ADB commands.

**Features:**
- 16 information categories
- Hardware specs (CPU, Memory, Display)
- System info (Android version, Build, Kernel)
- Network status (WiFi, Bluetooth)
- Device identifiers (Serial, IMEI)
- Security information (Patch level)
- Color-coded categories

**Run:**
```bash
cargo run --example device_info
```

**Use Case:** Learning how to query various device properties and system information.

---

### 4. Package Manager (`package_manager.rs`)

Comprehensive package management example with 20 different package operations.

**Features:**
- Query packages (all, user, system, enabled, disabled)
- Package information and permissions
- Install and uninstall applications
- Enable/disable apps
- Grant/revoke permissions
- Clear cache and data
- Force stop apps
- Safety warnings for destructive operations

**Run:**
```bash
cargo run --example package_manager
```

**Use Case:** Understanding Android package management through ADB.

---

## Example Architecture

All examples follow a similar architecture pattern:

```rust
struct App {
    selected: usize,
    items: Vec<MenuItem>,
    should_quit: bool,
    show_help: bool,
}
```

### Key Components

1. **State Management**: Each example maintains its own application state
2. **Event Handling**: Keyboard input processing for navigation and actions
3. **UI Rendering**: Ratatui-based terminal UI with consistent styling
4. **Help System**: Toggle-able help screen with keyboard shortcuts

### Common Keyboard Shortcuts

All examples share these common shortcuts:

| Key | Action |
|-----|--------|
| `↑` or `k` | Move up |
| `↓` or `j` | Move down |
| `Enter` or `Space` | Select/Execute |
| `h` or `?` | Toggle help |
| `q` or `Esc` | Quit |
| `Backspace` | Go back (in full app) |

### Color Scheme

Examples use the Android green theme:
- **Android Green**: `RGB(61, 220, 132)` - Primary color for highlights
- **White**: Default text color
- **Yellow**: Command text
- **Gray**: Secondary information
- **Color-coded categories**: For organization and visual clarity

## VHS Demos

Each example has a corresponding VHS tape file for generating animated GIF demos. These are located in `examples/vhs/`:

- `main_menu.tape` - Main menu demo
- `streaming.tape` - Streaming options demo
- `device_info.tape` - Device info queries demo
- `package_manager.tape` - Package management demo
- `navigation_showcase.tape` - Navigation features demo
- `quickstart.tape` - Quick 30-second demo
- `full_demo.tape` - Complete application demo

### Generating Demos

Generate all demo GIFs:
```bash
./examples/vhs/generate_all.sh
```

Generate a specific demo:
```bash
vhs examples/vhs/main_menu.tape
```

See [vhs/README.md](vhs/README.md) for detailed information about VHS demos.

## Creating New Examples

To create a new example:

1. **Create the example file**: `examples/your_example.rs`
2. **Add the example to Cargo.toml**:
   ```toml
   [[example]]
   name = "your_example"
   path = "examples/your_example.rs"
   ```
3. **Follow the established pattern**:
   - Use the App struct pattern
   - Implement common keyboard shortcuts
   - Use Android green theming
   - Include a help screen
4. **Create a VHS tape**: `examples/vhs/your_example.tape`
5. **Update documentation**:
   - Add to this README
   - Update `vhs/README.md`
   - Update `generate_all.sh`

### Example Template

```rust
//! Your Example
//!
//! Description of what this example demonstrates.
//!
//! Run with: cargo run --example your_example

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io::{self, stdout};

const ANDROID_GREEN: Color = Color::Rgb(61, 220, 132);

struct App {
    selected: usize,
    items: Vec<Item>,
    should_quit: bool,
    show_help: bool,
}

// ... implement your example
```

## Learning Path

Recommended order for exploring examples:

1. **main_menu.rs** - Start here to understand basic navigation
2. **device_info.rs** - Learn about querying device information
3. **package_manager.rs** - Explore package management operations
4. **streaming.rs** - Understand streaming integration

## Requirements

All examples require:
- Rust 1.70 or later
- A terminal with true color support
- For actual command execution: ADB installed and configured

Some examples have additional requirements:
- **streaming.rs**: scrcpy installed on your system
- **Full functionality**: Android device connected via USB or WiFi

## Tips

- **Development Mode**: Examples don't actually execute ADB commands by default. Check `stderr` to see what would be executed.
- **Testing**: Connect an Android device to test actual ADB command execution
- **Customization**: Feel free to modify examples to test different commands or UI layouts
- **Performance**: Examples are optimized for smooth 60fps rendering

## Contributing

When contributing new examples:

1. Follow the established code style and architecture
2. Include comprehensive documentation
3. Create a corresponding VHS tape for demos
4. Update all relevant README files
5. Test on multiple terminal emulators
6. Ensure examples work without device connected (demo mode)

## Resources

- [Main DroidTUI README](../README.md)
- [VHS Documentation](vhs/README.md)
- [Ratatui Documentation](https://docs.rs/ratatui)
- [ADB Documentation](https://developer.android.com/studio/command-line/adb)

## License

Examples are part of DroidTUI and licensed under the same terms as the main project.

---

**Happy coding! 🤖✨**