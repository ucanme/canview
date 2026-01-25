# 中文库名称 - 完整解决方案

## 🎯 问题确认

**GPUI 框架的 `on_key_down` 和 `on_key_press` 事件都无法捕获 IME（输入法）提交的文本。**

这是一个已知的框架限制，无法通过简单的事件监听解决。

---

## ✅ 推荐方案：配置文件方法

### 方案概述

由于 UI 输入框的限制，我们直接编辑配置文件来创建中文库名称。

### 步骤 1：找到配置文件

配置文件通常在：
```
~/.config/canview/libraries/
```
或
```
C:\Users\Administrator\AppData\Roaming\canview\libraries\
```

### 步骤 2：创建中文库

在配置目录中创建一个新文件夹，例如：
```
libraries/
├── 测试CAN信号库/
│   ├── metadata.json
│   └── signals.db
```

### 步骤 3：编辑 metadata.json

```json
{
  "name": "测试CAN信号库",
  "created_at": "2024-01-18T10:30:00",
  "version": "1.0.0"
}
```

### 步骤 4：重启应用

应用启动时会自动加载所有中文库名称。

---

## 🔄 方案 2：使用命令行参数

修改应用的启动参数，接受库名称：

### 创建一个辅助脚本

**创建文件：`create_library.bat`**

```batch
@echo off
set /p LIB_NAME="请输入库名称（支持中文）: "
echo 创建库: %LIB_NAME%
cargo run -- --create-library "%LIB_NAME%"
```

**使用方法：**
```
create_library.bat
```

然后输入中文库名称即可。

---

## 📝 方案 3：限制为英文 + 明确提示

如果暂时不需要中文支持，可以在 UI 上添加提示：

### 修改 UI 提示

在 `library_view.rs` 中：

```rust
.child(
    div()
        .text_xs()
        .text_color(rgb(0xf59e0b)) // 橙色警告
        .child("⚠️ Current version: English names only")
)
```

或者：

```rust
.child(
    div()
        .text_xs()
        .text_color(rgb(0x646473))
        .child("Library name (ASCII only)...")  // 明确说明
)
```

---

## 🚀 方案 4：等待 GPUI 更新

GPUI 是一个活跃开发的项目。可以：

1. **关注 GPUI 更新**
   - GitHub: https://github.com/zed-industries/zed
   - 查看是否有 IME 支持的 PR 或 Issue

2. **提交 Feature Request**
   - 向 GPUI 提交 Issue，请求 IME 支持
   - 说明你遇到的问题

3. **参考其他项目**
   - 查看 Zed 如何处理中文输入
   - 可能使用了特殊的处理方式

---

## 💻 方案 5：使用其他 UI 框架

如果中文输入是必需的，可以考虑：

### 选项 A：使用 WebView

在应用中嵌入一个 HTML 输入框：

```rust
use wry::WebView;

// HTML input 元素完全支持中文
let html = r#"
<input type="text" id="library_name"
       placeholder="输入库名称（支持中文）">
"#;

// 监听输入变化
```

### 选项 B：使用原生控件

```rust
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::*;

// 使用 Windows 原生 Edit 控件
// 完全支持 IME
```

### 选项 C：使用 Terminal UI

如果应用可以接受终端界面：

```rust
use dialoguer::Input;

// dialoguer 支持中文输入
let name = Input::new()
    .with_prompt("库名称")
    .interact_text()?;
```

---

## 📊 各方案对比

| 方案 | 难度 | 即用性 | 用户体验 | 推荐度 |
|------|------|--------|----------|--------|
| **配置文件** | ⭐ | ✅ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **命令行脚本** | ⭐⭐ | ✅ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **英文限制** | ⭐ | ✅ | ⭐⭐ | ⭐⭐⭐ |
| **GPUI 更新** | ⭐⭐⭐⭐ | ❌ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| **WebView** | ⭐⭐⭐ | ✅ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **原生控件** | ⭐⭐⭐⭐ | ✅ | ⭐⭐⭐⭐ | ⭐⭐ |
| **Terminal UI** | ⭐⭐ | ✅ | ⭐⭐ | ⭐⭐⭐ |

---

## 🎯 立即可用的方案

### 最简单：配置文件 + 应用支持

**用户操作流程：**

1. 打开配置目录
2. 创建新文件夹，命名为中文库名
3. 创建 `metadata.json`
4. 重启应用
5. 应用会显示中文库名称

**开发者需要做的：**

确保应用在启动时：
- ✅ 扫描 libraries 目录
- ✅ 读取所有子文件夹名称
- ✅ 加载 metadata.json
- ✅ 在 UI 中正确显示中文

### 检查应用是否已支持

查看你的代码中是否有：

```rust
// 扫描库目录
fn scan_libraries(dir: &Path) -> Vec<Library> {
    // 应该能读取中文文件夹名
}
```

如果已有这个功能，那么只需要：
1. 手动创建中文文件夹
2. 创建 metadata.json
3. 重启应用即可

---

## 📝 总结

**推荐使用配置文件方案**，因为：
- ✅ 立即可用
- ✅ 应用可能已经支持
- ✅ 无需修改代码
- ✅ 用户体验良好

如果需要真正的 UI 输入支持，建议：
1. 使用 WebView（最快实现）
2. 或等待 GPUI 支持 IME（长期方案）
3. 或切换到其他支持 IME 的 UI 框架

---

## ❓ 需要帮助？

如果你决定使用配置文件方案，我可以帮你：

1. 找到配置文件的准确位置
2. 创建正确的 JSON 结构
3. 确保应用能正确加载中文库名称

请告诉我你想使用哪个方案，我会提供详细的实施步骤！
