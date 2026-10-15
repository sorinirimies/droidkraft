//! Logs tab: per-device logcat buffer + filters.

use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent, Context, KeyDownEvent};

use droidkraft_core::features::logcat::FilterField;
use droidkraft_core::{LogEntry, LogcatFilter};

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
        let (s1, s2, s3, s4) = (
            serial.to_string(),
            serial.to_string(),
            serial.to_string(),
            serial.to_string(),
        );
        let streaming = d.log_streaming;
        let toolbar = div()
            .flex()
            .flex_wrap()
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
            .child(button(
                "log-mode",
                d.log_filter.mode_label(),
                theme::BG_ELEV,
                theme::TEXT,
                cx,
                move |this, _cx| {
                    if let Some(d) = this.dev(&s4) {
                        d.log_filter.cycle_mode();
                    }
                },
            ));
        let searching = d.log_filter.active_field == FilterField::Search;
        let (q, cur) = (
            d.log_filter.search_query.clone(),
            d.log_filter.search_cursor,
        );
        let focus = self.log_focus.clone();
        let sb = serial.to_string();
        let search_box = div()
            .id("log-search")
            .track_focus(&self.log_focus)
            .flex()
            .items_center()
            .flex_1()
            .min_w(px(120.))
            .px_2()
            .py_1()
            .rounded_md()
            .bg(rgb(theme::BG))
            .border_1()
            .border_color(rgb(if searching {
                theme::ACCENT
            } else {
                theme::BORDER
            }))
            .text_sm()
            .overflow_hidden()
            .on_click(cx.listener({
                let sb = sb.clone();
                move |this, _ev: &ClickEvent, w, cx| {
                    w.focus(&focus);
                    if let Some(d) = this.dev(&sb) {
                        d.log_filter.active_field = FilterField::Search;
                    }
                    cx.notify();
                }
            }))
            .on_key_down(cx.listener({
                let sb = sb.clone();
                move |this, ev: &KeyDownEvent, _w, cx| {
                    if let Some(d) = this.dev(&sb) {
                        let k = &ev.keystroke;
                        let m = &k.modifiers;
                        let char = k.key_char.as_deref();
                        if apply_search_key(
                            &mut d.log_filter,
                            &k.key,
                            char,
                            m.platform || m.control,
                        ) {
                            cx.notify();
                        }
                    }
                }
            }))
            .child(search_text(&q, cur, searching));
        let total = d.log_entries.len();
        let shown_n = d
            .log_entries
            .iter()
            .filter(|e| d.log_filter.matches(e))
            .count();
        let toolbar = toolbar.child(search_box).child(
            div()
                .text_xs()
                .text_color(rgb(theme::TEXT_DIM))
                .child(format!("{shown_n} / {total}")),
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

/// Search text with a caret when the box is active.
fn search_text(q: &str, cursor: usize, active: bool) -> AnyElement {
    if q.is_empty() && !active {
        return div()
            .text_color(rgb(theme::TEXT_FAINT))
            .child("Search logs…  (click, then type)")
            .into_any_element();
    }
    let chars: Vec<char> = q.chars().collect();
    let c = cursor.min(chars.len());
    let (a, b): (String, String) = (chars[..c].iter().collect(), chars[c..].iter().collect());
    div()
        .flex()
        .child(a)
        .children(active.then(|| div().text_color(rgb(theme::ACCENT)).child("│")))
        .child(b)
        .into_any_element()
}

/// Apply a key press to the search field. Returns true when handled.
pub(crate) fn apply_search_key(
    f: &mut LogcatFilter,
    key: &str,
    key_char: Option<&str>,
    cmd: bool,
) -> bool {
    f.active_field = FilterField::Search;
    match key {
        "backspace" => f.delete_char(),
        "delete" => f.delete_char_forward(),
        "left" => f.move_cursor_left(),
        "right" => f.move_cursor_right(),
        "home" => f.search_cursor = 0,
        "end" => f.search_cursor = f.search_query.chars().count(),
        "escape" => {
            f.clear_search();
            f.active_field = FilterField::None;
        }
        "enter" => f.active_field = FilterField::None,
        _ if cmd => return false,
        _ => match key_char {
            Some(c) if !c.chars().any(char::is_control) => {
                c.chars().for_each(|ch| f.insert_char(ch))
            }
            _ => return false,
        },
    }
    true
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_and_editing() {
        let mut f = LogcatFilter::default();
        for c in ["a", "b", "c"] {
            assert!(apply_search_key(&mut f, c, Some(c), false));
        }
        assert_eq!(f.search_query, "abc");
        apply_search_key(&mut f, "left", None, false);
        apply_search_key(&mut f, "backspace", None, false);
        assert_eq!(f.search_query, "ac");
        apply_search_key(&mut f, "home", None, false);
        apply_search_key(&mut f, "x", Some("x"), false);
        assert_eq!(f.search_query, "xac");
        apply_search_key(&mut f, "end", None, false);
        assert_eq!(f.search_cursor, 3);
    }

    #[test]
    fn escape_clears_and_shortcuts_pass_through() {
        let mut f = LogcatFilter::default();
        apply_search_key(&mut f, "a", Some("a"), false);
        assert!(!apply_search_key(&mut f, "q", Some("q"), true));
        assert_eq!(f.search_query, "a");
        apply_search_key(&mut f, "escape", None, false);
        assert!(f.search_query.is_empty());
        assert_eq!(f.active_field, FilterField::None);
    }
}
