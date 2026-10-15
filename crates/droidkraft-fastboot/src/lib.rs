//! # droidkraft-fastboot
//!
//! Fastboot for Rust. The default [`FastbootManager`] talks to the device
//! directly over USB (pure Rust, via [`nusb`](https://docs.rs/nusb)) or TCP,
//! and falls back to the `fastboot` binary only when USB access fails
//! (permissions, drivers) and the binary is installed.
//!
//! * **Commands** – flash, erase, getvar, reboot, lock/unlock, A/B slots, raw
//!   `oem ...` lines. See the convenience methods on [`FastbootManager`] or the
//!   [`FastbootCommand`] enum.
//! * **Large images** – sparse images are split to the device's
//!   `max-download-size`; raw images are converted to sparse when needed.
//! * **Factory images** – [`flash_all`] runs the whole `flash-all.sh` sequence
//!   (bootloader, radio, A/B slots, logical partitions) with progress callbacks.
//! * **TCP** – target `tcp:HOST[:PORT]` (default port 5554) as the serial, e.g.
//!   for Cuttlefish or emulators.
//!
//! ```no_run
//! use droidkraft_fastboot::FastbootManager;
//!
//! # fn main() -> Result<(), droidkraft_fastboot::FastbootError> {
//! let fb = FastbootManager::new();
//! let vars = fb.variables()?;
//! println!("product = {:?}", vars.get("product"));
//!
//! fb.flash("boot", "boot.img")?;
//! fb.reboot()?;
//! # Ok(()) }
//! ```
//!
//! ## Flashing a factory image
//!
//! ```no_run
//! use droidkraft_fastboot::{FactoryOptions, FastbootManager};
//!
//! # fn main() -> Result<(), droidkraft_fastboot::FastbootError> {
//! let fb = FastbootManager::new().with_serial("0A1B2C3D");
//! let opts = FactoryOptions::new().wipe(true);
//! fb.flash_factory("husky-ap1a.zip", &opts, |p| {
//!     println!("[{}/{}] {}", p.step_index, p.total_steps, p.label);
//! })?;
//! # Ok(()) }
//! ```
//!
//! ## Choosing a backend
//!
//! Use [`FastbootManager::with_backend`] or the `DROIDKRAFT_FASTBOOT_BACKEND`
//! environment variable (`auto` | `native` | `binary`).
//!
//! ## Safety
//!
//! [`FastbootCommand::is_destructive`] tells you which commands wipe data or
//! change the lock state; ask the user before running those.

mod binary;
mod factory;
mod native;
mod sparse;
mod tcp;
mod transport;
mod types;
mod usb;

pub use factory::{flash_all, flash_factory_zip, FactoryOptions, FlashProgress};
pub use types::{parse_variables, FastbootCommand, FastbootError, FastbootResult};

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

    // ── convenience API ────────────────────────────────────────────────────

    /// Read one variable (`getvar <name>`), e.g. `product` or `current-slot`.
    pub fn getvar(&self, name: &str) -> FastbootResult<String> {
        let out = self.execute(FastbootCommand::GetVar { name: name.into() })?;
        Ok(parse_variables(&out)
            .remove(name)
            .unwrap_or_else(|| out.trim().to_string()))
    }

    /// All variables (`getvar all`) as a sorted map. Repeated keys (e.g.
    /// `partition-size:boot_a`) keep their full name.
    pub fn variables(&self) -> FastbootResult<std::collections::BTreeMap<String, String>> {
        Ok(parse_variables(&self.execute(FastbootCommand::GetVarAll)?))
    }

    /// Flash an image file to `partition`.
    pub fn flash(
        &self,
        partition: impl Into<String>,
        image: impl AsRef<std::path::Path>,
    ) -> FastbootResult<String> {
        self.execute(FastbootCommand::flash(partition, image))
    }

    /// Erase a partition. ⚠ destroys its contents.
    pub fn erase(&self, partition: impl Into<String>) -> FastbootResult<String> {
        self.execute(FastbootCommand::Erase {
            partition: partition.into(),
        })
    }

    /// Select the active A/B slot (`"a"` or `"b"`).
    pub fn set_active_slot(&self, slot: impl Into<String>) -> FastbootResult<String> {
        self.execute(FastbootCommand::SetActiveSlot { slot: slot.into() })
    }

    /// Unlock the bootloader (`flashing unlock`). ⚠ wipes the device.
    pub fn unlock(&self) -> FastbootResult<String> {
        self.execute(FastbootCommand::OemUnlock)
    }

    /// Re-lock the bootloader (`flashing lock`). ⚠ wipes the device.
    pub fn lock(&self) -> FastbootResult<String> {
        self.execute(FastbootCommand::OemLock)
    }

    /// Factory reset: erase `userdata` and `metadata`. ⚠ all data lost.
    pub fn wipe(&self) -> FastbootResult<String> {
        self.execute(FastbootCommand::WipeData)
    }

    /// Send an arbitrary command line such as `oem device-info`.
    pub fn raw(&self, line: impl Into<String>) -> FastbootResult<String> {
        self.execute(FastbootCommand::Raw(line.into()))
    }

    /// Reboot into the system.
    pub fn reboot(&self) -> FastbootResult<String> {
        self.execute(FastbootCommand::Reboot)
    }

    /// Reboot into the bootloader.
    pub fn reboot_bootloader(&self) -> FastbootResult<String> {
        self.execute(FastbootCommand::RebootBootloader)
    }

    /// Reboot into recovery.
    pub fn reboot_recovery(&self) -> FastbootResult<String> {
        self.execute(FastbootCommand::RebootRecovery)
    }

    /// Reboot into userspace fastboot (fastbootd).
    pub fn reboot_fastbootd(&self) -> FastbootResult<String> {
        self.execute(FastbootCommand::RebootFastbootd)
    }

    /// Flash a factory image (`.zip` or extracted directory) with this
    /// manager's serial. Native backend only; see [`flash_all`].
    pub fn flash_factory(
        &self,
        source: impl AsRef<std::path::Path>,
        options: &FactoryOptions,
        mut progress: impl FnMut(FlashProgress),
    ) -> FastbootResult<String> {
        flash_all(
            source.as_ref(),
            self.serial.as_deref(),
            options,
            &mut progress,
        )
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
    fn builder_sets_fields() {
        let m = FastbootManager::new()
            .with_backend(Backend::Binary)
            .with_serial("tcp:127.0.0.1:5554");
        assert_eq!(m.backend, Some(Backend::Binary));
        assert_eq!(m.serial.as_deref(), Some("tcp:127.0.0.1:5554"));
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
