//! Navigation commands
//!
//! Command handlers for view navigation and window management.

use crate::app::state::AppView;

/// Navigate to a specific view
pub struct NavigateToView {
    pub target_view: AppView,
}

impl NavigateToView {
    pub fn new(target_view: AppView) -> Self {
        Self { target_view }
    }
}

/// Toggle window maximize state
pub struct ToggleMaximize;

impl ToggleMaximize {
    pub fn new() -> Self {
        Self
    }
}

/// Update container height calculation
pub struct UpdateContainerHeight {
    pub window_height: f32,
}

impl UpdateContainerHeight {
    pub fn new(window_height: f32) -> Self {
        Self { window_height }
    }

    /// Calculate the list container height based on window size
    ///
    /// This accounts for:
    /// - Top bar (37px): 36px + 1px border
    /// - Status bar (25px): 24px + 1px border
    /// - Log header (29px): 28px + 1px border
    /// - Total: 91px
    pub fn calculate_container_height(&self) -> f32 {
        self.window_height - 37.0 - 25.0 - 29.0
    }
}
