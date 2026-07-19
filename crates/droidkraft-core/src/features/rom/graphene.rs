//! GrapheneOS build resolution via the `releases.grapheneos.org` server.
//!
//! GrapheneOS publishes a small metadata file per device
//! (`{device}-stable`) and signed factory-image zips
//! (`{device}-factory-{build}.zip`) flashed with fastboot.

use super::types::{ArtifactKind, RomBuild, RomOs};

const RELEASES_BASE: &str = "https://releases.grapheneos.org";

/// Fetch the latest stable GrapheneOS factory image for a Pixel device.
///
/// Returns an empty vec if the device is unsupported (HTTP 404).
pub fn fetch_builds(codename: &str) -> Result<Vec<RomBuild>, String> {
    let url = format!("{RELEASES_BASE}/{codename}-stable");
    match ureq::get(&url).call() {
        Ok(r) => {
            let body = r
                .into_string()
                .map_err(|e| format!("read GrapheneOS metadata: {e}"))?;
            Ok(parse_metadata(codename, &body).into_iter().collect())
        }
        Err(ureq::Error::Status(404, _)) => Ok(Vec::new()),
        Err(e) => Err(format!("GrapheneOS request failed: {e}")),
    }
}

/// Parse the `{device}-stable` metadata line into a factory [`RomBuild`].
///
/// The metadata is whitespace-separated: `<build_number> <channel> <version>`.
/// Kept pure so it can be unit-tested.
pub fn parse_metadata(codename: &str, text: &str) -> Option<RomBuild> {
    let mut tokens = text.split_whitespace();
    let build = tokens.next()?.to_string();
    if !build.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let _channel = tokens.next().unwrap_or("stable");
    let android_version = tokens.next().unwrap_or("").to_string();

    let url = format!("{RELEASES_BASE}/{codename}-factory-{build}.zip");
    // Build date is the first 8 digits of the build number (YYYYMMDD).
    let build_date = if build.len() >= 8 {
        Some(format!(
            "{}-{}-{}",
            &build[0..4],
            &build[4..6],
            &build[6..8]
        ))
    } else {
        None
    };

    Some(RomBuild {
        os: RomOs::GrapheneOs,
        device_codename: codename.to_string(),
        version: build.clone(),
        android_version,
        build_date,
        kind: ArtifactKind::Factory,
        download_url: url,
        size_bytes: None,
        // GrapheneOS ships a detached signature rather than a plain SHA-256.
        sha256: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_metadata_line() {
        let b = parse_metadata("sunfish", "2024081200 stable 14").unwrap();
        assert_eq!(b.os, RomOs::GrapheneOs);
        assert_eq!(b.version, "2024081200");
        assert_eq!(b.android_version, "14");
        assert_eq!(b.kind, ArtifactKind::Factory);
        assert_eq!(
            b.download_url,
            "https://releases.grapheneos.org/sunfish-factory-2024081200.zip"
        );
        assert_eq!(b.build_date.as_deref(), Some("2024-08-12"));
    }

    #[test]
    fn rejects_non_numeric_build() {
        assert!(parse_metadata("sunfish", "garbage stable 14").is_none());
    }

    #[test]
    fn empty_metadata_is_none() {
        assert!(parse_metadata("sunfish", "").is_none());
    }
}
