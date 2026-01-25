use gpui::{prelude::*, *};

// Declare modules
mod app;
mod config;
mod handlers;
mod rendering;
mod library;
mod models;
mod ui;
mod chart;

// Import rendering utilities and app types
use rendering::calculate_column_widths;
use rendering::get_message_strings;
use app::CanViewApp;

// Re-export common types from models for use in other modules
pub use models::{ChannelType, ChannelMapping, AppConfig};

fn main() {
    env_logger::init();

    let app = Application::new();
    app.run(move |cx| {
        // This must be called before using any GPUI Component features
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::new(px(200.0), px(150.0)),
                    size: gpui::Size {
                        width: px(1600.0),
                        height: px(1000.0),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("CANVIEW - Bus Data Analyzer".into()),
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                kind: gpui::WindowKind::Normal,
                ..Default::default()
            };
            cx.open_window(options, |window, cx| {
                let view = cx.new(|_cx| CanViewApp::new());
                // This first level on the window should be a Root for gpui-component
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            })?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
