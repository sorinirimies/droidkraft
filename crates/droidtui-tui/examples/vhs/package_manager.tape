# VHS tape file for demonstrating droidtui package manager example
# Run with: vhs examples/vhs/package_manager.tape

# Output settings
Output examples/vhs/package_manager.gif
Set Shell "bash"
Set FontSize 14
Set Width 1400
Set Height 900
Set Padding 20
Set Theme "Dracula"

# Show intro screen
Type "# 📦 DroidTUI Package Manager Demo"
Sleep 1s
Enter
Sleep 500ms
Type "# Manage Android apps with ADB package manager"
Sleep 1.5s
Enter
Sleep 500ms

# Clear and run the example
Type "clear"
Enter
Sleep 500ms
Type "cargo run --example package_manager"
Enter

# Wait for application to load
Sleep 2.5s

# Navigate through package actions
Type "j"
Sleep 900ms

# List User Apps
Sleep 1s

# List System Apps
Type "j"
Sleep 900ms

# List Enabled Apps
Type "j"
Sleep 900ms

# List Disabled Apps
Type "j"
Sleep 900ms

# List with Paths
Type "j"
Sleep 1s

# List with Installers
Type "j"
Sleep 900ms

# Package Info
Type "j"
Sleep 1s

# Package Permissions
Type "j"
Sleep 900ms

# Clear App Data (requires confirmation)
Type "j"
Sleep 1s

# Clear App Cache
Type "j"
Sleep 900ms

# Disable App (requires confirmation)
Type "j"
Sleep 1s

# Enable App
Type "j"
Sleep 900ms

# Uninstall App (requires confirmation)
Type "j"
Sleep 1.5s

# Navigate back up
Type "k"
Sleep 800ms
Type "k"
Sleep 800ms
Type "k"
Sleep 800ms

# Install APK
Type "j"
Sleep 1s

# Grant Permission
Type "j"
Sleep 900ms

# Revoke Permission (requires confirmation)
Type "j"
Sleep 1s

# List Permissions
Type "j"
Sleep 900ms

# App Data Usage
Type "j"
Sleep 900ms

# Force Stop App
Type "j"
Sleep 1s

# Show help screen
Type "h"
Sleep 3.5s

# Return from help
Type "h"
Sleep 1s

# Navigate to different actions
Up 10
Sleep 800ms

# List All Packages
Sleep 1s

# Navigate to Query actions
Down 3
Sleep 900ms

# Navigate to Manage actions
Down 6
Sleep 1s

# Clear App Data - show warning
Sleep 1.5s

# Navigate to Permissions
Down 5
Sleep 1s

# Revoke Permission - show warning
Sleep 1.5s

# Show help briefly
Type "?"
Sleep 2.5s
Type "?"
Sleep 800ms

# Navigate to Force Stop
Down 2
Sleep 1s

# Back to top
Up 20
Sleep 1s

# Navigate through a few more actions
Down 4
Sleep 800ms
Down 2
Sleep 800ms

# Quit application
Type "q"
Sleep 1s

# Show completion message
Type "# Demo complete! ✨"
Sleep 1s
Enter
Sleep 500ms
Type "# Package Manager features:"
Sleep 800ms
Enter
Type "#   • List packages by type (user, system, enabled)"
Sleep 800ms
Enter
Type "#   • Query detailed package information"
Sleep 800ms
Enter
Type "#   • Install and uninstall applications"
Sleep 800ms
Enter
Type "#   • Manage app permissions"
Sleep 800ms
Enter
Type "#   • Clear cache and data"
Sleep 800ms
Enter
Type "#   • Enable/disable apps"
Sleep 2s
