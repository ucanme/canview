#![recursion_limit = "256"]

use gpui::{prelude::*, *};

// Declare modules
mod app;
mod config;
mod domain;
mod handlers;
mod library;
mod models;
mod platform;
mod rendering;
pub mod server;
mod ui;

// Import rendering utilities and app types
use app::CanViewApp;

// Re-export common types from models for use in other modules
pub use models::{AppConfig, ChannelMapping, ChannelType};

// Shared stash of paths dropped on the Dock icon while no window was
// available to dispatch them. Drained by the render tick via
// `crate::handlers::drag_drop::drain_dock_drop_queue` (registered below
// as an cx observer on each new CanViewApp). The callback signature for
// `Application::on_open_urls` is `FnMut(Vec<String>)` with no cx arg,
// so we can't dispatch directly — stash and drain instead.
static DOCK_DROP_QUEUE: std::sync::Mutex<Vec<std::path::PathBuf>> = std::sync::Mutex::new(Vec::new());

/// Parse `file://` URL strings to PathBuf. Strips `file://localhost/` or
/// `file:///` prefix and percent-decodes the rest. Non-`file://` schemes
/// are skipped (silently).
fn parse_file_urls(urls: Vec<String>) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    for u in urls {
        let u = u.strip_prefix("file://").unwrap_or(&u);
        // After stripping `file://`, the remainder is `localhost/path` or `/path`.
        let path_str = u.strip_prefix("localhost/").map(|p| format!("/{}", p))
            .unwrap_or_else(|| u.to_string());
        match urlencoding::decode(&path_str) {
            Ok(cow) => paths.push(std::path::PathBuf::from(cow.into_owned())),
            Err(_) => eprintln!("⚠️ failed to decode dropped URL: {}", u),
        }
    }
    paths
}

fn main() {
    // Capture panics with a full backtrace so crashes can be diagnosed.
    // Set RUST_BACKTRACE=1 (or =full) in the environment for symbol names.
    std::panic::set_hook(Box::new(|info| {
        eprintln!("PANIC: {info}");
        eprintln!("{}", std::backtrace::Backtrace::capture());
    }));

    // Initialize logger with debug level enabled
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let app = Application::new();
    // macOS Dock-icon drop → on_open_urls fires with file:// URLs.
    // The callback has no cx, so we stash paths to a global queue. The
    // first CanViewApp's render tick drains it via drain_dock_drop_queue.
    app.on_open_urls(move |urls: Vec<String>| {
        let paths = parse_file_urls(urls);
        if paths.is_empty() { return; }
        if let Ok(mut q) = DOCK_DROP_QUEUE.lock() {
            q.extend(paths);
        }
    });
    // When the user clicks the Dock icon while the app is already running
    // but no windows are visible (minimized, hidden, or on another space),
    // GPUI fires on_reopen. Without a handler, the app appears "stuck open
    // with no window". We open a fresh main window here so the user always
    // gets a visible window on Dock click.
    app.on_reopen(move |cx| {
        eprintln!("🔄 App re-opened (Dock click) — opening a new window");
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
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            })?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
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
