//! Custom-ROM flasher screen state for the TUI.
//!
//! Mirrors the GUI flow: detect the device → list compatible downloadable ROM
//! builds → download (with progress) → run a consent-gated [`FlashSession`].
//! All blocking work runs on a background worker thread; the UI polls results
//! on each tick.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use droidkraft_core::features::fastboot::FastbootManager;
use droidkraft_core::features::rom::{
    self, build_plan, DeviceProfile, DownloadProgress, FlashOptions, FlashSession, FlashStep,
    InstallMethod, RomBuild, StepStatus,
};
use droidkraft_core::AdbManager;

/// Request sent to the ROM worker thread.
enum RomReq {
    Detect,
    Download {
        build: RomBuild,
        dest: PathBuf,
    },
    RunStep {
        idx: usize,
        step: FlashStep,
        serial: String,
    },
}

/// Response from the ROM worker thread.
enum RomRes {
    Profile(DeviceProfile),
    Builds(Result<Vec<RomBuild>, String>),
    Progress(DownloadProgress),
    Downloaded(Result<PathBuf, String>),
    StepDone {
        idx: usize,
        result: Result<String, String>,
    },
}

/// Full state of the ROM flasher screen.
pub struct RomFlashState {
    req_tx: Sender<RomReq>,
    res_rx: Receiver<RomRes>,
    pub profile: Option<DeviceProfile>,
    pub builds: Vec<RomBuild>,
    pub selected: usize,
    pub status: String,
    pub download_progress: Option<DownloadProgress>,
    pub downloaded_path: Option<PathBuf>,
    pending_method: Option<InstallMethod>,
    pub session: Option<FlashSession>,
    pub busy: bool,
}

impl std::fmt::Debug for RomFlashState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RomFlashState")
            .field("builds", &self.builds.len())
            .field("selected", &self.selected)
            .field("busy", &self.busy)
            .finish()
    }
}

impl Default for RomFlashState {
    fn default() -> Self {
        Self::new()
    }
}

impl RomFlashState {
    pub fn new() -> Self {
        let (req_tx, req_rx) = channel::<RomReq>();
        let (res_tx, res_rx) = channel::<RomRes>();
        thread::spawn(move || worker_loop(req_rx, res_tx));
        Self {
            req_tx,
            res_rx,
            profile: None,
            builds: Vec::new(),
            selected: 0,
            status: "Press 'd' to detect this device and find compatible ROMs.".to_string(),
            download_progress: None,
            downloaded_path: None,
            pending_method: None,
            session: None,
            busy: false,
        }
    }

    /// Kick off device detection + ROM lookup.
    pub fn detect(&mut self) {
        self.builds.clear();
        self.session = None;
        self.downloaded_path = None;
        self.selected = 0;
        self.status = "Detecting device…".to_string();
        let _ = self.req_tx.send(RomReq::Detect);
    }

    pub fn select_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_down(&mut self) {
        if !self.builds.is_empty() {
            self.selected = (self.selected + 1).min(self.builds.len() - 1);
        }
    }

    /// Download the currently selected build.
    pub fn download_selected(&mut self) {
        if self.session.is_some() {
            return;
        }
        if let Some(build) = self.builds.get(self.selected).cloned() {
            let dest = std::env::temp_dir()
                .join("droidkraft-roms")
                .join(build.file_name());
            self.pending_method = Some(build.os.install_method());
            self.status = format!("Downloading {}…", build.file_name());
            self.download_progress = Some(DownloadProgress {
                downloaded: 0,
                total: build.size_bytes,
            });
            let _ = self.req_tx.send(RomReq::Download { build, dest });
        }
    }

    /// Run the next pending flash step (caller has confirmed if destructive).
    pub fn run_next_step(&mut self) {
        if self.busy {
            return;
        }
        let serial = self
            .profile
            .as_ref()
            .map(|p| p.serial.clone())
            .unwrap_or_default();
        if let Some(session) = &self.session {
            if let Some((idx, step)) = session.next_step() {
                self.busy = true;
                self.status = format!("Running: {}", step.label());
                let _ = self.req_tx.send(RomReq::RunStep {
                    idx,
                    step: step.clone(),
                    serial,
                });
            }
        }
    }

    /// Whether the next step is destructive and needs explicit confirmation.
    pub fn next_step_needs_confirmation(&self) -> bool {
        self.session
            .as_ref()
            .and_then(|s| s.next_step())
            .map(|(_, step)| step.requires_confirmation())
            .unwrap_or(false)
    }

    fn build_session(&mut self, rom_zip: PathBuf) {
        let serial = self
            .profile
            .as_ref()
            .map(|p| p.serial.clone())
            .unwrap_or_default();
        let mut opts = match self.pending_method {
            Some(InstallMethod::FastbootFactory) => FlashOptions::factory(rom_zip),
            _ => FlashOptions::new(rom_zip),
        };
        if self.profile.as_ref().and_then(|p| p.bootloader_unlocked) == Some(false) {
            opts.unlock_bootloader = true;
        }
        self.session = Some(FlashSession::new(build_plan(&opts), serial));
    }

    /// Drain worker responses into state. Call once per tick.
    pub fn poll(&mut self) {
        while let Ok(res) = self.res_rx.try_recv() {
            match res {
                RomRes::Profile(p) => {
                    self.status = if p.codename.is_empty() {
                        "Could not detect device codename.".to_string()
                    } else {
                        format!("Detected {} — searching ROMs…", p.display())
                    };
                    self.profile = Some(p);
                }
                RomRes::Builds(Ok(builds)) => {
                    self.status = if builds.is_empty() {
                        "No downloadable builds found for this device.".to_string()
                    } else {
                        format!(
                            "{} build(s) found. ↑/↓ select, Enter to download.",
                            builds.len()
                        )
                    };
                    self.builds = builds;
                }
                RomRes::Builds(Err(e)) => self.status = format!("ROM lookup failed: {e}"),
                RomRes::Progress(p) => self.download_progress = Some(p),
                RomRes::Downloaded(Ok(path)) => {
                    self.download_progress = None;
                    self.status = "Download verified. Review the flash plan below.".to_string();
                    self.downloaded_path = Some(path.clone());
                    self.build_session(path);
                }
                RomRes::Downloaded(Err(e)) => {
                    self.download_progress = None;
                    self.status = format!("Download failed: {e}");
                }
                RomRes::StepDone { idx, result } => {
                    self.busy = false;
                    if let Some(session) = &mut self.session {
                        session.statuses[idx] = match result {
                            Ok(_) => StepStatus::Done,
                            Err(e) => StepStatus::Failed(e),
                        };
                    }
                    if self
                        .session
                        .as_ref()
                        .map(|s| s.is_complete())
                        .unwrap_or(false)
                    {
                        self.status = "✅ Flash complete — device rebooting.".to_string();
                    }
                }
            }
        }
    }
}

fn worker_loop(req_rx: Receiver<RomReq>, res_tx: Sender<RomRes>) {
    let mut adb = AdbManager::new();
    let fastboot = FastbootManager::new();
    while let Ok(req) = req_rx.recv() {
        match req {
            RomReq::Detect => {
                if let Ok(profile) = adb.detect_device_profile() {
                    let codename = profile.codename.clone();
                    let _ = res_tx.send(RomRes::Profile(profile));
                    let builds = rom::resolve_all(&codename);
                    let _ = res_tx.send(RomRes::Builds(Ok(builds)));
                }
            }
            RomReq::Download { build, dest } => {
                let tx = res_tx.clone();
                let result = rom::download_and_verify(
                    &build.download_url,
                    &dest,
                    build.sha256.as_deref(),
                    |p| {
                        let _ = tx.send(RomRes::Progress(p));
                    },
                );
                let _ = res_tx.send(RomRes::Downloaded(result));
            }
            RomReq::RunStep { idx, step, serial } => {
                let result = rom::run_flash_step(&step, &serial, &mut adb, &fastboot);
                let _ = res_tx.send(RomRes::StepDone { idx, result });
            }
        }
    }
}
