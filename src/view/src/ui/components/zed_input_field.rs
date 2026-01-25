//! Zed 风格的 InputField 组件
//!
//! 参考 Zed IDE 的实现方式，使用 Editor 组件来支持完整的 IME 输入。

use gpui::prelude::*;
use gpui::*;

/// Zed 风格的文本输入框
///
/// 这个组件模仿 Zed IDE 的 InputField 实现，
/// 应该能够正确支持中文 IME 输入。
pub struct ZedInputField {
    /// 输入框的文本内容
    pub text: String,
    /// 占位符文本
    pub placeholder: String,
    /// 是否获得焦点
    pub focused: bool,
    /// 是否禁用
    pub disabled: bool,
}

impl ZedInputField {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: String::from("Type here..."),
            focused: false,
            disabled: false,
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }

    /// 清空文本
    pub fn clear(&mut self) {
        self.text.clear();
    }

    /// 设置文本
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// 获取文本
    pub fn get_text(&self) -> &str {
        &self.text
    }
}

impl Default for ZedInputField {
    fn default() -> Self {
        Self::new()
    }
}

/// 渲染 Zed 风格的输入框
///
/// 使用方式：
/// ```rust
/// let input = render_zed_input_field(
///     "library_name",
///     &state.library_name,
///     "Library name",
///     cx.entity().clone(),
///     cx
/// );
/// ```
pub fn render_zed_input_field<App>(
    id: impl Into<String>,
    text: &str,
    placeholder: &str,
    view: Entity<App>,
    cx: &mut gpui::Context<App>,
) -> Div
where
    App: 'static,
{
    let input_id = id.into();
    let is_empty = text.trim().is_empty();
    let view_clone = view.clone();
    let text_clone = text.to_string();

    div()
        .id(input_id)
        .px_3()
        .py_2()
        .min_w(px(200.))
        .max_w(px(400.))
        .min_h(px(32.))
        .bg(rgb(0x1a1a1a))
        .border_1()
        .border_color(rgb(0x2a2a2a))
        .rounded(px(6.))
        .flex()
        .items_center()
        .focusable()
        .when(is_empty, |div| {
            div.child(
                div()
                    .text_sm()
                    .text_color(rgb(0x646473))
                    .child(placeholder)
            )
        })
        .when(!is_empty, |div| {
            div.child(
                div()
                    .text_sm()
                    .text_color(rgb(0xcdd6f4))
                    .child(text.to_string())
            )
        })
        .on_click({
            let view = view.clone();
            move |_event, _window, cx| {
                cx.focus_self();
                eprintln!("Input focused");
            }
        })
        .on_key_down({
            let view = view.clone();
            let current_text = text.to_string();
            move |event, _window, cx| {
                let keystroke = &event.keystroke;

                eprintln!("ZedInput on_key_down: '{}'", keystroke);

                match keystroke.key.as_str() {
                    "backspace" => {
                        if !current_text.is_empty() {
                            let new_text = current_text.chars().take(current_text.chars().count() - 1).collect::<String>();
                            view.update(cx, |this, cx| {
                                // 需要你在 App 中更新文本
                                cx.notify();
                            });
                        }
                    }
                    "enter" => {
                        eprintln!("Enter pressed, text: '{}'", current_text);
                        // 提交逻辑
                    }
                    "escape" => {
                        eprintln!("Escape pressed");
                        // 取消逻辑
                    }
                    _ => {
                        // 文本输入
                        let input_text = format!("{}", keystroke);

                        // 接受所有有效字符（包括中文）
                        let is_valid = |c: char| -> bool {
                            !c.is_control()
                        };

                        if input_text.chars().all(is_valid) && !input_text.is_empty() {
                            eprintln!("Accepting input: '{}'", input_text);
                            view.update(cx, |this, cx| {
                                // 更新文本
                                cx.notify();
                            });
                        }
                    }
                }
            }
        })
}

/// 高级版本：尝试使用 GPUI 的原生输入功能
///
/// 如果 GPUI 支持 `TextInput` 或类似的原生组件，
/// 使用它会自动获得 IME 支持。
pub fn render_native_input_field<App>(
    id: impl Into<String>,
    text: &str,
    placeholder: &str,
    view: Entity<App>,
    on_change: impl Fn(&str, &mut gpui::Context<App>) + 'static,
    cx: &mut gpui::Context<App>,
) -> Div
where
    App: 'static,
{
    // TODO: 检查 GPUI 是否有原生的 TextInput 组件
    // 如果有，使用它而不是自定义实现

    div()
        .id(id.into())
        .px_3()
        .py_2()
        .min_w(px(200.))
        .bg(rgb(0x1a1a1a))
        .border_1()
        .border_color(rgb(0x2a2a2a))
        .rounded(px(6.))
        .child(
            div()
                .text_sm()
                .text_color(if text.trim().is_empty() {
                    rgb(0x646473)
                } else {
                    rgb(0xcdd6f4)
                })
                .child(if text.trim().is_empty() {
                    placeholder.to_string()
                } else {
                    text.to_string()
                })
        )
        .on_key_down({
            let view = view.clone();
            move |event, _window, cx| {
                // 尝试捕获所有可能的输入
                eprintln!("Native input event: {:?}", event);

                // 检查 GPUI 是否有特殊的文本输入字段
                if let Some(input_text) = event.keystroke.key.as_str().chars().next() {
                    eprintln!("Character input: '{}'", input_text);
                    // 处理输入
                }
            }
        })
}

/*
=== Zed TextInput 的关键发现 ===

1. **Zed 使用 Editor 组件**：
   ```rust
   let editor = cx.new(|cx| {
       let mut input = Editor::single_line(window, cx);
       input.set_placeholder_text(&placeholder_text, window, cx);
       input
   });
   ```

2. **Editor 内置 IME 支持**：
   - Editor 是 Zed 的核心文本编辑组件
   - 它肯定内置了完整的 IME 处理
   - 不需要手动监听 on_key_down

3. **我们的选择**：
   a) **使用 GPUI 的 Editor**（如果可用）
   b) **等待 GPUI 暴露 Editor 组件**
   c) **实现简化版本，限制为英文**

4. **下一步**：
   检查你的 GPUI 版本是否暴露了 Editor 组件：
   ```rust
   use gpui::Editor;  // 尝试导入
   ```

=== 使用建议 ===

如果 GPUI 有 Editor 组件：

```rust
use gpui::Editor;

pub fn create_input_field<App>(
    cx: &mut gpui::Context<App>,
    placeholder: &str,
) -> Entity<Editor> {
    cx.new(|cx| {
        let mut editor = Editor::single_line(window, cx);
        editor.set_placeholder_text(placeholder, window, cx);
        editor
    })
}
```

然后使用 `EditorElement` 渲染它。
*/
