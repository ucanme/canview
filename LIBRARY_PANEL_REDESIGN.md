# Library Panel Redesign

## Overview
重新设计了左侧面板，不再显示所有库的列表，而是显示当前选中的库信息，并支持折叠/展开版本列表。

## Changes Made

### 1. State Management (main.rs)

**Added new field:**
```rust
struct CanViewApp {
    // ... other fields
    library_versions_expanded: bool,  // 控制版本列表的折叠/展开状态
}
```

**Initialization:**
```rust
library_versions_expanded: true,  // 默认展开
```

### 2. Left Panel Redesign (library_view.rs)

**Before:**
- 显示所有库的列表
- 显示搜索框
- 需要滚动查看所有库

**After:**
- 只显示当前选中的库
- 库名称作为标题，带版本数量
- 可折叠的版本列表
- 点击标题可切换折叠状态

### 3. New Function: `render_selected_library_info`

这个新函数负责渲染左侧面板的内容：

**显示内容：**
1. **库标题区域**
   - 库名称（粗体）
   - 版本数量显示（例如：`(3)`）
   - 折叠/展开图标（▶ / ▼）
   - 可点击切换折叠状态

2. **版本列表**（可折叠）
   - 每个版本显示名称和日期
   - 激活的版本显示绿点（●）
   - 点击版本可激活
   - 鼠标悬停有高亮效果

3. **空状态**
   - 没有选中库时显示 "No library selected"
   - 没有版本时显示 "No versions"

### 4. Visual Design

**库标题样式：**
- 背景：`0x1a1a1a`（深灰色）
- 圆角：`4px`
- 悬停背景：`0x252525`
- 标题文字：白色粗体
- 版本数量：灰色小字
- 图标：蓝色箭头

**版本列表样式：**
- 每个版本项目可点击
- 悬停时背景高亮
- 激活版本显示绿色圆点
- 版本名称：白色
- 日期：灰色

## User Interaction

### 折叠/展开版本列表
- **操作**：点击库名称标题栏
- **效果**：切换版本列表的显示/隐藏状态
- **图标变化**：
  - 展开：▼ (向下箭头)
  - 折叠：▶ (向右箭头)

### 选择版本
- **操作**：点击版本名称
- **效果**：
  - 更新 channel mapping 的版本
  - 如果不存在 mapping，创建新的
  - 显示激活状态（绿点）

### 视觉反馈
1. **鼠标悬停**：
   - 标题栏：背景变亮
   - 版本项：背景变亮

2. **激活状态**：
   - 绿点（●）表示当前激活的版本

## Benefits

1. **更清晰的界面**：左侧面板不再拥挤，只显示当前相关内容
2. **节省空间**：折叠版本列表时占用更少空间
3. **快速切换**：一键折叠/展开所有版本
4. **更好的聚焦**：用户专注于当前选中的库

## Code Changes Summary

### Files Modified:
1. **src/view/src/main.rs**
   - Added `library_versions_expanded: bool` field
   - Initialized in both constructors
   - Updated `render_config_view` to pass the new parameter

2. **src/view/src/library_view.rs**
   - Updated `render_library_management_view` signature
   - Replaced left panel content from list to selected library info
   - Added `render_selected_library_info` function
   - Removed unused functions: `render_library_search`, `render_library_list`

### New Function Signature:
```rust
fn render_selected_library_info(
    library_manager: &LibraryManager,
    selected_id: &Option<String>,
    mappings: &[ChannelMapping],
    versions_expanded: bool,
    cx: &mut gpui::Context<crate::CanViewApp>
) -> impl IntoElement
```

## Compilation

✅ **Build successful**
```bash
cargo build --release
# Finished in 0.34s
```

## Testing

### Test Cases:
1. **选择库**
   - 在右侧面板选择一个库
   - 左侧面板应显示该库的名称和版本

2. **折叠/展开**
   - 点击左侧库名称标题
   - 版本列表应该折叠/展开
   - 箭头图标应该相应改变

3. **选择版本**
   - 点击左侧版本列表中的版本
   - 应该激活该版本（显示绿点）

4. **空状态**
   - 没有选中库时显示 "No library selected"
   - 库没有版本时显示 "No versions"

## Future Enhancements (Optional)

1. **库切换器**：添加下拉菜单在左侧切换不同的库
2. **版本管理**：在左侧版本列表添加删除按钮
3. **拖放排序**：支持拖动版本进行排序
4. **搜索功能**：在版本较多时添加搜索过滤
5. **版本详情**：点击版本显示更多详细信息

## Screenshots Description

### Before (旧设计):
```
┌─────────────────┬──────────────────────────┐
│ Libraries + New │                          │
│ [Search box]    │  Library Details         │
│                 │                          │
│ ▼ Library 1     │  [Detailed info]         │
│ ▼ Library 2     │                          │
│ ▶ Library 3     │                          │
│ ▼ Library 4     │                          │
│ ...             │                          │
└─────────────────┴──────────────────────────┘
```

### After (新设计):
```
┌─────────────────┬──────────────────────────┐
│ Libraries + New │                          │
│                 │  Library Details         │
│ ┌─────────────┐ │                          │
│ │ Library 1 (3)│ │  [Detailed info]         │
│ │   ▼         │ │                          │
│ │  v1.0       │ │                          │
│ │  v1.1       │ │                          │
│ │  v2.0 ●     │ │                          │
│ └─────────────┘ │                          │
│                 │                          │
└─────────────────┴──────────────────────────┘
```

### Collapsed State (折叠状态):
```
┌─────────────────┬──────────────────────────┐
│ Libraries + New │                          │
│                 │  Library Details         │
│ ┌─────────────┐ │                          │
│ │ Library 1 (3)│ │  [Detailed info]         │
│ │   ▶         │ │                          │
│ └─────────────┘ │                          │
│                 │                          │
└─────────────────┴──────────────────────────┘
```

## Implementation Details

### Click Handler
点击库名称标题时触发折叠状态切换：
```rust
.on_mouse_down(gpui::MouseButton::Left, {
    let view = view.clone();
    move |_event, _window, cx| {
        view.update(cx, |this, cx| {
            this.library_versions_expanded = !this.library_versions_expanded;
            cx.notify();
        });
    }
})
```

### Conditional Rendering
使用 `.when()` 来条件渲染版本列表：
```rust
.when(versions_expanded && !library.versions.is_empty(), |d| {
    // 渲染版本列表
})
```

### Icon Selection
根据折叠状态选择不同的箭头图标：
```rust
.child(if versions_expanded { "▼" } else { "▶" })
```

## Migration Notes

这是一个界面改进，不需要数据迁移：
- 所有现有数据保持不变
- 只是改变了显示方式
- 用户体验更清晰、更聚焦
