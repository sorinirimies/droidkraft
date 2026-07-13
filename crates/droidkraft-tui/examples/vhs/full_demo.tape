# VHS tape file for demonstrating full droidkraft application
# Run with: vhs examples/vhs/full_demo.tape

# Output settings
Output examples/vhs/full_demo.gif
Set Shell "bash"
Set FontSize 14
Set Width 1400
Set Height 900
Set Padding 20
Set Theme "Dracula"

# Show intro screen
Type "# 🤖 DroidKraft - Full Application Demo"
Sleep 1s
Enter
Sleep 500ms
Type "# A beautiful TUI for Android development with ADB"
Sleep 1s
Enter
Sleep 500ms
Type "# Features: Menu navigation, command execution, screen streaming"
Sleep 1.5s
Enter
Sleep 500ms

# Clear and run the application
Type "clear"
Enter
Sleep 500ms
Type "droidkraft-tui"
Enter

# Wait for startup animation to complete
Sleep 3s

# Press any key to continue past startup screen
Space
Sleep 1s

# Now we're in the main menu
# Navigate through different menu items
Type "j"
Sleep 700ms
Type "j"
Sleep 700ms

# Enter List Packages submenu
Enter
Sleep 1s

# Navigate through child options
Type "j"
Sleep 700ms
Type "j"
Sleep 700ms

# Go back to main menu
Backspace
Sleep 1s

# Navigate to Battery Info
Type "j"
Sleep 700ms

# Enter Battery Info submenu
Enter
Sleep 1s

# Navigate child options
Type "j"
Sleep 700ms

# Select Battery Level Only option
Sleep 1s

# Go back to main menu
Backspace
Sleep 1s

# Navigate down to Memory Usage
Type "j"
Sleep 700ms
Type "j"
Sleep 700ms

# Enter Memory Usage submenu
Enter
Sleep 1s

# Go back
Backspace
Sleep 1s

# Navigate to CPU Info
Type "j"
Sleep 700ms

# Navigate to Network Info
Type "j"
Sleep 700ms

# Navigate to Device Properties
Type "j"
Sleep 700ms

# Enter Device Properties submenu
Enter
Sleep 1s

# Navigate child menu
Type "j"
Sleep 700ms
Type "j"
Sleep 700ms

# Go back to main menu
Backspace
Sleep 1s

# Navigate down to Screenshot
Down 2
Sleep 700ms

# Navigate to System Log
Type "j"
Sleep 700ms
Type "j"
Sleep 700ms

# Enter System Log submenu
Enter
Sleep 1s

# Navigate options
Type "j"
Sleep 700ms

# Go back
Backspace
Sleep 1s

# Navigate to Screen Stream
Type "j"
Sleep 700ms

# Enter Screen Stream submenu
Enter
Sleep 1.5s

# Show different streaming options
Type "j"
Sleep 700ms
Type "j"
Sleep 700ms

# Go back to main menu
Backspace
Sleep 1s

# Navigate to ADB Version
Type "j"
Sleep 700ms

# Enter ADB Version submenu
Enter
Sleep 1.5s

# Navigate through version options
Type "j"
Sleep 700ms

# Go back to main menu
Backspace
Sleep 1s

# Navigate back to top
Up 5
Sleep 700ms
Up 5
Sleep 700ms

# Final position on List Devices
Sleep 1.5s

# Quit application
Type "q"
Sleep 1s

# Show completion message
Type "# Demo complete! ✨"
Sleep 1s
Enter
Sleep 500ms
Type "# DroidKraft features:"
Sleep 800ms
Enter
Type "#   • 14 ADB command categories"
Sleep 800ms
Enter
Type "#   • Expandable sub-menus"
Sleep 800ms
Enter
Type "#   • TachyonFX animations"
Sleep 800ms
Enter
Type "#   • Screen streaming support"
Sleep 800ms
Enter
Type "#   • Vim-like navigation"
Sleep 800ms
Enter
Type "#   • Beautiful Android green theme"
Sleep 2s
Enter
Sleep 500ms
Type "# Install: cargo install droidkraft"
Sleep 2s
