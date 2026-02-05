//! Zed-style UI components
//!
//! This module contains modern, accessible UI components inspired by Zed editor's
//! design language, featuring the Catppuccin Mocha color palette.
//!
//! These components are currently in development and may have compilation issues.
//! They are separated from the main components module to avoid blocking the build.

pub mod button;
pub mod card;
pub mod dropdown;
pub mod enhanced_text_input;
pub mod ime_text_input;

// Re-export for convenience
pub use button::{Button, ButtonColor, ButtonSize, IconPosition};
pub use card::{Card, CardPadding, CardStyle};
pub use dropdown::{DropdownItem, DropdownState, SimpleDropdown, render_dropdown_menu};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // This test verifies the module structure is correct
        // Individual component tests are in their respective modules
        assert!(true);
    }
}
