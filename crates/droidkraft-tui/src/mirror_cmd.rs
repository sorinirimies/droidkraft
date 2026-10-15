//! Launching the external scrcpy-like GUI mirror window (`droidkraft --mirror`).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const BUILD_HINT: &str =
    "droidkraft GUI not found: build it with `cargo build -p droidkraft --release` or `cargo install --path crates/droidkraft-gui`";

/// File name of the GUI binary for the current platform.
pub fn gui_binary_name() -> &'static str {
    if cfg!(windows) {
        "droidkraft.exe"
    } else {
        "droidkraft"
    }
}

/// Locate the GUI binary: sibling of the running exe first, then `PATH`.
pub fn locate_gui(exe_dir: Option<&Path>, path_var: Option<&OsStr>) -> Option<PathBuf> {
    let name = gui_binary_name();
    if let Some(dir) = exe_dir {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    path_var
        .into_iter()
        .flat_map(std::env::split_paths)
        .map(|d| d.join(name))
        .find(|p| p.is_file())
}

/// Arguments passed to the GUI binary.
pub fn build_args(serial: Option<&str>, size: Option<(u32, u32)>) -> Vec<String> {
    let mut a = vec!["--mirror".to_string()];
    if let Some(s) = serial {
        a.push("--serial".into());
        a.push(s.into());
    }
    if let Some((w, h)) = size {
        a.push("--size".into());
        a.push(format!("{w}x{h}"));
    }
    a
}

/// Build the detached command (stdio null).
pub fn build_command(bin: &Path, serial: Option<&str>, size: Option<(u32, u32)>) -> Command {
    let mut c = Command::new(bin);
    c.args(build_args(serial, size))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    c
}

/// Spawn the external mirror window without waiting for it.
pub fn spawn(serial: Option<&str>, size: Option<(u32, u32)>) -> Result<(), String> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let bin = locate_gui(exe_dir.as_deref(), std::env::var_os("PATH").as_deref())
        .ok_or_else(|| BUILD_HINT.to_string())?;
    build_command(&bin, serial, size)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("failed to start {}: {e}", bin.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_variants() {
        assert_eq!(build_args(None, None), ["--mirror"]);
        assert_eq!(
            build_args(Some("ABC"), Some((800, 600))),
            ["--mirror", "--serial", "ABC", "--size", "800x600"]
        );
    }

    #[test]
    fn locate_sibling_then_path_then_none() {
        let base = std::env::temp_dir().join(format!("dk-mirror-test-{}", std::process::id()));
        let (a, b) = (base.join("a"), base.join("b"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        assert_eq!(locate_gui(Some(&a), None), None);
        std::fs::write(b.join(gui_binary_name()), b"").unwrap();
        let path = std::env::join_paths([&b]).unwrap();
        assert_eq!(
            locate_gui(Some(&a), Some(&path)),
            Some(b.join(gui_binary_name()))
        );
        std::fs::write(a.join(gui_binary_name()), b"").unwrap();
        assert_eq!(
            locate_gui(Some(&a), Some(&path)),
            Some(a.join(gui_binary_name()))
        );
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn command_has_args() {
        let c = build_command(Path::new("/x/droidkraft"), Some("S"), None);
        let args: Vec<_> = c
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, ["--mirror", "--serial", "S"]);
    }
}
