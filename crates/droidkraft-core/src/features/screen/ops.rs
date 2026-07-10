//! Screen operations on [`AdbManager`].

use crate::client::AdbManager;
use crate::error::AdbResult;
use crate::features::screen::types::ScreenResolution;

impl AdbManager {
    /// Take a screenshot and save it to the device at `/sdcard/screenshot.png`.
    pub fn take_screenshot(&mut self) -> AdbResult<String> {
        self.shell_command("screencap -p /sdcard/screenshot.png")
    }

    /// Capture the current screen as raw PNG bytes.
    ///
    /// This streams `screencap -p` directly to stdout (no on-device file),
    /// making it suitable for the GUI's live screen-mirroring loop.
    pub fn capture_frame_png(&mut self) -> AdbResult<Vec<u8>> {
        self.shell_command_raw("screencap -p")
    }

    /// Query the device screen resolution and density.
    pub fn get_screen_resolution(&mut self) -> AdbResult<String> {
        let size = self.shell_command("wm size")?;
        let density = self.shell_command("wm density")?;
        Ok(format!("{}\n{}", size, density))
    }

    /// Query the device screen resolution as a structured value.
    pub fn screen_resolution(&mut self) -> AdbResult<Option<ScreenResolution>> {
        let size = self.shell_command("wm size")?;
        let density = self.shell_command("wm density")?;
        Ok(ScreenResolution::parse(&size, &density))
    }
}
