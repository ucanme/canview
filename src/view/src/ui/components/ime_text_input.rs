//! 使用 GPUI EntityInputHandler 的中文输入支持
//!
//! 这是 Zed IDE 支持中文输入的正确方式！

use gpui::*;
use std::ops::Range;

/// 支持中文 IME 输入的文本框状态
pub struct ImeTextInputState {
    pub text: String,
    pub cursor_position: usize,
    pub selection_range: Option<Range<usize>>,
    pub marked_range: Option<Range<usize>>, // IME 组合窗口文本范围
}

impl Default for ImeTextInputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            selection_range: None,
            marked_range: None,
        }
    }
}

impl ImeTextInputState {
    pub fn new(text: String) -> Self {
        let cursor_position = text.chars().count();
        Self {
            text,
            cursor_position,
            selection_range: None,
            marked_range: None,
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
        self.selection_range = None;
        self.marked_range = None;
    }
}

// Note: EntityInputHandler should be implemented by CanViewApp (in entity_input_handler.rs)
// ImeTextInputState is just a data structure that stores input state

