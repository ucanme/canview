// 中文输入示例
//
// 这个示例展示如何使用 Zed 风格的 TextInput 组件进行中文输入

use crate::ui::components::TextInputValidation;
use crate::ui::components::zed_style_text_input::ZedStyleTextInputBuilder;
use gpui::*;

/// 示例 1: 基础中文输入（库名称）
///
/// 支持中文、英文、数字、空格的混合输入
pub fn render_library_name_input<App>(
    state_text: String,
    is_focused: bool,
    view: Entity<App>,
    cx: &mut gpui::Context<App>,
) -> impl IntoElement
where
    App: 'static,
{
    ZedStyleTextInputBuilder::new()
        .text(state_text)
        .placeholder("例如：测试CAN信号库")
        .validation(TextInputValidation::LibraryName) // ✅ 支持中文
        .focused(is_focused)
        .min_width(px(300.))
        .build(
            "library_name_input",
            view,
            {
                let view = view.clone();
                move |new_text, cx| {
                    // new_text 可以包含中文，例如："测试CAN信号库"
                    view.update(cx, |this, cx| {
                        // 更新状态
                        cx.notify();
                    });
                }
            },
            {
                let view = view.clone();
                move |text, cx| {
                    view.update(cx, |this, cx| {
                        // 提交逻辑
                        cx.notify();
                    });
                }
            },
            {
                move |cx| {
                    // 取消逻辑
                }
            },
        )
}

/// 示例 2: 仅 ASCII 输入（版本号）
///
/// 不支持中文，只允许 ASCII 字符
pub fn render_version_input<App>(
    state_text: String,
    is_focused: bool,
    view: Entity<App>,
    cx: &mut gpui::Context<App>,
) -> impl IntoElement
where
    App: 'static,
{
    ZedStyleTextInputBuilder::new()
        .text(state_text)
        .placeholder("v1.0.0")
        .validation(TextInputValidation::VersionName) // ❌ 不支持中文
        .focused(is_focused)
        .min_width(px(150.))
        .build("version_input", view, on_change, on_submit, on_cancel)
}

/// 示例 3: 无验证（接受所有输入）
pub fn render_free_input<App>(
    state_text: String,
    is_focused: bool,
    view: Entity<App>,
    cx: &mut gpui::Context<App>,
) -> impl IntoElement
where
    App: 'static,
{
    ZedStyleTextInputBuilder::new()
        .text(state_text)
        .placeholder("输入任何内容...")
        .validation(TextInputValidation::None) // ✅ 支持所有字符（包括中文）
        .focused(is_focused)
        .min_width(px(300.))
        .build("free_input", view, on_change, on_submit, on_cancel)
}

/// 示例 4: 自定义验证（只允许中文和数字）
pub fn render_chinese_only_input<App>(
    state_text: String,
    is_focused: bool,
    view: Entity<App>,
    cx: &mut gpui::Context<App>,
) -> impl IntoElement
where
    App: 'static,
{
    // 自定义验证：只允许中文、数字和空格
    let chinese_only_validation = TextInputValidation::Custom(|ch| {
        ch.is_ascii_digit() || ch == ' ' || (ch >= '\u{4E00}' && ch <= '\u{9FFF}') // CJK Unified Ideographs
    });

    ZedStyleTextInputBuilder::new()
        .text(state_text)
        .placeholder("只允许中文和数字")
        .validation(chinese_only_validation)
        .focused(is_focused)
        .min_width(px(300.))
        .build("chinese_only_input", view, on_change, on_submit, on_cancel)
}

// ============================================================================
// 中文输入支持说明
// ============================================================================

/*
## 支持的验证模式

### 1. TextInputValidation::LibraryName ✅ 推荐用于中文输入

支持的字符：
- ✅ 中文字符（如：测试、中文、你好）
- ✅ 英文字母（如：Test、ABC）
- ✅ 数字（如：123、456）
- ✅ 空格
- ✅ 其他 Unicode 字符（如：日文、韩文、emoji）

示例输入：
- "测试CAN信号库"
- "CAN总线测试工具 v1.0"
- "📊 数据分析库"
- "Test测试123"

### 2. TextInputValidation::VersionName ❌ 不支持中文

支持的字符：
- ✅ ASCII 字母数字
- ✅ 点号（.）
- ✅ 下划线（_）
- ✅ 连字符（-）
- ❌ 中文字符

示例输入：
- "v1.0.0" ✅
- "version_1.2" ✅
- "测试" ❌

### 3. TextInputValidation::None ✅ 支持所有字符

接受的任何字符（除了控制字符），包括：
- ✅ 中文
- ✅ 英文
- ✅ 特殊符号
- ✅ Emoji

### 4. TextInputValidation::Custom 🎯 自定义验证

可以定义自己的验证规则，例如：

// 只允许中文
TextInputValidation::Custom(|ch| {
    (ch >= '\u{4E00}' && ch <= '\u{9FFF}') // CJK Unified Ideographs
})

// 允许中文和英文
TextInputValidation::Custom(|ch| {
    ch.is_ascii_alphanumeric() || (ch >= '\u{4E00}' && ch <= '\u{9FFF}')
})

## IME 输入法支持

组件支持中文输入法的多字符输入：

1. 用户输入拼音（例如：ceshi）
2. IME 显示候选词窗口
3. 用户选择"测试"
4. 完整的"测试"字符串会一次性插入到文本框中

代码会自动处理这种多字符输入：
```rust
if keystroke.len() > 1 {
    // 多字符字符串（来自 IME）
    // 例如："测试"、"你好"、"中国"
    let all_valid = keystroke.chars().all(|c| validation.is_valid_char(c));
    if all_valid {
        new_text.push_str(&keystroke); // 一次性插入整个中文字符串
    }
}
```

## 测试中文输入

运行单元测试验证中文支持：
```bash
cargo test library_name_validation
cargo test multi_character_validation
cargo test input_state_insert
```

所有测试都应该通过，证明中文输入功能正常工作。
*/
