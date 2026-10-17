//! Screen tab: per-device live mirror (detachable) + recording & screenshots.

use std::path::{Path, PathBuf};
use std::process::Command;

use gpui::{div, prelude::*, rgb, AnyElement, Context};

use droidkraft_mirror::RecordStatus;

use crate::app::DroidGui;
use crate::fmt::{ellipsize, fmt_elapsed};
use crate::layout::Layout;
use crate::mirror_view::empty_state;
use crate::state::{
    next_bitrate, next_dir, next_max_height, output_dir_choices, CaptureKind, DeviceState,
};
use crate::theme;
use crate::ui::widgets::{button, card, kv, section_title, small_button};

/// Program + args that reveal `path` in the platform file manager.
pub fn reveal_command(path: &Path) -> (String, Vec<String>) {
    let p = path.to_string_lossy().to_string();
    if cfg!(target_os = "macos") {
        ("open".into(), vec!["-R".into(), p])
    } else if cfg!(target_os = "windows") {
        ("explorer".into(), vec![format!("/select,{p}")])
    } else {
        let dir = path
            .parent()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or(p);
        ("xdg-open".into(), vec![dir])
    }
}

/// Program + args that open `path` with its default application.
pub fn open_command(path: &Path) -> (String, Vec<String>) {
    let p = path.to_string_lossy().to_string();
    if cfg!(target_os = "macos") {
        ("open".into(), vec![p])
    } else if cfg!(target_os = "windows") {
        ("explorer".into(), vec![p])
    } else {
        ("xdg-open".into(), vec![p])
    }
}

fn spawn_detached((prog, args): (String, Vec<String>)) {
    let _ = Command::new(prog).args(args).spawn();
}

impl DroidGui {
    pub(crate) fn render_screen(
        &self,
        serial: &str,
        lay: &Layout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(d) = self.fleet.get(serial) else {
            return div().into_any_element();
        };
        let stack = lay.stack_panes();
        let detached = d.detached.is_some();
        // Width left for the mirror next to the (side-by-side) recording panel.
        let mirror_w = lay.content_width() - 32.0 - if stack { 0.0 } else { 292.0 };
        if let Some(m) = &d.mirror {
            m.update(cx, |v, _| v.set_avail_width(mirror_w));
        }

        let mirror_area: AnyElement = if detached {
            let (s1, s2) = (serial.to_string(), serial.to_string());
            div()
                .flex()
                .flex_col()
                .flex_1()
                .items_center()
                .justify_center()
                .gap_3()
                .rounded_lg()
                .bg(rgb(theme::BG_PANEL))
                .border_1()
                .border_color(rgb(theme::BORDER))
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(theme::TEXT_DIM))
                        .child("Mirror detached to its own window"),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(button(
                            "attach",
                            "⇲ Attach",
                            theme::ACCENT_DIM,
                            0xffffff,
                            cx,
                            move |t, cx| t.attach(&s1, cx),
                        ))
                        .child(button(
                            "focus-window",
                            "Focus window",
                            theme::BG_ELEV,
                            theme::TEXT,
                            cx,
                            move |t, cx| {
                                if let Some(h) = t.fleet.get(&s2).and_then(|d| d.detached) {
                                    let _ = h.update(cx, |_, w, _| w.activate_window());
                                }
                            },
                        )),
                )
                .into_any_element()
        } else if let Some(m) = &d.mirror {
            let s = serial.to_string();
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap_2()
                .child(div().flex().justify_end().child(button(
                    "detach",
                    "⇱ Detach  ⌘D",
                    theme::ACCENT_DIM,
                    0xffffff,
                    cx,
                    move |t, cx| t.detach(&s, cx),
                )))
                .child(div().flex().flex_1().min_h_0().child(m.clone()))
                .into_any_element()
        } else {
            empty_state("Starting mirror…", "Preparing the live screen view.")
        };

        let mut root = div().flex().gap_3().flex_1().min_h_0();
        root = if stack {
            root.flex_col()
        } else {
            root.flex_row()
        };
        root.child(mirror_area)
            .child(self.render_recording(serial, d, stack, lay.height < 560.0, cx))
            .into_any_element()
    }

    fn render_recording(
        &self,
        serial: &str,
        d: &DeviceState,
        stack: bool,
        lay_short: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let status = d.recorder.status();
        let recording = matches!(status, RecordStatus::Recording { .. });
        let finishing = matches!(status, RecordStatus::Finishing);

        let state_row: AnyElement = if recording {
            let el = d.recorder.elapsed().map(fmt_elapsed).unwrap_or_default();
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().size_3().rounded_full().bg(rgb(theme::ERR)))
                .child(div().text_sm().text_color(rgb(theme::ERR)).child("REC"))
                .child(div().text_lg().text_color(rgb(theme::TEXT)).child(el))
                .into_any_element()
        } else if finishing {
            div()
                .text_sm()
                .text_color(rgb(theme::WARN))
                .child("Finishing — pulling file…")
                .into_any_element()
        } else {
            div()
                .text_sm()
                .text_color(rgb(theme::TEXT_DIM))
                .child("Not recording")
                .into_any_element()
        };

        let s = serial.to_string();
        let rec_btn = if recording || finishing {
            button(
                "rec-stop",
                "■ Stop",
                theme::DANGER,
                0xffffff,
                cx,
                move |t, _| t.stop_recording(&s),
            )
        } else {
            button(
                "rec-start",
                "● Record",
                theme::ACCENT_DIM,
                0xffffff,
                cx,
                move |t, _| t.start_recording(&s),
            )
        };
        let s = serial.to_string();
        let shot_btn = button(
            "shot",
            "📷 Screenshot",
            theme::BG_ELEV,
            theme::TEXT,
            cx,
            move |t, _| t.take_screenshot(&s),
        );

        let busy = recording || finishing;
        let (s1, s2, s3, s4) = (
            serial.to_string(),
            serial.to_string(),
            serial.to_string(),
            serial.to_string(),
        );
        let o = &d.rec_opts;
        let opts = div()
            .flex()
            .flex_col()
            .gap_1()
            .child(small_button(
                "opt-rate",
                format!("Bit-rate: {} Mbps", o.bit_rate_mbps),
                false,
                cx,
                move |t, _| {
                    if let Some(d) = t.dev(&s1).filter(|d| !d.recorder.is_recording()) {
                        d.rec_opts.bit_rate_mbps = next_bitrate(d.rec_opts.bit_rate_mbps);
                    }
                },
            ))
            .child(small_button(
                "opt-height",
                format!(
                    "Max height: {}",
                    o.max_height
                        .map(|h| format!("{h}p"))
                        .unwrap_or_else(|| "native".into())
                ),
                false,
                cx,
                move |t, _| {
                    if let Some(d) = t.dev(&s2).filter(|d| !d.recorder.is_recording()) {
                        d.rec_opts.max_height = next_max_height(d.rec_opts.max_height);
                    }
                },
            ))
            .child(small_button(
                "opt-touches",
                format!(
                    "Show touches: {}",
                    if o.show_touches { "on" } else { "off" }
                ),
                o.show_touches,
                cx,
                move |t, _| {
                    if let Some(d) = t.dev(&s3).filter(|d| !d.recorder.is_recording()) {
                        d.rec_opts.show_touches = !d.rec_opts.show_touches;
                    }
                },
            ))
            .child(small_button(
                "opt-dir",
                "Change folder",
                false,
                cx,
                move |t, _| {
                    if let Some(d) = t.dev(&s4).filter(|d| !d.recorder.is_recording()) {
                        d.out_dir = next_dir(&d.out_dir, &output_dir_choices());
                    }
                },
            ));

        let mut files = div().flex().flex_col().gap_1();
        if d.captures.is_empty() {
            files = files.child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_FAINT))
                    .child("No captures yet."),
            );
        }
        for (i, c) in d.captures.iter().rev().take(8).enumerate() {
            let name = c
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let (p1, p2): (PathBuf, PathBuf) = (c.path.clone(), c.path.clone());
            files = files.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_xs().text_color(rgb(theme::TEXT)).child(format!(
                        "{} {}",
                        if c.kind == CaptureKind::Recording {
                            "🎞"
                        } else {
                            "🖼"
                        },
                        ellipsize(&name, 34)
                    )))
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(small_button(
                                ("reveal", i),
                                "Reveal",
                                false,
                                cx,
                                move |_, _| spawn_detached(reveal_command(&p1)),
                            ))
                            .child(small_button(("open", i), "Open", false, cx, move |_, _| {
                                spawn_detached(open_command(&p2))
                            })),
                    ),
            );
        }

        let mut panel = card().id("rec-panel").flex_shrink_0().overflow_y_scroll();
        panel = if stack {
            panel
                .w_full()
                .max_h(gpui::rems(if lay_short { 8. } else { 9.4 }))
        } else {
            panel.w(gpui::rems(17.5))
        };
        panel = panel
            .child(section_title("Recording"))
            .child(state_row)
            .child(div().flex().gap_2().child(rec_btn).child(shot_btn))
            .child(section_title(if busy {
                "Options (locked while recording)"
            } else {
                "Options"
            }))
            .child(opts)
            .child(kv("Folder", d.out_dir.display().to_string()))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_FAINT))
                    .child("Works with or without the live mirror."),
            );
        if let Some(msg) = &d.capture_status {
            panel = panel.child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(msg.clone()),
            );
        }
        if let RecordStatus::Failed(e) = &status {
            panel = panel.child(div().text_xs().text_color(rgb(theme::ERR)).child(e.clone()));
        }
        panel
            .child(section_title("Captures"))
            .child(files)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_and_open_commands() {
        let p = Path::new("/tmp/a b/x.mp4");
        let (prog, args) = reveal_command(p);
        assert!(!prog.is_empty() && !args.is_empty());
        let (prog, args) = open_command(p);
        assert!(!prog.is_empty());
        assert!(args.iter().any(|a| a.contains("x.mp4")));
        if cfg!(target_os = "macos") {
            assert_eq!(reveal_command(p).1[0], "-R");
        }
    }
}
