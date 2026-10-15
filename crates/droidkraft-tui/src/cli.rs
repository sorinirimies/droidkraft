//! Hand-rolled CLI parser.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Tui,
    Query(Vec<String>),
    Devices,
    Info {
        serial: Option<String>,
        json: bool,
    },
    Screenshot {
        path: String,
        serial: Option<String>,
    },
    Mirror {
        serial: Option<String>,
        size: Option<(u32, u32)>,
    },
    Help,
    Version,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cli {
    pub action: Action,
    pub theme: Option<String>,
    pub graphics: Option<crate::gfx::GraphicsMode>,
}

pub fn parse_size(s: &str) -> Option<(u32, u32)> {
    let (w, h) = s.split_once(['x', 'X'])?;
    let (w, h) = (w.parse().ok()?, h.parse().ok()?);
    (w > 0 && h > 0).then_some((w, h))
}

/// Parse args (without argv[0]).
pub fn parse(args: &[String]) -> Result<Cli, String> {
    let mut theme = None;
    let mut graphics = None;
    let mut serial = None;
    let mut size = None;
    let mut positional: Vec<String> = Vec::new();
    let mut cmd: Option<String> = None;
    let mut mirror_flag = false;
    let mut query = false;
    let mut help = false;
    let mut version = false;
    let mut json = false;
    let mut i = 0;
    let val = |i: &mut usize, flag: &str| -> Result<String, String> {
        *i += 1;
        args.get(*i)
            .cloned()
            .ok_or_else(|| format!("{flag} requires a value"))
    };
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--help" | "-h" => help = true,
            "--version" | "-V" => version = true,
            "--theme" => theme = Some(val(&mut i, a)?),
            "--graphics" => {
                let v = val(&mut i, a)?;
                graphics = Some(crate::gfx::GraphicsMode::parse(&v).ok_or_else(|| {
                    format!(
                        "invalid --graphics '{v}' (expected auto|kitty|iterm2|sixel|halfblocks)"
                    )
                })?);
            }
            "--serial" => serial = Some(val(&mut i, a)?),
            "--size" => {
                let v = val(&mut i, a)?;
                size = Some(
                    parse_size(&v).ok_or_else(|| format!("invalid --size '{v}' (expected WxH)"))?,
                );
            }
            "--json" => json = true,
            "--mirror" => mirror_flag = true,
            "--query" => query = true,
            "--last" | "--level" | "--tag" | "--grep" if query || cmd.is_none() => {
                positional.push(a.to_string());
                positional.push(val(&mut i, a)?);
            }
            _ if a.starts_with('-') => return Err(format!("unknown option '{a}'")),
            _ => {
                if cmd.is_none() {
                    cmd = Some(a.to_string());
                } else {
                    positional.push(a.to_string());
                }
            }
        }
        i += 1;
    }
    let action = if help {
        Action::Help
    } else if version {
        Action::Version
    } else if query {
        Action::Query(positional)
    } else if mirror_flag {
        Action::Mirror { serial, size }
    } else {
        match cmd.as_deref() {
            None => Action::Tui,
            Some("devices") => Action::Devices,
            Some("info") => Action::Info { serial, json },
            Some("screenshot") => Action::Screenshot {
                path: positional
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "droidkraft-screenshot.png".into()),
                serial,
            },
            Some("mirror") => Action::Mirror { serial, size },
            Some(c) => return Err(format!("unknown command '{c}'")),
        }
    };
    Ok(Cli {
        action,
        theme,
        graphics,
    })
}

pub const USAGE: &str = "Usage: droidkraft-tui [OPTIONS] [COMMAND]

Commands:
  (none)                         Launch the TUI
  devices                        List devices (serial, state, model)
  info [--serial S] [--json]     Print device details (battery, memory, CPU, system…)
  screenshot [PATH] [--serial S] Save a PNG screenshot (default droidkraft-screenshot.png)
  mirror [--serial S] [--size WxH]  Open the external mirror window (GUI binary)

Options:
  --mirror                       Same as the `mirror` command
  --theme NAME                   Start with a theme (e.g. porsche)
  --graphics MODE                Screen mirror renderer: auto|kitty|iterm2|sixel|halfblocks
                                 (env DROIDKRAFT_GRAPHICS; default auto)
  --query [--last N] [--level L] [--tag T] [--grep P]
                                 Stream logcat as JSON lines (no TUI)
  -V, --version                  Print version
  -h, --help                     Show this help
";

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Result<Cli, String> {
        parse(&s.split_whitespace().map(String::from).collect::<Vec<_>>())
    }

    #[test]
    fn default_is_tui() {
        assert_eq!(
            p("").unwrap(),
            Cli {
                action: Action::Tui,
                theme: None,
                graphics: None
            }
        );
    }

    #[test]
    fn theme_and_flags() {
        let c = p("--theme nord").unwrap();
        assert_eq!(c.theme.as_deref(), Some("nord"));
        assert_eq!(p("--help").unwrap().action, Action::Help);
        assert_eq!(p("-V").unwrap().action, Action::Version);
        assert!(p("--theme").is_err());
    }

    #[test]
    fn graphics_flag() {
        use crate::gfx::GraphicsMode;
        assert_eq!(
            p("--graphics kitty").unwrap().graphics,
            Some(GraphicsMode::Kitty)
        );
        assert!(p("--graphics bogus").is_err());
        assert!(p("--graphics").is_err());
    }

    #[test]
    fn devices_and_screenshot() {
        assert_eq!(p("devices").unwrap().action, Action::Devices);
        assert_eq!(
            p("screenshot a.png --serial X").unwrap().action,
            Action::Screenshot {
                path: "a.png".into(),
                serial: Some("X".into())
            }
        );
        assert_eq!(
            p("screenshot").unwrap().action,
            Action::Screenshot {
                path: "droidkraft-screenshot.png".into(),
                serial: None
            }
        );
    }

    #[test]
    fn info_command() {
        assert_eq!(
            p("info").unwrap().action,
            Action::Info {
                serial: None,
                json: false
            }
        );
        assert_eq!(
            p("info --serial S --json").unwrap().action,
            Action::Info {
                serial: Some("S".into()),
                json: true
            }
        );
    }

    #[test]
    fn mirror_variants() {
        let want = Action::Mirror {
            serial: Some("S".into()),
            size: Some((800, 600)),
        };
        assert_eq!(p("mirror --serial S --size 800x600").unwrap().action, want);
        assert_eq!(
            p("--mirror --serial S --size 800x600").unwrap().action,
            want
        );
        assert_eq!(
            p("--mirror").unwrap().action,
            Action::Mirror {
                serial: None,
                size: None
            }
        );
        assert!(p("mirror --size bad").is_err());
        assert!(p("mirror --size 0x5").is_err());
    }

    #[test]
    fn query_and_errors() {
        assert_eq!(
            p("--query --last 5 --level E").unwrap().action,
            Action::Query(vec![
                "--last".into(),
                "5".into(),
                "--level".into(),
                "E".into()
            ])
        );
        assert!(p("bogus").is_err());
        assert!(p("--nope").is_err());
    }
}
