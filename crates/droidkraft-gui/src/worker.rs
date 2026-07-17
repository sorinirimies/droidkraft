//! Background worker thread that owns an [`AdbManager`] and services requests
//! from the GUI without ever blocking the render thread.
//!
//! Communication is over `std::sync::mpsc` channels:
//! the UI sends [`WorkerRequest`]s and drains [`WorkerResponse`]s each tick.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use droidkraft_core::features::fastboot::{FastbootCommand, FastbootManager};
use droidkraft_core::features::flash::{RebootTarget, RootStatus};
use droidkraft_core::features::rom::{self, DeviceProfile, DownloadProgress, FlashStep, RomBuild};
use droidkraft_core::{AdbManager, DeviceStatus};

use crate::commands::CommandAction;

/// Convenience extension: map any `Display` error to `String`.
///
/// A trait (not a macro) is the right tool here because it chains onto a
/// `Result` — `foo().str_err()` reads better than a wrapping macro.
trait StringErr<T> {
    fn str_err(self) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> StringErr<T> for Result<T, E> {
    fn str_err(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

/// How often the worker refreshes device status while idle.
const STATUS_INTERVAL: Duration = Duration::from_secs(2);

/// A request from the UI to the worker.
pub enum WorkerRequest {
    /// Run a catalogue command, tagged with its button label.
    Run {
        label: String,
        action: CommandAction,
    },
    /// Run a raw shell command, tagged with a label.
    Shell { label: String, command: String },
    /// Detect root status.
    DetectRoot,
    /// Remount `/` read-write (requires root).
    Remount,
    /// Execute a fastboot command.
    Fastboot {
        label: String,
        command: FastbootCommand,
    },
    /// Force an immediate device-status refresh.
    RefreshStatus,
    /// Detect the connected device profile (codename, bootloader state).
    DetectProfile,
    /// Resolve downloadable custom-ROM builds for a device codename.
    FetchRoms { codename: String },
    /// Download a ROM/recovery build to `dest` (streams progress).
    Download { build: RomBuild, dest: PathBuf },
    /// Run a single flash step, tracked by the UI's session at `idx`.
    RunFlashStep {
        idx: usize,
        step: FlashStep,
        serial: String,
    },
}

/// A response from the worker to the UI.
pub enum WorkerResponse {
    /// A fresh device status snapshot.
    Status(DeviceStatus),
    /// Output (or error) from a command, tagged with its label.
    Output {
        label: String,
        result: Result<String, String>,
    },
    /// The result of a root-detection request.
    Root(Result<RootStatus, String>),
    /// The detected device profile.
    Profile(DeviceProfile),
    /// Resolved downloadable ROM builds for the device.
    Roms(Result<Vec<RomBuild>, String>),
    /// Progress of an in-flight download.
    DownloadProgress(DownloadProgress),
    /// A finished download (path on success).
    Downloaded(Result<PathBuf, String>),
    /// Result of a single flash step, keyed by the UI session index.
    FlashStepDone {
        idx: usize,
        result: Result<String, String>,
    },
}

/// Handle to the background worker.
pub struct Worker {
    req_tx: Sender<WorkerRequest>,
    res_rx: Receiver<WorkerResponse>,
}

impl Worker {
    /// Spawn the worker thread.
    pub fn spawn() -> Self {
        let (req_tx, req_rx) = channel::<WorkerRequest>();
        let (res_tx, res_rx) = channel::<WorkerResponse>();

        thread::spawn(move || worker_loop(req_rx, res_tx));

        Self { req_tx, res_rx }
    }

    /// Send a request to the worker (ignored if the worker has stopped).
    pub fn send(&self, req: WorkerRequest) {
        let _ = self.req_tx.send(req);
    }

    /// Drain all currently available responses.
    pub fn drain(&self) -> Vec<WorkerResponse> {
        self.res_rx.try_iter().collect()
    }
}

fn worker_loop(req_rx: Receiver<WorkerRequest>, res_tx: Sender<WorkerResponse>) {
    let mut adb = AdbManager::new();

    // Emit an initial snapshot immediately.
    let _ = res_tx.send(WorkerResponse::Status(adb.fetch_device_status()));

    loop {
        match req_rx.recv_timeout(STATUS_INTERVAL) {
            Ok(req) => {
                handle_request(&mut adb, &res_tx, req);
            }
            Err(RecvTimeoutError::Timeout) => {
                let _ = res_tx.send(WorkerResponse::Status(adb.fetch_device_status()));
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

fn handle_request(adb: &mut AdbManager, res_tx: &Sender<WorkerResponse>, req: WorkerRequest) {
    match req {
        WorkerRequest::RefreshStatus => {
            let _ = res_tx.send(WorkerResponse::Status(adb.fetch_device_status()));
        }
        WorkerRequest::Run { label, action } => {
            let result = run_action(adb, action);
            let _ = res_tx.send(WorkerResponse::Output { label, result });
        }
        WorkerRequest::Shell { label, command } => {
            let result = adb.shell_command(&command).str_err();
            let _ = res_tx.send(WorkerResponse::Output { label, result });
        }
        WorkerRequest::DetectRoot => {
            let result = adb.detect_root().str_err();
            let _ = res_tx.send(WorkerResponse::Root(result));
        }
        WorkerRequest::Remount => {
            let result = adb.remount().str_err();
            let _ = res_tx.send(WorkerResponse::Output {
                label: "Remount".into(),
                result,
            });
        }
        WorkerRequest::Fastboot { label, command } => {
            let result = FastbootManager::new().execute(command).str_err();
            let _ = res_tx.send(WorkerResponse::Output { label, result });
        }
        WorkerRequest::DetectProfile => {
            if let Ok(profile) = adb.detect_device_profile() {
                let _ = res_tx.send(WorkerResponse::Profile(profile));
            }
        }
        WorkerRequest::FetchRoms { codename } => {
            let _ = res_tx.send(WorkerResponse::Roms(fetch_roms(&codename)));
        }
        WorkerRequest::Download { build, dest } => {
            // Download on a dedicated thread so the worker loop keeps serving
            // status/flash requests; progress streams back over the channel.
            let tx = res_tx.clone();
            thread::spawn(move || {
                let result = rom::download_and_verify(
                    &build.download_url,
                    &dest,
                    build.sha256.as_deref(),
                    |p| {
                        let _ = tx.send(WorkerResponse::DownloadProgress(p));
                    },
                );
                let _ = tx.send(WorkerResponse::Downloaded(result));
            });
        }
        WorkerRequest::RunFlashStep { idx, step, serial } => {
            let fastboot = FastbootManager::new();
            let result = rom::run_flash_step(&step, &serial, adb, &fastboot);
            let _ = res_tx.send(WorkerResponse::FlashStepDone { idx, result });
        }
    }
}

/// Resolve downloadable ROM builds for a device codename.
///
/// Only projects with a live download API (LineageOS) yield directly
/// downloadable builds; other catalog ROMs are informational.
fn fetch_roms(codename: &str) -> Result<Vec<RomBuild>, String> {
    droidkraft_core::features::rom::lineage::fetch_builds(codename)
}

fn run_action(adb: &mut AdbManager, action: CommandAction) -> Result<String, String> {
    match action {
        CommandAction::Adb(cmd) => adb.execute(cmd).str_err(),
        CommandAction::Shell(cmd) => adb.shell_command(&cmd).str_err(),
        CommandAction::Reboot(target) => reboot(adb, target),
    }
}

fn reboot(adb: &mut AdbManager, target: RebootTarget) -> Result<String, String> {
    adb.reboot(target).str_err()
}
