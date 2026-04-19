//! Button component usage examples
//!
//! This file demonstrates how to use the Button component in various scenarios.

use gpui::{prelude::*, *};
use crate::ui::components::button::{Button, ButtonSize, ButtonVariant};
use crate::app::{AppView, CanViewApp};

/// Example: Create a simple primary button
pub fn example_simple_button(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    Button::new("Click Me")
        .size(ButtonSize::Medium)
        .variant(ButtonVariant::Primary)
        .build()
        .on_mouse_down(gpui::MouseButton::Left, {
            let view = cx.entity().clone();
            move |_event, _window, cx| {
                view.update(cx, |app, cx| {
                    app.status_msg = "Button clicked!".into();
                    cx.notify();
                });
            }
        })
}

/// Example: Create navigation buttons (like File, Library, Plot tabs)
pub fn example_nav_buttons(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();

    div()
        .flex()
        .gap_2()
        .child(
            Button::new("File")
                .size(ButtonSize::Medium)
                .variant(ButtonVariant::Ghost)
                .active(false)
                .build()
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_event, _window, cx| {
                        view.update(cx, |app, cx| {
                            // Handle File button click
                            app.show_file_menu = !app.show_file_menu;
                            cx.notify();
                        });
                    }
                })
        )
        .child(
            Button::new("Library")
                .size(ButtonSize::Medium)
                .variant(ButtonVariant::Ghost)
                .active(false)
                .build()
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_event, _window, cx| {
                        view.update(cx, |app, cx| {
                            app.current_view = AppView::LibraryView;
                            cx.notify();
                        });
                    }
                })
        )
        .child(
            Button::new("Plot")
                .size(ButtonSize::Medium)
                .variant(ButtonVariant::Ghost)
                .active(false)
                .build()
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_event, _window, cx| {
                        view.update(cx, |app, cx| {
                            app.current_view = AppView::PlotView;
                            cx.notify();
                        });
                    }
                })
        )
}

/// Example: Button with different sizes
pub fn example_button_sizes(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    div()
        .flex()
        .gap_2()
        .items_center()
        .child(
            Button::new("Small")
                .size(ButtonSize::Small)
                .variant(ButtonVariant::Secondary)
                .build()
        )
        .child(
            Button::new("Medium")
                .size(ButtonSize::Medium)
                .variant(ButtonVariant::Secondary)
                .build()
        )
        .child(
            Button::new("Large")
                .size(ButtonSize::Large)
                .variant(ButtonVariant::Secondary)
                .build()
        )
}

/// Example: Button with different variants
pub fn example_button_variants(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    div()
        .flex()
        .gap_2()
        .child(
            Button::new("Primary")
                .variant(ButtonVariant::Primary)
                .build()
        )
        .child(
            Button::new("Secondary")
                .variant(ButtonVariant::Secondary)
                .build()
        )
        .child(
            Button::new("Ghost")
                .variant(ButtonVariant::Ghost)
                .build()
        )
        .child(
            Button::new("Danger")
                .variant(ButtonVariant::Danger)
                .build()
        )
}

/// Example: Active state button (for tabs)
pub fn example_active_tab_button(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let is_active = true;

    Button::new("Active Tab")
        .size(ButtonSize::Medium)
        .variant(ButtonVariant::Ghost)
        .active(is_active)
        .build()
        .on_mouse_down(gpui::MouseButton::Left, {
            let view = cx.entity().clone();
            move |_event, _window, cx| {
                view.update(cx, |app, cx| {
                    app.status_msg = "Tab clicked!".into();
                    cx.notify();
                });
            }
        })
}

/// Example: Disabled button
pub fn example_disabled_button() -> impl IntoElement {
    Button::new("Disabled")
        .size(ButtonSize::Medium)
        .variant(ButtonVariant::Primary)
        .disabled(true)
        .build()
}

/// Example: Convenience functions
pub fn example_convenience_functions(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();

    div()
        .flex()
        .gap_2()
        .child(
            crate::ui::components::primary_button("Save")
                .build()
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_event, _window, cx| {
                        view.update(cx, |app, cx| {
                            app.save_config(cx);
                        });
                    }
                })
        )
        .child(
            crate::ui::components::secondary_button("Cancel")
                .build()
        )
        .child(
            crate::ui::components::ghost_button("Help")
                .build()
        )
        .child(
            crate::ui::components::danger_button("Delete")
                .disabled(false)
                .build()
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_event, _window, cx| {
                        view.update(cx, |app, cx| {
                            app.status_msg = "Delete action".into();
                            cx.notify();
                        });
                    }
                })
        )
}

/// Example: Replace existing manual button styling with Button component
///
/// Before (old manual style):
/// ```rust
/// div()
///     .px_3()
///     .py_2()
///     .h(px(32.))
///     .bg(rgb(0x89b4fa))
///     .rounded(px(4.))
///     .cursor_pointer()
///     .child("Click me")
/// ```
///
/// After (using Button component):
/// ```rust
/// Button::new("Click me")
///     .size(ButtonSize::Medium)
///     .variant(ButtonVariant::Primary)
///     .build()
/// ```
pub fn example_migration_guide() {
    // This is just documentation, no actual code needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        // Test that buttons can be created
        let _btn = Button::new("Test");
        let _btn_primary = Button::new("Test").variant(ButtonVariant::Primary);
        let _btn_small = Button::new("Test").size(ButtonSize::Small);
        let _btn_disabled = Button::new("Test").disabled(true);
        let _btn_active = Button::new("Test").active(true);
    }

    #[test]
    fn test_convenience_functions() {
        // Test convenience functions
        let _primary = crate::ui::components::primary_button("Primary");
        let _secondary = crate::ui::components::secondary_button("Secondary");
        let _ghost = crate::ui::components::ghost_button("Ghost");
        let _danger = crate::ui::components::danger_button("Danger");
    }

    #[test]
    fn test_button_chaining() {
        // Test method chaining
        let _btn = Button::new("Test")
            .size(ButtonSize::Medium)
            .variant(ButtonVariant::Primary)
            .disabled(false)
            .active(false);
    }
}
