# VHS tape file for highlighting best features of droidkraft
# Run with: vhs examples/vhs/features_highlight.tape

# Output settings
Output examples/vhs/features_highlight.gif
Set Shell "bash"
Set FontSize 14
Set Width 1400
Set Height 900
Set Padding 20
Set Theme "Dracula"

# Show intro screen
Type "# ✨ DroidKraft - Features Highlight"
Sleep 1.5s
Enter
Sleep 500ms
Type "# The most powerful Android TUI experience"
Sleep 1.5s
Enter
Sleep 500ms

# Clear and show feature overview
Type "clear"
Enter
Sleep 500ms
Type "# 🎯 Key Features:"
Sleep 1s
Enter
Type "#   ✅ 14+ ADB command categories"
Sleep 700ms
Enter
Type "#   ✅ TachyonFX animations"
Sleep 700ms
Enter
Type "#   ✅ Screen streaming with scrcpy"
Sleep 700ms
Enter
Type "#   ✅ Vim-like navigation"
Sleep 700ms
Enter
Type "#   ✅ Multi-level menus"
Sleep 700ms
Enter
Type "#   ✅ Android green theming"
Sleep 1.5s
Enter
Sleep 500ms

# === Feature 1: Startup Animation ===
Type "clear"
Enter
Sleep 500ms
Type "# Feature 1: Beautiful Startup Animation 🎬"
Sleep 1.5s
Enter
Type "droidkraft-tui"
Enter

# Wait for full startup animation
Sleep 3.5s

# Continue
Space
Sleep 1s

# === Feature 2: Smooth Navigation ===
Type "# Smooth vim-like navigation"
Sleep 500ms

# Demonstrate smooth scrolling
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

Type "k"
Sleep 400ms
Type "k"
Sleep 400ms
Type "k"
Sleep 400ms

Sleep 1s

# === Feature 3: Multi-level Menus ===
Type "# Multi-level menu system"
Sleep 500ms

# Enter submenu
Enter
Sleep 1s

# Navigate child options
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms

Sleep 1s

# Back to main menu
Backspace
Sleep 1s

# === Feature 4: Rich Command Categories ===
Type "# 14+ command categories"
Sleep 500ms

# Quick tour through categories
Down 3
Sleep 600ms
Down 2
Sleep 600ms
Down 3
Sleep 600ms
Down 2
Sleep 600ms

Sleep 1s

# === Feature 5: Detailed Submenus ===
Type "# Detailed sub-options"
Sleep 500ms

# Enter a feature-rich submenu
Enter
Sleep 1s

# Show multiple options
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms

Sleep 1.5s

# Back
Backspace
Sleep 1s

# === Feature 6: Navigation Variety ===
Type "# Multiple navigation methods"
Sleep 500ms

# Navigate with arrows
Down 5
Sleep 600ms

# Navigate with j
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms

# Navigate with k
Type "k"
Sleep 600ms

# Quick scroll up
Up 8
Sleep 800ms

Sleep 1s

# === Feature 7: Consistent UI ===
Type "# Beautiful Android green theme"
Sleep 500ms

# Navigate to show consistent styling
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms
Enter
Sleep 1s
Type "j"
Sleep 600ms
Backspace
Sleep 1s

# === Feature 8: Deep Menu Support ===
Type "# Deep menu navigation"
Sleep 500ms

Down 6
Sleep 600ms
Enter
Sleep 1s
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms
Sleep 1s
Backspace
Sleep 1s

# === Finale: Quick Feature Demo ===
Type "# Lightning fast workflow"
Sleep 500ms

# Rapid navigation
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Enter
Sleep 800ms
Type "j"
Sleep 300ms
Backspace
Sleep 800ms
Down 5
Sleep 500ms
Enter
Sleep 800ms
Backspace
Sleep 800ms

# Quit
Type "q"
Sleep 1s

# Show feature summary
Type "# DroidKraft Features Complete! 🚀"
Sleep 1s
Enter
Sleep 500ms
Type "# What makes DroidKraft special:"
Sleep 800ms
Enter
Type "#   🎨 TachyonFX powered animations"
Sleep 700ms
Enter
Type "#   ⚡ Lightning fast vim navigation"
Sleep 700ms
Enter
Type "#   📱 14 ADB command categories"
Sleep 700ms
Enter
Type "#   🎯 Multi-level expandable menus"
Sleep 700ms
Enter
Type "#   📺 Screen streaming integration"
Sleep 700ms
Enter
Type "#   🎬 60fps smooth rendering"
Sleep 700ms
Enter
Type "#   🎨 Beautiful Android green theme"
Sleep 700ms
Enter
Type "#   🔧 Comprehensive device control"
Sleep 1s
Enter
Sleep 500ms
Type "# Install now: cargo install droidkraft"
Sleep 2s
