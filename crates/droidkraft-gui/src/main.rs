//! DroidKraft GUI entry point.

use std::sync::Arc;

use gpui::prelude::*;
use gpui::{px, size, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions};

use droidkraft_gui::cli::{self, Cli};
use droidkraft_gui::mirror_view::{self, Chrome, MirrorView, StandaloneRoot};
use droidkraft_gui::screen::ScreenStream;
use droidkraft_gui::DroidGui;

fn main() {
    let parsed = match cli::parse(std::env::args().skip(1)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}\n\n{}", cli::USAGE);
            std::process::exit(2);
        }
    };
    if parsed == Cli::Help {
        println!("{}", cli::USAGE);
        return;
    }
    Application::new().run(move |cx| {
        if let Cli::Mirror(m) = parsed {
            mirror_view::init(cx);
            let (w, h) = m.size.unwrap_or((400, 860));
            let bounds = Bounds::centered(None, size(px(w as f32), px(h as f32)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(m.title.unwrap_or_else(|| "DroidKraft Mirror".into()).into()),
                        ..Default::default()
                    }),
                    window_min_size: Some(size(px(240.), px(320.))),
                    app_id: Some("droidkraft".into()),
                    ..Default::default()
                },
                move |_, cx| {
                    let serial = m.serial.clone();
                    let mirror = cx.new(|cx| {
                        let mut v =
                            MirrorView::new(Arc::new(ScreenStream::new()), Chrome::Compact, cx);
                        v.set_discover(serial.is_none());
                        v.set_serial(serial);
                        v
                    });
                    cx.new(|cx| StandaloneRoot::new(mirror, cx))
                },
            )
            .expect("failed to open window");
            cx.activate(true);
            return;
        }
        let bounds = Bounds::centered(None, size(px(1100.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(640.), px(420.))),
                // Wayland app_id / X11 WM_CLASS: must match the .desktop file.
                app_id: Some("droidkraft".into()),
                ..Default::default()
            },
            |_, cx| cx.new(DroidGui::new),
        )
        .expect("failed to open window");
        cx.activate(true);
    });
}
