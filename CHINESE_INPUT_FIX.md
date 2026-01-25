# 修复中文输入的补丁

## 📋 修复步骤

1. **备份你的代码**
   ```bash
   cp src/view/src/library_view.rs src/view/src/library_view.rs.backup
   ```

2. **应用以下修改**

在 `library_view.rs` 的 `render_library_header` 函数中，找到 `.on_key_down(...)` 部分，替换为以下代码：

```rust
.on_key_down({
    let view = view.clone();
    let text = new_library_name.clone();
    move |event, _window, cx| {
        // ========== 调试输出 ==========
        let keystroke = &event.keystroke;
        eprintln!("Key Event: keystroke='{}', key='{}', to_string='{}'",
            keystroke, keystroke.key.as_str(), keystroke.to_string());
        // ================================

        match keystroke.key.as_str() {
            "backspace" => {
                view.update(cx, |this, cx| {
                    if this.library_cursor_position > 0 {
                        let mut chars: Vec<char> = this.new_library_name.chars().collect();
                        chars.remove(this.library_cursor_position - 1);
                        this.new_library_name = chars.into_iter().collect();
                        this.library_cursor_position -= 1;
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
                    let text_len = this.new_library_name.chars().count();
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
                    this.library_cursor_position = this.new_library_name.chars().count();
                    cx.notify();
                });
            }
            _ => {
                // ========== 修复：改进的输入处理 ==========

                // 尝试多种方式获取输入文本
                let input_candidates = vec![
                    format!("{}", keystroke),
                    keystroke.key.as_str().to_string(),
                    keystroke.to_string(),
                ];

                // 找到第一个非空、非控制字符的输入
                for input_text in input_candidates {
                    if input_text.is_empty() {
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
                        continue;
                    }

                    // 检查是否有控制字符
                    if input_text.chars().any(|c| c.is_control()) {
                        continue;
                    }

                    // 验证字符
                    let is_valid_char = |c: char| -> bool {
                        !c.is_control() && (c.is_ascii_alphanumeric() || c == ' ' || !c.is_ascii())
                    };

                    let all_valid = input_text.chars().all(is_valid_char);

                    if all_valid {
                        eprintln!("ACCEPTING input: '{}'", input_text);
                        view.update(cx, |this, cx| {
                            let mut chars: Vec<char> = this.new_library_name.chars().collect();
                            for (i, ch) in input_text.chars().enumerate() {
                                chars.insert(this.library_cursor_position + i, ch);
                            }
                            this.new_library_name = chars.into_iter().collect();
                            this.library_cursor_position += input_text.chars().count();
                            eprintln!("Library name is now: '{}'", this.new_library_name);
                            cx.notify();
                        });
                        break; // 成功处理后退出循环
                    } else {
                        eprintln!("REJECTED: Invalid characters in '{}'", input_text);
                    }
                }
                // ===========================================
            }
        }
    }
})
```

3. **测试修复**

```bash
cd src/view
cargo run
```

4. **输入中文并查看终端输出**

如果输入成功，你应该看到：
```
Key Event: keystroke='测试', key='测试', to_string='测试'
ACCEPTING input: '测试'
Library name is now: ' 测试'
```

---

## 🔍 如果仍然无法输入

### 1. 确认输入法

确保你使用的是中文输入法（不是英文键盘）：
- Windows: `Win + Space` 切换输入法
- 确认任务栏显示 "中文" 或 "CH"

### 2. 测试其他字符

- 输入英文字母：`test`
- 输入数字：`123`
- 输入符号：`@#$`

如果其他字符可以输入，但中文不行，说明是 IME 问题。

### 3. 查看完整调试输出

请复制并粘贴以下信息：

```
当你尝试输入"测试"时，终端显示了什么？
```

---

## 🚀 最终方案：使用 GPUI TextInput

如果上述修复仍然无效，建议使用 GPUI 的内置 TextInput 组件。

### 新的实现方式

创建新文件 `src/view/src/ui/components/chinese_text_input.rs`：

```rust
//! 支持中文输入的文本框组件

use gpui::prelude::*;
use gpui::*;

pub fn render_chinese_input<App>(
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
    // TODO: 使用 GPUI 的 TextInput 组件（如果可用）
    // 或者查找 GPUI 文档中关于 IME 支持的说明

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
        .id(id.into())
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
                    text.clone()
                })
        )
        // 关键：需要找到正确的事件来捕获 IME 输入
        .on_key_down({
            let view = view.clone();
            let text = text.clone();
            move |event, _window, cx| {
                // 调试
                eprintln!("Event: {:?}", event);

                // ... 处理逻辑
            }
        })
}
```

---

## 📮 需要帮助？

如果上述步骤都无法解决问题，请提供：

1. **完整的终端输出**（当你输入中文时）
2. **GPUI 版本**（从 Cargo.toml）
3. **操作系统和输入法信息**
4. **你尝试过的所有步骤**

我会根据这些信息提供更具体的解决方案。
