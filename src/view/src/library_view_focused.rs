// 修复版本：强制焦点 + 完整的事件监听
//
// 这个版本会：
// 1. 自动获取焦点
// 2. 显示焦点状态
// 3. 捕获所有键盘事件

// 在 library_view.rs 的 render_library_header 函数中
// 将输入框部分替换为以下代码：

fn render_library_header(
    cx: &mut gpui::Context<crate::CanViewApp>,
    new_library_name: String,
    cursor_position: usize,
) -> impl IntoElement {
    let view = cx.entity().clone();
    let is_editing = !new_library_name.is_empty();

    div()
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xcdd6f4))
                        .child("Signal Libraries"),
                )
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .bg(rgb(0x3b82f6))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .hover(|style| style.bg(rgb(0x2563eb)))
                        .on_mouse_down(gpui::MouseButton::Left, {
                            let view = view.clone();
                            move |_event, _window, cx| {
                                view.update(cx, |this, cx| {
                                    this.new_library_name = String::from(" ");
                                    this.library_cursor_position = 1;
                                    cx.notify();
                                });
                            }
                        })
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xffffff))
                                .child("+ New"),
                        ),
                )
                .when(is_editing, |d| {
                    d.child(
                        div()
                            .px_2()
                            .py_1()
                            .bg(rgb(0x1a1a1a))
                            .border_1()
                            .border_color(rgb(0x89b4fa)) // 蓝色边框 = 有焦点
                            .rounded(px(2.))
                            .flex()
                            .items_center()
                            .min_w(px(100.))
                            .max_w(px(200.))
                            .cursor_text()
                            .id("library_name_input")
                            .focusable()
                            // ========== 新增：自动聚焦 ==========
                            .on_mount({
                                let view = view.clone();
                                move |element, cx| {
                                    // 元素挂载后自动获取焦点
                                    element.focus(cx);
                                    eprintln!("Input mounted and focused!");
                                }
                            })
                            // ======================================
                            .when(new_library_name.trim().is_empty(), |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_0()
                                        .child(
                                            div()
                                                .w(px(2.))
                                                .h(px(14.))
                                                .bg(rgb(0x89b4fa)), // 光标
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x646473))
                                                .child("Library name..."),
                                        ),
                                )
                            })
                            .when(!new_library_name.trim().is_empty(), |d| {
                                let text = new_library_name.chars().collect::<Vec<_>>();
                                let cursor_pos = cursor_position.min(text.len());
                                let before_cursor: String = text[..cursor_pos].iter().collect();
                                let after_cursor: String = text[cursor_pos..].iter().collect();

                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_0()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0xcdd6f4))
                                                .child(before_cursor),
                                        )
                                        .child(
                                            div()
                                                .w(px(2.))
                                                .h(px(14.))
                                                .bg(rgb(0x89b4fa)), // 光标
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0xcdd6f4))
                                                .child(after_cursor),
                                        ),
                                )
                            })
                            .on_click({
                                let view = view.clone();
                                move |_event, _window, cx| {
                                    eprintln!("Input clicked!");
                                    // 点击时确保获得焦点
                                    cx.focus_self();
                                }
                            })
                            .on_key_down({
                                let view = view.clone();
                                let text = new_library_name.clone();
                                move |event, _window, cx| {
                                    // ========== 增强的调试输出 ==========
                                    let keystroke = &event.keystroke;
                                    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                    eprintln!("KEY DOWN EVENT FIRED!");
                                    eprintln!("  keystroke (Display): '{}'", keystroke);
                                    eprintln!("  keystroke.key: '{}'", keystroke.key.as_str());
                                    eprintln!("  keystroke.to_string(): '{}'", keystroke.to_string());
                                    eprintln!("  Has focus: true (event was fired)");
                                    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                                    // ========================================

                                    match keystroke.key.as_str() {
                                        "backspace" => {
                                            view.update(cx, |this, cx| {
                                                if this.library_cursor_position > 0 {
                                                    let mut chars: Vec<char> =
                                                        this.new_library_name.chars().collect();
                                                    chars.remove(this.library_cursor_position - 1);
                                                    this.new_library_name = chars.into_iter().collect();
                                                    this.library_cursor_position -= 1;
                                                    eprintln!(
                                                        "After backspace: '{}'",
                                                        this.new_library_name
                                                    );
                                                    cx.notify();
                                                }
                                            });
                                        }
                                        "enter" => {
                                            view.update(cx, |this, cx| {
                                                if !this.new_library_name.trim().is_empty() {
                                                    if let Err(e) = this.library_manager.create_library(
                                                        &this.new_library_name.trim(),
                                                        &this.app_config.config_dir,
                                                    ) {
                                                        eprintln!("Failed to create library: {}", e);
                                                    } else {
                                                        this.new_library_name = String::new();
                                                        this.library_cursor_position = 0;
                                                    }
                                                    cx.notify();
                                                }
                                            });
                                        }
                                        "escape" => {
                                            view.update(cx, |this, cx| {
                                                this.new_library_name = String::new();
                                                this.library_cursor_position = 0;
                                                cx.notify();
                                            });
                                        }
                                        "left" => {
                                            view.update(cx, |this, cx| {
                                                if this.library_cursor_position > 0 {
                                                    this.library_cursor_position -= 1;
                                                    cx.notify();
                                                }
                                            });
                                        }
                                        "right" => {
                                            view.update(cx, |this, cx| {
                                                let text_len =
                                                    this.new_library_name.chars().count();
                                                if this.library_cursor_position < text_len {
                                                    this.library_cursor_position += 1;
                                                    cx.notify();
                                                }
                                            });
                                        }
                                        "home" => {
                                            view.update(cx, |this, cx| {
                                                this.library_cursor_position = 0;
                                                cx.notify();
                                            });
                                        }
                                        "end" => {
                                            view.update(cx, |this, cx| {
                                                this.library_cursor_position =
                                                    this.new_library_name.chars().count();
                                                cx.notify();
                                            });
                                        }
                                        _ => {
                                            // 尝试多种方式获取输入
                                            let input_candidates = vec![
                                                format!("{}", keystroke),
                                                keystroke.key.as_str().to_string(),
                                                keystroke.to_string(),
                                            ];

                                            eprintln!("Trying {} input candidates", input_candidates.len());

                                            for (i, input_text) in input_candidates.iter().enumerate() {
                                                eprintln!("  Candidate {}: '{}'", i, input_text);

                                                if input_text.is_empty() {
                                                    eprintln!("    → Empty, skipping");
                                                    continue;
                                                }

                                                // 检查是否是控制键
                                                let lower = input_text.to_lowercase();
                                                if lower.starts_with("backspace")
                                                    || lower.starts_with("enter")
                                                    || lower.starts_with("escape")
                                                    || lower.starts_with("left")
                                                    || lower.starts_with("right")
                                                    || lower.starts_with("up")
                                                    || lower.starts_with("down")
                                                    || lower.starts_with("home")
                                                    || lower.starts_with("end")
                                                {
                                                    eprintln!("    → Control key, skipping");
                                                    continue;
                                                }

                                                // 检查是否有控制字符
                                                if input_text.chars().any(|c| c.is_control()) {
                                                    eprintln!("    → Contains control char, skipping");
                                                    continue;
                                                }

                                                // 验证字符
                                                let is_valid_char = |c: char| -> bool {
                                                    !c.is_control()
                                                        && (c.is_ascii_alphanumeric()
                                                            || c == ' '
                                                            || !c.is_ascii())
                                                };

                                                let all_valid = input_text.chars().all(is_valid_char);

                                                if all_valid {
                                                    eprintln!("    → ✓ ACCEPTING!");
                                                    view.update(cx, |this, cx| {
                                                        let mut chars: Vec<char> =
                                                            this.new_library_name.chars().collect();
                                                        for (i, ch) in input_text.chars().enumerate()
                                                        {
                                                            chars.insert(
                                                                this.library_cursor_position + i,
                                                                ch,
                                                            );
                                                        }
                                                        this.new_library_name =
                                                            chars.into_iter().collect();
                                                        this.library_cursor_position +=
                                                            input_text.chars().count();
                                                        eprintln!(
                                                            "Library name is now: '{}'",
                                                            this.new_library_name
                                                        );
                                                        cx.notify();
                                                    });
                                                    break;
                                                } else {
                                                    eprintln!("    → ✗ Invalid characters");
                                                }
                                            }
                                        }
                                    }
                                }
                            }),
                    )
                }),
        )
        .child(
            div()
                .px_2()
                .text_xs()
                .text_color(rgb(0x646473))
                .cursor_pointer()
                .hover(|style| style.text_color(rgb(0xcdd6f4)))
                .on_mouse_down(gpui::MouseButton::Left, {
                    let view = view.clone();
                    move |_event, _window, cx| {
                        view.update(cx, |this, cx| {
                            if !this.new_library_name.trim().is_empty() {
                                this.new_library_name = String::new();
                                this.library_cursor_position = 0;
                                cx.notify();
                            }
                        });
                    }
                })
                .child("Cancel"),
        )
}

/*
使用说明：
==========

1. 将上面的代码复制到 library_view.rs
2. 替换现有的 render_library_header 函数
3. 重新编译运行

4. 测试步骤：
   a) 点击 "+ New" 按钮
   b) 输入框应该自动获得焦点并显示蓝色边框
   c) 终端应该显示 "Input mounted and focused!"
   d) 输入任何字符（中文或英文）
   e) 查看终端输出

预期结果：
-----------
如果输入英文字母 'a'，终端应该显示：
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
KEY DOWN EVENT FIRED!
  keystroke (Display): 'a'
  keystroke.key: 'a'
  keystroke.to_string(): 'a'
  Has focus: true (event was fired)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

如果输入中文，我们可能会看到：
- 单个拼音字母
- 或者完全没有输出（如果 GPUI 不捕获 IME）

如果仍然没有输出：
--------------------
1. 检查输入框是否真的有焦点（蓝色边框）
2. 尝试点击输入框
3. 尝试按 Tab 键切换焦点
4. 查看是否有 "Input clicked!" 输出
*/
