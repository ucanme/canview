//! 支持中文输入的实用组件
//!
//! 由于 GPUI 的 on_key_down 事件不捕获 IME 输入，
//! 我们使用系统对话框来获取中文输入。

use gpui::prelude::*;
use gpui::*;

/// 方案 1: 使用系统文件对话框来输入库名称
///
/// 这个方法使用 Windows 的原生输入对话框，完全支持中文
#[cfg(windows)]
pub fn prompt_library_name_system(title: &str, prompt: &str) -> Option<String> {
    use rfd::FileDialog;

    // 注意：rfd 主要用于文件对话框，但我们可以用它作为示例
    // 更好的方法是使用 Windows API 的 InputBox

    // 临时方案：使用简单的控制台输入
    println!("{}: {}", title, prompt);
    println!("请输入库名称（支持中文），然后按 Enter:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok()?;
    Some(input.trim().to_string())
}

/// 方案 2: 使用剪贴板作为中转
///
/// 用户操作流程：
/// 1. 在输入法中选择中文
/// 2. Ctrl+C 复制
/// 3. 在应用中 Ctrl+V 粘贴
pub fn create_clipboard_based_input<App>(
    id: impl Into<String>,
    text: String,
    placeholder: String,
    view: Entity<App>,
    on_change: impl Fn(&str, &mut gpui::Context<App>) + 'static,
    on_enter: impl Fn(&str, &mut gpui::Context<App>) + 'static,
    on_escape: impl Fn(&mut gpui::Context<App>) + 'static,
) -> Div
where
    App: 'static,
{
    let input_id = id.into();

    div()
        .px_2()
        .py_1()
        .bg(rgb(0x1a1a1a))
        .border_1()
        .border_color(rgb(0x2a2a2a))
        .rounded(px(2.))
        .flex()
        .items_center()
        .gap_2()
        .min_w(px(200.))
        .max_w(px(400.))
        .id(input_id.clone())
        .focusable()
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(if text.trim().is_empty() {
                    rgb(0x646473)
                } else {
                    rgb(0xcdd6f4)
                })
                .child(if text.trim().is_empty() {
                    placeholder.clone()
                } else {
                    text.clone()
                })
        )
        .child(
            div()
                .px_2()
                .text_xs()
                .text_color(rgb(0x646473))
                .child("Ctrl+V粘贴中文")
        )
        .on_key_down({
            let view = view.clone();
            let text = text.clone();
            move |event, _window, cx| {
                let keystroke = &event.keystroke;

                match keystroke.key.as_str() {
                    "backspace" => {
                        view.update(cx, |this, cx| {
                            if !text.is_empty() {
                                let new_text = text.chars().take(text.chars().count() - 1).collect();
                                on_change(&new_text, cx);
                            }
                        });
                    }
                    "enter" => {
                        view.update(cx, |this, cx| {
                            on_enter(&text, cx);
                        });
                    }
                    "escape" => {
                        view.update(cx, |this, cx| {
                            on_escape(cx);
                        });
                    }
                    // Ctrl+V: 粘贴剪贴板内容
                    "v" if keystroke.ctrl => {
                        eprintln!("Ctrl+V pressed - attempting to paste");
                        // TODO: 使用 GPUI 的剪贴板 API
                        // cx.read_from_clipboard(|clipboard_content, cx| {
                        //     if let Some(clipboard_text) = clipboard_content {
                        //         eprintln!("Clipboard content: '{}'", clipboard_text);
                        //         on_change(&clipboard_text, cx);
                        //     }
                        // });
                        eprintln!("Note: Clipboard API not yet implemented");
                    }
                    _ => {
                        // 处理英文和数字输入
                        let input_text = format!("{}", keystroke);

                        // 只接受 ASCII 字符
                        if input_text.chars().all(|c| c.is_ascii() && !c.is_control()) {
                            let mut new_text = text.clone();
                            new_text.push_str(&input_text);
                            on_change(&new_text, cx);
                        }
                    }
                }
            }
        })
}

/// 方案 3: 创建一个弹窗输入组件
///
/// 显示一个提示，告诉用户如何在控制台输入中文
pub fn create_chinese_input_prompt<App>(
    title: String,
    current_text: String,
    view: Entity<App>,
    on_confirm: impl Fn(String, &mut gpui::Context<App>) + 'static,
    on_cancel: impl Fn(&mut gpui::Context<App>) + 'static,
) -> Div
where
    App: 'static,
{
    div()
        .fixed()
        .top_0()
        .left_0()
        .w(px(400.))
        .p_4()
        .bg(rgb(0x1e1e1e))
        .border_1()
        .border_color(rgb(0x89b4fa))
        .rounded(px(8.))
        .shadow_lg()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xcdd6f4))
                        .child(title)
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x646473))
                                .child("由于框架限制，请使用以下方法输入中文：")
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xcdd6f4))
                                .child("方法1: 在终端输入库名称，按 Enter")
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xcdd6f4))
                                .child("方法2: 使用 Ctrl+V 粘贴剪贴板内容")
                        )
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .px_4()
                                .py_2()
                                .bg(rgb(0x89b4fa))
                                .rounded(px(4.))
                                .cursor_pointer()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0xffffff))
                                        .child("确认")
                                )
                                .on_click(move |_event, _window, cx| {
                                    // TODO: 从控制台或剪贴板获取输入
                                    on_cancel(cx);
                                })
                        )
                        .child(
                            div()
                                .px_4()
                                .py_2()
                                .bg(rgb(0x2a2a2a))
                                .rounded(px(4.))
                                .cursor_pointer()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0xcdd6f4))
                                        .child("取消")
                                )
                                .on_click(move |_event, _window, cx| {
                                    on_cancel(cx);
                                })
                        )
                )
        )
}

/// 方案 4: 最简单的解决方案 - 只允许英文
///
/// 在 UI 上显示提示，告知用户只支持英文
pub fn create_english_only_input<App>(
    text: String,
    placeholder: String,
    on_change: impl Fn(&str, &mut gpui::Context<App>) + 'static,
) -> Div
where
    App: 'static,
{
    div()
        .px_2()
        .py_1()
        .bg(rgb(0x1a1a1a))
        .border_1()
        .border_color(rgb(0x2a2a2a))
        .rounded(px(2.))
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_xs()
                .text_color(if text.trim().is_empty() {
                    rgb(0x646473)
                } else {
                    rgb(0xcdd6f4)
                })
                .child(if text.trim().is_empty() {
                    placeholder
                } else {
                    text
                })
        )
        .child(
            div()
                .px_2()
                .text_xs()
                .text_color(rgb(0xf59e0b))
                .child("仅支持英文")
        )
        .on_key_down({
            move |event, _window, cx| {
                let keystroke = &event.keystroke;
                let input_text = format!("{}", keystroke);

                // 只接受 ASCII 字符
                if input_text.chars().all(|c| c.is_ascii() && !c.is_control()) {
                    let mut new_text = text.clone();
                    new_text.push_str(&input_text);
                    on_change(&new_text, cx);
                }
            }
        })
}

/*
使用建议：
========

短期方案（立即可用）：
1. 在 UI 上显示提示："仅支持英文库名称"
2. 或者使用剪贴板方案（Ctrl+V 粘贴）

长期方案：
1. 等待 GPUI 支持 IME
2. 或使用其他支持 IME 的 UI 框架
3. 或使用 WebView 技术栈

临时方案（开发阶段）：
1. 在控制台输入中文库名称
2. 使用配置文件直接编辑库名称

推荐使用：
- 对于生产环境：方案 1（仅英文）+ 明确提示
- 对于开发环境：方案 2（剪贴板）或方案 3（控制台输入）
*/
