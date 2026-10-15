//! # droidkraft-fastboot
//!
//! Fastboot for DroidKraft. The default [`FastbootManager`] drives the device
//! over USB directly (pure Rust, via `nusb`) and falls back to the `fastboot`
//! binary for anything the native backend can't do yet (nothing for now: wipe and large-image sparse splitting are native) or when USB access fails (permissions,
//! drivers).
//!
//! Pick a backend with [`Backend`] or the `DROIDKRAFT_FASTBOOT_BACKEND`
//! environment variable (`auto` | `native` | `binary`).

mod binary;
mod factory;
mod native;
mod sparse;
mod tcp;
mod transport;
mod types;
mod usb;

pub use factory::{flash_all, flash_factory_zip, FactoryOptions, FlashProgress};
pub use types::{FastbootCommand, FastbootError, FastbootResult};

/// Which implementation executes commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Backend {
    /// Native USB first, fall back to the binary (default).
    #[default]
    Auto,
    /// Native USB only.
    Native,
    /// `fastboot` binary only.
    Binary,
}

impl Backend {
    fn from_env() -> Self {
        match std::env::var("DROIDKRAFT_FASTBOOT_BACKEND").as_deref() {
            Ok("native") => Self::Native,
            Ok("binary") => Self::Binary,
            _ => Self::Auto,
        }
    }
}

/// Executes fastboot operations.
#[derive(Debug, Clone, Default)]
pub struct FastbootManager {
    backend: Option<Backend>,
    serial: Option<String>,
}

impl FastbootManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Force a specific backend (overrides the environment variable).
    pub fn with_backend(mut self, backend: Backend) -> Self {
        self.backend = Some(backend);
        self
    }

    /// Target a specific device when several are attached.
    pub fn with_serial(mut self, serial: impl Into<String>) -> Self {
        self.serial = Some(serial.into());
        self
    }

    /// Always `true`: the native backend needs no external tool.
    pub fn is_available() -> bool {
        true
    }

    /// Whether the external `fastboot` binary is in `PATH` (needed only for
    /// factory-image flashing and the fallback paths).
    pub fn binary_available() -> bool {
        binary::available()
    }

    /// Serial numbers of devices currently in fastboot mode (native USB scan).
    pub fn devices() -> Vec<String> {
        usb::list().unwrap_or_default()
    }

    /// Block until a fastboot device (matching the configured serial) can be
    /// opened, polling every 500 ms. Returns [`FastbootError::NoDevice`] on timeout.
    pub fn wait_for_device(&self, timeout: std::time::Duration) -> FastbootResult<()> {
        let start = std::time::Instant::now();
        loop {
            match native::open(self.serial.as_deref()) {
                Ok(_) => return Ok(()),
                Err(e) if start.elapsed() >= timeout => return Err(e),
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(500)),
            }
        }
    }

    /// Execute a command, returning its output text.
    pub fn execute(&self, command: FastbootCommand) -> FastbootResult<String> {
        match self.backend.unwrap_or_else(Backend::from_env) {
            Backend::Binary => binary::execute(&command),
            Backend::Native => match native::execute(&command, self.serial.as_deref()) {
                native::Outcome::Done(r) => r,
                native::Outcome::Fallback(e) => Err(e),
            },
            Backend::Auto => match native::execute(&command, self.serial.as_deref()) {
                native::Outcome::Done(r) => r,
                native::Outcome::Fallback(native_err) => match binary::execute(&command) {
                    // No binary: the native error is the more useful one.
                    Err(FastbootError::NotInstalled) => Err(native_err),
                    r => r,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Needs a device in fastboot mode: `cargo test -p droidkraft-fastboot -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn live_native_getvar() {
        let m = FastbootManager::new().with_backend(Backend::Native);
        let out = m.execute(FastbootCommand::GetVarAll).expect("getvar all");
        println!("{out}");
        assert!(out.contains("product") || out.contains("version"));
    }

    #[test]
    fn missing_device_is_reported() {
        if FastbootManager::devices().is_empty() {
            let r = FastbootManager::new()
                .with_backend(Backend::Native)
                .execute(FastbootCommand::Reboot);
            assert!(matches!(r, Err(FastbootError::NoDevice)));
        }
    }
}
