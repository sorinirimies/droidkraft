//! Framework-free catalogue of one-click device commands (mirrors the GUI's
//! `commands.rs`), plus the sectioned layout used by the Commands panel.

use droidkraft_core::features::flash::RebootTarget;
use droidkraft_core::features::packages::PackageFilter;
use droidkraft_core::AdbCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Device,
    System,
    Packages,
    Screen,
    Power,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::Device => "Device",
            Category::System => "System",
            Category::Packages => "Packages",
            Category::Screen => "Screen",
            Category::Power => "Power",
        }
    }
}

#[derive(Debug, Clone)]
pub enum CmdAction {
    Adb(AdbCommand),
    Shell(String),
    Reboot(RebootTarget),
}

#[derive(Debug, Clone)]
pub struct Command {
    pub label: &'static str,
    pub category: Category,
    pub action: CmdAction,
    pub destructive: bool,
}

fn adb(label: &'static str, category: Category, c: AdbCommand) -> Command {
    Command {
        label,
        category,
        action: CmdAction::Adb(c),
        destructive: false,
    }
}
fn shell(label: &'static str, category: Category, c: &str) -> Command {
    Command {
        label,
        category,
        action: CmdAction::Shell(c.into()),
        destructive: false,
    }
}
fn reboot(label: &'static str, t: RebootTarget) -> Command {
    Command {
        label,
        category: Category::Power,
        action: CmdAction::Reboot(t),
        destructive: true,
    }
}

/// The full catalogue shown in the Commands panel (same set as the GUI).
pub fn catalog() -> Vec<Command> {
    use Category::*;
    let pk = |f| AdbCommand::ListPackages {
        include_path: false,
        filter: f,
    };
    vec![
        adb("List Devices", Device, AdbCommand::ListDevices),
        adb("Device State", Device, AdbCommand::GetDeviceState),
        adb("ADB Version", Device, AdbCommand::GetAdbVersion),
        adb("Properties", Device, AdbCommand::GetDeviceProperties),
        adb("Battery", System, AdbCommand::GetBatteryInfo),
        adb("Memory", System, AdbCommand::GetMemoryInfo),
        adb("CPU", System, AdbCommand::GetCpuInfo),
        adb("Processes", System, AdbCommand::ListProcesses),
        adb("Network", System, AdbCommand::GetNetworkInfo),
        adb("Wi-Fi", System, AdbCommand::GetWifiStatus),
        adb("User Packages", Packages, pk(PackageFilter::User)),
        adb("System Packages", Packages, pk(PackageFilter::System)),
        adb("Resolution", Screen, AdbCommand::GetScreenResolution),
        adb("Screenshot", Screen, AdbCommand::TakeScreenshot),
        shell("Wake", Screen, "input keyevent KEYCODE_WAKEUP"),
        shell("Sleep", Screen, "input keyevent KEYCODE_SLEEP"),
        reboot("Reboot", RebootTarget::System),
        reboot("Bootloader", RebootTarget::Bootloader),
        reboot("Recovery", RebootTarget::Recovery),
    ]
}

/// Category sections in display order.
pub const CATEGORIES: [Category; 5] = [
    Category::Device,
    Category::System,
    Category::Packages,
    Category::Screen,
    Category::Power,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_category_has_commands() {
        let c = catalog();
        for cat in CATEGORIES {
            assert!(c.iter().any(|x| x.category == cat), "{cat:?} empty");
        }
    }

    #[test]
    fn power_is_destructive() {
        assert!(catalog()
            .iter()
            .filter(|c| c.category == Category::Power)
            .all(|c| c.destructive));
    }
}
