//! The window frame, drawn by the app when the compositor does not draw one.
//!
//! gpui asks the compositor for server-side decorations. KDE, Sway, Hyprland and the
//! X11 window managers draw them; GNOME on Wayland does not, and gpui then hands the
//! window to the app undecorated: no title bar, no close button, no way to move or
//! resize it. [`frame`] draws all of that, but only when
//! [`Window::window_decorations`] says the app must, so macOS, Windows, X11 and the
//! decorating Wayland compositors keep their native frames.

use gpui::{
    AnyElement, Bounds, CursorStyle, Decorations, HitboxBehavior, MouseButton, Pixels, Point,
    ResizeEdge, SharedString, Size, Tiling, TitlebarOptions, Window, WindowBounds, WindowOptions,
    canvas, div, point, prelude::*, px, rgb,
};

use crate::theme;

/// Wayland `app_id` and X11 `WM_CLASS` of every DroidKraft window. Desktops match it
/// against the `.desktop` file's name and `StartupWMClass` for the icon and grouping.
pub const APP_ID: &str = "io.github.sorinirimies.DroidKraft";

/// Width of the border that takes resize drags when the app draws the frame.
pub const RESIZE_INSET: f32 = 6.0;

/// Height of the drawn title bar.
pub const TITLE_BAR: f32 = 32.0;

/// Options shared by every DroidKraft window: bounds, title, minimum size and [`APP_ID`].
#[must_use]
pub fn options(bounds: Bounds<Pixels>, title: &str, min: Size<Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some(SharedString::from(title.to_string())),
            ..Default::default()
        }),
        window_min_size: Some(min),
        app_id: Some(APP_ID.into()),
        ..Default::default()
    }
}

/// The resize edge under `pos` in a window of `size` whose frame border is `inset`
/// wide, or `None` inside the window. Tiled edges sit against the screen or a
/// neighbour, so they never resize.
#[must_use]
pub fn resize_edge(
    pos: Point<Pixels>,
    inset: Pixels,
    size: Size<Pixels>,
    tiling: Tiling,
) -> Option<ResizeEdge> {
    let top = !tiling.top && pos.y < inset;
    let bottom = !tiling.bottom && pos.y > size.height - inset;
    let left = !tiling.left && pos.x < inset;
    let right = !tiling.right && pos.x > size.width - inset;
    match (top, bottom, left, right) {
        (true, _, true, _) => Some(ResizeEdge::TopLeft),
        (true, _, _, true) => Some(ResizeEdge::TopRight),
        (_, true, true, _) => Some(ResizeEdge::BottomLeft),
        (_, true, _, true) => Some(ResizeEdge::BottomRight),
        (true, ..) => Some(ResizeEdge::Top),
        (_, true, ..) => Some(ResizeEdge::Bottom),
        (_, _, true, _) => Some(ResizeEdge::Left),
        (.., true) => Some(ResizeEdge::Right),
        _ => None,
    }
}

/// The cursor shown over a resize edge.
#[must_use]
pub fn resize_cursor(edge: ResizeEdge) -> CursorStyle {
    match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    }
}

/// `content` inside the window frame: unchanged when the platform draws the frame,
/// else wrapped in a title bar (move, double-click to maximize, right-click for the
/// window menu, minimize / maximize / close) and a resize border.
pub fn frame(window: &mut Window, title: SharedString, content: impl IntoElement) -> AnyElement {
    let Decorations::Client { tiling } = window.window_decorations() else {
        return content.into_any_element();
    };
    let inset = px(RESIZE_INSET);
    window.set_client_inset(inset);
    div()
        .id("frame")
        .size_full()
        .flex()
        .flex_col()
        .bg(rgb(theme::BG))
        .when(!tiling.top, |d| d.pt(inset))
        .when(!tiling.bottom, |d| d.pb(inset))
        .when(!tiling.left, |d| d.pl(inset))
        .when(!tiling.right, |d| d.pr(inset))
        .child(
            canvas(
                |_bounds, window, _cx| {
                    let size = window.window_bounds().get_bounds().size;
                    window.insert_hitbox(
                        Bounds::new(point(px(0.0), px(0.0)), size),
                        HitboxBehavior::Normal,
                    )
                },
                move |_bounds, hitbox, window, _cx| {
                    let size = window.window_bounds().get_bounds().size;
                    if let Some(edge) = resize_edge(window.mouse_position(), inset, size, tiling) {
                        window.set_cursor_style(resize_cursor(edge), &hitbox);
                    }
                },
            )
            .size_full()
            .absolute(),
        )
        .on_mouse_move(|_, window, _| window.refresh())
        .on_mouse_down(MouseButton::Left, move |e, window, _| {
            let size = window.window_bounds().get_bounds().size;
            if let Some(edge) = resize_edge(e.position, inset, size, tiling) {
                window.start_window_resize(edge);
            }
        })
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .overflow_hidden()
                .border_color(rgb(theme::BORDER))
                .when(!tiling.is_tiled(), |d| d.border_1().rounded_md())
                .child(title_bar(window, title))
                .child(div().flex_1().min_h_0().w_full().child(content)),
        )
        .into_any_element()
}

/// The drawn title bar: title on the left, the controls the platform offers on the right.
fn title_bar(window: &Window, title: SharedString) -> impl IntoElement {
    let controls = window.window_controls();
    let maximized = window.is_maximized();
    div()
        .id("frame-title")
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .h(px(TITLE_BAR))
        .pl_3()
        .bg(rgb(theme::BG_PANEL))
        .border_b_1()
        .border_color(rgb(theme::BORDER))
        .on_mouse_down(MouseButton::Left, |e, window, _| {
            if e.click_count == 2 {
                window.zoom_window();
            } else {
                window.start_window_move();
            }
        })
        .on_mouse_down(MouseButton::Right, move |e, window, _| {
            if controls.window_menu {
                window.show_window_menu(e.position);
            }
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .text_size(px(13.0))
                .text_color(rgb(theme::TEXT_DIM))
                .child(title),
        )
        .when(controls.minimize, |d| {
            d.child(control("frame-min", "–", theme::BG_HOVER, |w| {
                w.minimize_window();
            }))
        })
        .when(controls.maximize, |d| {
            let glyph = if maximized { "❐" } else { "□" };
            d.child(control("frame-max", glyph, theme::BG_HOVER, |w| {
                w.zoom_window();
            }))
        })
        .child(control("frame-close", "×", theme::DANGER, |w| {
            w.remove_window();
        }))
}

/// One title-bar button. It swallows the mouse-down so the bar does not start a move.
fn control(
    id: &'static str,
    glyph: &'static str,
    hover: u32,
    act: impl Fn(&mut Window) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(px(44.0))
        .h_full()
        .text_size(px(14.0))
        .text_color(rgb(theme::TEXT))
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(hover)))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |_, window, _| act(window))
        .child(glyph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    const FREE: Tiling = Tiling {
        top: false,
        left: false,
        right: false,
        bottom: false,
    };

    fn edge(x: f32, y: f32, tiling: Tiling) -> Option<ResizeEdge> {
        resize_edge(
            point(px(x), px(y)),
            px(RESIZE_INSET),
            size(px(800.0), px(600.0)),
            tiling,
        )
    }

    #[test]
    fn inside_the_window_is_no_edge() {
        assert_eq!(edge(400.0, 300.0, FREE), None);
        assert_eq!(edge(RESIZE_INSET, RESIZE_INSET, FREE), None);
    }

    #[test]
    fn sides_and_corners_resize() {
        assert_eq!(edge(400.0, 1.0, FREE), Some(ResizeEdge::Top));
        assert_eq!(edge(400.0, 599.0, FREE), Some(ResizeEdge::Bottom));
        assert_eq!(edge(1.0, 300.0, FREE), Some(ResizeEdge::Left));
        assert_eq!(edge(799.0, 300.0, FREE), Some(ResizeEdge::Right));
        assert_eq!(edge(1.0, 1.0, FREE), Some(ResizeEdge::TopLeft));
        assert_eq!(edge(799.0, 1.0, FREE), Some(ResizeEdge::TopRight));
        assert_eq!(edge(1.0, 599.0, FREE), Some(ResizeEdge::BottomLeft));
        assert_eq!(edge(799.0, 599.0, FREE), Some(ResizeEdge::BottomRight));
    }

    #[test]
    fn tiled_edges_never_resize() {
        let left = Tiling { left: true, ..FREE };
        assert_eq!(edge(1.0, 300.0, left), None);
        assert_eq!(edge(1.0, 1.0, left), Some(ResizeEdge::Top));
        let all = Tiling {
            top: true,
            left: true,
            right: true,
            bottom: true,
        };
        assert_eq!(edge(1.0, 1.0, all), None);
        assert_eq!(edge(799.0, 599.0, all), None);
    }

    #[test]
    fn every_edge_has_a_resize_cursor() {
        assert_eq!(resize_cursor(ResizeEdge::Top), CursorStyle::ResizeUpDown);
        assert_eq!(
            resize_cursor(ResizeEdge::Right),
            CursorStyle::ResizeLeftRight
        );
        assert_eq!(
            resize_cursor(ResizeEdge::BottomRight),
            CursorStyle::ResizeUpLeftDownRight
        );
        assert_eq!(
            resize_cursor(ResizeEdge::BottomLeft),
            CursorStyle::ResizeUpRightDownLeft
        );
    }

    #[test]
    fn app_id_matches_the_desktop_entry() {
        let desktop =
            include_str!("../../../packaging/linux/io.github.sorinirimies.DroidKraft.desktop");
        assert!(
            desktop
                .lines()
                .any(|l| l == format!("StartupWMClass={APP_ID}")),
            "StartupWMClass must equal APP_ID"
        );
        let opts = options(
            Bounds::new(point(px(0.0), px(0.0)), size(px(10.0), px(10.0))),
            "t",
            size(px(1.0), px(1.0)),
        );
        assert_eq!(opts.app_id.as_deref(), Some(APP_ID));
    }
}
