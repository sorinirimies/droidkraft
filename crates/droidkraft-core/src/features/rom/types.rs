//! Custom-ROM domain types: the OS catalog entries, concrete downloadable
//! builds, the detected device profile, and download progress.

use serde::{Deserialize, Serialize};

/// A supported custom-ROM operating system / project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RomOs {
    LineageOs,
    /// /e/OS (Murena).
    EOs,
    PixelExperience,
    CrDroid,
    EvolutionX,
    ParanoidAndroid,
}

impl RomOs {
    pub fn label(&self) -> &'static str {
        match self {
            RomOs::LineageOs => "LineageOS",
            RomOs::EOs => "/e/OS",
            RomOs::PixelExperience => "Pixel Experience",
            RomOs::CrDroid => "crDroid",
            RomOs::EvolutionX => "Evolution X",
            RomOs::ParanoidAndroid => "Paranoid Android",
        }
    }

    pub fn homepage(&self) -> &'static str {
        match self {
            RomOs::LineageOs => "https://lineageos.org",
            RomOs::EOs => "https://e.foundation",
            RomOs::PixelExperience => "https://pixelexperience.org",
            RomOs::CrDroid => "https://crdroid.net",
            RomOs::EvolutionX => "https://evolution-x.org",
            RomOs::ParanoidAndroid => "https://paranoidandroid.co",
        }
    }

    /// How this ROM is typically installed.
    pub fn install_method(&self) -> InstallMethod {
        // Most modern custom ROMs ship a recovery image and are installed by
        // sideloading the ROM zip from that recovery.
        InstallMethod::RecoverySideload
    }

    /// Whether device support for this ROM can be resolved live from an API.
    pub fn has_live_api(&self) -> bool {
        matches!(self, RomOs::LineageOs)
    }

    pub fn all() -> &'static [RomOs] {
        &[
            RomOs::LineageOs,
            RomOs::EOs,
            RomOs::PixelExperience,
            RomOs::CrDroid,
            RomOs::EvolutionX,
            RomOs::ParanoidAndroid,
        ]
    }
}

/// The installation strategy for a ROM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallMethod {
    /// Flash a recovery image, boot it, then `adb sideload` the ROM zip.
    RecoverySideload,
    /// Flash factory images directly with fastboot (`fastboot flashall`).
    FastbootFlashAll,
}

/// The role a downloadable artifact plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    /// The ROM zip itself (sideloaded).
    Rom,
    /// A recovery image (flashed via fastboot).
    Recovery,
}

/// A catalog entry: a ROM project and the (seed) devices it is known to support.
#[derive(Debug, Clone)]
pub struct CustomRom {
    pub os: RomOs,
    /// Curated list of supported device codenames (seed data; live projects
    /// such as LineageOS additionally resolve support via their API).
    pub devices: &'static [&'static str],
}

impl CustomRom {
    /// Whether this ROM is known to support the given device codename.
    pub fn supports(&self, codename: &str) -> bool {
        self.devices
            .iter()
            .any(|d| d.eq_ignore_ascii_case(codename))
    }
}

/// A concrete, downloadable ROM (or recovery) build for a specific device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RomBuild {
    pub os: RomOs,
    pub device_codename: String,
    /// Human version string, e.g. `"lineage-21.0"` or `"14"`.
    pub version: String,
    /// Android version, e.g. `"14"`.
    pub android_version: String,
    /// Build date (`YYYY-MM-DD`) when known.
    pub build_date: Option<String>,
    pub kind: ArtifactKind,
    pub download_url: String,
    pub size_bytes: Option<u64>,
    /// Expected SHA-256 (lowercase hex) for integrity verification.
    pub sha256: Option<String>,
}

impl RomBuild {
    /// A suggested local filename derived from the URL.
    pub fn file_name(&self) -> String {
        self.download_url
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                format!(
                    "{}-{}-{}.zip",
                    self.os.label().replace(' ', "").to_lowercase(),
                    self.version,
                    self.device_codename
                )
            })
    }
}

/// Information about the connected device used to filter compatible ROMs and
/// to gate the flash flow.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub serial: String,
    /// Device codename (`ro.product.device`) — the key used to match ROMs.
    pub codename: String,
    pub manufacturer: String,
    pub model: String,
    pub android_version: String,
    /// `Some(true)` if the bootloader is unlocked, `Some(false)` if locked,
    /// `None` if it could not be determined.
    pub bootloader_unlocked: Option<bool>,
}

impl DeviceProfile {
    /// A friendly one-line label, e.g. `"Google Pixel 4a (sunfish)"`.
    pub fn display(&self) -> String {
        let base = if self.model.is_empty() {
            self.codename.clone()
        } else {
            self.model.clone()
        };
        if self.codename.is_empty() {
            base
        } else {
            format!("{} ({})", base, self.codename)
        }
    }
}

/// Progress of a download.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
}

impl DownloadProgress {
    /// Fraction complete in `0.0..=1.0`, or `None` when the total is unknown.
    pub fn fraction(&self) -> Option<f32> {
        self.total.map(|t| {
            if t == 0 {
                0.0
            } else {
                (self.downloaded as f32 / t as f32).clamp(0.0, 1.0)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rom_os_labels_and_api() {
        assert_eq!(RomOs::LineageOs.label(), "LineageOS");
        assert!(RomOs::LineageOs.has_live_api());
        assert!(!RomOs::CrDroid.has_live_api());
        assert_eq!(RomOs::all().len(), 6);
    }

    #[test]
    fn custom_rom_supports_is_case_insensitive() {
        let rom = CustomRom {
            os: RomOs::LineageOs,
            devices: &["sunfish", "davinci"],
        };
        assert!(rom.supports("sunfish"));
        assert!(rom.supports("SUNFISH"));
        assert!(!rom.supports("cheeseburger"));
    }

    #[test]
    fn build_file_name_from_url() {
        let b = RomBuild {
            os: RomOs::LineageOs,
            device_codename: "sunfish".into(),
            version: "lineage-21.0".into(),
            android_version: "14".into(),
            build_date: None,
            kind: ArtifactKind::Rom,
            download_url: "https://mirror.example/lineage-21.0-sunfish.zip".into(),
            size_bytes: None,
            sha256: None,
        };
        assert_eq!(b.file_name(), "lineage-21.0-sunfish.zip");
    }

    #[test]
    fn device_profile_display() {
        let p = DeviceProfile {
            model: "Pixel 4a".into(),
            codename: "sunfish".into(),
            ..Default::default()
        };
        assert_eq!(p.display(), "Pixel 4a (sunfish)");
    }

    #[test]
    fn download_progress_fraction() {
        let p = DownloadProgress {
            downloaded: 50,
            total: Some(200),
        };
        assert_eq!(p.fraction(), Some(0.25));
        let unknown = DownloadProgress {
            downloaded: 10,
            total: None,
        };
        assert_eq!(unknown.fraction(), None);
    }
}
