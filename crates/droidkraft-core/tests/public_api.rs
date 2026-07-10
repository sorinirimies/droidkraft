#![allow(clippy::field_reassign_with_default)]
//! Integration tests exercising the public `droidkraft_core` API surface.
//!
//! These do not require a connected device — they lock the exported types and
//! their framework-free behaviour so refactors can't silently break the API.

use droidkraft_core::features::fastboot::FastbootCommand;
use droidkraft_core::features::flash::{RebootTarget, RootMethod, RootStatus};
use droidkraft_core::{
    AdbCommand, AdbManager, DeviceStatus, LogEntry, LogLevel, LogcatFilter, LogcatStream,
    PackageFilter, ScreenResolution,
};

#[test]
fn manager_constructs_without_device() {
    let m = AdbManager::new();
    assert!(m.selected_device().is_none());
}

#[test]
fn device_status_default_is_disconnected() {
    let s = DeviceStatus::default();
    assert!(!s.is_connected());
    assert!(s.active().is_none());
}

#[test]
fn adb_command_variants_are_constructible() {
    let _ = AdbCommand::ListDevices;
    let _ = AdbCommand::ListPackages {
        include_path: true,
        filter: PackageFilter::User,
    };
    let _ = AdbCommand::Shell {
        command: "echo hi".into(),
    };
}

#[test]
fn logcat_pipeline_parses_and_filters() {
    let raw = "01-15 12:34:56.789  1000  1001 E ActivityManager: crash detected";
    let entry = LogEntry::parse(raw);
    assert_eq!(entry.level, LogLevel::Error);
    assert_eq!(entry.tag.as_deref(), Some("ActivityManager"));

    let mut filter = LogcatFilter::default();
    filter.min_level = LogLevel::Warn;
    assert!(filter.matches(&entry));

    filter.min_level = LogLevel::Fatal;
    assert!(!filter.matches(&entry));
}

#[test]
fn logcat_stream_is_idle_until_started() {
    let mut stream = LogcatStream::new();
    assert!(!stream.is_running());
    let mut out = Vec::new();
    let (n, _status) = stream.drain_into(&mut out, 10);
    assert_eq!(n, 0);
}

#[test]
fn fastboot_command_args_are_stable() {
    assert_eq!(FastbootCommand::WipeData.args(), vec!["-w"]);
    assert!(FastbootCommand::OemUnlock.is_destructive());
}

#[test]
fn reboot_targets_cover_all_modes() {
    let modes: Vec<&str> = RebootTarget::all().iter().map(|t| t.arg()).collect();
    assert!(modes.contains(&"bootloader"));
    assert!(modes.contains(&"recovery"));
    assert!(modes.contains(&"sideload"));
}

#[test]
fn root_status_helpers() {
    let s = RootStatus::not_rooted();
    assert!(!s.is_rooted);
    assert_eq!(s.method, RootMethod::None);
}

#[test]
fn screen_resolution_parses_wm_output() {
    let r = ScreenResolution::parse("Physical size: 1440x3120", "Physical density: 560").unwrap();
    assert_eq!((r.width, r.height, r.density), (1440, 3120, 560));
    assert!(r.aspect_ratio() > 0.0);
}
