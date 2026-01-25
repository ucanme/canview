# TextInput 组件使用指南

## 📋 概述

`TextInput` 组件是一个功能完整、支持输入法（IME）的文本输入组件。它提供了灵活的字符验证、多字符字符串输入和一致的用户体验。

## ✨ 核心特性

- ✅ **多字符输入支持**：完整支持输入法，可以接收 "你好"、"测试信号库" 等多字符文本
- ✅ **灵活的字符验证**：提供预设的验证模式，也支持自定义验证函数
- ✅ **字符级操作**：正确处理 UTF-8 多字节字符（中文、日文、韩文、表情符号等）
- ✅ **易于集成**：简洁的 API，与现有代码无缝集成
- ✅ **调试友好**：详细的日志输出，便于诊断问题

## 🚀 快速开始

### 基础用法

```rust
use crate::ui::components::{TextInputBuilder, TextInputValidation};
use gpui::{prelude::*, *};

// 在你的 render 函数中
fn render_my_view(cx: &mut Context<MyApp>) -> impl IntoElement {
    let view = cx.entity().clone();
    let current_text = "当前文本".to_string();
    
    TextInputBuilder::new()
        .text(current_text.clone())
        .placeholder("请输入库名...")
        .focused(true)
        .validation(TextInputValidation::LibraryName)
        .build(
            "library_name_input",
            view.clone(),
            // on_change
            {
                let view = view.clone();
                move |new_text, cx| {
                    view.update(cx, |this, cx| {
                        this.library_name = new_text.to_string();
                        cx.notify();
                    });
                }
            },
            // on_submit (Enter 键)
            {
                let view = view.clone();
                move |text, cx| {
                    view.update(cx, |this, cx| {
                        this.create_library(text);
                        cx.notify();
                    });
                }
            },
            // on_cancel (Esc 键)
            {
                move |cx| {
                    view.update(cx, |this, cx| {
                        this.cancel_input();
                        cx.notify();
                    });
                }
            }
        )
}
```

## 📖 验证模式

### 1. LibraryName（库名验证）

**适用场景**：库名、项目名等需要支持多语言的场景

**规则**：
- ✅ 支持中文、日文、韩文等所有 Unicode 字符
- ✅ 支持英文字母和数字
- ✅ 支持空格
- ✅ 支持表情符号
- ❌ 不支持控制字符（换行、制表符等）

**示例**：
```rust
TextInputBuilder::new()
    .validation(TextInputValidation::LibraryName)
    // ...
```

**有效输入**：
- "测试CAN信号库"
- "Test测试库123"
- "CAN测试库2024"
- "📊 数据分析库"

### 2. VersionName（版本名验证）

**适用场景**：版本号、标签名等需要符合版本规范的场景

**规则**：
- ✅ 仅支持 ASCII 字母（a-z, A-Z）
- ✅ 支持数字（0-9）
- ✅ 支持点号（.）
- ✅ 支持下划线（_）
- ✅ 支持连字符（-）
- ❌ 不支持空格
- ❌ 不支持中文和其他 Unicode 字符

**示例**：
```rust
TextInputBuilder::new()
    .validation(TextInputValidation::VersionName)
    // ...
```

**有效输入**：
- "v1.0.0"
- "version_1.2"
- "release-2.0"
- "v1.2.3-beta"
- "1.0.3-beta_release"

### 3. Custom（自定义验证）

**适用场景**：需要特殊验证规则的场景

**示例**：
```rust
// 只允许数字
TextInputBuilder::new()
    .validation(TextInputValidation::Custom(|c| c.is_ascii_digit()))
    // ...

// 只允许大写字母和数字
TextInputBuilder::new()
    .validation(TextInputValidation::Custom(|c| {
        c.is_ascii_uppercase() || c.is_ascii_digit()
    }))
    // ...

// 允许字母、数字、@ 和 .
TextInputBuilder::new()
    .validation(TextInputValidation::Custom(|c| {
        c.is_ascii_alphanumeric() || c == '@' || c == '.'
    }))
    // ...
```

### 4. None（无验证）

**适用场景**：接受所有非控制字符

**示例**：
```rust
TextInputBuilder::new()
    .validation(TextInputValidation::None)
    // ...
```

## 🎨 完整示例

### 示例 1：库名输入

```rust
pub fn render_library_creation(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();
    
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_sm().text_color(rgb(0xcdd6f4)).child("创建新库"))
        .child(
            TextInputBuilder::new()
                .text(this.new_library_name.clone())
                .placeholder("输入库名称（支持中文）...")
                .focused(true)
                .validation(TextInputValidation::LibraryName)
                .max_w(px(300.))
                .build(
                    "new_library_input",
                    view.clone(),
                    {
                        let view = view.clone();
                        move |new_text, cx| {
                            view.update(cx, |this, cx| {
                                this.new_library_name = new_text.to_string();
                                cx.notify();
                            });
                        }
                    },
                    {
                        let view = view.clone();
                        move |text, cx| {
                            view.update(cx, |this, cx| {
                                if !text.is_empty() {
                                    this.create_library(text);
                                }
                                cx.notify();
                            });
                        }
                    },
                    {
                        move |cx| {
                            view.update(cx, |this, cx| {
                                this.cancel_library_creation();
                                cx.notify();
                            });
                        }
                    }
                )
        )
        .child(
            div()
                .flex()
                .gap_2()
                .child(create_button("创建", view.clone()))
                .child(cancel_button("取消", view))
        )
}
```

### 示例 2：版本名输入

```rust
pub fn render_version_input(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();
    
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().text_sm().child("版本名:"))
        .child(
            TextInputBuilder::new()
                .text(this.new_version_name.clone())
                .placeholder("v1.0.0")
                .validation(TextInputValidation::VersionName)
                .max_w(px(150.))
                .min_w(px(120.))
                .build(
                    "version_input",
                    view.clone(),
                    on_change,
                    on_submit,
                    on_cancel
                )
        )
}
```

### 示例 3：搜索框

```rust
pub fn render_search_box(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();
    
    div()
        .p_2()
        .bg(rgb(0x1a1a1a))
        .child(
            TextInputBuilder::new()
                .text(this.search_query.clone())
                .placeholder("搜索库名...")
                .validation(TextInputValidation::None) // 搜索无限制
                .max_w(px(400.))
                .build(
                    "search_input",
                    view.clone(),
                    {
                        let view = view.clone();
                        move |query, cx| {
                            view.update(cx, |this, cx| {
                                this.search_query = query.to_string();
                                this.perform_search();
                                cx.notify();
                            });
                        }
                    },
                    {
                        let view = view.clone();
                        move |query, cx| {
                            view.update(cx, |this, cx| {
                                this.navigate_to_search_result(query);
                                cx.notify();
                            });
                        }
                    },
                    {
                        move |cx| {
                            view.update(cx, |this, cx| {
                                this.clear_search();
                                cx.notify();
                            });
                        }
                    }
                )
        )
}
```

### 示例 4：自定义验证 - ID 输入

```rust
pub fn render_id_input(cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();
    
    // 自定义验证：只允许小写字母、数字和连字符
    let id_validation = TextInputValidation::Custom(|c| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'
    });
    
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().text_sm().child("ID:"))
        .child(
            TextInputBuilder::new()
                .text(this.item_id.clone())
                .placeholder("my-item-id-123")
                .validation(id_validation)
                .max_w(px(250.))
                .build(
                    "item_id_input",
                    view,
                    on_change,
                    on_submit,
                    on_cancel
                )
        )
}
```

## 🔧 高级用法

### 1. 动态宽度

```rust
TextInputBuilder::new()
    .text(text.clone())
    .min_w(px(150.))  // 最小宽度
    .max_w(px(400.))  // 最大宽度
    .build(/* ... */)
```

### 2. 条件性焦点

```rust
TextInputBuilder::new()
    .text(text.clone())
    .focused(this.is_editing)  // 根据状态设置焦点
    .build(/* ... */)
```

### 3. 动态占位符

```rust
let placeholder = if this.is_library_mode {
    "输入库名称..."
} else {
    "输入版本号..."
};

TextInputBuilder::new()
    .text(text.clone())
    .placeholder(placeholder)
    .build(/* ... */)
```

## 📊 API 参考

### TextInputBuilder

#### 方法

| 方法 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `new()` | - | `Self` | 创建新的构建器 |
| `text()` | `impl Into<String>` | `Self` | 设置初始文本 |
| `placeholder()` | `impl Into<String>` | `Self` | 设置占位符文本 |
| `focused()` | `bool` | `Self` | 设置是否聚焦 |
| `validation()` | `TextInputValidation` | `Self` | 设置字符验证模式 |
| `max_width()` | `Pixels` | `Self` | 设置最大宽度 |
| `min_width()` | `Pixels` | `Self` | 设置最小宽度 |
| `build()` | `id`, `view`, `on_change`, `on_submit`, `on_cancel` | `impl IntoElement` | 构建组件 |

### TextInputValidation

| 变体 | 描述 |
|------|------|
| `LibraryName` | 支持中文、英文、数字、空格、Unicode |
| `VersionName` | 仅 ASCII + .-_ |
| `Custom(fn(char) -> bool)` | 自定义验证函数 |
| `None` | 无验证（仅排除控制字符） |

## 🎯 最佳实践

### 1. 选择合适的验证模式

```rust
// ✅ 好的做法：使用预设模式
TextInputBuilder::new()
    .validation(TextInputValidation::LibraryName)

// ❌ 不推荐：除非有特殊需求，否则避免自定义验证
TextInputBuilder::new()
    .validation(TextInputValidation::Custom(|c| /* 复杂逻辑 */))
```

### 2. 提供清晰的占位符

```rust
// ✅ 好的做法：提供示例
.placeholder("测试CAN信号库")

// ❌ 不推荐：占位符不明确
.placeholder("输入...")
```

### 3. 处理空输入

```rust
// ✅ 好的做法：在 on_submit 中验证
move |text, cx| {
    if !text.trim().is_empty() {
        this.create_library(text);
    }
}

// ❌ 不推荐：不检查空输入
move |text, cx| {
    this.create_library(text);  // 可能创建空名称
}
```

### 4. 更新状态后通知

```rust
// ✅ 好的做法：每次状态改变都通知
move |new_text, cx| {
    this.text = new_text.to_string();
    cx.notify();  // 重要！触发重绘
}

// ❌ 不推荐：忘记通知
move |new_text, cx| {
    this.text = new_text.to_string();
    // 缺少 cx.notify()
}
```

## 🐛 调试

### 启用日志

组件会输出详细的调试日志：

```
TextInput clicked: library_name_input
TextInput key_down: keystroke='nihao' key='nihao' text=''
TextInput key_down: keystroke='你好' key='你好' text='nihao'
TextInput inserted: '你好', new_text: '你好'
```

### 查看日志

```bash
# 运行应用并查看日志
cargo run -p view --release 2>&1 | grep TextInput
```

### 常见问题

**问题 1：输入中文不显示**
- 检查控制台日志
- 确认是否看到 `TextInput inserted` 日志
- 如果只看到拼音，说明输入法未正确工作，使用剪贴板粘贴作为临时方案

**问题 2：字符被拒绝**
- 检查验证模式是否正确
- 查看日志中的 `TextInput rejected` 消息
- 根据需要调整验证模式

**问题 3：状态未更新**
- 确保在回调中调用了 `cx.notify()`
- 检查状态更新逻辑是否正确

## 📚 相关资源

- **源码**：`src/view/src/ui/components/text_input.rs`
- **测试**：`tests/test_ime_input.rs`
- **输入法支持**：`IME_INPUT_SUPPORT.md`
- **故障排除**：`TROUBLESHOOTING.md`

## 🔄 迁移指南

### 从旧代码迁移

**旧代码**（使用直接的事件处理）：
```rust
div()
    .on_key_down(move |event, _window, cx| {
        let keystroke = format!("{}", event.keystroke);
        if keystroke.len() == 1 {
            // 处理单字符输入...
        }
    })
```

**新代码**（使用 TextInput 组件）：
```rust
TextInputBuilder::new()
    .validation(TextInputValidation::LibraryName)
    .build(
        "my_input",
        view,
        on_change,
        on_submit,
        on_cancel
    )
```

## ✅ 总结

TextInput 组件提供了：
- 🎯 完整的输入法支持
- 🔒 灵活的字符验证
- 🎨 一致的 UI 样式
- 📝 详细的日志输出
- 🧪 完整的测试覆盖

使用这个组件可以大大简化文本输入的开发，同时获得完整的输入法支持和国际化能力。