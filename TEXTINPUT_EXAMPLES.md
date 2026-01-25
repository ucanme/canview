# TextInput 组件使用示例

## 概述

`TextInput` 组件是一个轻量级的文本输入 UI 组件，支持：
- ✅ 多字符输入（IME 支持）
- ✅ 灵活的字符验证
- ✅ 一致的视觉样式

**重要提示**：这是一个**展示型组件**，只负责渲染和基础键盘事件监听。实际的状态更新需要由父组件处理。

## 快速开始

### 基础用法

```rust
use crate::ui::components::{TextInputBuilder, TextInputValidation};
use gpui::{prelude::*, *};

// 在你的渲染函数中
fn render_my_input(cx: &mut Context<MyApp>) -> impl IntoElement {
    TextInputBuilder::new()
        .text(this.my_text.clone())
        .placeholder("请输入...")
        .validation(TextInputValidation::LibraryName)
        .focused(true)
        .build("my_input_id")
}
```

### 完整示例：带状态管理

```rust
fn render_library_input(
    text: String,
    is_editing: bool,
    cx: &mut Context<CanViewApp>
) -> impl IntoElement {
    let view = cx.entity().clone();
    
    div()
        .flex()
        .items_center()
        .gap_2()
        .when(!is_editing, |d| {
            // 显示按钮
            d.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x646473))
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, {
                        let view = view.clone();
                        move |_event, _window, cx| {
                            view.update(cx, |this, cx| {
                                this.is_editing = true;
                                this.input_text = String::new();
                                cx.notify();
                            });
                        }
                    })
                    .child("+ New")
            )
        })
        .when(is_editing, |d| {
            // 显示输入框
            d.child(
                TextInputBuilder::new()
                    .text(text.clone())
                    .placeholder("库名称...")
                    .validation(TextInputValidation::LibraryName)
                    .focused(true)
                    .build("library_input")
            )
            .child(
                div()
                    .text_xs()
                    .cursor_pointer()
                    .on_mouse_down(gpui::MouseButton::Left, {
                        let view = view.clone();
                        move |_event, _window, cx| {
                            view.update(cx, |this, cx| {
                                this.create_library();
                                this.is_editing = false;
                                cx.notify();
                            });
                        }
                    })
                    .child("Create")
            )
        })
}
```

## 状态管理示例

### 方式 1：直接在父组件处理键盘事件

```rust
fn render_with_inline_handlers(
    text: String,
    cx: &mut Context<CanViewApp>
) -> impl IntoElement {
    let view = cx.entity().clone();
    
    div()
        .child(
            div()
                .px_2()
                .py_1()
                .bg(rgb(0x1a1a1a))
                .border_1()
                .border_color(rgb(0x89b4fa))
                .rounded(px(2.))
                .child(
                    div()
                        .text_xs()
                        .text_color(if text.is_empty() { rgb(0x646473) } else { rgb(0xcdd6f4) })
                        .child(if text.is_empty() { "输入..." } else { text.as_str() })
                )
                .on_key_down({
                    let view = view.clone();
                    move |event, _window, cx| {
                        let keystroke = format!("{}", event.keystroke);
                        
                        view.update(cx, |this, cx| {
                            use crate::ui::components::TextInputValidation;
                            
                            match keystroke.as_str() {
                                "backspace" => {
                                    this.input_text.pop();
                                    cx.notify();
                                }
                                "enter" => {
                                    this.submit();
                                    cx.notify();
                                }
                                "escape" => {
                                    this.cancel();
                                    cx.notify();
                                }
                                _ => {
                                    // 多字符输入支持（IME）
                                    if keystroke.len() > 0 
                                        && !keystroke.to_lowercase().starts_with("backspace")
                                        && keystroke.chars().all(|c| !c.is_control()) {
                                        
                                        // 验证字符
                                        let is_valid = |c: char| -> bool {
                                            !c.is_control() && (c.is_ascii_alphanumeric() || c == ' ' || !c.is_ascii())
                                        };
                                        
                                        if keystroke.chars().all(is_valid) {
                                            this.input_text.push_str(&keystroke);
                                            eprintln!("Inserted: '{}'", keystroke);
                                            cx.notify();
                                        }
                                    }
                                }
                            }
                        });
                    }
                })
        )
}
```

### 方式 2：使用辅助函数

```rust
use crate::ui::components::{TextInputBuilder, TextInputValidation, handle_key_down};

fn render_with_helper(
    text: String,
    cx: &mut Context<CanViewApp>
) -> impl IntoElement {
    let view = cx.entity().clone();
    
    div()
        .child(
            TextInputBuilder::new()
                .text(text.clone())
                .placeholder("输入...")
                .validation(TextInputValidation::LibraryName)
                .build("my_input")
        )
        .on_key_down({
            let view = view.clone();
            move |event, _window, cx| {
                let keystroke = format!("{}", event.keystroke);
                
                view.update(cx, |this, cx| {
                    let (should_update, new_text) = handle_key_down(
                        &this.input_text,
                        &keystroke,
                        TextInputValidation::LibraryName
                    );
                    
                    if should_update {
                        this.input_text = new_text;
                        cx.notify();
                    }
                    
                    // 处理特殊键
                    match keystroke.as_str() {
                        "enter" => this.submit(),
                        "escape" => this.cancel(),
                        _ => {}
                    }
                });
            }
        })
}
```

## 验证模式示例

### LibraryName - 支持中文和所有 Unicode

```rust
TextInputBuilder::new()
    .validation(TextInputValidation::LibraryName)
    .build("library_input")

// ✅ 有效输入：
// - "测试CAN信号库"
// - "Test测试库123"
// - "📊 数据分析库"
// - "CAN测试库2024"
```

### VersionName - 仅 ASCII 和版本号字符

```rust
TextInputBuilder::new()
    .validation(TextInputValidation::VersionName)
    .build("version_input")

// ✅ 有效输入：
// - "v1.0.0"
// - "version_1.2"
// - "release-2.0"
// - "v1.2.3-beta"
```

### Custom - 自定义验证

```rust
// 只允许数字
TextInputBuilder::new()
    .validation(TextInputValidation::Custom(|c| c.is_ascii_digit()))
    .build("number_input")

// 只允许大写字母和数字
TextInputBuilder::new()
    .validation(TextInputValidation::Custom(|c| {
        c.is_ascii_uppercase() || c.is_ascii_digit()
    }))
    .build("id_input")

// 允许字母、数字、@ 和 .
TextInputBuilder::new()
    .validation(TextInputValidation::Custom(|c| {
        c.is_ascii_alphanumeric() || c == '@' || c == '.'
    }))
    .build("email_input")
```

## 实际应用示例

### 示例 1：创建库对话框

```rust
pub fn render_create_library_dialog(
    is_open: bool,
    library_name: String,
    cx: &mut Context<CanViewApp>
) -> impl IntoElement {
    let view = cx.entity().clone();
    
    div()
        .when(is_open, |d| {
            d.child(
                div()
                    .fixed()
                    .top_8()
                    .left_8()
                    .w(px(400.))
                    .p_4()
                    .bg(rgb(0x1a1a1a))
                    .border_1()
                    .border_color(rgb(0x89b4fa))
                    .rounded(px(8.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("创建新库")
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
                                    .child("库名称")
                            )
                            .child(
                                TextInputBuilder::new()
                                    .text(library_name.clone())
                                    .placeholder("支持中文、英文、数字...")
                                    .validation(TextInputValidation::LibraryName)
                                    .max_w(px(380.))
                                    .build("new_library_input")
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .justify_end()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(rgb(0x89b4fa))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view = view.clone();
                                        move |_event, _window, cx| {
                                            view.update(cx, |this, cx| {
                                                this.create_library();
                                                this.close_dialog();
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0x1a1a1a))
                                            .child("创建")
                                    )
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .cursor_pointer()
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view = view.clone();
                                        move |_event, _window, cx| {
                                            view.update(cx, |this, cx| {
                                                this.close_dialog();
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child("取消")
                            )
                    )
            )
        })
}
```

### 示例 2：搜索框

```rust
pub fn render_search_box(
    search_query: String,
    cx: &mut Context<CanViewApp>
) -> impl IntoElement {
    let view = cx.entity().clone();
    
    div()
        .px_4()
        .py_2()
        .bg(rgb(0x1a1a1a))
        .rounded(px(8.))
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x646473))
                .child("🔍")
        )
        .child(
            TextInputBuilder::new()
                .text(search_query.clone())
                .placeholder("搜索库名称...")
                .validation(TextInputValidation::None)
                .max_w(px(300.))
                .build("search_input")
        )
        .on_key_down({
            let view = view.clone();
            move |event, _window, cx| {
                let keystroke = format!("{}", event.keystroke);
                
                if keystroke == "enter" {
                    view.update(cx, |this, cx| {
                        this.perform_search();
                        cx.notify();
                    });
                }
            }
        })
}
```

### 示例 3：表单中的多个输入框

```rust
pub fn render_settings_form(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_6()
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::BOLD)
                .child("设置")
        )
        // 输入框 1：项目名称（支持中文）
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_sm().child("项目名称"))
                .child(
                    TextInputBuilder::new()
                        .text(this.project_name.clone())
                        .placeholder("例如：测试项目")
                        .validation(TextInputValidation::LibraryName)
                        .max_w(px(400.))
                        .build("project_name")
                )
        )
        // 输入框 2：版本号（仅 ASCII）
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_sm().child("版本号"))
                .child(
                    TextInputBuilder::new()
                        .text(this.version.clone())
                        .placeholder("v1.0.0")
                        .validation(TextInputValidation::VersionName)
                        .max_w(px(200.))
                        .build("version")
                )
        )
        // 输入框 3：ID（自定义验证）
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_sm().child("项目 ID"))
                .child(
                    TextInputBuilder::new()
                        .text(this.project_id.clone())
                        .placeholder("project-123")
                        .validation(TextInputValidation::Custom(|c| {
                            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'
                        }))
                        .max_w(px(300.))
                        .build("project_id")
                )
        )
}
```

## 调试技巧

### 查看键盘事件

组件会输出详细的日志：

```
TextInput key_down: id='library_input' keystroke='nihao' key='nihao' text=''
TextInput key_down: id='library_input' keystroke='你好' key='你好' text='nihao'
TextInput inserted: '你好', new_text: '你好'
```

### 验证字符

```rust
use crate::ui::components::TextInputValidation;

let validation = TextInputValidation::LibraryName;

// 检查单个字符
assert!(validation.is_valid_char('测'));
assert!(validation.is_valid_char('A'));
assert!(!validation.is_valid_char('\n'));

// 检查字符串
let text = "测试库";
assert!(text.chars().all(|c| validation.is_valid_char(c)));
```

## 常见问题

### Q: 如何处理 Enter 和 Escape 键？

A: 在父组件的 `.on_key_down()` 中处理：

```rust
.on_key_down({
    let view = view.clone();
    move |event, _window, cx| {
        let keystroke = format!("{}", event.keystroke);
        
        view.update(cx, |this, cx| {
            match keystroke.as_str() {
                "enter" => this.submit(),
                "escape" => this.cancel(),
                _ => {}
            }
        });
    }
})
```

### Q: 如何获取输入的文本？

A: TextInput 组件不存储状态。需要在父组件中维护：

```rust
struct MyState {
    input_text: String,
}

impl MyState {
    fn handle_input(&mut self, new_text: String) {
        self.input_text = new_text;
        // 处理输入...
    }
}
```

### Q: 为什么输入中文不工作？

A: 
1. 检查日志是否显示 `TextInput inserted`
2. 确认使用了多字符支持（查看上面的"方式 1"示例）
3. 如果仍不工作，使用剪贴板粘贴作为临时方案

## 总结

TextInput 组件提供了：
- 🎨 一致的视觉样式
- 🔒 灵活的字符验证
- 📝 调试日志输出
- 🧪 完整的测试覆盖

记住：这是一个**展示型组件**，状态管理需要由父组件完成！