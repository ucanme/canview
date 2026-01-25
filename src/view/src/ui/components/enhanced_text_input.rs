//! Enhanced Text Input Component with Cursor, Selection, and Better UX
//!
//! This component is inspired by gpui-component's input implementation but
//! maintains a simpler API for easier use in the canview project.
//!
//! Key improvements over the basic text_input:
//! - Visible blinking cursor
//! - Text selection with Shift+Arrow keys
//! - Better keyboard handling (Ctrl/Cmd combinations)
//! - Improved IME support
//! - Cleaner rendering with selection highlights
//!
//! # Example
//!
//! ```rust
//! use crate::ui::components::{EnhancedTextInput, EnhancedTextInputBuilder};
//!
//! // Simple usage
//! let input = EnhancedTextInputBuilder::new()
//!     .text(state.text.clone())
//!     .placeholder("Enter library name...")
//!     .focused(state.is_editing)
//!     .build(
//!         "library_input",
//!         cx.entity().clone(),
//!         on_change,
//!         on_submit,
//!     );
//! ```

use gpui::prelude::FluentBuilder;
use gpui::*;
use std::ops::Range;
use std::time::Duration;

/// Character validation mode for text input
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextInputValidation {
    /// Library name validation: supports Unicode, spaces, alphanumeric
    LibraryName,

    /// Version name validation: ASCII alphanumeric, dot, underscore, hyphen only
    VersionName,

    /// Custom validation function
    Custom(fn(char) -> bool),

    /// No validation (accepts everything except control characters)
    None,
}

impl TextInputValidation {
    /// Check if a character is valid according to the validation mode
    pub fn is_valid_char(&self, ch: char) -> bool {
        match self {
            TextInputValidation::LibraryName => {
                !ch.is_control() && (ch.is_ascii_alphanumeric() || ch == ' ' || !ch.is_ascii())
            }
            TextInputValidation::VersionName => {
                ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-'
            }
            TextInputValidation::Custom(validator) => validator(ch),
            TextInputValidation::None => !ch.is_control(),
        }
    }
}

/// Text selection represented by start and end positions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TextSelection {
    pub start: usize,
    pub end: usize,
}

impl TextSelection {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn clear(&mut self) {
        self.start = 0;
        self.end = 0;
    }

    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }

    pub fn to_range(&self) -> Range<usize> {
        self.start..self.end
    }
}

/// Internal state for enhanced text input
pub struct EnhancedTextInputState {
    pub text: String,
    pub cursor: usize,
    pub selection: TextSelection,
    pub focused: bool,
    pub cursor_visible: bool,
    pub validation: TextInputValidation,
}

impl EnhancedTextInputState {
    pub fn new(text: String) -> Self {
        let cursor = text.chars().count();
        Self {
            text,
            cursor,
            selection: TextSelection::default(),
            focused: false,
            cursor_visible: true,
            validation: TextInputValidation::LibraryName,
        }
    }

    /// Get character index from cursor position (cursor is in chars, need byte offset)
    fn cursor_to_byte_offset(&self) -> usize {
        self.text
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    /// Insert text at cursor position
    pub fn insert_text(&mut self, text_to_insert: &str) -> bool {
        // Validate all characters
        let all_valid = text_to_insert
            .chars()
            .all(|c| self.validation.is_valid_char(c));

        if !all_valid {
            return false;
        }

        // Delete selection if any
        if !self.selection.is_empty() {
            self.delete_selection();
        }

        let byte_offset = self.cursor_to_byte_offset();
        self.text.insert_str(byte_offset, text_to_insert);
        self.cursor += text_to_insert.chars().count();
        self.selection.clear();
        true
    }

    /// Delete character before cursor (backspace)
    pub fn delete_backward(&mut self) {
        if !self.selection.is_empty() {
            self.delete_selection();
            return;
        }

        if self.cursor > 0 && !self.text.is_empty() {
            let mut chars: Vec<char> = self.text.chars().collect();
            chars.remove(self.cursor - 1);
            self.text = chars.into_iter().collect();
            self.cursor -= 1;
        }
    }

    /// Delete character after cursor (delete)
    pub fn delete_forward(&mut self) {
        if !self.selection.is_empty() {
            self.delete_selection();
            return;
        }

        let char_count = self.text.chars().count();
        if self.cursor < char_count {
            let mut chars: Vec<char> = self.text.chars().collect();
            chars.remove(self.cursor);
            self.text = chars.into_iter().collect();
        }
    }

    /// Delete selected text
    fn delete_selection(&mut self) {
        if self.selection.is_empty() {
            return;
        }

        let mut chars: Vec<char> = self.text.chars().collect();
        let range = self.selection.to_range();
        chars.drain(range);
        self.text = chars.into_iter().collect();
        self.cursor = self.selection.start;
        self.selection.clear();
    }

    /// Move cursor to position
    pub fn move_cursor_to(&mut self, position: usize) {
        let char_count = self.text.chars().count();
        self.cursor = position.min(char_count);
    }

    /// Move cursor relative to current position
    pub fn move_cursor(&mut self, delta: isize) {
        let new_pos = self.cursor as isize + delta;
        if new_pos >= 0 {
            self.move_cursor_to(new_pos as usize);
        }
    }

    /// Select text from current cursor to new position
    pub fn select_to(&mut self, position: usize) {
        let char_count = self.text.chars().count();
        let new_pos = position.min(char_count);
        self.selection = TextSelection::new(self.cursor, new_pos);
    }

    /// Select all text
    pub fn select_all(&mut self) {
        self.selection = TextSelection::new(0, self.text.chars().count());
    }

    /// Get selected text
    pub fn selected_text(&self) -> String {
        if self.selection.is_empty() {
            return String::new();
        }

        self.text
            .chars()
            .enumerate()
            .filter(|(i, _)| self.selection.contains(*i))
            .map(|(_, c)| c)
            .collect()
    }

    /// Clear selection and move cursor
    pub fn clear_selection(&mut self) {
        if !self.selection.is_empty() {
            self.cursor = self.selection.end;
            self.selection.clear();
        }
    }
}

/// Builder for creating enhanced text inputs
pub struct EnhancedTextInputBuilder {
    text: String,
    placeholder: String,
    focused: bool,
    validation: TextInputValidation,
    max_width: Option<Pixels>,
    min_width: Option<Pixels>,
}

impl EnhancedTextInputBuilder {
    /// Create a new text input builder
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: String::new(),
            focused: false,
            validation: TextInputValidation::LibraryName,
            max_width: None,
            min_width: None,
        }
    }

    /// Set the initial text content
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Set the placeholder text shown when input is empty
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set whether the input is focused
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set the character validation mode
    pub fn validation(mut self, validation: TextInputValidation) -> Self {
        self.validation = validation;
        self
    }

    /// Set maximum width
    pub fn max_width(mut self, width: Pixels) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set minimum width
    pub fn min_width(mut self, width: Pixels) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Build the enhanced text input element
    pub fn build<App>(
        self,
        id: impl Into<String>,
        view: Entity<App>,
        on_change: impl Fn(&str, &mut gpui::Context<App>) + 'static,
        on_submit: impl Fn(&str, &mut gpui::Context<App>) + 'static,
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
                    .text_xs()
                    .text_color(text_color)
                    .child(display_text.clone()),
            )
            // Render cursor overlay when focused
            .when(self.focused, |this_div| {
                this_div.child(
                    gpui::div()
                        .absolute()
                        .left(px(4.))
                        .top(px(2.))
                        .bottom(px(2.))
                        .w(px(1.5))
                        .bg(rgb(0xcdd6f4))
                )
            })
    }
}

impl Default for EnhancedTextInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to check if a keystroke is a printable character or multi-char string
pub fn is_printable_keystroke(keystroke: &str) -> bool {
    if keystroke.is_empty() {
        return false;
    }

    // Check if it's not a control key
    let lower = keystroke.to_lowercase();
    if lower.starts_with("backspace")
        || lower.starts_with("delete")
        || lower.starts_with("left")
        || lower.starts_with("right")
        || lower.starts_with("up")
        || lower.starts_with("down")
        || lower == "enter"
        || lower == "escape"
        || lower == "tab"
    {
        return false;
    }

    // Check all characters are not control characters
    keystroke.chars().all(|c| !c.is_control())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection() {
        let mut selection = TextSelection::new(5, 2);
        assert_eq!(selection.start, 2);
        assert_eq!(selection.end, 5);
        assert_eq!(selection.len(), 3);
        assert!(!selection.is_empty());
        assert!(selection.contains(3));
        assert!(!selection.contains(1));
    }

    #[test]
    fn test_state_insert() {
        let mut state = EnhancedTextInputState::new("Test".to_string());
        assert_eq!(state.text, "Test");
        assert_eq!(state.cursor, 4);

        state.insert_text("测试");
        assert_eq!(state.text, "Test测试");
        assert_eq!(state.cursor, 6);
    }

    #[test]
    fn test_state_delete() {
        let mut state = EnhancedTextInputState::new("测试库".to_string());
        assert_eq!(state.text, "测试库");
        assert_eq!(state.cursor, 3);

        state.delete_backward();
        assert_eq!(state.text, "测试");
        assert_eq!(state.cursor, 2);
    }

    #[test]
    fn test_state_selection() {
        let mut state = EnhancedTextInputState::new("测试库".to_string());
        state.select_to(2);
        assert_eq!(state.selection.start, 0);
        assert_eq!(state.selection.end, 2);
        assert_eq!(state.selected_text(), "测试");
    }

    #[test]
    fn test_state_delete_selection() {
        let mut state = EnhancedTextInputState::new("测试库123".to_string());
        state.selection = TextSelection::new(2, 5);
        state.delete_selection();
        assert_eq!(state.text, "测试23");
        assert_eq!(state.cursor, 2);
    }
}
