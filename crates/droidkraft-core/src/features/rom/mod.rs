//! Custom-ROM feature — catalog, device-compatibility filtering, build
//! resolution (LineageOS live API), downloading with verification, and a
//! consent-gated flash orchestrator.

pub mod catalog;
pub mod download;
pub mod flash;
pub mod lineage;
pub mod ops;
pub mod types;

pub use catalog::{catalog, roms_for_device, supported_roms};
pub use download::{download_and_verify, download_to, sha256_file, sha256_hex, verify_sha256};
pub use flash::{
    build_plan, preflight, FlashOptions, FlashPlan, FlashSession, FlashStep, StepStatus,
};
pub use types::{
    ArtifactKind, CustomRom, DeviceProfile, DownloadProgress, InstallMethod, RomBuild, RomOs,
};
