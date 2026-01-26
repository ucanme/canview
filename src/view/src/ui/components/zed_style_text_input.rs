//! Zed-style Enhanced Text Input Component
//!
//! A modern text input inspired by Zed IDE with better visual feedback and state management.

use gpui::prelude::FluentBuilder;
use gpui::*;

pub use super::text_input::TextInputValidation;

/// Enhanced text input state with cursor and selection support
#[derive(Clone, Debug)]
pub struct ZedStyleTextInputState {
    pub text: String,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub ime_composition: Option<String>,
}

impl Default for ZedStyleTextInputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            selection_start: None,
            ime_composition: None,
        }
    }
}

impl ZedStyleTextInputState {
    pub fn new(text: String) -> Self {
        let cursor_position = text.chars().count();
        Self {
            text,
            cursor_position,
            ..Default::default()
        }
    }

    pub fn has_selection(&self) -> bool {
        if let Some(start) = self.selection_start {
            start != self.cursor_position
        } else {
            false
        }
    }

    pub fn get_selected_range(&self) -> Option<(usize, usize)> {
        if let Some(start) = self.selection_start {
            let end = self.cursor_position;
            let (start, end) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            Some((start, end))
        } else {
            None
        }
    }

    pub fn select_all(&mut self) {
        self.selection_start = Some(0);
        self.cursor_position = self.text.chars().count();
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.get_selected_range() {
            let mut chars: Vec<char> = self.text.chars().collect();
            chars.drain(start..end);
            self.text = chars.into_iter().collect();
            self.cursor_position = start;
            self.clear_selection();
            true
        } else {
            false
        }
    }
}

/// Zed-style text input builder
pub struct ZedStyleTextInputBuilder {
    text: String,
    placeholder: String,
    focused: bool,
    validation: TextInputValidation,
    max_width: Option<Pixels>,
    min_width: Option<Pixels>,
    show_cursor: bool,
}

impl ZedStyleTextInputBuilder {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: String::new(),
            focused: false,
            validation: TextInputValidation::None,
            max_width: None,
            min_width: None,
            show_cursor: true,
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn validation(mut self, validation: TextInputValidation) -> Self {
        self.validation = validation;
        self
    }

    pub fn max_width(mut self, width: Pixels) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn min_width(mut self, width: Pixels) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn show_cursor(mut self, show: bool) -> Self {
        self.show_cursor = show;
        self
    }

    /// Build the text input with keyboard handling (compatible with existing API)
    pub fn build<App>(
        self,
        id: impl Into<String>,
        view: Entity<App>,
        on_change: impl Fn(&str, &mut gpui::Context<App>) + 'static,
        on_submit: impl Fn(&str, &mut gpui::Context<App>) + 'static,
        on_cancel: impl Fn(&mut gpui::Context<App>) + 'static,
    ) -> impl IntoElement
    where
        App: 'static,
    {
        let text = self.text.clone();
        let placeholder = self.placeholder.clone();
        let input_id = id.into();
        let validation = self.validation;
        let max_width = self.max_width.unwrap_or(px(200.));
        let min_width = self.min_width.unwrap_or(px(100.));

        let border_color = if self.focused {
            rgb(0x89b4fa)
        } else {
            rgb(0x2a2a2a)
        };

        let is_empty = text.trim().is_empty();
        let text_color = if is_empty {
            rgb(0x646473)
        } else {
            rgb(0xcdd6f4)
        };

        let display_text = if is_empty {
            if placeholder.is_empty() {
                String::from("Type here...")
            } else {
                placeholder
            }
        } else {
            text.clone()
        };

        // Calculate cursor position for visual feedback
        let cursor_offset = text.chars().count() as f32 * 8.0; // Approximate char width

        div()
            .px_2()
            .py_1()
            .bg(rgb(0x1a1a1a))
            .border_1()
            .border_color(border_color)
            .rounded(px(2.))
            .flex()
            .items_center()
            .min_w(min_width)
            .max_w(max_width)
            .cursor_text()
            .id(input_id.clone())
            .focusable()
            .relative()
            .child(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .w_full()
                    .child(
                        div()
                            .text_sm()
                            .text_color(text_color)
                            .child(display_text.clone()),
                    )
                    .when(self.focused && self.show_cursor && !is_empty, |el| {
                        // Simple cursor indicator
                        el.child(
                            div()
                                .absolute()
                                .left(px(cursor_offset))
                                .top(px(2.))
                                .w(px(2.))
                                .h(px(16.))
                                .bg(rgb(0xcdd6f4)),
                        )
                    }),
            )
            .on_key_down({
                let view = view.clone();
                move |event, _window, cx| {
                    let keystroke = format!("{}", event.keystroke);

                    match keystroke.as_str() {
                        "backspace" => {
                            view.update(cx, |this, cx| {
                                if !text.is_empty() {
                                    let mut chars: Vec<char> = text.chars().collect();
                                    chars.pop();
                                    let new_text: String = chars.into_iter().collect();
                                    on_change(&new_text, cx);
                                }
                            });
                        }
                        "enter" => {
                            view.update(cx, |this, cx| {
                                on_submit(&text, cx);
                            });
                        }
                        "escape" => {
                            view.update(cx, |this, cx| {
                                on_cancel(cx);
                            });
                        }
                        "left" | "right" | "up" | "down" | "home" | "end" => {
                            // Navigation keys - can be extended for cursor movement
                        }
                        _ => {
                            // Handle character input (including multi-character from IME)
                            let is_printable = if keystroke.len() == 1 {
                                keystroke
                                    .chars()
                                    .next()
                                    .map(|c| !c.is_control())
                                    .unwrap_or(false)
                            } else if keystroke.len() > 1 {
                                !keystroke.to_lowercase().starts_with("backspace")
                                    && !keystroke.to_lowercase().starts_with("delete")
                                    && !keystroke.to_lowercase().starts_with("left")
                                    && !keystroke.to_lowercase().starts_with("right")
                                    && !keystroke.to_lowercase().starts_with("up")
                                    && !keystroke.to_lowercase().starts_with("down")
                                    && !keystroke.to_lowercase().starts_with("home")
                                    && !keystroke.to_lowercase().starts_with("end")
                                    && keystroke.chars().all(|c| !c.is_control())
                            } else {
                                false
                            };

                            if is_printable {
                                let all_valid =
                                    keystroke.chars().all(|c| validation.is_valid_char(c));

                                if all_valid {
                                    let mut new_text = text.clone();
                                    new_text.push_str(&keystroke);

                                    view.update(cx, |this, cx| {
                                        on_change(&new_text, cx);
                                    });
                                }
                            }
                        }
                    }
                }
            })
    }
}

impl Default for ZedStyleTextInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = ZedStyleTextInputState::new("Hello".to_string());
        assert_eq!(state.text, "Hello");
        assert_eq!(state.cursor_position, 5);
        assert!(!state.has_selection());
    }

    #[test]
    fn test_selection() {
        let mut state = ZedStyleTextInputState::new("Hello".to_string());
        state.selection_start = Some(0);
        state.cursor_position = 3;

        assert!(state.has_selection());
        assert_eq!(state.get_selected_range(), Some((0, 3)));
    }

    #[test]
    fn test_select_all() {
        let mut state = ZedStyleTextInputState::new("Hello".to_string());
        state.select_all();

        assert!(state.has_selection());
        assert_eq!(state.get_selected_range(), Some((0, 5)));
    }

    #[test]
    fn test_clear_selection() {
        let mut state = ZedStyleTextInputState::new("Hello".to_string());
        state.select_all();
        state.clear_selection();

        assert!(!state.has_selection());
    }

    #[test]
    fn test_delete_selection() {
        let mut state = ZedStyleTextInputState::new("Hello".to_string());
        state.selection_start = Some(1);
        state.cursor_position = 4;

        let deleted = state.delete_selection();
        assert!(deleted);
        assert_eq!(state.text, "Ho");
        assert_eq!(state.cursor_position, 1);
    }
}
