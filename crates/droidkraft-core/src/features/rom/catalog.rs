//! A curated seed catalog of custom ROMs and device-compatibility filtering.
//!
//! The device lists here are a *seed* of common codenames so the picker works
//! offline. Projects with a live API (LineageOS) additionally resolve real,
//! current device support and builds through [`super::lineage`].

use super::types::{CustomRom, DeviceProfile, RomOs};

/// The full catalog of known custom ROMs with seed device support.
pub fn catalog() -> Vec<CustomRom> {
    vec![
        CustomRom {
            os: RomOs::LineageOs,
            // A small seed; the live API is authoritative (see `lineage`).
            devices: &[
                "sunfish",
                "bramble",
                "redfin",
                "barbet",
                "davinci",
                "cheeseburger",
                "dumpling",
                "enchilada",
                "fajita",
                "guacamole",
                "lmi",
                "alioth",
                "raven",
                "oriole",
                "bluejay",
            ],
        },
        CustomRom {
            os: RomOs::PixelExperience,
            devices: &[
                "sunfish",
                "davinci",
                "lmi",
                "alioth",
                "cheeseburger",
                "raven",
                "oriole",
            ],
        },
        CustomRom {
            os: RomOs::CrDroid,
            devices: &[
                "davinci", "lmi", "alioth", "sunfish", "raven", "oriole", "miatoll",
            ],
        },
        CustomRom {
            os: RomOs::EvolutionX,
            devices: &["davinci", "alioth", "sunfish", "raven", "oriole"],
        },
        CustomRom {
            os: RomOs::EOs,
            devices: &[
                "sunfish",
                "bramble",
                "redfin",
                "cheeseburger",
                "enchilada",
                "davinci",
            ],
        },
        CustomRom {
            os: RomOs::ParanoidAndroid,
            devices: &["davinci", "alioth", "cheeseburger", "enchilada"],
        },
    ]
}

/// The custom ROMs from the seed catalog that support the given device codename.
pub fn supported_roms(codename: &str) -> Vec<CustomRom> {
    if codename.trim().is_empty() {
        return Vec::new();
    }
    catalog()
        .into_iter()
        .filter(|rom| rom.supports(codename))
        .collect()
}

/// The custom ROMs compatible with a detected [`DeviceProfile`].
pub fn roms_for_device(profile: &DeviceProfile) -> Vec<CustomRom> {
    supported_roms(&profile.codename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_covers_all_known_os() {
        let cat = catalog();
        for os in RomOs::all() {
            assert!(cat.iter().any(|r| r.os == *os), "missing {:?}", os);
        }
    }

    #[test]
    fn supported_roms_filters_by_codename() {
        let roms = supported_roms("sunfish");
        assert!(roms.iter().any(|r| r.os == RomOs::LineageOs));
        // sunfish (Pixel 4a) is a Google device — not in the crDroid seed.
        assert!(supported_roms("nonexistent-codename").is_empty());
    }

    #[test]
    fn empty_codename_yields_nothing() {
        assert!(supported_roms("").is_empty());
        assert!(supported_roms("   ").is_empty());
    }

    #[test]
    fn roms_for_device_uses_codename() {
        let profile = DeviceProfile {
            codename: "davinci".into(),
            ..Default::default()
        };
        let roms = roms_for_device(&profile);
        assert!(roms.len() >= 3);
        assert!(roms.iter().all(|r| r.supports("davinci")));
    }
}
