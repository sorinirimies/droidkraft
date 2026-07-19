//! Custom-ROM domain types: the OS catalog entries, concrete downloadable
//! builds, the detected device profile, and download progress.

use serde::{Deserialize, Serialize};

/// A supported custom-ROM operating system / project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RomOs {
    LineageOs,
    /// GrapheneOS (hardened, Pixel-only, fastboot factory install).
    GrapheneOs,
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
            RomOs::GrapheneOs => "GrapheneOS",
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
            RomOs::GrapheneOs => "https://grapheneos.org",
            RomOs::EOs => "https://e.foundation",
            RomOs::PixelExperience => "https://pixelexperience.org",
            RomOs::CrDroid => "https://crdroid.net",
            RomOs::EvolutionX => "https://evolution-x.org",
            RomOs::ParanoidAndroid => "https://paranoidandroid.co",
        }
    }

    /// How this ROM is installed.
    pub fn install_method(&self) -> InstallMethod {
        match self {
            // GrapheneOS ships signed factory images flashed with fastboot.
            RomOs::GrapheneOs => InstallMethod::FastbootFactory,
            // Most others ship a recovery and are installed by sideloading.
            _ => InstallMethod::RecoverySideload,
        }
    }

    /// Where downloadable builds are resolved from.
    pub fn build_source(&self) -> BuildSource {
        match self {
            RomOs::LineageOs => BuildSource::LineageApi,
            RomOs::GrapheneOs => BuildSource::GrapheneReleases,
            // crDroid & Evolution X publish LineageOS-updater-style OTA JSON on
            // GitHub; `{codename}` is substituted into the template.
            RomOs::CrDroid => BuildSource::OtaJson(&[
                "https://raw.githubusercontent.com/crdroidandroid/android_vendor_crDroidOTA/14.0/{codename}.json",
                "https://raw.githubusercontent.com/crdroidandroid/android_vendor_crDroidOTA/13.0/{codename}.json",
            ]),
            RomOs::EvolutionX => BuildSource::OtaJson(&[
                "https://raw.githubusercontent.com/Evolution-X/OTA/main/builds/{codename}.json",
                "https://raw.githubusercontent.com/Evolution-X/OTA/udc/builds/{codename}.json",
            ]),
            // No stable machine-readable download API wired — catalog/info only.
            RomOs::EOs | RomOs::PixelExperience | RomOs::ParanoidAndroid => {
                BuildSource::WebsiteOnly
            }
        }
    }

    /// Whether builds for this ROM can be resolved and downloaded in-app.
    pub fn is_downloadable(&self) -> bool {
        !matches!(self.build_source(), BuildSource::WebsiteOnly)
    }

    /// Kept for API compatibility — whether device support is resolved live.
    pub fn has_live_api(&self) -> bool {
        matches!(self, RomOs::LineageOs | RomOs::GrapheneOs)
    }

    pub fn all() -> &'static [RomOs] {
        &[
            RomOs::LineageOs,
            RomOs::GrapheneOs,
            RomOs::EOs,
            RomOs::PixelExperience,
            RomOs::CrDroid,
            RomOs::EvolutionX,
            RomOs::ParanoidAndroid,
        ]
    }
}

/// Where a ROM's downloadable builds come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildSource {
    /// The LineageOS v2 download API.
    LineageApi,
    /// The GrapheneOS releases server (fastboot factory images).
    GrapheneReleases,
    /// A LineageOS-updater-style OTA JSON at one of these URL templates
    /// (`{codename}` is substituted); the first that resolves wins.
    OtaJson(&'static [&'static str]),
    /// No machine-readable download source — shown for compatibility only.
    WebsiteOnly,
}

/// The installation strategy for a ROM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallMethod {
    /// Flash a recovery image, boot it, then `adb sideload` the ROM zip.
    RecoverySideload,
    /// Flash signed factory images with fastboot (GrapheneOS / Pixel factory).
    FastbootFactory,
}

/// The role a downloadable artifact plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    /// The ROM zip itself (sideloaded).
    Rom,
    /// A recovery image (flashed via fastboot).
    Recovery,
    /// A signed factory-image zip (flashed via fastboot; GrapheneOS/Pixel).
    Factory,
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
        assert!(RomOs::LineageOs.is_downloadable());
        assert!(RomOs::GrapheneOs.is_downloadable());
        assert!(!RomOs::PixelExperience.is_downloadable());
        assert_eq!(RomOs::all().len(), 7);
    }

    #[test]
    fn install_methods() {
        assert_eq!(
            RomOs::GrapheneOs.install_method(),
            InstallMethod::FastbootFactory
        );
        assert_eq!(
            RomOs::LineageOs.install_method(),
            InstallMethod::RecoverySideload
        );
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
