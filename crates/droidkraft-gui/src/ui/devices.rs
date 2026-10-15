//! Devices home: fleet summary + one rich card per device.

use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent, Context};

use crate::app::DroidGui;
use crate::fmt::{
    battery_color, connection_kind, fmt_mib, nonempty, state_hint, temp_color, usage_color,
};
use crate::layout::{Layout, RateStyle};
use crate::state::{DeviceState, Tab};
use crate::theme;
use crate::ui::widgets::{
    bar, button, card, chip, device_icon, heading, pill, section_title, small_button, sparkline,
};

impl DroidGui {
    pub(crate) fn render_devices(&self, lay: &Layout, cx: &mut Context<Self>) -> AnyElement {
        let fleet = &self.fleet;
        let (ram_used, ram_total) = fleet.ram_totals();
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .flex_wrap()
            .gap_2()
            .child(heading("Devices"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(self.rate_control(RateStyle::Full, cx))
                    .child(button(
                        "refresh",
                        "Refresh",
                        theme::BG_ELEV,
                        theme::TEXT,
                        cx,
                        |this, _cx| {
                            this.worker
                                .send(crate::worker::WorkerRequest::RefreshStatus)
                        },
                    )),
            );

        let summary = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap_3()
            .child(pill(
                "Online",
                format!("{} / {}", fleet.online_count(), fleet.len()),
                theme::OK,
            ))
            .child(match fleet.avg_battery() {
                Some(b) => pill("Avg battery", format!("{b}%"), battery_color(b)),
                None => pill("Avg battery", "—", theme::TEXT_DIM),
            })
            .child(pill(
                "RAM in use",
                if ram_total > 0 {
                    format!("{} / {}", fmt_mib(ram_used), fmt_mib(ram_total))
                } else {
                    "—".into()
                },
                theme::TEXT,
            ));

        let body: AnyElement = if fleet.is_empty() {
            crate::mirror_view::empty_state(
                "No device connected",
                "Connect a device over USB with debugging enabled (or `adb connect` over Wi-Fi).",
            )
        } else {
            let mut grid = div().flex().flex_row().flex_wrap().gap_3().items_start();
            let ncols = lay.card_columns();
            let cw = ((lay.content_width() - 32.0 - 12.0 * (ncols as f32 - 1.0)) / ncols as f32)
                .clamp(200.0, 460.0);
            let detail = lay.card_detail();
            for (i, d) in fleet.ordered().into_iter().enumerate() {
                grid = grid.child(self.device_card(d, i, cw, detail, cx));
            }
            div()
                .id("devices-scroll")
                .flex()
                .flex_col()
                .gap_3()
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .child(section_title("Fleet"))
                .child(grid)
                .into_any_element()
        };

        crate::ui::widgets::panel()
            .child(header)
            .child(summary)
            .child(body)
            .into_any_element()
    }

    fn device_card(
        &self,
        d: &DeviceState,
        idx: usize,
        cw: f32,
        detail: u8,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let serial = d.serial.clone();
        let online = d.is_online();
        let status_color = if online {
            theme::OK
        } else if d.connected {
            theme::WARN
        } else {
            theme::TEXT_FAINT
        };

        let mut sub = format!("{} · {}", d.serial, connection_kind(&d.serial));
        if let Some(det) = d.details.as_ref().filter(|x| x.online) {
            sub.push_str(&format!(
                " · Android {} (SDK {})",
                nonempty(&det.system.android_release, "?"),
                det.system.sdk
            ));
        }

        let mut c = card()
            .id(("card", idx))
            .w(px(cw))
            .overflow_hidden()
            .cursor_pointer()
            .hover(|s| s.border_color(rgb(theme::ACCENT_DIM)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(device_icon(status_color, d.battery_pct()))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_color(rgb(theme::TEXT))
                            .child(d.display_name()),
                    )
                    .child(chip(d.adb_state.clone(), status_color)),
            )
            .child(div().text_xs().text_color(rgb(theme::TEXT_DIM)).child(sub));

        match (online, d.details.as_ref().filter(|x| x.online)) {
            (true, Some(det)) => {
                let b = &det.battery;
                let charging = matches!(b.status, droidkraft_core::BatteryStatus::Charging);
                let temp = b
                    .temperature_c
                    .map(|t| (format!("{t:.0}°C"), temp_color(t)));
                let mut brow = div().flex().items_center().gap_2().text_xs().child(
                    div().text_color(rgb(battery_color(b.level))).child(format!(
                        "{}{}%",
                        if charging { "⚡ " } else { "🔋 " },
                        b.level
                    )),
                );
                brow = brow.child(div().flex_1().child(bar(
                    b.level as f32,
                    battery_color(b.level),
                    8.0,
                )));
                if let Some((t, col)) = temp {
                    brow = brow.child(div().text_color(rgb(col)).child(t));
                }
                let cpu = d.history.cpu_pct.last().copied().unwrap_or(0.0) as f32;
                let ram = det.memory.used_pct();
                let spark = detail >= 2;
                let sto = det.storage.used_pct();
                c = c
                    .child(brow)
                    .children((detail >= 1).then(|| {
                        div()
                            .flex()
                            .gap_3()
                            .child(div().flex_1().child(mini("RAM", ram)))
                            .child(div().flex_1().child(mini("CPU", cpu)))
                            .child(div().flex_1().child(mini("Disk", sto)))
                    }))
                    .children(spark.then(|| {
                        div()
                            .flex()
                            .gap_3()
                            .child(div().flex_1().child(sparkline(
                                &d.history.ram_pct.as_vec_f64(),
                                100.0,
                                usage_color(ram),
                                22.0,
                                30,
                            )))
                            .child(div().flex_1().child(sparkline(
                                &d.history.cpu_pct.as_vec_f64(),
                                100.0,
                                usage_color(cpu),
                                22.0,
                                30,
                            )))
                            .child(div().flex_1().child(sparkline(
                                &d.history.battery.as_vec_f64(),
                                100.0,
                                battery_color(b.level),
                                22.0,
                                30,
                            )))
                    }));
            }
            (true, None) => {
                c = c.child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme::TEXT_FAINT))
                        .child("Collecting details…"),
                );
            }
            _ => {
                let hint = state_hint(&d.adb_state).unwrap_or("");
                c = c.child(div().text_xs().text_color(rgb(theme::WARN)).child(hint));
            }
        }

        // Quick actions.
        let act = |tab: Tab, label: &'static str, key: &'static str, cx: &mut Context<Self>| {
            let s = serial.clone();
            small_button((key, idx), label, false, cx, move |this, cx| {
                this.open_device(&s, tab, cx)
            })
        };
        let mut row = div().flex().flex_row().flex_wrap().gap_1();
        row = row
            .child(act(Tab::Overview, "Overview", "qa-ov", cx))
            .child(act(Tab::Logs, "Logs", "qa-logs", cx))
            .child(act(Tab::Commands, "Commands", "qa-cmd", cx))
            .child(act(Tab::Screen, "Mirror", "qa-mirror", cx));
        if online {
            let s = serial.clone();
            row = row.child(small_button(
                ("qa-rec", idx),
                "● Record",
                true,
                cx,
                move |this, cx| {
                    this.open_device(&s, Tab::Screen, cx);
                    this.start_recording(&s);
                },
            ));
        }
        row = row.child(act(Tab::Flash, "Flash", "qa-flash", cx));
        if detail >= 1 {
            c = c.child(row);
        }

        if !d.connected {
            c = c.opacity(0.6);
        }
        c.on_click(cx.listener(move |this, _ev: &ClickEvent, _w, cx| {
            let tab = this
                .fleet
                .get(&serial)
                .map(|d| d.tab)
                .unwrap_or(Tab::Overview);
            this.open_device(&serial, tab, cx);
            cx.notify();
        }))
        .into_any_element()
    }
}

fn mini(label: &str, pct: f32) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .flex()
                .justify_between()
                .text_xs()
                .child(
                    div()
                        .text_color(rgb(theme::TEXT_DIM))
                        .child(label.to_string()),
                )
                .child(
                    div()
                        .text_color(rgb(theme::TEXT))
                        .child(format!("{pct:.0}%")),
                ),
        )
        .child(bar(pct, usage_color(pct), 5.0))
        .into_any_element()
}
