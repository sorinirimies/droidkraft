//! Unified build resolution across the whole ROM catalog.
//!
//! Frontends call [`resolve_all`] with a device codename and get back every
//! downloadable build from every compatible ROM — a single, simple entry point
//! backing the "big catalog" experience.

use serde_json::Value;

use super::catalog::supported_roms;
use super::lineage;
use super::{graphene, ArtifactKind, BuildSource, RomBuild, RomOs};

/// Resolve downloadable builds for a single ROM + device codename.
pub fn resolve_builds(os: RomOs, codename: &str) -> Result<Vec<RomBuild>, String> {
    match os.build_source() {
        BuildSource::LineageApi => lineage::fetch_builds(codename),
        BuildSource::GrapheneReleases => graphene::fetch_builds(codename),
        BuildSource::OtaJson(templates) => fetch_ota_json(os, codename, templates),
        BuildSource::WebsiteOnly => Ok(Vec::new()),
    }
}

/// Resolve builds across **all** ROMs compatible with the device, aggregating
/// the results. Individual resolver failures are ignored so one flaky source
/// never hides the rest.
pub fn resolve_all(codename: &str) -> Vec<RomBuild> {
    let mut out = Vec::new();
    // Always try the live-API ROMs even if the seed catalog doesn't list the
    // device (the API is authoritative).
    let mut seen_os = std::collections::HashSet::new();
    for rom in supported_roms(codename) {
        seen_os.insert(rom.os);
        if let Ok(mut builds) = resolve_builds(rom.os, codename) {
            out.append(&mut builds);
        }
    }
    for os in [RomOs::LineageOs, RomOs::GrapheneOs] {
        if !seen_os.contains(&os) {
            if let Ok(mut builds) = resolve_builds(os, codename) {
                out.append(&mut builds);
            }
        }
    }
    out
}

/// Fetch and parse a LineageOS-updater-style OTA JSON, trying each template URL
/// in turn (`{codename}` substituted) and returning the first non-empty result.
fn fetch_ota_json(os: RomOs, codename: &str, templates: &[&str]) -> Result<Vec<RomBuild>, String> {
    let mut last_err = None;
    for template in templates {
        let url = template.replace("{codename}", codename);
        match ureq::get(&url).call() {
            Ok(r) => {
                if let Ok(body) = r.into_string() {
                    let builds = parse_ota_json(os, codename, &body);
                    if !builds.is_empty() {
                        return Ok(builds);
                    }
                }
            }
            Err(ureq::Error::Status(404, _)) => continue,
            Err(e) => last_err = Some(e.to_string()),
        }
    }
    match last_err {
        Some(e) => Err(e),
        None => Ok(Vec::new()),
    }
}

/// Parse the common `{"response":[{filename,url,size,version,datetime,…}]}`
/// OTA JSON used by LineageOS-updater-derived ROMs (crDroid, Evolution X, …).
///
/// Pure (no I/O) so it can be unit-tested.
pub fn parse_ota_json(os: RomOs, codename: &str, json: &str) -> Vec<RomBuild> {
    let root: Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    // Accept either {"response":[...]} or a bare [...] array.
    let items = root
        .get("response")
        .and_then(Value::as_array)
        .or_else(|| root.as_array());
    let items = match items {
        Some(a) => a,
        None => return Vec::new(),
    };

    let mut out = Vec::new();
    for it in items {
        let url = it.get("url").and_then(Value::as_str).unwrap_or("");
        if url.is_empty() {
            continue;
        }
        let version = it
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let android_version = it
            .get("android_version")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| version.clone());
        let build_date = it
            .get("datetime")
            .and_then(Value::as_i64)
            .map(format_unix_date);
        let size_bytes = it.get("size").and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        });
        let sha256 = it
            .get("sha256")
            .or_else(|| it.get("hash"))
            .and_then(Value::as_str)
            .map(str::to_string);

        out.push(RomBuild {
            os,
            device_codename: codename.to_string(),
            version: if version.is_empty() {
                os.label().to_string()
            } else {
                version
            },
            android_version,
            build_date,
            kind: ArtifactKind::Rom,
            download_url: url.to_string(),
            size_bytes,
            sha256,
        });
    }
    out
}

/// Format a unix timestamp as `YYYY-MM-DD` (UTC) without a chrono dependency.
fn format_unix_date(ts: i64) -> String {
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

    const OTA: &str = r#"{
        "response": [
            {
                "datetime": 1700000000,
                "filename": "crDroid-14.0-sunfish.zip",
                "id": "abc",
                "romtype": "stable",
                "size": 987654321,
                "url": "https://sourceforge.net/crDroid-14.0-sunfish.zip",
                "version": "14.0",
                "sha256": "aa11"
            }
        ]
    }"#;

    #[test]
    fn parses_ota_response_format() {
        let builds = parse_ota_json(RomOs::CrDroid, "sunfish", OTA);
        assert_eq!(builds.len(), 1);
        let b = &builds[0];
        assert_eq!(b.os, RomOs::CrDroid);
        assert_eq!(b.version, "14.0");
        assert_eq!(b.size_bytes, Some(987654321));
        assert_eq!(b.sha256.as_deref(), Some("aa11"));
        assert_eq!(b.kind, ArtifactKind::Rom);
        assert_eq!(b.build_date.as_deref(), Some("2023-11-14"));
    }

    #[test]
    fn parses_bare_array_format() {
        let json = r#"[{"url":"https://x/rom.zip","version":"1.0"}]"#;
        let builds = parse_ota_json(RomOs::EvolutionX, "davinci", json);
        assert_eq!(builds.len(), 1);
        assert_eq!(builds[0].download_url, "https://x/rom.zip");
    }

    #[test]
    fn ignores_entries_without_url() {
        let json = r#"{"response":[{"version":"1.0"}]}"#;
        assert!(parse_ota_json(RomOs::CrDroid, "x", json).is_empty());
    }

    #[test]
    fn invalid_json_yields_empty() {
        assert!(parse_ota_json(RomOs::CrDroid, "x", "not json").is_empty());
    }

    #[test]
    fn resolve_builds_website_only_is_empty() {
        assert!(resolve_builds(RomOs::PixelExperience, "sunfish")
            .unwrap()
            .is_empty());
    }
}
