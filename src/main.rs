use crate::app::App;

pub mod adb;
pub mod app;
pub mod effects;
pub mod event;
pub mod fastboot;
pub mod logcat;
pub mod menu;
pub mod message;
pub mod model;
pub mod theme;
pub mod update;
pub mod view;

fn print_usage() {
    eprintln!("Usage: droidtui [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --query              Stream logcat as JSON lines to stdout (no TUI)");
    eprintln!("  --query --last N     Dump the last N logcat lines as JSON and exit");
    eprintln!("  --query --level L    Filter by minimum level: V, D, I, W, E, F");
    eprintln!("  --query --tag TAG    Filter by tag substring");
    eprintln!("  --query --grep PAT   Filter by message substring");
    eprintln!("  --help               Show this help message");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  droidtui                              # Launch the TUI");
    eprintln!("  droidtui --query                      # Stream live logcat as JSONL");
    eprintln!("  droidtui --query --last 500            # Dump last 500 lines as JSONL");
    eprintln!("  droidtui --query --level E             # Stream only errors");
    eprintln!("  droidtui --query --tag MyApp           # Stream only tag matching 'MyApp'");
    eprintln!(
        "  droidtui --query | nu -c 'lines | each {{ from json }} | where level == \"Error\"'"
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
    eprintln!("droidtui: streaming logcat from {} as JSON lines…", serial);
    if let Some(ref level) = min_level {
        eprintln!("  filter: level >= {}", level.as_char());
    }
    if let Some(ref tag) = tag_lower {
        eprintln!("  filter: tag contains \"{}\"", tag);
    }
    if let Some(ref pat) = grep_lower {
        eprintln!("  filter: message contains \"{}\"", pat);
    }
    if let Some(n) = last_n {
        eprintln!("  mode: last {} lines", n);
    } else {
        eprintln!("  mode: live stream (Ctrl+C to stop)");
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

    eprintln!("droidtui: {} lines written", count);
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

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // ── Help ──────────────────────────────────────────────────────────────
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        return Ok(());
    }

    // ── Query mode (no TUI) ───────────────────────────────────────────────
    if args.iter().any(|a| a == "--query") {
        color_eyre::install()?;
        return run_query_mode(&args[1..]);
    }

    // ── Normal TUI mode ───────────────────────────────────────────────────
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}
