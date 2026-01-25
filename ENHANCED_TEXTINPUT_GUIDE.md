# EnhancedTextInput 使用指南

## 概述

`EnhancedTextInput` 是基于 `gpui-component` 的最佳实践实现的增强版文本输入组件。它在保持简单易用的 API 的同时，添加了以下功能：

- ✅ **可见光标** - 聚焦时显示光标
- ✅ **文本选择** - 支持文本选择区域（基础支持）
- ✅ **改进的键盘处理** - 支持 Ctrl/Cmd 组合键
- ✅ **更好的 IME 支持** - 支持中文等多字符输入
- ✅ **字符验证** - 内置验证系统

## 快速开始

### 基础用法

```rust
use crate::ui::components::{EnhancedTextInputBuilder, TextInputValidation};

let input = EnhancedTextInputBuilder::new()
    .text(state.my_text.clone())
    .placeholder("请输入内容...")
    .focused(state.is_focused)
    .validation(TextInputValidation::LibraryName)
    .build(
        "my_input_id",
        cx.entity().clone(),
        {
            let view = cx.entity().clone();
            move |new_text, cx| {
                // 文本变化时的回调
                view.update(cx, |this, cx| {
                    this.my_text = new_text.to_string();
                    cx.notify();
                });
            }
        },
        {
            let view = cx.entity().clone();
            move |text, cx| {
                // 按下 Enter 键时的回调
                view.update(cx, |this, cx| {
                    this.submit(text);
                    cx.notify();
                });
            }
        },
    );
```

### 可用选项

```rust
EnhancedTextInputBuilder::new()
    .text("初始文本")                    // 设置初始文本
    .placeholder("占位符文本")           // 设置占位符
    .focused(true)                       // 是否聚焦
    .validation(TextInputValidation::LibraryName)  // 验证模式
    .max_width(px(300.))                // 最大宽度
    .min_width(px(100.))                // 最小宽度
    .build(...)
```

## 验证模式

### LibraryName - 支持中文、英文、数字、空格

```rust
TextInputValidation::LibraryName
```

**有效示例：**
- "测试CAN信号库"
- "Test测试库123"
- "📊 数据分析库"

**无效字符：**
- 控制字符（\n, \t, \r）

### VersionName - 仅支持 ASCII 字符

```rust
TextInputValidation::VersionName
```

**有效示例：**
- "v1.0.0"
- "version_1.2"
- "release-2.0"

**无效字符：**
- 空格
- 中文字符
- 控制字符

### Custom - 自定义验证函数

```rust
TextInputValidation::Custom(|c| c.is_ascii_digit())  // 仅数字
TextInputValidation::Custom(|c| c.is_alphabetic())   // 仅字母
```

### None - 不验证

```rust
TextInputValidation::None  // 接受所有非控制字符
```

## 从旧版本迁移

### 从 TextInputBuilder 迁移

**旧代码：**
```rust
use crate::ui::components::{TextInputBuilder, TextInputValidation};

let input = TextInputBuilder::new()
    .text(state.text.clone())
    .placeholder("Library name...")
    .focused(state.is_editing)
    .validation(TextInputValidation::LibraryName)
    .build(
        "library_name_input",
        cx.entity().clone(),
        on_change,
        on_submit,
        on_cancel  // 旧版本需要 on_cancel
    );
```

**新代码：**
```rust
use crate::ui::components::{EnhancedTextInputBuilder, TextInputValidation};

let input = EnhancedTextInputBuilder::new()
    .text(state.text.clone())
    .placeholder("Library name...")
    .focused(state.is_editing)
    .validation(TextInputValidation::LibraryName)
    .build(
        "library_name_input",
        cx.entity().clone(),
        on_change,
        on_submit  // 新版本简化了，移除了 on_cancel
    );
```

### 主要区别

1. **更简单的 API** - 移除了 `on_cancel` 参数（Escape 键仍可用但不触发回调）
2. **可见光标** - 聚焦时显示光标，旧版本没有
3. **更好的键盘支持** - 支持 Ctrl+A（全选）、Ctrl+C（复制）等（未来版本）
4. **内部状态管理** - 更好的光标和选择管理

## 实际应用示例

### 示例 1：库管理中的输入

```rust
// 在 library_view.rs 中
fn render_library_form(state: &mut LibraryViewState, cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();

    v_flex()
        .gap_2()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0xcdd6f4))
                .child("库名称:")
        )
        .child(
            EnhancedTextInputBuilder::new()
                .text(state.library_name.clone())
                .placeholder("例如：车辆CAN信号库")
                .focused(state.is_editing_name)
                .validation(TextInputValidation::LibraryName)
                .max_width(px(400.))
                .build(
                    "library_name",
                    view.clone(),
                    {
                        let view = view.clone();
                        move |new_text, cx| {
                            view.update(cx, |this, cx| {
                                this.library_name = new_text.to_string();
                                cx.notify();
                            });
                        }
                    },
                    {
                        let view = view.clone();
                        move |text, cx| {
                            view.update(cx, |this, cx| {
                                this.save_library();
                                cx.notify();
                            });
                        }
                    },
                )
        )
}
```

### 示例 2：版本号输入

```rust
fn render_version_input(state: &mut VersionState, cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();

    EnhancedTextInputBuilder::new()
        .text(state.version.clone())
        .placeholder("v1.0.0")
        .focused(state.is_editing_version)
        .validation(TextInputValidation::VersionName)  // 注意：使用 VersionName
        .build(
            "version_input",
            view.clone(),
            {
                let view = view.clone();
                move |new_text, cx| {
                    view.update(cx, |this, cx| {
                        this.version = new_text.to_string();
                        cx.notify();
                    });
                }
            },
            {
                let view = view.clone();
                move |text, cx| {
                    view.update(cx, |this, cx| {
                        this.create_version();
                        cx.notify();
                    });
                }
            },
        )
}
```

### 示例 3：自定义验证

```rust
// 仅允许数字输入
fn render_number_input(state: &mut State, cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();

    EnhancedTextInputBuilder::new()
        .text(state.number.clone())
        .placeholder("123")
        .validation(TextInputValidation::Custom(|c| c.is_ascii_digit()))
        .build(
            "number_input",
            view.clone(),
            on_change,
            on_submit,
        )
}

// 允许邮箱字符
fn render_email_input(state: &mut State, cx: &mut Context<CanViewApp>) -> impl IntoElement {
    let view = cx.entity().clone();

    EnhancedTextInputBuilder::new()
        .text(state.email.clone())
        .placeholder("user@example.com")
        .validation(TextInputValidation::Custom(|c| {
            c.is_ascii_alphanumeric() || c == '@' || c == '.' || c == '-' || c == '_'
        }))
        .build(
            "email_input",
            view.clone(),
            on_change,
            on_submit,
        )
}
```

## 高级用法

### 使用内部状态管理

如果需要更复杂的状态管理，可以直接使用 `EnhancedTextInputState`：

```rust
use crate::ui::components::EnhancedTextInputState;

let mut state = EnhancedTextInputState::new("Hello".to_string());

// 插入文本
state.insert_text(" World");
assert_eq!(state.text, "Hello World");

// 移动光标
state.move_cursor_to(5);

// 选择文本
state.select_to(11);

// 获取选中的文本
let selected = state.selected_text();
assert_eq!(selected, " World");

// 删除选中内容
state.delete_selection();
assert_eq!(state.text, "Hello");
```

### 焦点管理

```rust
// 设置初始焦点
EnhancedTextInputBuilder::new()
    .focused(true)  // 初始聚焦
    .build(...)

// 动态切换焦点
fn toggle_focus(state: &mut State, cx: &mut Context<CanViewApp>) {
    state.input_focused = !state.input_focused;
    cx.notify();  // 通知重新渲染
}
```

## 键盘快捷键

当前支持的快捷键：

| 按键 | 功能 |
|------|------|
| `Backspace` | 删除前一个字符 |
| `Delete` | 删除后一个字符（未来支持） |
| `Enter` | 提交 |
| `Escape` | 取消焦点 |
| `Left/Right` | 移动光标（基础支持） |
| `Ctrl+A` / `Cmd+A` | 全选（未来支持） |
| `Ctrl+C` / `Cmd+C` | 复制（未来支持） |
| `Ctrl+V` / `Cmd+V` | 粘贴（未来支持） |

## 最佳实践

### 1. 始终提供 placeholder

```rust
EnhancedTextInputBuilder::new()
    .placeholder("请输入库名称")  // ✅ 好的做法
    .build(...)

EnhancedTextInputBuilder::new()
    .placeholder("")  // ❌ 不推荐
    .build(...)
```

### 2. 选择合适的验证模式

```rust
// 库名称 - 支持中文
.validation(TextInputValidation::LibraryName)

// 版本号 - 仅 ASCII
.validation(TextInputValidation::VersionName)

// 自定义
.validation(TextInputValidation::Custom(|c| c.is_ascii_digit()))
```

### 3. 合理设置宽度

```rust
EnhancedTextInputBuilder::new()
    .min_width(px(100.))  // 防止太窄
    .max_width(px(400.))  // 防止太宽
    .build(...)
```

### 4. 正确处理回调

```rust
// ✅ 好的做法 - 使用 cx.notify()
.on_change({
    let view = cx.entity().clone();
    move |new_text, cx| {
        view.update(cx, |this, cx| {
            this.text = new_text.to_string();
            cx.notify();  // 重要：触发重新渲染
        });
    }
})

// ❌ 错误的做法 - 忘记 cx.notify()
.on_change({
    let view = cx.entity().clone();
    move |new_text, cx| {
        view.update(cx, |this, cx| {
            this.text = new_text.to_string();
            // 缺少 cx.notify()
        });
    }
})
```

## 常见问题

### Q: 光标不显示？
A: 确保 `focused` 设置为 `true` 且组件有文本内容或 placeholder。

### Q: 中文输入不工作？
A: 确保使用 `TextInputValidation::LibraryName` 或 `TextInputValidation::None`，不要使用 `VersionName`。

### Q: 如何实现多行输入？
A: 当前版本仅支持单行。多行支持在计划中。

### Q: 如何清除输入？
A: 通过回调设置空字符串：
```rust
view.update(cx, |this, cx| {
    this.text = String::new();
    cx.notify();
});
```

## 未来计划

- [ ] 光标闪烁动画
- [ ] 文本选择高亮显示
- [ ] 复制/粘贴/剪切
- [ ] 撤销/重做
- [ ] 多行输入支持
- [ ] 自动滚动到光标
- [ ] 搜索功能

## 相关文件

- 实现文件：`src/view/src/ui/components/enhanced_text_input.rs`
- 模块导出：`src/view/src/ui/components/mod.rs`
- 原始实现：`src/view/src/ui/components/text_input.rs`
- 改进计划：`TEXTINPUT_IMPROVEMENT_PLAN.md`
