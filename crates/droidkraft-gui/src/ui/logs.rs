//! Logs tab: per-device logcat buffer + filters.

use gpui::{div, prelude::*, px, rgb, AnyElement, Context};

use droidkraft_core::LogEntry;

use crate::app::DroidGui;
use crate::theme;
use crate::ui::widgets::button;

/// Maximum number of visible (filtered) log rows rendered per frame.
const MAX_VISIBLE_LOGS: usize = 500;

impl DroidGui {
    pub(crate) fn render_logs(&self, serial: &str, cx: &mut Context<Self>) -> AnyElement {
        let Some(d) = self.fleet.get(serial) else {
            return div().into_any_element();
        };
        let (s1, s2, s3) = (serial.to_string(), serial.to_string(), serial.to_string());
        let streaming = d.log_streaming;
        let toolbar = div()
            .flex()
            .items_center()
            .gap_2()
            .child(button(
                "log-toggle",
                if streaming { "Stop" } else { "Start" },
                if streaming {
                    theme::DANGER
                } else {
                    theme::ACCENT_DIM
                },
                0xffffff,
                cx,
                move |this, _cx| this.toggle_log_stream(&s1),
            ))
            .child(button(
                "log-clear",
                "Clear",
                theme::BG_ELEV,
                theme::TEXT,
                cx,
                move |this, _cx| {
                    if let Some(d) = this.dev(&s2) {
                        d.log_entries.clear();
                    }
                },
            ))
            .child(button(
                "log-level",
                format!("Level ≥ {}", d.log_filter.min_level.as_char()),
                theme::BG_ELEV,
                theme::TEXT,
                cx,
                move |this, _cx| {
                    if let Some(d) = this.dev(&s3) {
                        d.log_filter.cycle_level();
                    }
                },
            ))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(format!("{} lines", d.log_entries.len())),
            );

        let mut rows: Vec<AnyElement> = d
            .log_entries
            .iter()
            .filter(|e| d.log_filter.matches(e))
            .map(log_row)
            .collect();
        let start = rows.len().saturating_sub(MAX_VISIBLE_LOGS);
        let visible = rows.split_off(start);

        let list = div()
            .id("log-list")
            .flex()
            .flex_col()
            .w_full()
            .flex_1()
            .min_h_0()
            .p_2()
            .rounded_md()
            .bg(rgb(theme::BG))
            .border_1()
            .border_color(rgb(theme::BORDER))
            .overflow_y_scroll()
            .children(visible);

        div()
            .flex()
            .flex_col()
            .gap_2()
            .flex_1()
            .min_h_0()
            .child(toolbar)
            .child(list)
            .into_any_element()
    }
}

fn log_row(entry: &LogEntry) -> AnyElement {
    let mut row = div().flex().flex_row().gap_2().py(px(1.));
    row = row.child(
        div()
            .w(px(14.))
            .text_xs()
            .text_color(theme::level_color(entry.level))
            .child(entry.level.as_char().to_string()),
    );
    if let Some(tag) = &entry.tag {
        row = row.child(
            div()
                .min_w(px(120.))
                .max_w(px(180.))
                .text_xs()
                .text_color(theme::tag_color(tag))
                .overflow_hidden()
                .child(tag.clone()),
        );
    }
    row.child(
        div()
            .flex_grow()
            .font_family("monospace")
            .text_xs()
            .text_color(theme::level_color(entry.level))
            .child(entry.message.clone()),
    )
    .into_any_element()
}
