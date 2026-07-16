//! DroidKraft GUI entry point.

use gpui::prelude::*;
use gpui::{px, size, Application, Bounds, WindowBounds, WindowOptions};

use droidkraft_gui::DroidGui;

fn main() {
    Application::new().run(|cx| {
        let bounds = Bounds::centered(None, size(px(1100.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(DroidGui::new),
        )
        .expect("failed to open window");
        cx.activate(true);
    });
}
