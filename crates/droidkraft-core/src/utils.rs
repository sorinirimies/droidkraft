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

/// Word-wrap a single line to `max_width`, breaking over-long words.
///
/// Tabs are expanded to 4 spaces first (terminals/renderers treat `\t`
/// inconsistently). Returns at least one line.
pub fn wrap_text(line: &str, max_width: usize) -> Vec<String> {
    let line = line.replace('\t', "    ");
    let line = line.as_str();

    if max_width < 4 || line.len() <= max_width {
        return vec![line.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_len = 0;

    for word in line.split_whitespace() {
        if current_len + word.len() < max_width {
            if !current.is_empty() {
                current.push(' ');
                current_len += 1;
            }
            current.push_str(word);
            current_len += word.len();
        } else {
            if !current.is_empty() {
                chunks.push(std::mem::take(&mut current));
            }
            if word.len() > max_width {
                for chunk in word.chars().collect::<Vec<char>>().chunks(max_width) {
                    chunks.push(chunk.iter().collect::<String>());
                }
                current_len = 0;
            } else {
                current = word.to_string();
                current_len = word.len();
            }
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    if chunks.is_empty() {
        vec![line.to_string()]
    } else {
        chunks
    }
}

/// Escape a string for safe inclusion inside single quotes in a POSIX shell.
///
/// Produces the body only (no surrounding quotes): each `'` becomes `'\''`,
/// so `wrap it` in `'...'` at the call site to build e.g. `su -c '...'`.
pub fn shell_single_quote(s: &str) -> String {
    s.replace('\'', "'\\''")
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

    #[test]
    fn wrap_text_short_is_single_line() {
        assert_eq!(
            wrap_text("hello world", 80),
            vec!["hello world".to_string()]
        );
    }

    #[test]
    fn wrap_text_wraps_at_word_boundary() {
        let out = wrap_text("aaaa bbbb cccc", 9);
        assert!(out.len() >= 2);
        assert!(out.iter().all(|l| l.len() <= 9));
    }

    #[test]
    fn wrap_text_breaks_overlong_word() {
        let out = wrap_text("abcdefghijklmnop", 5);
        assert!(out.iter().all(|l| l.len() <= 5));
        assert_eq!(out.concat(), "abcdefghijklmnop");
    }

    #[test]
    fn wrap_text_expands_tabs() {
        assert!(!wrap_text("a\tb", 80)[0].contains('\t'));
    }

    #[test]
    fn shell_single_quote_escapes_quotes() {
        assert_eq!(shell_single_quote("it's"), "it'\\''s");
        assert_eq!(shell_single_quote("plain"), "plain");
    }
}
