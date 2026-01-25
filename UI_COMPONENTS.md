# 公共UI组件使用文档

## 概述

已创建可复用的 UI 组件来简化 main.rs 中的代码。

## 已创建的组件

### 1. Button 组件

**文件**: `src/view/src/ui/components/button.rs`

#### 特性
- 4 种预定义样式
- 统一的悬停效果
- 简洁的 API

#### 使用方法

```rust
use crate::ui::components::Button;

// 基础按钮（灰色）
Button::new("Click Me")
    .build(on_click);

// 主要按钮（蓝色）
Button::new("Create")
    .primary()
    .build(on_click);

// 危险按钮（红色）
Button::new("Delete")
    .danger()
    .build(on_click);

// 幽灵按钮（透明背景）
Button::new("Cancel")
    .ghost()
    .build(on_click);
```

#### 颜色规格

| 样式 | 背景色 | 文字色 | 悬停背景 |
|------|--------|--------|----------|
| Primary | #89b4fa (蓝) | #1a1a1a | #6a9cda |
| Secondary | #2a2a2a (灰) | #cdd6f4 | #3a3a3a |
| Danger | #f38ba8 (红) | #1a1a1a | #e78aa7 |
| Ghost | transparent | #89b4fa | transparent |

### 2. TextInput 组件

**文件**: `src/view/src/ui/components/text_input.rs`

#### 使用方法

```rust
use crate::ui::components::TextInputBuilder;

TextInputBuilder::new()
    .text(initial_text)
    .placeholder("Enter text...")
    .focused(true)
    .build(
        "unique_id",
        on_change,
        on_submit,
        on_cancel
    )
```

详见 `TEXTINPUT_TEST.md`

### 3. Panel 组件

**文件**: `src/view/src/ui/components/panel.rs`

#### 特性
- 3 种预定义样式
- 可自定义内边距
- 可选圆角

#### 使用方法

```rust
use crate::ui::components::{Panel, PanelStyle};

// 默认面板
Panel::new()
    .build()
    .child(content)

// 自定义样式和内边距
Panel::new()
    .style(PanelStyle::Elevated)
    .padding(24.0, 16.0)
    .build()
    .child(content)

// 无圆角面板
Panel::new()
    .no_round()
    .build()
    .child(content)
```

#### 样式规格

| 样式 | 背景色 |
|------|--------|
| Default | #0c0c0e |
| Elevated | #1a1a1a |
| Subtle | #1e1e2e |

### 4. Label 组件

**文件**: `src/view/src/ui/components/label.rs`

#### 特性
- 5 种尺寸
- 6 种颜色
- 可选字重
- 便捷方法

#### 使用方法

```rust
use crate::ui::components::{Label, LabelSize, LabelColor};

// 基础标签
Label::new("Hello World")
    .build()

// 自定义尺寸和颜色
Label::new("Important")
    .size(LabelSize::LG)
    .color(LabelColor::Accent)
    .font_weight(FontWeight::BOLD)
    .build()

// 便捷方法
Label::small_muted("Optional text")  // 小号灰色文本
Label::accent("Highlight")           // 蓝色强调文本
```

#### 尺寸规格

| 尺寸 | 字体大小 |
|------|----------|
| XS | 11px |
| SM | 13px |
| Base | 14px |
| LG | 16px |
| XL | 18px |

#### 颜色规格

| 颜色 | 色值 |
|------|------|
| Default | #cdd6f4 |
| Muted | #646473 |
| Accent | #89b4fa |
| Success | #a6e3a1 |
| Warning | #f9e2af |
| Error | #f38ba8 |

### 5. Divider 组件

**文件**: `src/view/src/ui/components/divider.rs`

#### 特性
- 水平和垂直方向
- 可自定义颜色和粗细

#### 使用方法

```rust
use crate::ui::components::{Divider, DividerOrientation};

// 水平分隔线
Divider::horizontal()
    .build()

// 垂直分隔线
Divider::vertical()
    .build()

// 自定义样式
Divider::horizontal()
    .color(0x404040)
    .thickness(2.0)
    .build()
```

### 6. Card 组件

**文件**: `src/view/src/ui/components/card.rs`

#### 特性
- 3 种样式
- 可点击交互
- 带边框和圆角

#### 使用方法

```rust
use crate::ui::components::{Card, CardStyle};

// 基础卡片
Card::new()
    .build()
    .child(content)

// 悬停效果
Card::new()
    .style(CardStyle::Hover)
    .build()
    .child(content)

// 可点击卡片
Card::new()
    .clickable(on_click)
    .build()
    .child(content)
```

## 组件位置

```
src/view/src/ui/components/
├── mod.rs          ← 导出所有组件
├── button.rs       ✅ Button 组件
├── card.rs         ✅ Card 组件
├── divider.rs      ✅ Divider 组件
├── label.rs        ✅ Label 组件
├── panel.rs        ✅ Panel 组件
└── text_input.rs   ✅ TextInput 组件
```

## 编译状态

```bash
✅ cargo build --release
   Finished in 0.44s
```

## 使用示例

### 示例 1: 面板带标题和分隔线

```rust
Panel::new()
    .build()
    .child(
        div()
            .child(Label::new("Settings").size(LabelSize::LG).build())
            .child(Divider::horizontal().build())
            .child(content)
    )
```

### 示例 2: 卡片布局

```rust
div()
    .gap_3()
    .children(
        (1..=3).map(|i| {
            Card::new()
                .clickable(on_click)
                .padding(16.0, 12.0)
                .build()
                .child(
                    div()
                        .child(Label::new(format!("Item {}", i)).build())
                        .child(Label::small_muted("Description"))
                )
        })
    )
```

### 示例 3: 按钮组

```rust
div()
    .flex()
    .gap_2()
    .children(vec![
        Button::new("Cancel").ghost().build(on_cancel),
        Button::new("Delete").danger().build(on_delete),
        Button::new("Save").primary().build(on_save),
    ])
```

## 代码对比

### 重构前

```rust
div()
    .px_4()
    .py_3()
    .rounded(px(6.))
    .bg(rgb(0x0c0c0e))
    .child(content)

div()
    .text_sm()
    .text_color(rgb(0xcdd6f4))
    .font_weight(FontWeight::MEDIUM)
    .child("Title")

div()
    .px_3()
    .py_1()
    .bg(rgb(0x89b4fa))
    .on_mouse_down(...)
    .child("Create")
```

### 重构后

```rust
Panel::new().build().child(content)

Label::new("Title")
    .size(LabelSize::SM)
    .font_weight(FontWeight::MEDIUM)
    .build()

Button::new("Create")
    .primary()
    .build(on_click)
```

## 导入路径

```rust
use crate::ui::components::{
    Button, ButtonColor,
    Card, CardStyle,
    Divider, DividerOrientation,
    Label, LabelColor, LabelSize,
    Panel, PanelStyle,
    TextInputBuilder,
};
```

---

**创建日期**: 2026-01-17
**更新日期**: 2026-01-17
**状态**: ✅ 编译通过，可立即使用
