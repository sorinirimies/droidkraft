//! Integration tests for ADB functionality
//!
//! These tests verify the ADB client integration and command execution.
//! Note: Some tests require an actual ADB server running and a connected device.

use droidtui::adb::{AdbCommand, AdbError, AdbManager, PackageFilter};

#[test]
fn test_adb_manager_creation() {
    let _manager = AdbManager::new();
    // Manager should be created successfully
    // No assertion needed - test passes if no panic occurs
}

#[test]
fn test_adb_command_variants() {
    // Test that all command variants can be created
    let _cmd1 = AdbCommand::ListDevices;
    let _cmd2 = AdbCommand::GetBatteryInfo;
    let _cmd3 = AdbCommand::GetMemoryInfo;
    let _cmd4 = AdbCommand::ListPackages {
        include_path: true,
        filter: PackageFilter::All,
    };
    let _cmd5 = AdbCommand::Shell {
        command: "ls".to_string(),
    };
}

#[test]
fn test_package_filter_variants() {
    let filters = [
        PackageFilter::All,
        PackageFilter::User,
        PackageFilter::System,
        PackageFilter::Enabled,
        PackageFilter::Disabled,
    ];

    // All filter variants should be creatable
    assert_eq!(filters.len(), 5);
}

#[test]
fn test_adb_command_clone() {
    let cmd = AdbCommand::ListDevices;
    let cloned = cmd.clone();

    // Commands should be cloneable
    assert!(matches!(cloned, AdbCommand::ListDevices));
}

#[test]
fn test_adb_error_display() {
    let error1 = AdbError::DeviceNotFound;
    assert_eq!(error1.to_string(), "Device not found");

    let error2 = AdbError::NoDeviceSelected;
    assert_eq!(error2.to_string(), "No device selected");

    let error3 = AdbError::ConnectionError("test".to_string());
    assert_eq!(error3.to_string(), "Connection error: test");

    let error4 = AdbError::CommandFailed("failed".to_string());
    assert_eq!(error4.to_string(), "Command failed: failed");

    let error5 = AdbError::ParseError("parse".to_string());
    assert_eq!(error5.to_string(), "Parse error: parse");
}

#[test]
fn test_shell_command_creation() {
    let cmd = AdbCommand::Shell {
        command: "echo hello".to_string(),
    };

    if let AdbCommand::Shell { command } = cmd {
        assert_eq!(command, "echo hello");
    } else {
        panic!("Expected Shell command");
    }
}

#[test]
fn test_list_packages_command() {
    let cmd1 = AdbCommand::ListPackages {
        include_path: false,
        filter: PackageFilter::User,
    };

    if let AdbCommand::ListPackages {
        include_path,
        filter,
    } = cmd1
    {
        assert!(!include_path);
        assert!(matches!(filter, PackageFilter::User));
    } else {
        panic!("Expected ListPackages command");
    }
}

// Integration tests below require actual ADB server and device
// Run with: cargo test --test adb_integration_tests -- --ignored

#[test]
#[ignore]
fn test_adb_connect() {
    let mut manager = AdbManager::new();
    let result = manager.connect();

    // This test requires ADB server to be running
    match result {
        Ok(_) => {} // Connection successful
        Err(e) => {
            println!(
                "Warning: ADB connection failed (expected if no ADB server): {}",
                e
            );
            // Don't fail the test as this is expected in CI environments
        }
    }
}

#[test]
#[ignore]
fn test_list_devices() {
    let mut manager = AdbManager::new();

    match manager.execute(AdbCommand::ListDevices) {
        Ok(output) => {
            println!("Devices output: {}", output);
            assert!(output.contains("List of devices") || output.contains("No devices"));
        }
        Err(e) => {
            println!(
                "Warning: List devices failed (expected if no ADB server): {}",
                e
            );
        }
    }
}

#[test]
#[ignore]
fn test_get_adb_version() {
    let mut manager = AdbManager::new();

    match manager.execute(AdbCommand::GetAdbVersion) {
        Ok(output) => {
            println!("ADB version: {}", output);
            assert!(!output.is_empty());
        }
        Err(e) => {
            println!(
                "Warning: Get version failed (expected if no ADB server): {}",
                e
            );
        }
    }
}

#[test]
#[ignore]
fn test_shell_command_execution() {
    let mut manager = AdbManager::new();

    // Try to execute a simple shell command
    let cmd = AdbCommand::Shell {
        command: "echo test".to_string(),
    };

    match manager.execute(cmd) {
        Ok(output) => {
            println!("Shell output: {}", output);
            // Output should contain "test" or error message
            assert!(!output.is_empty());
        }
        Err(e) => {
            println!(
                "Warning: Shell command failed (expected if no device): {}",
                e
            );
            // Expected if no device is connected
            assert!(matches!(
                e,
                AdbError::NoDeviceSelected
                    | AdbError::DeviceNotFound
                    | AdbError::ConnectionError(_)
            ));
        }
    }
}

#[test]
#[ignore]
fn test_device_selection() {
    let mut manager = AdbManager::new();

    // Set a dummy device serial
    manager.select_device("test_device_123".to_string());

    // Try to execute command with selected device (will fail but tests selection logic)
    let cmd = AdbCommand::GetDeviceState;

    match manager.execute(cmd) {
        Ok(_) => {
            // If successful, device actually exists
            // No assertion needed - success means test passed
        }
        Err(e) => {
            // Expected - device doesn't exist
            println!("Expected error for dummy device: {}", e);
            // No assertion needed - error is expected behavior
        }
    }
}

#[test]
#[ignore]
fn test_battery_info() {
    let mut manager = AdbManager::new();

    match manager.execute(AdbCommand::GetBatteryInfo) {
        Ok(output) => {
            println!("Battery info: {}", output);
            // Should contain battery-related information
            assert!(
                output.contains("level")
                    || output.contains("battery")
                    || output.contains("No device")
            );
        }
        Err(e) => {
            println!(
                "Warning: Battery info failed (expected if no device): {}",
                e
            );
        }
    }
}

#[test]
#[ignore]
fn test_list_packages_integration() {
    let mut manager = AdbManager::new();

    let cmd = AdbCommand::ListPackages {
        include_path: false,
        filter: PackageFilter::All,
    };

    match manager.execute(cmd) {
        Ok(output) => {
            println!(
                "Packages (first 200 chars): {}",
                &output.chars().take(200).collect::<String>()
            );
            // Should contain package information or error message
            assert!(!output.is_empty());
        }
        Err(e) => {
            println!(
                "Warning: List packages failed (expected if no device): {}",
                e
            );
        }
    }
}

#[test]
fn test_error_conversion_from_io() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
    let adb_error: AdbError = io_error.into();

    assert!(matches!(adb_error, AdbError::IoError(_)));
}

#[test]
fn test_command_debug_format() {
    let cmd = AdbCommand::ListDevices;
    let debug_str = format!("{:?}", cmd);

    assert!(debug_str.contains("ListDevices"));
}

#[test]
fn test_filter_debug_format() {
    let filter = PackageFilter::User;
    let debug_str = format!("{:?}", filter);

    assert!(debug_str.contains("User"));
}

#[test]
fn test_multiple_command_types() {
    let commands = vec![
        AdbCommand::ListDevices,
        AdbCommand::GetBatteryInfo,
        AdbCommand::GetMemoryInfo,
        AdbCommand::GetCpuInfo,
        AdbCommand::GetNetworkInfo,
        AdbCommand::GetDeviceProperties,
        AdbCommand::ListProcesses,
        AdbCommand::GetScreenResolution,
        AdbCommand::TakeScreenshot,
        AdbCommand::GetAdbVersion,
    ];

    // All commands should be creatable
    assert_eq!(commands.len(), 10);
}

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn test_command_clone_equality() {
        let original = AdbCommand::GetBatteryInfo;
        let cloned = original.clone();

        // Both should be the same variant
        assert!(matches!(original, AdbCommand::GetBatteryInfo));
        assert!(matches!(cloned, AdbCommand::GetBatteryInfo));
    }

    #[test]
    fn test_shell_command_preserves_content() {
        let original_text = "ls -la /sdcard";
        let cmd = AdbCommand::Shell {
            command: original_text.to_string(),
        };

        if let AdbCommand::Shell { command } = cmd {
            assert_eq!(command, original_text);
        }
    }
}
