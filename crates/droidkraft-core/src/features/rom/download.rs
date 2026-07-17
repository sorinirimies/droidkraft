//! Downloading ROM/recovery artifacts with progress reporting and SHA-256
//! integrity verification.

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::types::DownloadProgress;

/// Compute the lowercase-hex SHA-256 of a byte slice.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex(&hasher.finalize())
}

/// Compute the lowercase-hex SHA-256 of a file (streamed, constant memory).
pub fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(&hasher.finalize()))
}

/// Verify a file against an expected SHA-256 (case-insensitive hex).
pub fn verify_sha256(path: &Path, expected: &str) -> io::Result<bool> {
    Ok(sha256_file(path)?.eq_ignore_ascii_case(expected.trim()))
}

/// Download `url` to `dest`, invoking `on_progress` as bytes arrive.
///
/// The file is written to a `<dest>.part` temp file and renamed on success so
/// a partial download never masquerades as a complete one.
pub fn download_to(
    url: &str,
    dest: &Path,
    mut on_progress: impl FnMut(DownloadProgress),
) -> Result<PathBuf, String> {
    let resp = ureq::get(url)
        .call()
        .map_err(|e| format!("download request failed: {e}"))?;

    let total = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok());

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
    }
    let part = dest.with_extension("part");
    let mut out = File::create(&part).map_err(|e| format!("create file: {e}"))?;

    let mut reader = resp.into_reader();
    let mut buf = [0u8; 64 * 1024];
    let mut downloaded: u64 = 0;
    on_progress(DownloadProgress {
        downloaded: 0,
        total,
    });
    loop {
        let n = reader.read(&mut buf).map_err(|e| format!("read: {e}"))?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n]).map_err(|e| format!("write: {e}"))?;
        downloaded += n as u64;
        on_progress(DownloadProgress { downloaded, total });
    }
    out.flush().map_err(|e| format!("flush: {e}"))?;
    drop(out);

    std::fs::rename(&part, dest).map_err(|e| format!("finalize: {e}"))?;
    Ok(dest.to_path_buf())
}

/// Download then verify against an expected SHA-256, deleting the file and
/// erroring if the checksum does not match.
pub fn download_and_verify(
    url: &str,
    dest: &Path,
    expected_sha256: Option<&str>,
    on_progress: impl FnMut(DownloadProgress),
) -> Result<PathBuf, String> {
    let path = download_to(url, dest, on_progress)?;
    if let Some(expected) = expected_sha256 {
        match verify_sha256(&path, expected) {
            Ok(true) => {}
            Ok(false) => {
                let _ = std::fs::remove_file(&path);
                return Err("SHA-256 verification failed — download corrupt".into());
            }
            Err(e) => return Err(format!("checksum error: {e}")),
        }
    }
    Ok(path)
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vectors() {
        // Well-known: SHA-256("") and SHA-256("abc").
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_file_and_verify() {
        let dir = std::env::temp_dir();
        let path = dir.join("droidkraft_sha_test.bin");
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(
            sha256_file(&path).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(verify_sha256(
            &path,
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
        )
        .unwrap());
        assert!(!verify_sha256(&path, "deadbeef").unwrap());
        let _ = std::fs::remove_file(&path);
    }
}
