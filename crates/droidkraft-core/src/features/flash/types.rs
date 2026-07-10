//! Flash & root domain types.

/// A target mode to reboot the device into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebootTarget {
    /// Normal reboot into Android.
    System,
    /// Reboot into the bootloader / fastboot mode.
    Bootloader,
    /// Reboot into recovery mode.
    Recovery,
    /// Reboot into sideload mode (recovery ADB sideload).
    Sideload,
    /// Reboot into fastbootd (userspace fastboot).
    Fastboot,
}

impl RebootTarget {
    /// The `adb reboot` argument for this target (empty for a normal reboot).
    pub fn arg(&self) -> &'static str {
        match self {
            RebootTarget::System => "",
            RebootTarget::Bootloader => "bootloader",
            RebootTarget::Recovery => "recovery",
            RebootTarget::Sideload => "sideload",
            RebootTarget::Fastboot => "fastboot",
        }
    }

    /// A short human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            RebootTarget::System => "Reboot System",
            RebootTarget::Bootloader => "Reboot to Bootloader",
            RebootTarget::Recovery => "Reboot to Recovery",
            RebootTarget::Sideload => "Reboot to Sideload",
            RebootTarget::Fastboot => "Reboot to Fastbootd",
        }
    }

    /// All reboot targets, for menu generation.
    pub fn all() -> &'static [RebootTarget] {
        &[
            RebootTarget::System,
            RebootTarget::Bootloader,
            RebootTarget::Recovery,
            RebootTarget::Sideload,
            RebootTarget::Fastboot,
        ]
    }
}

/// How root access was detected on the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootMethod {
    /// Magisk `su` present.
    Magisk,
    /// A generic `su` binary is present.
    SuBinary,
    /// `adbd` is already running as root.
    AdbdRoot,
    /// No root detected.
    None,
}

impl RootMethod {
    pub fn label(&self) -> &'static str {
        match self {
            RootMethod::Magisk => "Magisk",
            RootMethod::SuBinary => "su binary",
            RootMethod::AdbdRoot => "adbd root",
            RootMethod::None => "not rooted",
        }
    }
}

/// The detected root status of a device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootStatus {
    /// Whether any form of root access is available.
    pub is_rooted: bool,
    /// How root was detected.
    pub method: RootMethod,
    /// Detected Magisk version, if any.
    pub magisk_version: Option<String>,
}

impl RootStatus {
    /// A "not rooted" status.
    pub fn not_rooted() -> Self {
        Self {
            is_rooted: false,
            method: RootMethod::None,
            magisk_version: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reboot_args() {
        assert_eq!(RebootTarget::System.arg(), "");
        assert_eq!(RebootTarget::Bootloader.arg(), "bootloader");
        assert_eq!(RebootTarget::Recovery.arg(), "recovery");
        assert_eq!(RebootTarget::Sideload.arg(), "sideload");
        assert_eq!(RebootTarget::Fastboot.arg(), "fastboot");
    }

    #[test]
    fn all_targets_present() {
        assert_eq!(RebootTarget::all().len(), 5);
    }

    #[test]
    fn not_rooted_default() {
        let s = RootStatus::not_rooted();
        assert!(!s.is_rooted);
        assert_eq!(s.method, RootMethod::None);
    }
}
