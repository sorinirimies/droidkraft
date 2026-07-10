# VHS tape file for demonstrating droidtui device info example
# Run with: vhs examples/vhs/device_info.tape

# Output settings
Output examples/vhs/device_info.gif
Set Shell "bash"
Set FontSize 14
Set Width 1400
Set Height 900
Set Padding 20
Set Theme "Dracula"

# Show intro screen
Type "# 📱 DroidTUI Device Information Demo"
Sleep 1s
Enter
Sleep 500ms
Type "# Query comprehensive device info with ADB"
Sleep 1.5s
Enter
Sleep 500ms

# Clear and run the example
Type "clear"
Enter
Sleep 500ms
Type "cargo run --example device_info"
Enter

# Wait for application to load
Sleep 2.5s

# Navigate through different info categories
Type "j"
Sleep 900ms

# Android Version
Sleep 1s

# Build Number
Type "j"
Sleep 900ms

# Serial Number
Type "j"
Sleep 900ms

# Screen Resolution
Type "j"
Sleep 1s

# CPU Architecture
Type "j"
Sleep 900ms

# Memory Info
Type "j"
Sleep 1s

# Storage Space
Type "j"
Sleep 900ms

# Battery Status
Type "j"
Sleep 1s

# WiFi Status
Type "j"
Sleep 900ms

# Bluetooth Info
Type "j"
Sleep 900ms

# IMEI Number
Type "j"
Sleep 900ms

# Uptime
Type "j"
Sleep 1s

# Kernel Version
Type "j"
Sleep 900ms

# Language & Locale
Type "j"
Sleep 900ms

# Security Patch
Type "j"
Sleep 1.5s

# Navigate back up
Type "k"
Sleep 800ms
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

# Navigate to different categories
Down 4
Sleep 800ms

# Memory Info
Sleep 1s

# Go down to Storage
Type "j"
Sleep 1s

# Battery
Type "j"
Sleep 1s

# Back to top
Up 10
Sleep 1s

# Device Model
Sleep 1.5s

# Show help briefly
Type "?"
Sleep 2s
Type "?"
Sleep 800ms

# Navigate to Battery Status
Down 8
Sleep 1s

# Navigate to Security Patch
Down 7
Sleep 1s

# Quit application
Type "q"
Sleep 1s

# Show completion message
Type "# Demo complete! ✨"
Sleep 1s
Enter
Sleep 500ms
Type "# Device Information categories:"
Sleep 800ms
Enter
Type "#   • Hardware specs (CPU, Memory, Display)"
Sleep 800ms
Enter
Type "#   • System info (Android version, Kernel)"
Sleep 800ms
Enter
Type "#   • Network status (WiFi, Bluetooth)"
Sleep 800ms
Enter
Type "#   • Device IDs (Serial, IMEI)"
Sleep 800ms
Enter
Type "#   • Security & Updates"
Sleep 2s
