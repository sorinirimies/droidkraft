# DroidKraft Quick Reference - adb_cli API

## 🚀 Quick Start

### Using ADB Commands

```rust
use droidkraft_tui::adb::{AdbCommand, AdbManager, PackageFilter};

// Create manager
let mut manager = AdbManager::new();

// Execute command
let result = manager.execute(AdbCommand::ListDevices)?;
println!("{}", result);
```

---

## 📋 Command Reference

### Device Commands

```rust
// List all connected devices
AdbCommand::ListDevices

// Get device state (online, offline, etc.)
AdbCommand::GetDeviceState

// Get device serial number
AdbCommand::GetSerialNumber
```

### Package Commands

```rust
// List all packages
AdbCommand::ListPackages {
    include_path: false,
    filter: PackageFilter::All,
}

// List user packages only
AdbCommand::ListPackages {
    include_path: false,
    filter: PackageFilter::User,
}

// List system packages
AdbCommand::ListPackages {
    include_path: false,
    filter: PackageFilter::System,
}

// List with file paths
AdbCommand::ListPackages {
    include_path: true,
    filter: PackageFilter::All,
}

// Get package info
AdbCommand::GetPackageInfo {
    package_name: "com.example.app".to_string(),
}

// Uninstall package
AdbCommand::UninstallPackage {
    package_name: "com.example.app".to_string(),
}

// Clear package data
AdbCommand::ClearPackageData {
    package_name: "com.example.app".to_string(),
}
```

### System Commands

```rust
// Battery information
AdbCommand::GetBatteryInfo

// Memory usage statistics
AdbCommand::GetMemoryInfo

// CPU information
AdbCommand::GetCpuInfo

// Device properties
AdbCommand::GetDeviceProperties

// System logs (last N lines)
AdbCommand::GetSystemLog { lines: 100 }
```

### Network Commands

```rust
// Network connectivity info
AdbCommand::GetNetworkInfo

// WiFi status
AdbCommand::GetWifiStatus
```

### Screen Commands

```rust
// Take screenshot
AdbCommand::TakeScreenshot

// Get screen resolution and density
AdbCommand::GetScreenResolution
```

### Process Commands

```rust
// List all processes
AdbCommand::ListProcesses

// Force stop an app
AdbCommand::ForceStop {
    package_name: "com.example.app".to_string(),
}
```

### Shell Commands

```rust
// Execute custom shell command
AdbCommand::Shell {
    command: "ls -la /sdcard".to_string(),
}

// Multiple arguments
AdbCommand::Shell {
    command: "dumpsys wifi".to_string(),
}
```

### Version Commands

```rust
// Get ADB server version
AdbCommand::GetAdbVersion
```

---

## 🎯 Package Filters

```rust
pub enum PackageFilter {
    All,        // All packages
    User,       // User-installed packages (-3)
    System,     // System packages (-s)
    Enabled,    // Enabled packages (-e)
    Disabled,   // Disabled packages (-d)
}
```

---

## ⚠️ Error Handling

```rust
use droidkraft_tui::adb::{AdbError, AdbResult};

match manager.execute(command) {
    Ok(output) => println!("Success: {}", output),
    Err(e) => match e {
        AdbError::ConnectionError(msg) => {
            eprintln!("Connection failed: {}", msg);
        }
        AdbError::DeviceNotFound => {
            eprintln!("No device found");
        }
        AdbError::NoDeviceSelected => {
            eprintln!("Please select a device first");
        }
        AdbError::CommandFailed(msg) => {
            eprintln!("Command failed: {}", msg);
        }
        _ => eprintln!("Error: {}", e),
    }
}
```

---

## 🔧 Common Patterns

### Execute Multiple Commands

```rust
let mut manager = AdbManager::new();

let commands = vec![
    AdbCommand::ListDevices,
    AdbCommand::GetBatteryInfo,
    AdbCommand::GetMemoryInfo,
];

for cmd in commands {
    match manager.execute(cmd) {
        Ok(output) => println!("{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Check Device Connection

```rust
let mut manager = AdbManager::new();

match manager.execute(AdbCommand::ListDevices) {
    Ok(output) => {
        if output.contains("No devices") {
            println!("Please connect a device");
        } else {
            println!("Device connected!");
        }
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### Get Specific Property

```rust
// Get Android version
let cmd = AdbCommand::Shell {
    command: "getprop ro.build.version.release".to_string(),
};
let version = manager.execute(cmd)?;

// Get device model
let cmd = AdbCommand::Shell {
    command: "getprop ro.product.model".to_string(),
};
let model = manager.execute(cmd)?;
```

### Filter Command Output

```rust
// Get battery level only
let cmd = AdbCommand::Shell {
    command: "dumpsys battery | grep level".to_string(),
};
let battery_level = manager.execute(cmd)?;

// Get WiFi IP address
let cmd = AdbCommand::Shell {
    command: "ip addr show wlan0 | grep inet".to_string(),
};
let ip_addr = manager.execute(cmd)?;
```

---

## 📱 Device Management

### Select Device

```rust
let mut manager = AdbManager::new();

// Auto-selects first device on ListDevices
manager.execute(AdbCommand::ListDevices)?;

// Manually select device
manager.select_device("ABC123XYZ".to_string());
```

### Multiple Devices

```rust
// List all devices
let output = manager.execute(AdbCommand::ListDevices)?;

// Parse device serials (simplified)
for line in output.lines() {
    if line.contains("device") && !line.contains("List") {
        println!("Found device: {}", line);
    }
}
```

---

## 🧪 Testing

### Unit Test Example

```rust
#[test]
fn test_command_creation() {
    let cmd = AdbCommand::ListDevices;
    assert!(matches!(cmd, AdbCommand::ListDevices));
}
```

### Integration Test Example

```rust
#[test]
#[ignore] // Requires device
fn test_battery_info() {
    let mut manager = AdbManager::new();
    let result = manager.execute(AdbCommand::GetBatteryInfo);
    assert!(result.is_ok());
}
```

---

## 💡 Tips & Tricks

### 1. Error Recovery

```rust
// Retry on connection error
let mut retries = 3;
loop {
    match manager.execute(command.clone()) {
        Ok(output) => break Ok(output),
        Err(AdbError::ConnectionError(_)) if retries > 0 => {
            retries -= 1;
            std::thread::sleep(Duration::from_secs(1));
        }
        Err(e) => break Err(e),
    }
}
```

### 2. Command Chaining

```rust
// Get device info in one go
let cmd = AdbCommand::Shell {
    command: "echo 'Model:' && getprop ro.product.model && echo 'Android:' && getprop ro.build.version.release".to_string(),
};
```

### 3. Performance

```rust
// Reuse manager instance
let mut manager = AdbManager::new();
manager.connect()?; // Connect once

// Execute many commands
for _ in 0..100 {
    manager.execute(AdbCommand::GetBatteryInfo)?;
}
```

---

## 🔗 Related Commands

### Package Management

```rust
// Install → Uses file system (not yet implemented via API)
// List → AdbCommand::ListPackages
// Uninstall → AdbCommand::UninstallPackage
// Clear data → AdbCommand::ClearPackageData
// Force stop → AdbCommand::ForceStop
```

### System Information

```rust
// Battery → AdbCommand::GetBatteryInfo
// Memory → AdbCommand::GetMemoryInfo
// CPU → AdbCommand::GetCpuInfo
// Properties → AdbCommand::GetDeviceProperties
// Network → AdbCommand::GetNetworkInfo
```

---

## 📚 Examples

### Full Example: Device Monitor

```rust
use droidkraft_tui::adb::{AdbCommand, AdbManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = AdbManager::new();
    
    // Check connection
    println!("Checking devices...");
    let devices = manager.execute(AdbCommand::ListDevices)?;
    println!("{}", devices);
    
    // Get battery info
    println!("\nBattery Status:");
    let battery = manager.execute(AdbCommand::GetBatteryInfo)?;
    println!("{}", battery);
    
    // Get memory info
    println!("\nMemory Status:");
    let memory = manager.execute(AdbCommand::GetMemoryInfo)?;
    println!("{}", memory);
    
    Ok(())
}
```

---

## 🆘 Troubleshooting

### "Connection Error"
```rust
// Start ADB server first
// Run: adb start-server

let result = manager.connect();
if result.is_err() {
    println!("Make sure ADB server is running");
}
```

### "No Device Selected"
```rust
// List devices first to auto-select
manager.execute(AdbCommand::ListDevices)?;

// Or manually select
manager.select_device("DEVICE_SERIAL".to_string());
```

### "Command Failed"
```rust
// Check device authorization
// Verify USB debugging enabled
// Check device connection (adb devices)
```

---

## 📖 See Also

- `REFACTORING.md` - Complete refactoring documentation
- `examples/` - Standalone examples
- `tests/adb_integration_tests.rs` - More examples
- [adb_client docs](https://docs.rs/adb_client/2.1.17)

---

**Version**: 0.3.0  
**Last Updated**: December 2024  
**Status**: Production Ready ✅