//! Controller layer
//!
//! This module contains business logic controllers that act as a bridge
//! between the domain layer and the UI layer.
//!
//! NOTE: `ui_controller` is currently excluded from the build because it
//! references symbols (`crate::views::log_view::render_message_row_static_with_widths`,
//! `crate::app::FilterType`, `CanViewerApp::render_filter_dropdown`) that no
//! longer exist in the current codebase. It was dead code (the `controllers`
//! module was not declared in `main.rs`) until Task 6 wired it in. Re-enabling
//! `ui_controller` requires porting its filter-dropdown rendering to the
//! current `ui::views` structure — left for a future task.

pub mod library_controller;
pub mod config_controller;
pub mod window_controller;
// pub mod ui_controller; // broken: references removed APIs; see note above
pub mod signal_set_controller;

pub use library_controller::*;
pub use config_controller::*;
pub use window_controller::*;
// pub use ui_controller::*;
pub use signal_set_controller::*;
