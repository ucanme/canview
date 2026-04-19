//! Window management controller
//!
//! Handles business logic for window management operations.

use crate::app::CanViewApp;
use gpui::{px, Context, Bounds, Point, Size, Window};

/// Toggle window maximize state
pub fn toggle_maximize(app: &mut CanViewApp, window: &mut Window, cx: &mut Context<CanViewApp>) {
    // Initialize display bounds on first use
    if app.display_bounds.is_none() {
        let displays = cx.displays();
        if let Some(display) = displays.first() {
            let display_bounds = display.bounds();
            // Leave a small margin for the task bar and dock
            let margin = px(4.0);
            app.display_bounds = Some(Bounds {
                origin: Point::new(margin, margin),
                size: Size {
                    width: display_bounds.size.width - margin * 2.0,
                    height: display_bounds.size.height - margin * 2.0,
                },
            });
        }
    }

    if app.is_maximized {
        // Restore to normal size
        if let Some(saved_bounds) = app.saved_window_bounds {
            app.is_maximized = false;
            app.saved_window_bounds = None;

            // Use window.resize to restore the saved size
            window.resize(gpui::Size {
                width: saved_bounds.size.width,
                height: saved_bounds.size.height,
            });
            cx.notify();
        }
    } else {
        // Maximize
        let current_bounds = window.bounds();
        app.saved_window_bounds = Some(current_bounds);
        app.is_maximized = true;

        if let Some(maximized_bounds) = app.display_bounds {
            window.resize(gpui::Size {
                width: maximized_bounds.size.width,
                height: maximized_bounds.size.height,
            });
            cx.notify();
        }
    }
}

/// Update container height based on window size
pub fn update_container_height(app: &mut CanViewApp, window: &mut Window) {
    let bounds = window.bounds();
    let header_height = px(80.0); // Approximate header height
    let padding = px(100.0); // Bottom padding
    let available_height = bounds.size.height - header_height - padding;

    // Update the container height with a minimum value
    let min_height = px(500.0);
    let final_height = available_height.max(min_height);
    app.list_container_height = final_height.into();
}
