//! Hand-rolled command-line parsing (no extra dependencies).

/// Options for the standalone mirror-only window.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MirrorArgs {
    /// Device serial; `None` = first connected device.
    pub serial: Option<String>,
    /// Initial window size in logical pixels.
    pub size: Option<(u32, u32)>,
    /// Window title override.
    pub title: Option<String>,
}

/// What the process was asked to do.
#[derive(Debug, Clone, PartialEq)]
pub enum Cli {
    /// Full application window (default).
    Gui,
    /// Mirror-only window (`--mirror`).
    Mirror(MirrorArgs),
    /// Print usage and exit.
    Help,
}

pub const USAGE: &str = "\
DroidKraft — Android device monitor

USAGE:
    droidkraft                      Open the full application
    droidkraft --mirror [OPTIONS]   Open only a screen-mirror window (scrcpy-like)

MIRROR OPTIONS:
    --serial SERIAL    Device to mirror (default: first connected device)
    --size WxH         Initial window size, e.g. 400x820
    --title TITLE      Window title

    -h, --help         Print this help
";

/// Parse arguments (excluding the program name).
pub fn parse<I, S>(args: I) -> Result<Cli, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut mirror = false;
    let mut m = MirrorArgs::default();
    let mut it = args.into_iter().map(Into::into);
    while let Some(a) = it.next() {
        let (key, inline) = match a.split_once('=') {
            Some((k, v)) if k.starts_with("--") => (k.to_string(), Some(v.to_string())),
            _ => (a.clone(), None),
        };
        let mut value = |name: &str| -> Result<String, String> {
            inline
                .clone()
                .or_else(|| it.next())
                .ok_or_else(|| format!("{name} requires a value"))
        };
        match key.as_str() {
            "-h" | "--help" => return Ok(Cli::Help),
            "--mirror" => mirror = true,
            "--serial" => m.serial = Some(value("--serial")?),
            "--title" => m.title = Some(value("--title")?),
            "--size" => m.size = Some(parse_size(&value("--size")?)?),
            _ => return Err(format!("unknown argument: {a}")),
        }
    }
    if !mirror && m != MirrorArgs::default() {
        return Err("--serial/--size/--title require --mirror".into());
    }
    Ok(if mirror { Cli::Mirror(m) } else { Cli::Gui })
}

fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let bad = || format!("invalid --size '{s}', expected WxH (e.g. 400x820)");
    let (w, h) = s.split_once(['x', 'X']).ok_or_else(bad)?;
    let (w, h): (u32, u32) = (w.parse().map_err(|_| bad())?, h.parse().map_err(|_| bad())?);
    if w < 100 || h < 100 {
        return Err(bad());
    }
    Ok((w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(a: &[&str]) -> Result<Cli, String> {
        parse(a.iter().copied())
    }

    #[test]
    fn default_is_gui() {
        assert_eq!(p(&[]), Ok(Cli::Gui));
    }

    #[test]
    fn help() {
        assert_eq!(p(&["--help"]), Ok(Cli::Help));
        assert_eq!(p(&["--mirror", "-h"]), Ok(Cli::Help));
    }

    #[test]
    fn mirror_full() {
        assert_eq!(
            p(&[
                "--mirror",
                "--serial",
                "ABC",
                "--size",
                "400x820",
                "--title=Hi"
            ]),
            Ok(Cli::Mirror(MirrorArgs {
                serial: Some("ABC".into()),
                size: Some((400, 820)),
                title: Some("Hi".into()),
            }))
        );
    }

    #[test]
    fn mirror_bare() {
        assert_eq!(p(&["--mirror"]), Ok(Cli::Mirror(MirrorArgs::default())));
    }

    #[test]
    fn errors() {
        assert!(p(&["--bogus"]).is_err());
        assert!(p(&["--mirror", "--serial"]).is_err());
        assert!(p(&["--mirror", "--size", "abc"]).is_err());
        assert!(p(&["--mirror", "--size", "10x10"]).is_err());
        assert!(p(&["--serial", "X"]).is_err());
    }
}
