//! Background worker thread owning an [`AdbManager`].
//!
//! The render thread never touches adb: it sends [`Req`]s and drains [`Res`]es
//! on every tick. Every request and response carries the device serial so the
//! UI can route it to the right per-device workspace. The worker also polls
//! the device list and every online device's [`DeviceDetails`] (all devices
//! every ~3 s, the focused one every ~1.5 s).

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use droidkraft_core::features::fastboot::{FastbootCommand, FastbootManager};
use droidkraft_core::features::flash::RootStatus;
use droidkraft_core::{collect_device_details, AdbCommand, AdbManager, CpuSample, DeviceDetails};

use crate::commands::CmdAction;

/// Poll interval for devices that are not focused.
pub const POLL_INTERVAL: Duration = Duration::from_secs(3);
/// Poll interval for the device whose workspace is open.
pub const FOCUS_INTERVAL: Duration = Duration::from_millis(1500);
/// How often the device list itself is refreshed.
pub const LIST_INTERVAL: Duration = Duration::from_secs(2);

/// Request from the UI to the worker. `serial` is the target workspace.
pub enum Req {
    Run {
        serial: String,
        label: String,
        action: CmdAction,
    },
    Shell {
        serial: String,
        label: String,
        command: String,
    },
    DetectRoot {
        serial: String,
    },
    Remount {
        serial: String,
    },
    Fastboot {
        serial: String,
        label: String,
        command: FastbootCommand,
    },
    FastbootDevices {
        serial: String,
    },
    FactoryFlash {
        serial: String,
        source: std::path::PathBuf,
        wipe: bool,
    },
    Screenshot {
        serial: String,
        path: std::path::PathBuf,
    },
    /// Tell the worker which workspace is open (polled faster). `None` = home.
    Focus(Option<String>),
    /// Poll the list and all devices right now.
    Refresh,
}

/// One entry of `adb devices`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedDevice {
    pub serial: String,
    pub state: String,
}

/// Response from the worker to the UI.
pub enum Res {
    Devices(Vec<ListedDevice>),
    Details(Box<DeviceDetails>),
    Output {
        serial: String,
        label: String,
        result: Result<String, String>,
    },
    Root {
        serial: String,
        result: Result<RootStatus, String>,
    },
    FastbootDevices {
        serial: String,
        devices: Vec<String>,
        available: bool,
    },
    /// Result of a Flash & Root operation (fastboot, remount, factory flash).
    Flash {
        serial: String,
        label: String,
        result: Result<String, String>,
    },
    Screenshot {
        serial: String,
        result: Result<std::path::PathBuf, String>,
    },
    /// Streaming factory-flash progress.
    Progress {
        serial: String,
        done: usize,
        total: usize,
        label: String,
    },
}

/// Handle to the worker thread.
pub struct Worker {
    tx: Sender<Req>,
    rx: Receiver<Res>,
}

impl Worker {
    pub fn spawn() -> Self {
        let (tx, req_rx) = channel::<Req>();
        let (res_tx, rx) = channel::<Res>();
        thread::spawn(move || run(req_rx, res_tx));
        Self { tx, rx }
    }

    /// A worker with no thread behind it (tests): requests are dropped.
    pub fn detached() -> Self {
        let (tx, _r) = channel::<Req>();
        let (_t, rx) = channel::<Res>();
        Self { tx, rx }
    }

    pub fn send(&self, req: Req) {
        let _ = self.tx.send(req);
    }

    pub fn drain(&self) -> Vec<Res> {
        self.rx.try_iter().collect()
    }
}

/// Whether a device should be polled given time since its last poll.
pub fn is_due(since_last: Option<Duration>, focused: bool) -> bool {
    let iv = if focused {
        FOCUS_INTERVAL
    } else {
        POLL_INTERVAL
    };
    since_last.is_none_or(|d| d >= iv)
}

#[derive(Default)]
struct Poller {
    focus: Option<String>,
    last_list: Option<Instant>,
    devices: Vec<ListedDevice>,
    last_poll: HashMap<String, Instant>,
    prev_cpu: HashMap<String, CpuSample>,
    force: bool,
}

fn list_devices() -> Option<Vec<ListedDevice>> {
    use std::net::{Ipv4Addr, SocketAddrV4};
    let mut server = adb_client::ADBServer::new(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5037));
    let mut v: Vec<ListedDevice> = server
        .devices()
        .ok()?
        .into_iter()
        .map(|d| ListedDevice {
            serial: d.identifier,
            state: normalize_state(&format!("{:?}", d.state)),
        })
        .collect();
    v.sort_by(|a, b| a.serial.cmp(&b.serial));
    Some(v)
}

/// Map adb's debug state (`Device`, `Unauthorized`, …) to adb's wire names.
pub fn normalize_state(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "device" => "device".into(),
        o => o.to_string(),
    }
}

fn poll(adb: &mut AdbManager, rx: &Receiver<Req>, tx: &Sender<Res>, p: &mut Poller) {
    let now = Instant::now();
    if p.force
        || p.last_list
            .is_none_or(|t| now.duration_since(t) >= LIST_INTERVAL)
    {
        p.last_list = Some(now);
        match list_devices() {
            Some(l) => {
                p.devices = l.clone();
                let _ = tx.send(Res::Devices(l));
            }
            None => {
                p.devices.clear();
                let _ = tx.send(Res::Devices(vec![]));
            }
        }
    }
    let serials: Vec<String> = p
        .devices
        .iter()
        .filter(|d| d.state == "device")
        .map(|d| d.serial.clone())
        .collect();
    p.last_poll.retain(|k, _| serials.contains(k));
    p.prev_cpu.retain(|k, _| serials.contains(k));
    for s in serials {
        let focused = p.focus.as_deref() == Some(s.as_str());
        let since = p.last_poll.get(&s).map(|t| now.duration_since(*t));
        if !(p.force || is_due(since, focused)) {
            continue;
        }
        p.last_poll.insert(s.clone(), Instant::now());
        let (d, cpu) = collect_device_details(&s, p.prev_cpu.get(&s));
        p.prev_cpu.insert(s, cpu);
        let _ = tx.send(Res::Details(Box::new(d)));
        // Keep the UI responsive: serve queued requests between devices.
        while let Ok(req) = rx.try_recv() {
            handle(adb, tx, p, req);
        }
    }
    p.force = false;
}

fn run(req_rx: Receiver<Req>, res_tx: Sender<Res>) {
    let mut adb = AdbManager::new();
    let mut p = Poller {
        force: true,
        ..Default::default()
    };
    loop {
        poll(&mut adb, &req_rx, &res_tx, &mut p);
        match req_rx.recv_timeout(Duration::from_millis(250)) {
            Ok(req) => handle(&mut adb, &res_tx, &mut p, req),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn s_err<T, E: std::fmt::Display>(r: Result<T, E>) -> Result<T, String> {
    r.map_err(|e| e.to_string())
}

fn handle(adb: &mut AdbManager, tx: &Sender<Res>, p: &mut Poller, req: Req) {
    match req {
        Req::Refresh => p.force = true,
        Req::Focus(s) => {
            // Opening a workspace polls it immediately.
            if let Some(s) = &s {
                p.last_poll.remove(s);
            }
            p.focus = s;
        }
        Req::Run {
            serial,
            label,
            action,
        } => {
            adb.select_device(serial.clone());
            let result = match action {
                CmdAction::Adb(c) => s_err(adb.execute(c)),
                CmdAction::Shell(c) => s_err(adb.shell_command(&c)),
                CmdAction::Reboot(t) => s_err(adb.reboot(t)),
            };
            let _ = tx.send(Res::Output {
                serial,
                label,
                result,
            });
        }
        Req::Shell {
            serial,
            label,
            command,
        } => {
            adb.select_device(serial.clone());
            let result = s_err(adb.shell_command(&command));
            let _ = tx.send(Res::Output {
                serial,
                label,
                result,
            });
        }
        Req::Screenshot { serial, path } => {
            adb.select_device(serial.clone());
            let result = s_err(adb.capture_frame_png()).and_then(|png| {
                if let Some(dir) = path.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                std::fs::write(&path, png)
                    .map(|_| path.clone())
                    .map_err(|e| e.to_string())
            });
            let _ = tx.send(Res::Screenshot { serial, result });
        }
        Req::DetectRoot { serial } => {
            adb.select_device(serial.clone());
            let _ = tx.send(Res::Root {
                serial,
                result: s_err(adb.detect_root()),
            });
        }
        Req::Remount { serial } => {
            adb.select_device(serial.clone());
            let result = s_err(adb.remount());
            let _ = tx.send(Res::Flash {
                serial,
                label: "Remount".into(),
                result,
            });
        }
        Req::Fastboot {
            serial,
            label,
            command,
        } => {
            let result = s_err(FastbootManager::new().execute(command));
            let _ = tx.send(Res::Flash {
                serial,
                label,
                result,
            });
        }
        Req::FactoryFlash {
            serial,
            source,
            wipe,
        } => {
            let tx = tx.clone();
            thread::spawn(move || {
                let opts = droidkraft_fastboot::FactoryOptions {
                    wipe,
                    ..Default::default()
                };
                let ptx = tx.clone();
                let ps = serial.clone();
                let mut progress = |p: droidkraft_fastboot::FlashProgress| {
                    let _ = ptx.send(Res::Progress {
                        serial: ps.clone(),
                        done: p.step_index,
                        total: p.total_steps,
                        label: p.label,
                    });
                };
                let result = s_err(droidkraft_fastboot::flash_all(
                    &source,
                    None,
                    &opts,
                    &mut progress,
                ));
                let _ = tx.send(Res::Flash {
                    serial,
                    label: "Factory flash".into(),
                    result,
                });
            });
        }
        Req::FastbootDevices { serial } => {
            let _ = tx.send(Res::FastbootDevices {
                serial,
                devices: FastbootManager::devices(),
                available: FastbootManager::is_available(),
            });
        }
    }
}

/// Convenience used by tests and the CLI: run one adb command synchronously.
pub fn run_adb(cmd: AdbCommand) -> Result<String, String> {
    s_err(AdbManager::new().execute(cmd))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_logic() {
        assert!(is_due(None, false));
        assert!(!is_due(Some(Duration::from_secs(2)), false));
        assert!(is_due(Some(Duration::from_secs(3)), false));
        assert!(is_due(Some(Duration::from_millis(1600)), true));
        assert!(!is_due(Some(Duration::from_millis(500)), true));
    }

    #[test]
    fn state_names() {
        assert_eq!(normalize_state("Device"), "device");
        assert_eq!(normalize_state("Unauthorized"), "unauthorized");
    }
}
