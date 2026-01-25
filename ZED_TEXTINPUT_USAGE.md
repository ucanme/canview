# Zed-style TextInput Component Usage Guide

这是一个受 Zed IDE 启发的现代化 TextInput 组件，提供了完整的文本输入功能。

## 主要特性

### 1. **可见光标和动画**
- 闪烁的光标动画（530ms 间隔）
- 清晰显示当前输入位置
- 可自定义光标颜色和宽度

### 2. **文本选择**
- 支持 Shift+方向键选择文本
- Ctrl+A 全选
- 可视化选择区域高亮

### 3. **键盘快捷键**
- `Left/Right` - 移动光标
- `Shift+Left/Right` - 选择文本
- `Home/End` - 跳转到行首/行尾
- `Shift+Home/End` - 选择到行首/行尾
- `Backspace/Delete` - 删除字符
- `Ctrl+A` - 全选
- `Ctrl+C` - 复制（待实现剪贴板 API）
- `Ctrl+V` - 粘贴（待实现剪贴板 API）
- `Ctrl+X` - 剪切（待实现剪贴板 API）
- `Enter` - 提交
- `Escape` - 取消

### 4. **IME 输入法支持**
- 支持中文、日文、韩文等多字符输入
- 字符验证功能

### 5. **灵活的验证选项**
- `LibraryName` - 支持 Unicode、空格、字母数字
- `VersionName` - 仅 ASCII 字母数字、点、下划线、连字符
- `Custom` - 自定义验证函数
- `None` - 无验证

## 使用方法

### 基础用法

```rust
use crate::ui::components::zed_text_input_interactive::ZedTextInputInteractive;
use gpui::*;

// 在你的 render 方法中
fn render_text_input(cx: &mut Context<Self>) -> impl IntoElement {
    let view = cx.entity().clone();

    ZedTextInputInteractive::new()
        .text(self.state.text.clone())
        .placeholder("Enter text...")
        .focused(true)
        .build(
            view,
            {
                let view = cx.entity().clone();
                move |new_text, cx| {
                    view.update(cx, |this, cx| {
                        this.state.text = new_text.to_string();
                        cx.notify();
                    });
                }
            },
            {
                let view = cx.entity().clone();
                move |text, cx| {
                    view.update(cx, |this, cx| {
                        this.submit(text.to_string());
                        cx.notify();
                    });
                }
            },
            {
                let view = cx.entity().clone();
                move |cx| {
                    view.update(cx, |this, cx| {
                        this.cancel();
                        cx.notify();
                    });
                }
            },
            cx,
        )
}
```

### 带验证的输入

```rust
// 库名称输入（支持 Unicode）
let library_input = ZedTextInputInteractive::new()
    .text(state.library_name.clone())
    .placeholder("测试CAN信号库")
    .validation(TextInputValidation::LibraryName)
    .min_width(px(200.))
    .build(view, on_change, on_submit, on_cancel, cx);

// 版本号输入（仅 ASCII）
let version_input = ZedTextInputInteractive::new()
    .text(state.version.clone())
    .placeholder("v1.0.0")
    .validation(TextInputValidation::VersionName)
    .min_width(px(100.))
    .build(view, on_change, on_submit, on_cancel, cx);
```

### 自定义光标样式

```rust
use crate::ui::components::zed_text_input_interactive::{CursorConfig, ZedTextInputInteractive};

let custom_cursor = CursorConfig {
    blink_interval: Duration::from_millis(300),  // 更快的闪烁
    width: px(3.),                                // 更宽的光标
    color: rgb(0x89b4fa),                        // 蓝色光标
};

let input = ZedTextInputInteractive::new()
    .text(state.text.clone())
    .cursor_config(custom_cursor)
    .build(view, on_change, on_submit, on_cancel, cx);
```

### 使用状态管理

```rust
use crate::ui::components::zed_text_input::ZedTextInputState;

// 创建输入状态
let mut input_state = ZedTextInputState::new("Hello".to_string());

// 插入文本
input_state.insert_text(" World", TextInputValidation::None);

// 删除字符
input_state.delete_backward();

// 选择文本
input_state.select_all();

// 检查选择
if input_state.has_selection() {
    if let Some((start, end)) = input_state.get_selected_range() {
        println!("Selected: {}..{}", start, end);
    }
}
```

## 组件对比

### 简化版 (ZedTextInput)

适合简单的静态展示，不需要完整交互的场景。

```rust
use crate::ui::components::zed_text_input::{ZedTextInput, ZedTextInputBuilder};

// 使用 Builder
let input = ZedTextInputBuilder::new()
    .text("Hello")
    .placeholder("Enter text")
    .min_width(px(200.))
    .build_simple();
```

### 完整交互版 (ZedTextInputInteractive)

推荐用于大多数场景，提供完整的键盘交互和状态管理。

## 样式定制

### 边框颜色
- 焦点状态：`rgb(0x89b4fa)` (蓝色)
- 非焦点状态：`rgb(0x2a2a2a)` (深灰)

### 背景颜色
- 默认：`rgb(0x1a1a1a)` (深黑)

### 文本颜色
- 占位符：`rgb(0x646473)` (灰色)
- 正常文本：`rgb(0xcdd6f4)` (浅色)

### 选择高亮
- 半透明蓝色：`rgba(0x89b4fa).opacity(0.3)`

## 高级功能

### 文本选择操作

```rust
let mut input = ZedTextInputInteractive::new().text("Hello World");

// 选择部分文本
input.state.selection_start = Some(0);
input.state.cursor_position = 5;

// 获取选中的文本
if let Some(selected) = input.get_selected_text() {
    println!("Selected: {}", selected);  // "Hello"
}

// 替换选中的文本
input.replace_selection("Hi");
assert_eq!(input.state.text, "Hi World");

// 删除选中的文本
input.state.selection_start = Some(0);
input.state.cursor_position = 2;
input.handle_backspace();
assert_eq!(input.state.text, " World");
```

### 光标导航

```rust
let mut input = ZedTextInputInteractive::new().text("Hello");

// 向左移动
input.handle_left(false);

// 向右选择
input.handle_right(true);

// 跳转到行首
input.handle_home(false);

// 跳转到行尾
input.handle_end(false);
```

## 测试

组件包含完整的单元测试，可以运行：

```bash
cargo test zed_text_input
```

## 与现有组件的对比

### 相比原有的 TextInput 组件

1. **更好的视觉效果**
   - 可见的光标和闪烁动画
   - 文本选择高亮
   - 更清晰的焦点状态

2. **更完整的键盘支持**
   - 方向键导航
   - 文本选择快捷键
   - Home/End 键支持

3. **更好的状态管理**
   - 独立的状态结构
   - 更清晰的操作方法
   - 更容易测试

4. **更符合 Zed 风格**
   - 与 Zed IDE 一致的设计语言
   - 使用 GPUI 的最佳实践

## 未来改进

以下功能已经规划但尚未完全实现：

1. **剪贴板集成**
   - 完整的 Ctrl+C/V/X 支持
   - 使用系统剪贴板 API

2. **IME 组合窗口**
   - 显示输入法的组合文本
   - 更好的中文输入体验

3. **多行支持**
   - 支持换行
   - 行号显示
   - 垂直滚动

4. **自动完成**
   - 基于上下文的建议
   - 下拉选择列表

5. **撤销/重做**
   - 命令历史
   - Ctrl+Z / Ctrl+Y 支持

## 示例：完整的表单实现

```rust
use crate::ui::components::zed_text_input_interactive::ZedTextInputInteractive;
use crate::ui::components::TextInputValidation;
use gpui::*;

pub fn render_library_form(cx: &mut Context<Self>) -> impl IntoElement {
    let view = cx.entity().clone();

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_sm().text_color(rgb(0xcdd6f4)).child("库名称"))
        .child(
            ZedTextInputInteractive::new()
                .text(self.library_name.clone())
                .placeholder("输入库名称...")
                .validation(TextInputValidation::LibraryName)
                .min_width(px(300.))
                .build(
                    view.clone(),
                    on_change_library_name,
                    on_submit_library_name,
                    on_cancel_edit,
                    cx,
                ),
        )
        .child(div().text_sm().text_color(rgb(0xcdd6f4)).child("版本号"))
        .child(
            ZedTextInputInteractive::new()
                .text(self.version.clone())
                .placeholder("v1.0.0")
                .validation(TextInputValidation::VersionName)
                .min_width(px(300.))
                .build(
                    view,
                    on_change_version,
                    on_submit_version,
                    on_cancel_edit,
                    cx,
                ),
        )
}
```

## 性能建议

1. **避免频繁重建**
   - 尽量重用输入状态
   - 只在必要时创建新实例

2. **合理使用验证**
   - 简单验证使用内置模式
   - 复杂验证使用 `Custom` 模式

3. **光标动画**
   - 可以调整闪烁间隔以节省性能
   - 在大量输入框时考虑禁用动画

## 故障排查

### 光标不显示
- 检查 `.focused(true)` 是否设置
- 确认组件有 `.focusable()` 标记

### 文本输入无响应
- 确认 `on_change` 回调正确实现
- 检查是否有其他元素拦截了键盘事件

### 中文输入问题
- 确认验证模式支持 Unicode（使用 `LibraryName` 或 `None`）
- 检查 `.chars()` 处理是否正确

## 参考资源

- Zed IDE: https://github.com/zed-industries/zed
- GPUI 框架: https://github.com/zed-industries/zed
