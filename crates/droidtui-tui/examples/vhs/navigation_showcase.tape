# VHS tape file for demonstrating droidtui navigation features
# Run with: vhs examples/vhs/navigation_showcase.tape

# Output settings
Output examples/vhs/navigation_showcase.gif
Set Shell "bash"
Set FontSize 14
Set Width 1400
Set Height 900
Set Padding 20
Set Theme "Dracula"

# Show intro screen
Type "# ⌨️  DroidTUI Navigation Showcase"
Sleep 1s
Enter
Sleep 500ms
Type "# Vim-like navigation with smooth animations"
Sleep 1.5s
Enter
Sleep 500ms

# Clear and run the main application
Type "clear"
Enter
Sleep 500ms
Type "droidtui"
Enter

# Wait for startup animation
Sleep 3s

# Continue past startup screen
Space
Sleep 1s

# === Demonstrate basic j/k navigation ===
Type "# Basic j/k navigation"
Sleep 500ms
Enter

# Navigate down with j
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms

# Navigate up with k
Type "k"
Sleep 600ms
Type "k"
Sleep 600ms

# === Demonstrate arrow keys ===
Sleep 800ms

# Navigate with arrows
Down 3
Sleep 600ms
Down 2
Sleep 600ms

Up 2
Sleep 600ms

# === Enter and navigate submenus ===
Sleep 800ms

# Enter a submenu
Enter
Sleep 1s

# Navigate child menu with j
Type "j"
Sleep 700ms
Type "j"
Sleep 700ms
Type "j"
Sleep 700ms

# Navigate up in child menu
Type "k"
Sleep 700ms

# Go back to main menu with Backspace
Backspace
Sleep 1s

# === Demonstrate quick navigation ===
Sleep 800ms

# Quick scroll down
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

# Quick scroll up
Type "k"
Sleep 400ms
Type "k"
Sleep 400ms
Type "k"
Sleep 400ms

# === Navigate to different section ===
Sleep 800ms

Down 4
Sleep 700ms

# Enter another submenu
Enter
Sleep 1s

# Navigate submenu options
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms

# Exit submenu
Backspace
Sleep 1s

# === Show wrap-around navigation ===
Sleep 800ms

# Navigate to top
Up 20
Sleep 1s

# First item
Sleep 1s

# Navigate down through several items
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms

# === Enter deep submenu ===
Sleep 800ms

Enter
Sleep 1s

# Navigate deeply
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms
Type "j"
Sleep 600ms

Sleep 1s

# Back to main
Backspace
Sleep 1s

# === Navigate to middle section ===
Sleep 800ms

Down 6
Sleep 700ms

Enter
Sleep 1s

Type "j"
Sleep 600ms

Backspace
Sleep 1s

# === Navigate to bottom ===
Sleep 800ms

Down 10
Sleep 700ms

Sleep 1.5s

# Back to top for finale
Up 15
Sleep 1s

Sleep 1s

# Quit application
Type "q"
Sleep 1s

# Show completion message
Type "# Navigation Demo Complete! ✨"
Sleep 1s
Enter
Sleep 500ms
Type "# Navigation Features:"
Sleep 800ms
Enter
Type "#   • Vim-style j/k keys"
Sleep 800ms
Enter
Type "#   • Arrow key support"
Sleep 800ms
Enter
Type "#   • Smooth scrolling animations"
Sleep 800ms
Enter
Type "#   • Enter to select, Backspace to go back"
Sleep 800ms
Enter
Type "#   • Multi-level menu navigation"
Sleep 800ms
Enter
Type "#   • Intuitive and fast workflow"
Sleep 2s
