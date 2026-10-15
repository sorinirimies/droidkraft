//! Commands tab: one-click catalogue + this device's last output.

use gpui::{div, prelude::*, px, rgb, AnyElement, Context};

use crate::app::DroidGui;
use crate::commands::{by_category, CommandCategory};
use crate::fmt::truncate;
use crate::state::DeviceState;
use crate::theme;
use crate::ui::widgets::{button, section_title};

impl DroidGui {
    pub(crate) fn render_commands(&self, serial: &str, cx: &mut Context<Self>) -> AnyElement {
        let Some(d) = self.fleet.get(serial) else {
            return div().into_any_element();
        };
        let mut root = div()
            .id("cmd-scroll")
            .flex()
            .flex_col()
            .gap_3()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll();
        for cat in CommandCategory::all() {
            let mut row = div().flex().flex_row().flex_wrap().gap_2();
            for cmd in by_category(*cat) {
                let (bg, fg) = if cmd.destructive {
                    (theme::DANGER, 0xffffff)
                } else {
                    (theme::BG_ELEV, theme::TEXT)
                };
                let id: &'static str = cmd.label;
                let (c, s) = (cmd.clone(), serial.to_string());
                row = row.child(button(id, cmd.label, bg, fg, cx, move |this, _cx| {
                    this.run_command(&s, &c)
                }));
            }
            root = root.child(section_title(cat.label())).child(row);
        }
        root.child(render_output(d)).into_any_element()
    }
}

/// This device's last command / fastboot output.
pub fn render_output(d: &DeviceState) -> AnyElement {
    let content: AnyElement = match (&d.busy_label, &d.last_output) {
        (Some(label), _) => div()
            .text_sm()
            .text_color(rgb(theme::WARN))
            .child(format!("Running {label}…"))
            .into_any_element(),
        (None, None) => div()
            .text_sm()
            .text_color(rgb(theme::TEXT_FAINT))
            .child("Command output will appear here.")
            .into_any_element(),
        (None, Some((label, result))) => {
            let (color, text) = match result {
                Ok(out) => (theme::OK, out.clone()),
                Err(e) => (theme::ERR, e.clone()),
            };
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().text_xs().text_color(rgb(color)).child(label.clone()))
                .child(
                    div()
                        .font_family("monospace")
                        .text_xs()
                        .text_color(rgb(theme::TEXT))
                        .child(truncate(&text, 4000)),
                )
                .into_any_element()
        }
    };
    div()
        .id("cmd-output")
        .p_3()
        .h(px(240.))
        .flex_shrink_0()
        .w_full()
        .rounded_md()
        .bg(rgb(theme::BG))
        .border_1()
        .border_color(rgb(theme::BORDER))
        .overflow_y_scroll()
        .child(content)
        .into_any_element()
}
