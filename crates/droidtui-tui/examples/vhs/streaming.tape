# VHS tape file for demonstrating droidtui streaming example
# Run with: vhs examples/vhs/streaming.tape

# Output settings
Output examples/vhs/streaming.gif
Set Shell "bash"
Set FontSize 14
Set Width 1400
Set Height 900
Set Padding 20
Set Theme "Dracula"

# Show intro screen
Type "# 📺 DroidTUI Screen Streaming Demo"
Sleep 1s
Enter
Sleep 500ms
Type "# Stream your Android device screen with scrcpy integration"
Sleep 1.5s
Enter
Sleep 500ms

# Clear and run the example
Type "clear"
Enter
Sleep 500ms
Type "cargo run --example streaming"
Enter

# Wait for application to load
Sleep 2.5s

# Navigate through streaming options
Type "j"
Sleep 900ms

# High Quality option
Type "j"
Sleep 1s

# Low Latency option
Type "j"
Sleep 1s

# Gaming Mode - show details
Sleep 1.5s

# Navigate to Full Resolution
Type "j"
Sleep 900ms
Type "j"
Sleep 1s

# Read-Only Mode
Type "j"
Sleep 1s

# Show Borderless Window option
Type "j"
Sleep 900ms

# Always On Top
Type "j"
Sleep 900ms

# Record Session option
Type "j"
Sleep 1.5s

# Navigate back up a bit
Type "k"
Sleep 800ms
Type "k"
Sleep 800ms

# Go to Gaming Mode
Type "k"
Sleep 800ms
Type "k"
Sleep 800ms

# Show help screen
Type "h"
Sleep 3s

# Return from help
Type "h"
Sleep 1s

# Navigate to different options
Down 6
Sleep 800ms

# No Audio option
Type "j"
Sleep 1s

# Back to top
Up 10
Sleep 1s

# Standard Quality
Sleep 1.5s

# Show help one more time briefly
Type "?"
Sleep 2.5s
Type "?"
Sleep 800ms

# Navigate to Low Latency
Down 2
Sleep 1s

# Quit application
Type "q"
Sleep 1s

# Show completion message
Type "# Demo complete! ✨"
Sleep 1s
Enter
Sleep 500ms
Type "# Screen Streaming features:"
Sleep 800ms
Enter
Type "#   • Multiple quality presets (480p-1080p+)"
Sleep 800ms
Enter
Type "#   • Gaming mode with 60fps support"
Sleep 800ms
Enter
Type "#   • Recording capabilities"
Sleep 800ms
Enter
Type "#   • Read-only viewing mode"
Sleep 800ms
Enter
Type "#   • Powered by scrcpy"
Sleep 2s
