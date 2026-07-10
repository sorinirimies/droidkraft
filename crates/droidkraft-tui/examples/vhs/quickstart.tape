# VHS tape file for quick droidkraft demo
# Run with: vhs examples/vhs/quickstart.tape

# Output settings
Output examples/vhs/quickstart.gif
Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 700
Set Padding 20
Set Theme "Dracula"

# Title
Type "# 🤖 DroidKraft Quick Demo"
Sleep 1s
Enter
Type "# TUI for Android development"
Sleep 1.5s
Enter
Sleep 500ms

# Launch
Type "droidkraft"
Enter

# Wait for startup animation
Sleep 3s

# Continue past startup
Space
Sleep 1s

# Quick navigation demo
Down 2
Sleep 500ms

# Enter submenu
Enter
Sleep 800ms

# Navigate child
Down
Sleep 500ms

# Back to main
Backspace
Sleep 800ms

# More navigation
Down 3
Sleep 500ms

# Enter another submenu
Enter
Sleep 800ms

# Back
Backspace
Sleep 800ms

# Scroll down
Down 5
Sleep 500ms

# Quit
Type "q"
Sleep 1s

# Outro
Type "# Try it: cargo install droidkraft"
Sleep 2s
