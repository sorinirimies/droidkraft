//! LineageOS device-support and build resolution via the official
//! `download.lineageos.org` v2 API.

use serde_json::Value;

use super::types::{ArtifactKind, RomBuild, RomOs};

/// Base URL of the LineageOS download API.
const API_BASE: &str = "https://download.lineageos.org/api/v2/devices";

/// Fetch the available LineageOS builds for a device codename.
///
/// Returns an empty vec if the device is unsupported (HTTP 404).
pub fn fetch_builds(codename: &str) -> Result<Vec<RomBuild>, String> {
    let url = format!("{}/{}/builds", API_BASE, codename);
    let resp = ureq::get(&url).call();
    match resp {
        Ok(r) => {
            let body = r
                .into_string()
                .map_err(|e| format!("read LineageOS response: {e}"))?;
            parse_builds_json(codename, &body)
        }
        Err(ureq::Error::Status(404, _)) => Ok(Vec::new()),
        Err(e) => Err(format!("LineageOS API request failed: {e}")),
    }
}

/// Whether LineageOS officially supports a device (has at least one build).
pub fn is_supported(codename: &str) -> bool {
    fetch_builds(codename).map(|b| !b.is_empty()).unwrap_or(false)
}

/// Map a LineageOS major version to its Android release.
pub fn android_version_for(lineage_version: &str) -> String {
    let major = lineage_version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok());
    match major {
        Some(22) => "16",
        Some(21) => "14",
        Some(20) => "13",
        Some(19) => "12",
        Some(18) => "11",
        Some(17) => "10",
        Some(16) => "9",
        _ => "?",
    }
    .to_string()
}

/// Parse the LineageOS v2 `builds` JSON payload into [`RomBuild`]s.
///
/// Kept pure (no I/O) so it can be unit-tested against sample payloads.
pub fn parse_builds_json(codename: &str, json: &str) -> Result<Vec<RomBuild>, String> {
    let root: Value =
        serde_json::from_str(json).map_err(|e| format!("invalid LineageOS JSON: {e}"))?;
    let builds = root
        .as_array()
        .ok_or_else(|| "expected a JSON array of builds".to_string())?;

    let mut out = Vec::new();
    for build in builds {
        let version = build
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let android_version = android_version_for(&version);
        let build_date = build
            .get("date")
            .and_then(Value::as_i64)
            .map(format_unix_date)
            .or_else(|| {
                build
                    .get("datetime")
                    .and_then(Value::as_i64)
                    .map(format_unix_date)
            });

        let files = match build.get("files").and_then(Value::as_array) {
            Some(f) => f,
            None => continue,
        };

        for file in files {
            let url = file
                .get("url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if url.is_empty() {
                continue;
            }
            let filename = file
                .get("filename")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_lowercase();
            let kind = if filename.ends_with(".zip") {
                ArtifactKind::Rom
            } else if filename.contains("recovery") || filename.ends_with(".img") {
                ArtifactKind::Recovery
            } else {
                continue;
            };

            out.push(RomBuild {
                os: RomOs::LineageOs,
                device_codename: codename.to_string(),
                version: format!("lineage-{version}"),
                android_version: android_version.clone(),
                build_date: build_date.clone(),
                kind,
                download_url: url,
                size_bytes: file.get("size").and_then(Value::as_u64),
                sha256: file
                    .get("sha256")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            });
        }
    }
    Ok(out)
}

/// Format a unix timestamp as `YYYY-MM-DD` (UTC), avoiding a chrono dependency.
fn format_unix_date(ts: i64) -> String {
    // Days since epoch → civil date (Howard Hinnant's algorithm).
    let days = ts.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"[
        {
            "date": 1700000000,
            "type": "nightly",
            "version": "21.0",
            "files": [
                {
                    "filename": "lineage-21.0-20231114-nightly-sunfish-signed.zip",
                    "sha256": "abc123",
                    "size": 1234567890,
                    "url": "https://mirror.lineageos.org/full/sunfish/lineage-21.0-sunfish.zip"
                },
                {
                    "filename": "lineage-21.0-20231114-recovery-sunfish.img",
                    "sha256": "def456",
                    "size": 100000,
                    "url": "https://mirror.lineageos.org/full/sunfish/recovery.img"
                }
            ]
        }
    ]"#;

    #[test]
    fn android_version_mapping() {
        assert_eq!(android_version_for("21.0"), "14");
        assert_eq!(android_version_for("20.0"), "13");
        assert_eq!(android_version_for("weird"), "?");
    }

    #[test]
    fn parses_rom_and_recovery() {
        let builds = parse_builds_json("sunfish", SAMPLE).unwrap();
        assert_eq!(builds.len(), 2);
        let rom = &builds[0];
        assert_eq!(rom.kind, ArtifactKind::Rom);
        assert_eq!(rom.os, RomOs::LineageOs);
        assert_eq!(rom.version, "lineage-21.0");
        assert_eq!(rom.android_version, "14");
        assert_eq!(rom.sha256.as_deref(), Some("abc123"));
        assert_eq!(rom.size_bytes, Some(1234567890));
        assert_eq!(builds[1].kind, ArtifactKind::Recovery);
    }

    #[test]
    fn build_date_is_formatted() {
        let builds = parse_builds_json("sunfish", SAMPLE).unwrap();
        assert_eq!(builds[0].build_date.as_deref(), Some("2023-11-14"));
    }

    #[test]
    fn empty_array_yields_no_builds() {
        assert!(parse_builds_json("x", "[]").unwrap().is_empty());
    }

    #[test]
    fn invalid_json_errors() {
        assert!(parse_builds_json("x", "not json").is_err());
    }

    #[test]
    fn format_unix_date_epoch() {
        assert_eq!(format_unix_date(0), "1970-01-01");
    }
}
