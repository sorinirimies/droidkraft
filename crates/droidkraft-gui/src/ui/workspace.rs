//! Device workspace: header strip, tab bar, and the active tab's body.

use gpui::{div, prelude::*, rgb, AnyElement, ClickEvent, Context};

use crate::app::DroidGui;
use crate::fmt::{battery_color, connection_kind, nonempty, state_hint};
use crate::layout::{Layout, RateStyle, TabStyle};
use crate::state::{Route, Tab};
use crate::theme;
use crate::ui::overview::overview;
use crate::ui::widgets::{chip, dot};

impl DroidGui {
    pub(crate) fn render_workspace(
        &self,
        serial: &str,
        lay: &Layout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(d) = self.fleet.get(serial) else {
            return div().into_any_element();
        };
        let status_color = if d.is_online() {
            theme::OK
        } else if d.connected {
            theme::WARN
        } else {
            theme::TEXT_FAINT
        };

        // ── header strip ──
        let mut chips: Vec<AnyElement> = vec![
            chip(d.adb_state.clone(), status_color),
            chip(connection_kind(serial), theme::TEXT_DIM),
        ];
        if let Some(det) = d.details.as_ref().filter(|x| x.online) {
            chips.push(chip(
                format!("Android {}", nonempty(&det.system.android_release, "?")),
                theme::TEXT_DIM,
            ));
            match det.system.flash_locked {
                Some(true) => chips.push(chip("bootloader locked", theme::WARN)),
                Some(false) => chips.push(chip("bootloader unlocked", theme::OK)),
                None => {}
            }
            chips.push(chip(
                format!("{}%", det.battery.level),
                battery_color(det.battery.level),
            ));
            if let Some(ssid) = &det.network.wifi_ssid {
                chips.push(chip(format!("Wi-Fi {ssid}"), theme::ACCENT));
            }
            let root = d
                .root_status
                .as_ref()
                .map(|r| r.is_rooted)
                .or(det.system.is_rooted);
            match root {
                Some(true) => chips.push(chip("root", theme::OK)),
                Some(false) => chips.push(chip("no root", theme::TEXT_DIM)),
                None => {}
            }
        }
        chips.truncate(lay.header_chips());
        let wide = lay.mode == crate::layout::Mode::Wide;
        let header = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap_3()
            .child(
                div()
                    .id("back")
                    .flex_none()
                    .text_sm()
                    .text_color(rgb(theme::TEXT_DIM))
                    .cursor_pointer()
                    .child("← Devices")
                    .on_click(cx.listener(|this, _ev: &ClickEvent, _w, cx| {
                        this.set_route(Route::Devices);
                        cx.notify();
                    })),
            )
            .child(dot(status_color))
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_xl()
                    .text_color(rgb(theme::TEXT))
                    .child(d.display_name()),
            )
            .children(wide.then(|| {
                div()
                    .text_xs()
                    .text_color(rgb(theme::TEXT_FAINT))
                    .child(serial.to_string())
            }))
            .child(div().flex().flex_row().flex_wrap().gap_1().children(chips));

        // ── tab bar ──
        let mut tabs = div()
            .flex()
            .gap_1()
            .overflow_hidden()
            .justify_between()
            .border_b_1()
            .border_color(rgb(theme::BORDER));
        let full: Vec<&str> = Tab::all().iter().map(|t| t.label()).collect();
        let short: Vec<&str> = Tab::all().iter().map(|t| t.short_label()).collect();
        let init: Vec<&str> = Tab::all().iter().map(|t| t.initial()).collect();
        let bar = lay.tab_bar(&full, &short, &init);
        let mut tab_row = div()
            .id("tab-row")
            .flex()
            .gap_1()
            .min_w_0()
            .overflow_x_scroll();
        for tab in Tab::all() {
            let tab = *tab;
            let active = tab == d.tab;
            let s = serial.to_string();
            tab_row = tab_row.child(
                div()
                    .id(("tab", tab as usize))
                    .flex_none()
                    .min_w(gpui::px(36.))
                    .px(gpui::px(if bar.tabs == TabStyle::Full { 12. } else { 8. }))
                    .py_2()
                    .text_sm()
                    .cursor_pointer()
                    .text_color(rgb(if active {
                        theme::ACCENT
                    } else {
                        theme::TEXT_DIM
                    }))
                    .border_b_2()
                    .border_color(rgb(if active { theme::ACCENT } else { theme::BG }))
                    .hover(|s| s.text_color(rgb(theme::TEXT)))
                    .child(match bar.tabs {
                        TabStyle::Full => tab.label(),
                        TabStyle::Short => tab.short_label(),
                        TabStyle::Initial => tab.initial(),
                    })
                    .on_click(cx.listener(move |this, _ev: &ClickEvent, _w, cx| {
                        this.select_tab(&s, tab, cx);
                        cx.notify();
                    })),
            );
        }
        tabs = tabs.child(tab_row);
        if bar.rate != RateStyle::Hidden {
            tabs = tabs.child(self.rate_control(bar.rate, cx));
        }

        let banner: Option<AnyElement> = if d.is_online() {
            None
        } else {
            Some(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(rgb(theme::BG_ELEV))
                    .border_1()
                    .border_color(rgb(theme::WARN))
                    .text_sm()
                    .text_color(rgb(theme::WARN))
                    .child(
                        state_hint(&d.adb_state)
                            .unwrap_or("Device not ready.")
                            .to_string(),
                    )
                    .into_any_element(),
            )
        };

        let body: AnyElement = match d.tab {
            Tab::Overview => div()
                .id("overview-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .child(overview(d, lay))
                .into_any_element(),
            Tab::Logs => self.render_logs(serial, cx),
            Tab::Commands => self.render_commands(serial, cx),
            Tab::Flash => self.render_flash(serial, cx),
            Tab::Screen => self.render_screen(serial, lay, cx),
        };

        let mut root = div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .p_4()
            .overflow_hidden();
        if !d.connected {
            root = root.opacity(0.7);
        }
        root.child(header)
            .children(banner)
            .child(tabs)
            .child(body)
            .into_any_element()
    }
}

impl DroidGui {
    /// `Graphs  [−] 1.5s [+]` control: lower interval = faster graphs.
    pub(crate) fn rate_control(&self, style: RateStyle, cx: &mut Context<Self>) -> AnyElement {
        let r = self.rate;
        let step = |id: &'static str,
                    label: &'static str,
                    faster: bool,
                    off: bool,
                    cx: &mut Context<Self>| {
            let (bg, fg) = if off {
                (theme::BG, theme::TEXT_DIM)
            } else {
                (theme::BG_ELEV, theme::TEXT)
            };
            crate::ui::widgets::button(id, label, bg, fg, cx, move |this, cx| {
                let r = if faster {
                    this.rate.faster()
                } else {
                    this.rate.slower()
                };
                this.set_rate(r);
                cx.notify();
            })
        };
        div()
            .flex()
            .items_center()
            .gap_1()
            .flex_none()
            .text_xs()
            .text_color(rgb(theme::TEXT_DIM))
            .children((style == RateStyle::Full).then_some("Refresh"))
            .child(step("rate-slower", "−", false, r.is_slowest(), cx))
            .child(
                div()
                    .min_w(gpui::px(40.))
                    .flex()
                    .justify_center()
                    .text_color(rgb(theme::TEXT))
                    .child(r.label()),
            )
            .child(step("rate-faster", "+", true, r.is_fastest(), cx))
            .into_any_element()
    }
}
