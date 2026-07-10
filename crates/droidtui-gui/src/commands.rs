//! Framework-free catalogue of one-click device commands surfaced as buttons in
//! the GUI.  Kept independent of gpui so it can be unit-tested.

use droidtui_core::features::flash::RebootTarget;
use droidtui_core::features::packages::PackageFilter;
use droidtui_core::AdbCommand;

/// A logical grouping of commands, used to lay buttons out in sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    Device,
    System,
    Packages,
    Screen,
    Power,
}

impl CommandCategory {
    pub fn label(&self) -> &'static str {
        match self {
            CommandCategory::Device => "Device",
            CommandCategory::System => "System",
            CommandCategory::Packages => "Packages",
            CommandCategory::Screen => "Screen",
            CommandCategory::Power => "Power",
        }
    }

    /// Iteration order for rendering sections.
    pub fn all() -> &'static [CommandCategory] {
        &[
            CommandCategory::Device,
            CommandCategory::System,
            CommandCategory::Packages,
            CommandCategory::Screen,
            CommandCategory::Power,
        ]
    }
}

/// What a button actually does when clicked.
#[derive(Debug, Clone)]
pub enum CommandAction {
    /// A typed ADB command dispatched via `AdbManager::execute`.
    Adb(AdbCommand),
    /// A raw shell command.
    Shell(String),
    /// Reboot the device into a target mode.
    Reboot(RebootTarget),
}

/// A single labelled command button.
#[derive(Debug, Clone)]
pub struct GuiCommand {
    pub label: &'static str,
    pub category: CommandCategory,
    pub action: CommandAction,
    /// Whether the action is potentially destructive (needs confirmation styling).
    pub destructive: bool,
}

impl GuiCommand {
    const fn adb(label: &'static str, category: CommandCategory, cmd: AdbCommand) -> Self {
        Self {
            label,
            category,
            action: CommandAction::Adb(cmd),
            destructive: false,
        }
    }

    fn shell(label: &'static str, category: CommandCategory, cmd: &str) -> Self {
        Self {
            label,
            category,
            action: CommandAction::Shell(cmd.to_string()),
            destructive: false,
        }
    }

    const fn reboot(label: &'static str, target: RebootTarget) -> Self {
        Self {
            label,
            category: CommandCategory::Power,
            action: CommandAction::Reboot(target),
            destructive: true,
        }
    }
}

/// The full catalogue of quick commands shown in the Commands panel.
pub fn catalog() -> Vec<GuiCommand> {
    vec![
        // Device
        GuiCommand::adb(
            "List Devices",
            CommandCategory::Device,
            AdbCommand::ListDevices,
        ),
        GuiCommand::adb(
            "Device State",
            CommandCategory::Device,
            AdbCommand::GetDeviceState,
        ),
        GuiCommand::adb(
            "ADB Version",
            CommandCategory::Device,
            AdbCommand::GetAdbVersion,
        ),
        GuiCommand::adb(
            "Properties",
            CommandCategory::Device,
            AdbCommand::GetDeviceProperties,
        ),
        // System
        GuiCommand::adb(
            "Battery",
            CommandCategory::System,
            AdbCommand::GetBatteryInfo,
        ),
        GuiCommand::adb("Memory", CommandCategory::System, AdbCommand::GetMemoryInfo),
        GuiCommand::adb("CPU", CommandCategory::System, AdbCommand::GetCpuInfo),
        GuiCommand::adb(
            "Processes",
            CommandCategory::System,
            AdbCommand::ListProcesses,
        ),
        GuiCommand::adb(
            "Network",
            CommandCategory::System,
            AdbCommand::GetNetworkInfo,
        ),
        GuiCommand::adb("Wi-Fi", CommandCategory::System, AdbCommand::GetWifiStatus),
        // Packages
        GuiCommand::adb(
            "User Packages",
            CommandCategory::Packages,
            AdbCommand::ListPackages {
                include_path: false,
                filter: PackageFilter::User,
            },
        ),
        GuiCommand::adb(
            "System Packages",
            CommandCategory::Packages,
            AdbCommand::ListPackages {
                include_path: false,
                filter: PackageFilter::System,
            },
        ),
        // Screen
        GuiCommand::adb(
            "Resolution",
            CommandCategory::Screen,
            AdbCommand::GetScreenResolution,
        ),
        GuiCommand::adb(
            "Screenshot",
            CommandCategory::Screen,
            AdbCommand::TakeScreenshot,
        ),
        GuiCommand::shell(
            "Wake",
            CommandCategory::Screen,
            "input keyevent KEYCODE_WAKEUP",
        ),
        GuiCommand::shell(
            "Sleep",
            CommandCategory::Screen,
            "input keyevent KEYCODE_SLEEP",
        ),
        // Power
        GuiCommand::reboot("Reboot", RebootTarget::System),
        GuiCommand::reboot("Bootloader", RebootTarget::Bootloader),
        GuiCommand::reboot("Recovery", RebootTarget::Recovery),
    ]
}

/// Commands belonging to a given category, preserving catalogue order.
pub fn by_category(category: CommandCategory) -> Vec<GuiCommand> {
    catalog()
        .into_iter()
        .filter(|c| c.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_non_empty() {
        assert!(!catalog().is_empty());
    }

    #[test]
    fn every_category_has_commands() {
        for cat in CommandCategory::all() {
            assert!(
                !by_category(*cat).is_empty(),
                "category {:?} had no commands",
                cat
            );
        }
    }

    #[test]
    fn power_commands_are_destructive() {
        for c in by_category(CommandCategory::Power) {
            assert!(c.destructive, "{} should be destructive", c.label);
        }
    }

    #[test]
    fn reboot_actions_map_to_targets() {
        let power = by_category(CommandCategory::Power);
        assert!(power
            .iter()
            .any(|c| matches!(c.action, CommandAction::Reboot(RebootTarget::Bootloader))));
    }
}
