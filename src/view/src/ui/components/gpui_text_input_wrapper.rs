//! 使用 GPUI 内置功能的中文输入支持
//!
//! GPUI 的 on_key_down 事件可能无法正确捕获 IME 输入。
//! 这个模块提供了替代方案。

use gpui::prelude::*;
use gpui::*;

/// 方案 1: 使用 GPUI 的 TextInput 组件（如果可用）
///
/// GPUI 可能有内置的 TextInput 组件，它应该自动处理 IME 输入
///
/// 示例：
/// ```rust
/// use gpui::TextInput;
///
/// let text_input = TextInput::new()
///     .placeholder("输入库名称...")
///     .text(state.text.clone())
///     .on_change(cx.listener(|this, new_text, cx| {
///         this.library_name = new_text;
///         cx.notify();
///     }))
///     .build(cx);
/// ```

/// 方案 2: 使用 content_size 事件监听文本变化
///
/// 某些版本的 GPUI 使用 content_size 事件来传递 IME 输入
///
/// 示例：
/// ```rust
/// div()
///     .id("input")
///     .focusable()
///     .on_content_size_change(cx.listener(|this, size, cx| {
///         // 处理内容变化
///     }))
///     .on_key_down(...)
/// ```

/// 方案 3: 监听多个事件类型
///
/// IME 输入可能通过不同的事件传递
///
/// 示例：
/// ```rust
/// div()
///     .id("input")
///     .focusable()
///     .on_key_down(|event, window, cx| {
///         // 处理键盘输入
///     })
///     .on_key_press(|event, window, cx| {
///         // 处理按键事件
///     })
///     .on_text(|text, cx| {
///         // 处理文本输入（IME）
///     })
/// ```

/// 方案 4: 检查当前输入法状态
///
/// 示例：
/// ```rust
/// div()
///     .id("input")
///     .focusable()
///     .on_key_down({
///         let view = view.clone();
///         move |event, _window, cx| {
///             // 检查是否有 IME 组合文本
///             let has_composition = event.keystroke.is_composing;
///
///             if has_composition {
///                 // IME 正在输入中，不处理
///                 return;
///             }
///
///             // 处理正常输入
///             let keystroke = format!("{}", event.keystroke);
///             // ...
///         }
///     })
/// ```

/// 方案 5: 直接使用 DOM/平台 API
///
/// 在某些情况下，可能需要直接使用平台 API 来处理 IME
///
/// Windows 示例：
/// ```rust
/// #[cfg(windows)]
/// use winapi::um::imm::*;
///
/// // 获取 IME 组合窗口文本
/// fn get_ime_composition_text() -> String {
///     // Windows IME API 调用
/// }
/// ```

/// ============================================================================
/// 推荐的解决方案
/// ============================================================================

/// 临时解决方案：修改验证逻辑，移除对单个字符的依赖

pub fn create_chinese_friendly_input<App>(
    state_text: String,
    placeholder: String,
    view: Entity<App>,
    on_change: impl Fn(&str, &mut gpui::Context<App>) + 'static,
    on_submit: impl Fn(&str, &mut gpui::Context<App>) + 'static,
    on_cancel: impl Fn(&mut gpui::Context<App>) + 'static,
    cx: &mut gpui::Context<App>,
) -> Div
where
    App: 'static,
{
    let view_clone = view.clone();

    div()
        .px_2()
        .py_1()
        .bg(rgb(0x1a1a1a))
        .border_1()
        .border_color(rgb(0x2a2a2a))
        .rounded(px(2.))
        .flex()
        .items_center()
        .min_w(px(100.))
        .max_w(px(200.))
        .cursor_text()
        .id("chinese_input")
        .focusable()
        .when(state_text.trim().is_empty(), |d| {
            d.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x646473))
                    .child(placeholder),
            )
        })
        .when(!state_text.trim().is_empty(), |d| {
            d.child(
                div()
                    .text_xs()
                    .text_color(rgb(0xcdd6f4))
                    .child(state_text.clone()),
            )
        })
        .on_key_down({
            let view = view.clone();
            let text = state_text.clone();
            move |event, _window, cx| {
                // 修改：使用 event.keystroke 的原始键值
                let keystroke = &event.keystroke;

                // 调试：打印所有按键信息
                eprintln!("Key event:");
                eprintln!("  keystroke (Display): '{}'", keystroke);
                eprintln!("  key: '{}'", keystroke.key.as_str());

                match keystroke.key.as_str() {
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
                    _ => {
                        // 尝试使用不同的方式获取输入文本
                        let input_text = if let Some(text) = keystroke.key.as_str().chars().next() {
                            // 如果 key 包含有效字符
                            text.to_string()
                        } else {
                            // 否则使用 display 表示
                            format!("{}", keystroke)
                        };

                        // 验证并插入
                        let is_valid_char = |c: char| -> bool {
                            !c.is_control() && (c.is_ascii_alphanumeric() || c == ' ' || !c.is_ascii())
                        };

                        let all_valid = input_text.chars().all(is_valid_char);

                        if all_valid && !input_text.is_empty() {
                            eprintln!("Inserting text: '{}'", input_text);
                            view.update(cx, |this, cx| {
                                let mut new_text = text.clone();
                                new_text.push_str(&input_text);
                                on_change(&new_text, cx);
                            });
                        }
                    }
                }
            }
        })
}

// ============================================================================
/// 调试和诊断
/// ============================================================================

/// 添加详细的调试输出来诊断中文输入问题
pub fn create_debug_input<App>(
    state_text: String,
    view: Entity<App>,
    cx: &mut gpui::Context<App>,
) -> Div
where
    App: 'static,
{
    div()
        .px_2()
        .py_1()
        .border_1()
        .border_color(rgb(0xff0000))
        .min_w(px(300.))
        .id("debug_input")
        .focusable()
        .child(div().text_sm().child(format!("Debug: '{}'", state_text)))
        .on_key_down({
            let view = view.clone();
            move |event, _window, cx| {
                let keystroke = &event.keystroke;

                // 详细的调试信息
                eprintln!("╔════════════════════════════════════════════════════════════╗");
                eprintln!("║            KEY EVENT DEBUG INFORMATION                      ║");
                eprintln!("╚════════════════════════════════════════════════════════════╝");
                eprintln!("keystroke Display:  '{}'", keystroke);
                eprintln!("keystroke.key:      '{}'", keystroke.key.as_str());
                eprintln!("keystroke Modifiers: shift={}, ctrl={}, alt={}, meta={}",
                    keystroke.shift,
                    keystroke.ctrl,
                    keystroke.alt,
                    keystroke.meta
                );

                // 检查所有可能的字段
                if keystroke.key.as_str().len() > 1 {
                    eprintln!("Multi-char key detected!");
                    for (i, ch) in keystroke.key.as_str().chars().enumerate() {
                        eprintln!("  char[{}]: '{}' (is_ascii={}, is_control={})",
                            i, ch, ch.is_ascii(), ch.is_control());
                    }
                }

                eprintln!("═════════════════════════════════════════════════════════════\n");

                // 尝试不同的方式获取文本
                let method1 = format!("{}", keystroke);
                let method2 = keystroke.key.as_str().to_string();
                let method3 = keystroke.to_string();

                eprintln!("Method 1 (format!): '{}'", method1);
                eprintln!("Method 2 (key.as_str()): '{}'", method2);
                eprintln!("Method 3 (to_string()): '{}'", method3);
                eprintln!();
            }
        })
}

// ============================================================================
/// 使用建议
/// ============================================================================

/*
1. 运行调试版本查看实际输入

    在你的代码中使用 create_debug_input，然后：
    - 切换到中文输入法
    - 输入一些汉字
    - 查看终端输出

2. 根据调试输出选择正确的方法

    如果 method1/2/3 中有一个包含中文，就使用那个方法

3. 检查 GPUI 版本

    不同版本的 GPUI 可能有不同的 IME 处理方式

4. 考虑使用平台原生控件

    如果 GPUI 的文本输入不支持 IME，可以考虑：
    - 使用平台原生的文本输入控件
    - 使用 webview 技术栈
    - 使用其他 UI 框架
*/
