//! Flash & Root tab: root tools, reboot targets, fastboot, custom-ROM flow.

use gpui::{div, prelude::*, rgb, AnyElement, Context};

use droidkraft_core::features::fastboot::FastbootCommand;
use droidkraft_core::features::flash::RebootTarget;
use droidkraft_core::features::rom::StepStatus;

use crate::app::DroidGui;
use crate::state::DeviceState;
use crate::theme;
use crate::ui::commands::render_output;
use crate::ui::widgets::{button, section_title};

impl DroidGui {
    pub(crate) fn render_flash(&self, serial: &str, cx: &mut Context<Self>) -> AnyElement {
        let Some(d) = self.fleet.get(serial) else {
            return div().into_any_element();
        };
        let root_line: AnyElement = match &d.root_status {
            None => div()
                .text_sm()
                .text_color(rgb(theme::TEXT_DIM))
                .child("Root status unknown — run a check.")
                .into_any_element(),
            Some(rs) => {
                let (color, text) = if rs.is_rooted {
                    (
                        theme::OK,
                        format!(
                            "Rooted via {}{}",
                            rs.method.label(),
                            rs.magisk_version
                                .as_ref()
                                .map(|v| format!(" (Magisk {v})"))
                                .unwrap_or_default()
                        ),
                    )
                } else {
                    (theme::WARN, "Not rooted".to_string())
                };
                div()
                    .text_sm()
                    .text_color(rgb(color))
                    .child(text)
                    .into_any_element()
            }
        };

        let mut reboots = div().flex().flex_row().flex_wrap().gap_2();
        for &target in RebootTarget::all() {
            let id: &'static str = target.label();
            let s = serial.to_string();
            reboots = reboots.child(button(
                id,
                target.label(),
                theme::BG_ELEV,
                theme::TEXT,
                cx,
                move |this, _cx| this.reboot(&s, target),
            ));
        }

        let fastboot_cmds = [
            ("fb-unlock", "Unlock Bootloader", FastbootCommand::OemUnlock),
            ("fb-lock", "Lock Bootloader", FastbootCommand::OemLock),
            ("fb-wipe", "Wipe Data", FastbootCommand::WipeData),
            ("fb-reboot", "Fastboot Reboot", FastbootCommand::Reboot),
            ("fb-getvar", "Get Variables", FastbootCommand::GetVarAll),
            (
                "fb-slot-a",
                "Active Slot A",
                FastbootCommand::SetActiveSlot { slot: "a".into() },
            ),
            (
                "fb-slot-b",
                "Active Slot B",
                FastbootCommand::SetActiveSlot { slot: "b".into() },
            ),
            (
                "fb-bl",
                "Reboot Bootloader",
                FastbootCommand::RebootBootloader,
            ),
            (
                "fb-fbd",
                "Reboot Fastbootd",
                FastbootCommand::RebootFastbootd,
            ),
            ("fb-rec", "Reboot Recovery", FastbootCommand::RebootRecovery),
        ];
        let mut fb_row = div().flex().flex_row().flex_wrap().gap_2();
        for (id, label, cmd) in fastboot_cmds {
            let (bg, fg) = if cmd.is_destructive() {
                (theme::DANGER, 0xffffff)
            } else {
                (theme::BG_ELEV, theme::TEXT)
            };
            let s = serial.to_string();
            fb_row = fb_row.child(button(id, label, bg, fg, cx, move |this, _cx| {
                this.fastboot(&s, cmd.clone())
            }));
        }

        let (s1, s2) = (serial.to_string(), serial.to_string());
        div()
            .id("flash-scroll")
            .flex()
            .flex_col()
            .gap_3()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(button(
                        "root-check",
                        "Check Root",
                        theme::ACCENT_DIM,
                        0xffffff,
                        cx,
                        move |this, _cx| this.detect_root(&s1),
                    ))
                    .child(button(
                        "remount",
                        "Remount RW",
                        theme::BG_ELEV,
                        theme::TEXT,
                        cx,
                        move |this, _cx| this.remount(&s2),
                    ))
                    .child(root_line),
            )
            .child(section_title("Reboot into"))
            .child(reboots)
            .child(section_title("Fastboot (device must be in bootloader)"))
            .child(fb_row)
            .child(section_title("Custom ROM (one-stop flash)"))
            .child(self.render_rom_flash(serial, d, cx))
            .child(render_output(d))
            .into_any_element()
    }

    /// The custom-ROM catalog + download + guided flash UI.
    fn render_rom_flash(
        &self,
        serial: &str,
        d: &DeviceState,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut col = div().flex().flex_col().gap_2();

        let device_line = match &d.profile {
            Some(p) => format!(
                "{}  •  bootloader {}",
                p.display(),
                match p.bootloader_unlocked {
                    Some(true) => "unlocked",
                    Some(false) => "LOCKED",
                    None => "unknown",
                }
            ),
            None => "No device scanned yet.".to_string(),
        };
        let s = serial.to_string();
        col = col.child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(button(
                    "rom-detect",
                    "Find ROMs for this device",
                    theme::ACCENT_DIM,
                    0xffffff,
                    cx,
                    move |this, _cx| this.detect_roms(&s),
                ))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme::TEXT_DIM))
                        .child(device_line),
                ),
        );

        if let Some(s) = &d.rom_status {
            col = col.child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(s.clone()),
            );
        }

        if let Some(p) = &d.download_progress {
            let pct = p
                .fraction()
                .map(|f| format!("{:.0}%", f * 100.0))
                .unwrap_or_else(|| format!("{} bytes", p.downloaded));
            col = col.child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::ACCENT))
                    .child(format!("Downloading… {pct}")),
            );
        }

        if d.flash_session.is_none() {
            for (i, build) in d.rom_builds.iter().take(12).enumerate() {
                let label = format!(
                    "{} {}  •  Android {}  •  {}{}",
                    build.os.label(),
                    build.version,
                    build.android_version,
                    build.build_date.clone().unwrap_or_default(),
                    build
                        .size_bytes
                        .map(|b| format!("  •  {} MB", b / 1_000_000))
                        .unwrap_or_default(),
                );
                let (b, s) = (build.clone(), serial.to_string());
                col = col.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(rgb(theme::BG_ELEV))
                        .child(div().text_sm().text_color(rgb(theme::TEXT)).child(label))
                        .child(button(
                            ("rom-dl", i),
                            "Download",
                            theme::ACCENT_DIM,
                            0xffffff,
                            cx,
                            move |this, _cx| this.download_build(&s, &b),
                        )),
                );
            }
        }

        if let Some(session) = &d.flash_session {
            let (done, total) = session.progress();
            col = col.child(
                div()
                    .text_sm()
                    .text_color(rgb(theme::TEXT))
                    .child(format!("Flash plan — {done}/{total} steps done")),
            );
            for (i, step) in session.plan.steps.iter().enumerate() {
                let (icon, color) = match &session.statuses[i] {
                    StepStatus::Pending => ("○", theme::TEXT_FAINT),
                    StepStatus::Running => ("◐", theme::WARN),
                    StepStatus::Done => ("✔", theme::OK),
                    StepStatus::Failed(_) => ("✗", theme::ERR),
                };
                col = col.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_sm().text_color(rgb(color)).child(icon))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme::TEXT))
                                .child(step.label()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme::TEXT_FAINT))
                                .child(step.description()),
                        ),
                );
            }
            if let Some((_, step)) = session.next_step() {
                let (bg, verb) = if step.requires_confirmation() {
                    (theme::DANGER, "⚠ Confirm & run")
                } else {
                    (theme::ACCENT_DIM, "Run")
                };
                let label = format!("{verb}: {}", step.label());
                let s = serial.to_string();
                col = col.child(button(
                    "rom-flash-next",
                    label,
                    bg,
                    0xffffff,
                    cx,
                    move |this, _cx| this.run_next_flash_step(&s),
                ));
            } else if session.is_complete() {
                col = col.child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme::OK))
                        .child("✅ Flash complete — device rebooting into the new ROM."),
                );
            }
        }
        col.into_any_element()
    }
}
