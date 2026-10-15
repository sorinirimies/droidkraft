//! Left navigation rail: Devices home, one entry per device, Settings.

use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent, Context};

use crate::app::DroidGui;
use crate::fmt::{battery_color, ellipsize};
use crate::layout::Layout;
use crate::state::{Route, Tab};
use crate::theme;
use crate::ui::widgets::{device_icon, section_title};

fn nav_item(active: bool, labels: bool) -> gpui::Div {
    let d = div()
        .flex()
        .flex_shrink_0()
        .items_center()
        .gap_2()
        .py_2()
        .rounded_md()
        .overflow_hidden()
        .bg(rgb(if active {
            theme::BG_ELEV
        } else {
            theme::BG_PANEL
        }))
        .text_color(rgb(if active { theme::TEXT } else { theme::TEXT_DIM }))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme::BG_HOVER)));
    // Icon-only rail: centre the glyph.
    if labels {
        d.px_2()
    } else {
        d.px_1().justify_center()
    }
}

impl DroidGui {
    pub(crate) fn render_nav(
        &self,
        route: &Route,
        lay: &Layout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let labels = lay.nav_labels();
        let name_n = lay.nav_name_chars();
        let serial_n = lay.nav_serial_chars();
        let mut rail = div()
            .flex()
            .flex_col()
            .gap_1()
            .w(px(lay.nav_width()))
            .flex_shrink_0()
            .h_full()
            .overflow_hidden()
            .p_2()
            .bg(rgb(theme::BG_PANEL))
            .border_r_1()
            .border_color(rgb(theme::BORDER))
            .child(
                div()
                    .flex()
                    .justify_center()
                    .px_1()
                    .py_2()
                    .text_lg()
                    .text_color(rgb(theme::ACCENT))
                    .when(labels, |d| d.justify_start().px_2())
                    .child(if labels { "DroidKraft" } else { "DK" }),
            );

        let online = self.fleet.online_count();
        rail = rail.child(
            nav_item(*route == Route::Devices, labels)
                .id("nav-devices")
                .child(
                    div()
                        .flex()
                        .flex_shrink_0()
                        .justify_center()
                        .w(px(24.0))
                        .text_sm()
                        .child("▦"),
                )
                .children(labels.then(|| div().text_sm().flex_1().child("Devices")))
                .children(labels.then(|| {
                    div()
                        .text_xs()
                        .text_color(rgb(theme::TEXT_FAINT))
                        .child(format!("{online}/{}", self.fleet.len()))
                }))
                .on_click(cx.listener(|this, _ev: &ClickEvent, _w, cx| {
                    this.set_route(Route::Devices);
                    cx.notify();
                })),
        );

        if labels {
            rail = rail.child(div().px_2().pt_2().child(section_title("Connected")));
        } else if !self.fleet.is_empty() {
            rail = rail.child(div().mx_2().my_1().h(px(1.0)).bg(rgb(theme::BORDER)));
        }
        if self.fleet.is_empty() && labels {
            rail = rail.child(
                div()
                    .px_2()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_FAINT))
                    .child("No devices"),
            );
        }
        // Device list scrolls when there are more devices than rows.
        let mut list = div()
            .id("nav-list")
            .flex()
            .flex_col()
            .gap_1()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll();
        for (i, d) in self.fleet.ordered().into_iter().enumerate() {
            let serial = d.serial.clone();
            let active = *route == Route::Device(serial.clone());
            let color = if d.is_online() {
                theme::OK
            } else if d.connected {
                theme::WARN
            } else {
                theme::TEXT_FAINT
            };
            let battery = d.battery_pct();
            let mut item = nav_item(active, labels)
                .id(("nav-dev", i))
                .child(device_icon(color, battery));
            if labels {
                item = item.child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .child(
                            div()
                                .text_sm()
                                .whitespace_nowrap()
                                .child(ellipsize(&d.display_name(), name_n)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .whitespace_nowrap()
                                .text_color(rgb(theme::TEXT_FAINT))
                                .child(ellipsize(&d.serial, serial_n)),
                        ),
                );
                if let Some(b) = battery {
                    item = item.child(
                        div()
                            .flex_shrink_0()
                            .text_xs()
                            .text_color(rgb(battery_color(b)))
                            .child(format!("{b}%")),
                    );
                }
            }
            if !d.connected {
                item = item.opacity(0.55);
            }
            list = list.child(
                item.on_click(cx.listener(move |this, _ev: &ClickEvent, _w, cx| {
                    let tab = this
                        .fleet
                        .get(&serial)
                        .map(|d| d.tab)
                        .unwrap_or(Tab::Overview);
                    this.open_device(&serial, tab, cx);
                    cx.notify();
                })),
            );
        }
        rail = rail.child(list);

        rail = rail.child(
            nav_item(*route == Route::Settings, labels)
                .id("nav-settings")
                .child(
                    div()
                        .flex()
                        .flex_shrink_0()
                        .justify_center()
                        .w(px(24.0))
                        .text_sm()
                        .child("⚙"),
                )
                .children(labels.then(|| {
                    div().text_sm().child(if lay.nav_long_settings() {
                        "Settings & Help"
                    } else {
                        "Settings"
                    })
                }))
                .on_click(cx.listener(|this, _ev: &ClickEvent, _w, cx| {
                    this.set_route(Route::Settings);
                    cx.notify();
                })),
        );
        rail.into_any_element()
    }

    pub(crate) fn render_settings(&self) -> AnyElement {
        use crate::ui::widgets::{card, heading, kv, panel};
        panel()
            .child(heading("Settings & Help"))
            .child(
                card()
                    .child(section_title("About"))
                    .child(kv("Version", env!("CARGO_PKG_VERSION")))
                    .child(kv(
                        "Polling",
                        "all devices every ~3 s, open workspace every ~1.5 s",
                    )),
            )
            .child(
                card()
                    .child(section_title("Shortcuts"))
                    .child(kv("⌘/Ctrl + D", "Detach / attach the open device's mirror"))
                    .child(kv("⌘/Ctrl + W", "Close the detached mirror window")),
            )
            .child(
                card()
                    .child(section_title("Tips"))
                    .child(kv(
                        "Devices",
                        "Click a card or one of its quick actions to open its workspace.",
                    ))
                    .child(kv(
                        "Recording",
                        "Screen tab → Record; files go to the chosen output folder.",
                    ))
                    .child(kv(
                        "Standalone",
                        "`droidkraft --mirror [--serial S]` opens only a mirror window.",
                    )),
            )
            .into_any_element()
    }
}
