# VHS tape file for demonstrating droidtui main menu
# Run with: vhs examples/vhs/main_menu.tape

# Output settings
Output examples/vhs/main_menu.gif
Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 800
Set Padding 20
Set Theme "Dracula"

# Show intro screen
Type "# DroidTUI Main Menu Demo"
Sleep 1s
Enter
Sleep 500ms
Type "# A beautiful TUI for Android development"
Sleep 1s
Enter
Sleep 500ms
Type "# Let's explore the main menu..."
Sleep 1.5s
Enter
Sleep 500ms

# Clear and run the example
Type "clear"
Enter
Sleep 500ms
Type "cargo run --example main_menu"
Enter

# Wait for application to load
Sleep 2s

# Navigate down through menu items
Type "j"
Sleep 800ms
Type "j"
Sleep 800ms
Type "j"
Sleep 800ms

# Navigate back up
Type "k"
Sleep 800ms
Type "k"
Sleep 800ms

# Navigate to Battery Info
Type "j"
Sleep 1s

# Show the command details by staying on this item
Sleep 1.5s

# Navigate down to Memory Usage
Type "j"
Sleep 1s

# Navigate to CPU Info
Type "j"
Sleep 1s

# Navigate to Network Info
Type "j"
Sleep 1s

# Go back up to show navigation
Type "k"
Sleep 800ms
Type "k"
Sleep 800ms
Type "k"
Sleep 800ms

# Show help screen
Type "h"
Sleep 2s

# Return to menu
Type "h"
Sleep 1s

# Navigate down to System Log
Down 8
Sleep 1s

# Navigate to ADB Version (last item)
Type "j"
Sleep 1s

# Go back to top
Up 12
Sleep 1s

# Navigate to List Devices
Sleep 1.5s

# Simulate selection (in real app this would execute)
Sleep 500ms

# Show one more navigation sequence
Down 3
Sleep 800ms
Down 2
Sleep 800ms

# Show help one more time
Type "?"
Sleep 2s

# Exit help
Type "?"
Sleep 1s

# Quit application
Type "q"
Sleep 1s

# Show completion message
Type "# Demo complete!"
Sleep 1s
Enter
Type "# The main menu provides access to 13 ADB commands"
Sleep 1s
Enter
Type "# Each with expandable sub-options in the full app"
Sleep 2s
