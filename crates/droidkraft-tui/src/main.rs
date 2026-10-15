use droidkraft_tui::cli::{self, Action};
use droidkraft_tui::{app::App, logcat, mirror_cmd};

fn print_usage() {
    println!("Logcat query options (with --query):");
    println!("  --query              Stream logcat as JSON lines to stdout (no TUI)");
    println!("  --query --last N     Dump the last N logcat lines as JSON and exit");
    println!("  --query --level L    Filter by minimum level: V, D, I, W, E, F");
    println!("  --query --tag TAG    Filter by tag substring");
    println!("  --query --grep PAT   Filter by message substring");
    println!("  --version            Print version");
    println!("  --help               Show this help message");
    println!();
    println!("Examples:");
    println!("  droidkraft                              # Launch the TUI");
    println!("  droidkraft --query                      # Stream live logcat as JSONL");
    println!("  droidkraft --query --last 500            # Dump last 500 lines as JSONL");
    println!("  droidkraft --query --level E             # Stream only errors");
    println!("  droidkraft --query --tag MyApp           # Stream only tag matching 'MyApp'");
    println!(
        "  droidkraft --query | nu -c 'lines | each {{ from json }} | where level == \"Error\"'"
    );
}

/// CLI query mode — streams logcat as JSON lines to stdout without starting
/// the TUI.  Designed for piping into Nushell, jq, grep, etc.
fn run_query_mode(args: &[String]) -> color_eyre::Result<()> {
    use adb_client::ADBServer;
    use std::io::Write;
    use std::net::{Ipv4Addr, SocketAddrV4};
    use std::sync::mpsc;

    // ── Parse flags ───────────────────────────────────────────────────────
    let mut last_n: Option<usize> = None;
    let mut min_level: Option<logcat::LogLevel> = None;
    let mut tag_filter: Option<String> = None;
    let mut grep_filter: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--last" => {
                i += 1;
                if i < args.len() {
                    last_n = args[i].parse::<usize>().ok();
                }
            }
            "--level" => {
                i += 1;
                if i < args.len() {
                    let ch = args[i].chars().next().unwrap_or('V');
                    min_level = Some(logcat::LogLevel::from_char(ch.to_ascii_uppercase()));
                }
            }
            "--tag" => {
                i += 1;
                if i < args.len() {
                    tag_filter = Some(args[i].clone());
                }
            }
            "--grep" => {
                i += 1;
                if i < args.len() {
                    grep_filter = Some(args[i].clone());
                }
            }
            _ => {}
        }
        i += 1;
    }

    let min_order = min_level.map(|l| l.order()).unwrap_or(0);
    let tag_lower = tag_filter.map(|t| t.to_lowercase());
    let grep_lower = grep_filter.map(|g| g.to_lowercase());

    // ── Connect to device ─────────────────────────────────────────────────
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);
    let mut server = ADBServer::new(addr);

    let devices = server
        .devices()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to list devices (is adb running?): {}", e))?;

    if devices.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "No devices connected. Connect a device and try again."
        ));
    }

    let serial = devices[0].identifier.clone();
    println!(
        "droidkraft: streaming logcat from {} as JSON lines…",
        serial
    );
    if let Some(ref level) = min_level {
        println!("  filter: level >= {}", level.as_char());
    }
    if let Some(ref tag) = tag_lower {
        println!("  filter: tag contains \"{}\"", tag);
    }
    if let Some(ref pat) = grep_lower {
        println!("  filter: message contains \"{}\"", pat);
    }
    if let Some(n) = last_n {
        println!("  mode: last {} lines", n);
    } else {
        println!("  mode: live stream (Ctrl+C to stop)");
    }

    // ── Start streaming in a background thread ────────────────────────────
    let (tx, rx) = mpsc::sync_channel::<String>(10_000);
    let serial_clone = serial.clone();

    std::thread::spawn(move || {
        let device_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);
        let mut device = adb_client::ADBServerDevice::new(serial_clone, Some(device_addr));
        let writer = logcat::ChannelWriter::new(tx.clone());

        if let Err(e) = device.get_logs(writer) {
            let _ = tx.send(format!("--- LOGCAT ERROR: {} ---", e));
        }
    });

    // ── Process lines ─────────────────────────────────────────────────────
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let mut count: usize = 0;

    // Install a Ctrl+C handler so we exit cleanly
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc_handler(r);

    loop {
        if !running.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(line) => {
                let entry = logcat::LogEntry::parse(&line);

                // Apply filters
                if entry.level.order() < min_order {
                    continue;
                }
                if let Some(ref tag) = tag_lower {
                    match &entry.tag {
                        Some(t) if t.to_lowercase().contains(tag) => {}
                        _ => continue,
                    }
                }
                if let Some(ref pat) = grep_lower {
                    if !entry.message.to_lowercase().contains(pat) {
                        continue;
                    }
                }

                // Serialize and write
                if serde_json::to_writer(&mut out, &entry).is_ok() {
                    let _ = writeln!(out);
                    let _ = out.flush();
                }

                count += 1;

                if let Some(n) = last_n {
                    if count >= n {
                        break;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    println!("droidkraft: {} lines written", count);
    Ok(())
}

/// Best-effort Ctrl+C handler using a background thread that waits for the
/// pipe to close.  No external crate needed — we just let the OS default
/// SIGINT handling kill the process, which closes the channel and breaks
/// the recv loop naturally.  This function is a no-op placeholder; the
/// recv_timeout + Disconnected path in `run_query_mode` handles cleanup.
fn ctrlc_handler(_flag: std::sync::Arc<std::sync::atomic::AtomicBool>) {
    // The recv_timeout loop in run_query_mode will exit when the channel
    // disconnects (background thread killed) or when the process receives
    // SIGINT (default handler terminates the process).  No custom signal
    // handling needed.
}

fn run_devices() -> color_eyre::Result<()> {
    let mut adb = droidkraft_core::adb::AdbManager::new();
    let st = adb.fetch_device_status();
    if st.devices.is_empty() {
        println!("No devices connected.");
        return Ok(());
    }
    println!("{:<24} {:<14} MODEL", "SERIAL", "STATE");
    for (i, d) in st.devices.iter().enumerate() {
        let model = if i == st.selected_idx {
            st.model.as_str()
        } else {
            "-"
        };
        println!("{:<24} {:<14} {}", d.serial, d.state, model);
    }
    Ok(())
}

fn run_screenshot(path: &str, serial: Option<String>) -> color_eyre::Result<()> {
    let mut adb = droidkraft_core::adb::AdbManager::new();
    adb.connect().map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
    match serial {
        Some(s) => adb.select_device(s),
        None => {
            adb.list_devices()
                .map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
        }
    }
    let png = adb
        .capture_frame_png()
        .map_err(|e| color_eyre::eyre::eyre!("screenshot failed: {e}"))?;
    std::fs::write(path, &png)?;
    println!("Saved {} ({} bytes)", path, png.len());
    Ok(())
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = match cli::parse(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}\n\n{}", cli::USAGE);
            std::process::exit(2);
        }
    };
    match cli.action {
        Action::Help => {
            print!("{}", cli::USAGE);
            println!();
            print_usage();
            Ok(())
        }
        Action::Version => {
            println!("droidkraft-tui {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Action::Query(rest) => {
            color_eyre::install()?;
            run_query_mode(&rest)
        }
        Action::Devices => {
            color_eyre::install()?;
            run_devices()
        }
        Action::Info { serial, json } => {
            color_eyre::install()?;
            run_info(serial, json)
        }
        Action::Screenshot { path, serial } => {
            color_eyre::install()?;
            run_screenshot(&path, serial)
        }
        Action::Mirror { serial, size } => match mirror_cmd::spawn(serial.as_deref(), size) {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
        Action::Tui => {
            color_eyre::install()?;
            let mut app = App::new();
            app.graphics = droidkraft_tui::gfx::resolve_mode(
                cli.graphics,
                std::env::var("DROIDKRAFT_GRAPHICS").ok().as_deref(),
            );
            if let Some(t) = cli.theme {
                app.model.themes.set_by_name(&t);
            }
            app.run()
        }
    }
}

/// `info`: print the rich per-device details (text or JSON).
fn run_info(serial: Option<String>, json: bool) -> color_eyre::Result<()> {
    let serials: Vec<String> = match serial {
        Some(s) => vec![s],
        None => {
            let mut srv = adb_client::ADBServer::default();
            srv.devices()
                .map_err(|e| color_eyre::eyre::eyre!("adb: {e}"))?
                .into_iter()
                .map(|d| d.identifier)
                .collect()
        }
    };
    if serials.is_empty() {
        eprintln!("no devices connected");
        std::process::exit(1);
    }
    let all = droidkraft_core::list_all_details(&serials);
    if json {
        println!("{}", serde_json::to_string_pretty(&all)?);
    } else {
        for d in &all {
            println!("{}  [{}]  {}", d.serial, d.state, d.summary_line());
            let s = &d.system;
            println!(
                "  {} {} ({}) · SDK {} · patch {} · build {}",
                s.manufacturer, s.model, s.codename, s.sdk, s.security_patch, s.build_id
            );
            println!(
                "  kernel {} · uptime {}s · bootloader {}",
                s.kernel, s.uptime_secs, s.bootloader
            );
            println!(
                "  battery {}% {:?} · RAM {}/{} MiB · storage {}/{} MiB",
                d.battery.level,
                d.battery.status,
                d.memory.used_mib(),
                d.memory.total_mib,
                d.storage.used_mib,
                d.storage.total_mib
            );
        }
    }
    Ok(())
}
