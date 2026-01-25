//! Enhanced Text Input Component with Full IME Support
//!
//! This component provides comprehensive text input functionality including:
//! - Multi-character string input (IME support for Chinese, Japanese, Korean, etc.)
//! - Inline keyboard event handling (no parent component key handler needed)
//! - Flexible character validation options
//! - Cursor position management
//! - Consistent styling and behavior
//!
//! # Example
//!
//! ```rust
//! use crate::ui::components::{TextInputBuilder, TextInputValidation};
//!
//! // Library name input (supports Unicode)
//! let input = TextInputBuilder::new()
//!     .text(state.library_name.clone())
//!     .placeholder("Library name...")
//!     .focused(state.is_editing)
//!     .validation(TextInputValidation::LibraryName)
//!     .build(
//!         "library_name_input",
//!         cx.entity().clone(),
//!         on_change,
//!         on_submit,
//!         on_cancel
//!     );
//!
//! // Version name input (ASCII only)
//! let input = TextInputBuilder::new()
//!     .text(state.version_name.clone())
//!     .placeholder("v1.0.0")
//!     .validation(TextInputValidation::VersionName)
//!     .build(
//!         "version_name_input",
//!         cx.entity().clone(),
//!         on_change,
//!         on_submit,
//!         on_cancel
//!     );
//! ```

use gpui::prelude::FluentBuilder;
use gpui::*;

/// Character validation mode for text input
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextInputValidation {
    /// Library name validation: supports Unicode, spaces, alphanumeric
    /// Example: "测试CAN信号库", "Test测试库123", "📊 数据分析库"
    LibraryName,

    /// Version name validation: ASCII alphanumeric, dot, underscore, hyphen only
    /// Example: "v1.0.0", "version_1.2", "release-2.0", "v1.2.3-beta"
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
                // Support Chinese, English, numbers, spaces, and any Unicode
                !ch.is_control() && (ch.is_ascii_alphanumeric() || ch == ' ' || !ch.is_ascii())
            }
            TextInputValidation::VersionName => {
                // Only ASCII alphanumeric, dot, underscore, hyphen
                ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-'
            }
            TextInputValidation::Custom(validator) => validator(ch),
            TextInputValidation::None => !ch.is_control(),
        }
    }
}

/// Builder for creating text inputs with flexible options
pub struct TextInputBuilder {
    text: String,
    placeholder: String,
    focused: bool,
    validation: TextInputValidation,
    max_width: Option<Pixels>,
    min_width: Option<Pixels>,
}

impl TextInputBuilder {
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

    /// Build the text input element with full keyboard handling
    ///
    /// This version handles all keyboard events internally and updates state
    /// through callbacks. No parent component key handler needed.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the input element
    /// * `view` - Entity to update via view.update()
    /// * `on_change` - Callback when text changes, receives (new_text, cx)
    /// * `on_submit` - Callback when Enter is pressed, receives (text, cx)
    /// * `on_cancel` - Callback when Escape is pressed, receives (cx)
    ///
    /// # Example
    ///
    /// ```rust
    /// TextInputBuilder::new()
    ///     .text(state.text.clone())
    ///     .placeholder("Enter text...")
    ///     .validation(TextInputValidation::LibraryName)
    ///     .build(
    ///         "my_input",
    ///         cx.entity().clone(),
    ///         {
    ///             let view = cx.entity().clone();
    ///             move |new_text, cx| {
    ///                 view.update(cx, |this, cx| {
    ///                     this.text = new_text.to_string();
    ///                     cx.notify();
    ///                 });
    ///             }
    ///         },
    ///         {
    ///             let view = cx.entity().clone();
    ///             move |text, cx| {
    ///                 view.update(cx, |this, cx| {
    ///                     this.submit(text);
    ///                     cx.notify();
    ///                 });
    ///             }
    ///         },
    ///         {
    ///             move |cx| {
    ///                 view.update(cx, |this, cx| {
    ///                     this.cancel();
    ///                     cx.notify();
    ///                 });
    ///             }
    ///         }
    ///     )
    /// ```
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
            .child(div().text_xs().text_color(text_color).child(display_text))
            .on_key_down({
                let view = view.clone();
                let text = text.clone();
                move |event, _window, cx| {
                    let keystroke = format!("{}", event.keystroke);
                    let key_text = event.keystroke.key.as_str();

                    eprintln!(
                        "TextInput key_down: id='{}' keystroke='{}' key='{}' text='{}'",
                        input_id, keystroke, key_text, text
                    );

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
                            // Navigation keys - let parent handle or ignore
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
                                // Multi-character string (possibly from IME)
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
                                // Validate all characters
                                let all_valid =
                                    keystroke.chars().all(|c| validation.is_valid_char(c));

                                if all_valid {
                                    let mut new_text = text.clone();
                                    new_text.push_str(&keystroke);

                                    eprintln!(
                                        "TextInput inserted: '{}', new_text: '{}'",
                                        keystroke, new_text
                                    );

                                    view.update(cx, |this, cx| {
                                        on_change(&new_text, cx);
                                    });
                                } else {
                                    eprintln!(
                                        "TextInput rejected: invalid chars in '{}'",
                                        keystroke
                                    );
                                }
                            }
                        }
                    }
                }
            })
    }
}

impl Default for TextInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper struct for managing text input state with cursor
pub struct TextInputState {
    pub text: String,
    pub cursor_position: usize,
}

impl TextInputState {
    /// Create a new input state
    pub fn new(text: String) -> Self {
        let cursor_position = text.chars().count();
        Self {
            text,
            cursor_position,
        }
    }

    /// Insert text at cursor position (supports multi-character strings from IME)
    pub fn insert_text(&mut self, text_to_insert: &str, validation: TextInputValidation) -> bool {
        // Validate all characters
        let all_valid = text_to_insert.chars().all(|c| validation.is_valid_char(c));

        if !all_valid {
            eprintln!("TextInput rejected: invalid chars in '{}'", text_to_insert);
            return false;
        }

        let mut chars: Vec<char> = self.text.chars().collect();
        for (i, ch) in text_to_insert.chars().enumerate() {
            chars.insert(self.cursor_position + i, ch);
        }
        self.text = chars.into_iter().collect();
        self.cursor_position += text_to_insert.chars().count();

        eprintln!(
            "TextInput inserted: '{}' at position {}, new_text: '{}'",
            text_to_insert,
            self.cursor_position - text_to_insert.chars().count(),
            self.text
        );

        true
    }

    /// Delete character before cursor (backspace)
    pub fn delete_backward(&mut self) -> bool {
        if self.cursor_position > 0 && !self.text.is_empty() {
            let mut chars: Vec<char> = self.text.chars().collect();
            chars.remove(self.cursor_position - 1);
            self.text = chars.into_iter().collect();
            self.cursor_position -= 1;
            true
        } else {
            false
        }
    }

    /// Clear all text
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
    }

    /// Move cursor to position (clamped to valid range)
    pub fn move_cursor_to(&mut self, position: usize) {
        self.cursor_position = position.min(self.text.chars().count());
    }

    /// Move cursor relative to current position
    pub fn move_cursor(&mut self, delta: isize) {
        let new_pos = self.cursor_position as isize + delta;
        if new_pos >= 0 {
            self.move_cursor_to(new_pos as usize);
        }
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
        || lower.starts_with("home")
        || lower.starts_with("end")
        || lower == "enter"
        || lower == "escape"
        || lower == "tab"
    {
        return false;
    }

    // Check all characters are not control characters
    keystroke.chars().all(|c| !c.is_control())
}

/// Helper function to handle keyboard input for text input
///
/// Returns (should_update, new_text) where should_update indicates if the
/// text changed and new_text is the updated text content
pub fn handle_key_down(
    current_text: &str,
    keystroke: &str,
    validation: TextInputValidation,
) -> (bool, String) {
    let lower = keystroke.to_lowercase();

    match lower.as_str() {
        "backspace" => {
            if !current_text.is_empty() {
                let mut chars: Vec<char> = current_text.chars().collect();
                chars.pop();
                (true, chars.into_iter().collect())
            } else {
                (false, current_text.to_string())
            }
        }
        "enter" | "escape" => (false, current_text.to_string()),
        "left" | "right" | "up" | "down" | "home" | "end" => (false, current_text.to_string()),
        _ => {
            // Handle character input (including multi-character from IME)
            if is_printable_keystroke(keystroke) {
                // Validate all characters
                let all_valid = keystroke.chars().all(|c| validation.is_valid_char(c));

                if all_valid {
                    let mut new_text = current_text.to_string();
                    new_text.push_str(keystroke);
                    eprintln!(
                        "TextInput inserted: '{}', new_text: '{}'",
                        keystroke, new_text
                    );
                    (true, new_text)
                } else {
                    eprintln!("TextInput rejected: invalid chars in '{}'", keystroke);
                    (false, current_text.to_string())
                }
            } else {
                (false, current_text.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_name_validation() {
        let validation = TextInputValidation::LibraryName;

        // Valid characters
        assert!(validation.is_valid_char('测'));
        assert!(validation.is_valid_char('a'));
        assert!(validation.is_valid_char('Z'));
        assert!(validation.is_valid_char('0'));
        assert!(validation.is_valid_char(' '));
        assert!(validation.is_valid_char('📊'));

        // Invalid characters
        assert!(!validation.is_valid_char('\n'));
        assert!(!validation.is_valid_char('\t'));
        assert!(!validation.is_valid_char('\r'));
    }

    #[test]
    fn test_version_name_validation() {
        let validation = TextInputValidation::VersionName;

        // Valid characters
        assert!(validation.is_valid_char('a'));
        assert!(validation.is_valid_char('Z'));
        assert!(validation.is_valid_char('0'));
        assert!(validation.is_valid_char('.'));
        assert!(validation.is_valid_char('_'));
        assert!(validation.is_valid_char('-'));

        // Invalid characters
        assert!(!validation.is_valid_char(' '));
        assert!(!validation.is_valid_char('测'));
        assert!(!validation.is_valid_char('\n'));
    }

    #[test]
    fn test_custom_validation() {
        let validation = TextInputValidation::Custom(|c| c.is_ascii_digit());

        assert!(validation.is_valid_char('0'));
        assert!(validation.is_valid_char('9'));
        assert!(!validation.is_valid_char('a'));
        assert!(!validation.is_valid_char('测'));
    }

    #[test]
    fn test_multi_character_validation() {
        let validation = TextInputValidation::LibraryName;

        // Test multi-character strings
        let valid_strings = vec!["测试", "Test", "测试123", "Test测试"];

        for s in valid_strings {
            assert!(
                s.chars().all(|c| validation.is_valid_char(c)),
                "String '{}' should be valid",
                s
            );
        }

        let invalid_strings = vec!["Test\nLibrary", "Test\tLibrary"];

        for s in invalid_strings {
            assert!(
                !s.chars().all(|c| validation.is_valid_char(c)),
                "String '{}' should be invalid",
                s
            );
        }
    }

    #[test]
    fn test_input_state_insert() {
        let mut state = TextInputState::new("Test".to_string());
        assert_eq!(state.text, "Test");
        assert_eq!(state.cursor_position, 4);

        // Insert multi-char string
        state.insert_text("测试", TextInputValidation::LibraryName);
        assert_eq!(state.text, "Test测试");
        assert_eq!(state.cursor_position, 6);
    }

    #[test]
    fn test_input_state_delete() {
        let mut state = TextInputState::new("测试库".to_string());
        assert_eq!(state.text, "测试库");
        assert_eq!(state.cursor_position, 3);

        // Delete backward
        state.delete_backward();
        assert_eq!(state.text, "测试");
        assert_eq!(state.cursor_position, 2);
    }

    #[test]
    fn test_input_state_clear() {
        let mut state = TextInputState::new("Test".to_string());
        state.clear();
        assert_eq!(state.text, "");
        assert_eq!(state.cursor_position, 0);
    }

    #[test]
    fn test_cursor_movement() {
        let mut state = TextInputState::new("测试".to_string());
        assert_eq!(state.cursor_position, 2);

        state.move_cursor_to(0);
        assert_eq!(state.cursor_position, 0);

        state.move_cursor(1);
        assert_eq!(state.cursor_position, 1);

        state.move_cursor(-1);
        assert_eq!(state.cursor_position, 0);

        // Clamp to end
        state.move_cursor_to(100);
        assert_eq!(state.cursor_position, 2);
    }

    #[test]
    fn test_is_printable_keystroke() {
        // Printable characters
        assert!(is_printable_keystroke("a"));
        assert!(is_printable_keystroke("你好"));
        assert!(is_printable_keystroke("Test"));
        assert!(is_printable_keystroke("v1.0.0"));

        // Control keys
        assert!(!is_printable_keystroke("backspace"));
        assert!(!is_printable_keystroke("enter"));
        assert!(!is_printable_keystroke("escape"));
        assert!(!is_printable_keystroke("left"));
        assert!(!is_printable_keystroke(""));

        // Strings with control characters
        assert!(!is_printable_keystroke("Test\n"));
        assert!(!is_printable_keystroke("Test\t"));
    }

    #[test]
    fn test_handle_key_down() {
        let validation = TextInputValidation::LibraryName;

        // Backspace
        let (updated, text) = handle_key_down("Test", "backspace", validation);
        assert!(updated);
        assert_eq!(text, "Tes");

        // Enter (no change)
        let (updated, text) = handle_key_down("Test", "enter", validation);
        assert!(!updated);
        assert_eq!(text, "Test");

        // Printable character
        let (updated, text) = handle_key_down("Test", "你好", validation);
        assert!(updated);
        assert_eq!(text, "Test你好");

        // Invalid character
        let (updated, text) = handle_key_down("Test", "Test\n", validation);
        assert!(!updated);
        assert_eq!(text, "Test");
    }
}
