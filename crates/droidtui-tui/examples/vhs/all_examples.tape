# VHS tape file for showcasing all droidtui examples
# Run with: vhs examples/vhs/all_examples.tape

# Output settings
Output examples/vhs/all_examples.gif
Set Shell "bash"
Set FontSize 14
Set Width 1400
Set Height 900
Set Padding 20
Set Theme "Dracula"

# Show intro screen
Type "# 🤖 DroidTUI - All Examples Showcase"
Sleep 1.5s
Enter
Sleep 500ms
Type "# A comprehensive tour of all standalone examples"
Sleep 1.5s
Enter
Sleep 500ms

# === EXAMPLE 1: Main Menu ===
Type "clear"
Enter
Sleep 500ms
Type "# 📱 Example 1: Main Menu"
Sleep 1s
Enter
Type "cargo run --example main_menu"
Enter
Sleep 2.5s

# Quick navigation
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "k"
Sleep 500ms

# Show help
Type "h"
Sleep 1.5s
Type "h"
Sleep 500ms

# Quit
Type "q"
Sleep 1s

# === EXAMPLE 2: Device Info ===
Type "clear"
Enter
Sleep 500ms
Type "# 📱 Example 2: Device Information"
Sleep 1s
Enter
Type "cargo run --example device_info"
Enter
Sleep 2.5s

# Navigate through categories
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Down 3
Sleep 500ms

# Show help briefly
Type "?"
Sleep 1.5s
Type "?"
Sleep 500ms

# Quit
Type "q"
Sleep 1s

# === EXAMPLE 3: Package Manager ===
Type "clear"
Enter
Sleep 500ms
Type "# 📦 Example 3: Package Manager"
Sleep 1s
Enter
Type "cargo run --example package_manager"
Enter
Sleep 2.5s

# Navigate through actions
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Down 4
Sleep 500ms
Down 3
Sleep 500ms

# Show help
Type "h"
Sleep 1.5s
Type "h"
Sleep 500ms

# Quit
Type "q"
Sleep 1s

# === EXAMPLE 4: Streaming ===
Type "clear"
Enter
Sleep 500ms
Type "# 📺 Example 4: Screen Streaming"
Sleep 1s
Enter
Type "cargo run --example streaming"
Enter
Sleep 2.5s

# Navigate through streaming options
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Down 2
Sleep 500ms

# Show help
Type "?"
Sleep 1.5s
Type "?"
Sleep 500ms

# Quit
Type "q"
Sleep 1s

# === FINALE: Full App ===
Type "clear"
Enter
Sleep 500ms
Type "# 🚀 Bonus: Full Application"
Sleep 1s
Enter
Type "droidtui"
Enter
Sleep 3s

# Continue past startup
Space
Sleep 1s

# Quick navigation demo
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Enter
Sleep 800ms
Type "j"
Sleep 500ms
Backspace
Sleep 800ms

# Navigate more
Down 4
Sleep 500ms
Enter
Sleep 800ms
Backspace
Sleep 800ms

# Quit
Type "q"
Sleep 1s

# Show completion message
Type "# All Examples Complete! ✨"
Sleep 1s
Enter
Sleep 500ms
Type "# DroidTUI provides 4 standalone examples:"
Sleep 800ms
Enter
Type "#   1. 📱 Main Menu - Basic navigation & commands"
Sleep 800ms
Enter
Type "#   2. 📱 Device Info - Query device information"
Sleep 800ms
Enter
Type "#   3. 📦 Package Manager - Manage Android apps"
Sleep 800ms
Enter
Type "#   4. 📺 Streaming - Screen streaming options"
Sleep 800ms
Enter
Sleep 500ms
Type "# Plus the full DroidTUI application with all features!"
Sleep 1.5s
Enter
Sleep 500ms
Type "# Install: cargo install droidtui"
Sleep 2s
