//! Small reusable gpui building blocks: buttons, cards, gauges, sparklines.

use gpui::{
    div, prelude::*, px, relative, rgb, AnyElement, ClickEvent, Context, Div, ElementId, Hsla,
};

use crate::fmt::{spark_heights, tail};
use crate::theme;

/// Clickable button.
pub fn button<V: 'static>(
    id: impl Into<ElementId>,
    label: impl Into<String>,
    bg: u32,
    fg: u32,
    cx: &mut Context<V>,
    on_click: impl Fn(&mut V, &mut Context<V>) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .max_w_full()
        .min_h(gpui::rems(1.75))
        .px_3()
        .py_2()
        .rounded_md()
        .bg(rgb(bg))
        .text_sm()
        .text_color(rgb(fg))
        .cursor_pointer()
        .border_1()
        .border_color(rgb(theme::BORDER))
        .hover(|s| s.border_color(rgb(theme::ACCENT_DIM)))
        .child(label.into())
        .on_click(cx.listener(move |this, _ev: &ClickEvent, _window, cx| {
            on_click(this, cx);
            cx.notify();
        }))
        .into_any_element()
}

/// Compact button for card action rows.
pub fn small_button<V: 'static>(
    id: impl Into<ElementId>,
    label: impl Into<String>,
    accent: bool,
    cx: &mut Context<V>,
    on_click: impl Fn(&mut V, &mut Context<V>) + 'static,
) -> AnyElement {
    let (bg, fg) = if accent {
        (theme::ACCENT_DIM, 0xffffff)
    } else {
        (theme::BG_PANEL, theme::TEXT_DIM)
    };
    div()
        .id(id)
        .flex()
        .items_center()
        .flex_none()
        .min_h(gpui::rems(1.75))
        .px_2()
        .py_1()
        .rounded_md()
        .bg(rgb(bg))
        .text_xs()
        .text_color(rgb(fg))
        .cursor_pointer()
        .border_1()
        .border_color(rgb(theme::BORDER))
        .hover(|s| s.bg(rgb(theme::BG_HOVER)))
        .child(label.into())
        .on_click(cx.listener(move |this, ev: &ClickEvent, _window, cx| {
            let _ = ev;
            on_click(this, cx);
            cx.notify();
        }))
        .into_any_element()
}

pub fn section_title(text: &str) -> AnyElement {
    div()
        .text_xs()
        .text_color(rgb(theme::TEXT_FAINT))
        .child(text.to_uppercase())
        .into_any_element()
}

pub fn heading(text: impl Into<String>) -> AnyElement {
    div()
        .text_xl()
        .text_color(rgb(theme::TEXT))
        .child(text.into())
        .into_any_element()
}

/// Full-size container for a panel body.
pub fn panel() -> Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .size_full()
        .p_4()
        .overflow_hidden()
}

/// Card surface (elevated, subtle border).
pub fn card() -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .rounded_lg()
        .bg(rgb(theme::BG_ELEV))
        .border_1()
        .border_color(rgb(theme::BORDER))
}

/// Titled card for the statistics dashboard.
pub fn stat_card(title: &str) -> Div {
    card().child(
        div()
            .text_xs()
            .text_color(rgb(theme::TEXT_FAINT))
            .child(title.to_uppercase()),
    )
}

pub fn dot(color: u32) -> AnyElement {
    div()
        .size_2()
        .rounded_full()
        .bg(rgb(color))
        .into_any_element()
}

/// Small coloured status chip.
pub fn chip(label: impl Into<String>, color: u32) -> AnyElement {
    div()
        .px_2()
        .rounded_md()
        .bg(rgb(theme::BG_PANEL))
        .border_1()
        .border_color(rgb(theme::BORDER))
        .max_w(gpui::rems(13.75))
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_xs()
        .text_color(rgb(color))
        .child(label.into())
        .into_any_element()
}

pub fn pill(label: impl Into<String>, value: impl Into<String>, color: u32) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .px_3()
        .py_2()
        .min_w(px(130.))
        .rounded_lg()
        .bg(rgb(theme::BG_ELEV))
        .border_1()
        .border_color(rgb(theme::BORDER))
        .child(
            div()
                .text_xs()
                .text_color(rgb(theme::TEXT_DIM))
                .child(label.into()),
        )
        .child(div().text_lg().text_color(rgb(color)).child(value.into()))
        .into_any_element()
}

/// Horizontal gauge, `pct` in `0..=100`.
pub fn bar(pct: f32, color: u32, height: f32) -> AnyElement {
    let f = (pct / 100.0).clamp(0.0, 1.0);
    div()
        .w_full()
        .h(px(height))
        .rounded_full()
        .bg(rgb(theme::BG))
        .overflow_hidden()
        .child(div().h_full().w(relative(f)).rounded_full().bg(rgb(color)))
        .into_any_element()
}

/// `label ........ value` over a bar.
pub fn gauge(label: &str, value: impl Into<String>, pct: f32, color: u32) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .w_full()
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
                .child(div().text_color(rgb(theme::TEXT)).child(value.into())),
        )
        .child(bar(pct, color, 6.0))
        .into_any_element()
}

/// Key/value row used in info tables.
/// Like [`kv`] but the value wraps (for sentences / help text).
pub fn kv_wrap(key: &str, value: impl Into<String>) -> AnyElement {
    div()
        .flex()
        .gap_2()
        .text_xs()
        .child(
            div()
                .w(gpui::rems(6.))
                .flex_shrink_0()
                .text_color(rgb(theme::TEXT_FAINT))
                .child(key.to_string()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_color(rgb(theme::TEXT))
                .child(value.into()),
        )
        .into_any_element()
}

pub fn kv(key: &str, value: impl Into<String>) -> AnyElement {
    div()
        .flex()
        .gap_2()
        .text_xs()
        .child(
            div()
                .w(gpui::rems(6.))
                .flex_shrink_0()
                .text_color(rgb(theme::TEXT_FAINT))
                .child(key.to_string()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .text_color(rgb(theme::TEXT))
                .child(value.into()),
        )
        .into_any_element()
}

/// Sparkline drawn from div columns (newest sample on the right).
pub fn sparkline(values: &[f64], max: f64, color: u32, height: f32, cols: usize) -> AnyElement {
    let hs = spark_heights(tail(values, cols), max, height);
    let mut row = div().flex().items_end().gap(px(1.)).h(px(height)).w_full();
    // Pad on the left so bars keep a constant width while history fills up.
    for _ in hs.len()..cols {
        row = row.child(
            div()
                .flex_1()
                .h(px(1.))
                .bg(Hsla::from(rgb(theme::TEXT_DIM)).opacity(0.25)),
        );
    }
    let last = hs.len().saturating_sub(1);
    for (i, h) in hs.into_iter().enumerate() {
        // Older samples fade out; the newest is solid.
        let age = (last - i) as f32 / cols.max(1) as f32;
        row = row.child(
            div()
                .flex_1()
                .h(px(h))
                .rounded_sm()
                .bg(Hsla::from(rgb(color)).opacity((0.95 - age * 0.55).max(0.35))),
        );
    }
    row.into_any_element()
}

/// Sparkline inside a tinted track (for the overview charts).
pub fn chart(
    title: &str,
    now: String,
    values: &[f64],
    max: f64,
    color: u32,
    height: f32,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .w_full()
        .child(
            div()
                .flex()
                .justify_between()
                .text_xs()
                .child(
                    div()
                        .text_color(rgb(theme::TEXT_DIM))
                        .child(title.to_string()),
                )
                .child(div().text_color(rgb(color)).child(now)),
        )
        .child(
            div()
                .w_full()
                .p_1()
                .rounded_md()
                .bg(rgb(theme::BG))
                .child(sparkline(values, max, color, height, 60)),
        )
        .into_any_element()
}

/// Phone glyph for a device: outline coloured by connection state, filled
/// bottom-up by battery level, with a status dot in the corner.
pub fn device_icon(color: u32, battery: Option<u8>) -> AnyElement {
    let fill = f32::from(battery.unwrap_or(0).min(100)) / 100.0 * 16.0;
    div()
        .relative()
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .size(gpui::px(24.0))
        .child(
            div()
                .flex()
                .flex_col()
                .justify_end()
                .w(gpui::px(14.0))
                .h(gpui::px(22.0))
                .p(gpui::px(2.0))
                .rounded_sm()
                .border_2()
                .border_color(rgb(color))
                .child(
                    div()
                        .w_full()
                        .h(gpui::px(fill))
                        .rounded_sm()
                        .bg(rgb(color))
                        .opacity(0.55),
                ),
        )
        .child(
            div()
                .absolute()
                .bottom(gpui::px(0.0))
                .right(gpui::px(0.0))
                .size(gpui::px(7.0))
                .rounded_full()
                .border_1()
                .border_color(rgb(theme::BG_PANEL))
                .bg(rgb(color)),
        )
        .into_any_element()
}
