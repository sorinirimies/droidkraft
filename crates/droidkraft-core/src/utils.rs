//! Framework-free helper utilities shared across features.

/// Parse `/proc/meminfo` output and return `(total_mib, available_mib)`.
pub fn parse_meminfo(s: &str) -> (u64, u64) {
    let mut total = 0u64;
    let mut avail = 0u64;
    for line in s.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("MemTotal:") => {
                total = parts
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
                    / 1024;
            }
            Some("MemAvailable:") => {
                avail = parts
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
                    / 1024;
            }
            _ => {}
        }
    }
    (total, avail)
}

/// Convert a char-based cursor index to a byte index within a string.
pub fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// Copy text to the system clipboard using platform-native commands.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    use std::io::Write as _;
    use std::process::{Command, Stdio};

    #[cfg(target_os = "macos")]
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn pbcopy: {}", e))?;

    #[cfg(target_os = "linux")]
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .or_else(|_| {
            Command::new("xsel")
                .args(["--clipboard", "--input"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
        })
        .map_err(|e| format!("No clipboard tool found (xclip/xsel): {}", e))?;

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err("Clipboard not supported on this platform".into());

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| format!("Write to clipboard failed: {}", e))?;
        }
        drop(child.stdin.take());
        child
            .wait()
            .map_err(|e| format!("Clipboard command failed: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_meminfo_extracts_mib() {
        let s = "MemTotal:        8192000 kB\nMemAvailable:    4096000 kB\n";
        assert_eq!(parse_meminfo(s), (8000, 4000));
    }

    #[test]
    fn parse_meminfo_missing_fields() {
        assert_eq!(parse_meminfo("Foo: 1 kB"), (0, 0));
    }

    #[test]
    fn char_to_byte_index_ascii() {
        assert_eq!(char_to_byte_index("hello", 2), 2);
    }

    #[test]
    fn char_to_byte_index_multibyte() {
        // "é" is 2 bytes in UTF-8
        assert_eq!(char_to_byte_index("é_", 1), 2);
    }

    #[test]
    fn char_to_byte_index_out_of_range() {
        assert_eq!(char_to_byte_index("hi", 9), 2);
    }
}
