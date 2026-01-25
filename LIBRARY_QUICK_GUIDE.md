# 信号库管理功能实现状态

## ✅ 已实现的功能

### 1. 三栏布局
- ✅ 左栏：库列表
- ✅ 中栏：版本列表
- ✅ 右栏：通道配置
- ✅ 分割线隔开

**文件**: `src/view/src/ui/views/library_management.rs`

### 2. CAN/LIN 类型支持
- ✅ 创建库时选择类型
- ✅ 库数据结构支持类型字段

**文件**: `src/view/src/models/library.rs`

### 3. 通道配置
- ✅ 通道 ID 输入（已有解析验证）
- ✅ 通道名称输入
- ✅ 数据库文件路径选择

**文件**: `src/view/src/app/impls.rs` (第 3772 行)

```rust
let channel_id: u16 = match self.new_channel_id.trim().parse() {
    Ok(id) => id,
    Err(_) => {
        self.status_msg = "通道 ID 必须是整数".into();
        return;
    }
};
```

### 4. 本地存储模块
- ✅ SignalLibraryStorage 已实现
- ✅ 支持文件复制到本地目录
- ✅ 按库名/版本组织目录

**文件**: `src/view/src/library/storage.rs`

## 🔧 需要完善的功能

### 1. 集成本地存储到添加版本流程

**当前状态**: 添加版本时直接使用用户选择的路径

**需要改进**: 自动复制到本地存储

**修改位置**: `src/view/src/app/impls.rs` - 添加版本的处理函数

### 2. 配置文件自动保存/加载

**当前状态**: 配置保存功能已存在，但未集成到信号库操作

**需要改进**: 
- 添加/删除库时自动保存
- 应用启动时自动加载

**修改位置**: 
- `src/view/src/app/impls.rs` - 启动加载
- 各个操作函数 - 自动保存

### 3. UI 优化

**需要添加**:
- 库列表显示 CAN/LIN 标识
- 通道 ID 输入实时验证反馈
- 更好的错误提示

## 📝 快速实现指南

### 步骤 1: 在应用状态中添加存储管理器

**文件**: `src/view/src/app/state.rs`

```rust
pub struct CanViewApp {
    // ... 现有字段
    
    /// 信号库本地存储管理器
    pub signal_storage: Option<SignalLibraryStorage>,
}
```

初始化：

```rust
impl CanViewApp {
    pub fn new_state() -> Self {
        // 初始化存储管理器
        let signal_storage = SignalLibraryStorage::new().ok();
        
        Self {
            // ... 现有初始化
            signal_storage,
        }
    }
}
```

### 步骤 2: 修改添加版本逻辑

**文件**: `src/view/src/app/impls.rs`

找到添加版本的函数（搜索 "add_version" 或 "confirm_add_channel"），添加本地存储逻辑：

```rust
// 在确认添加通道时
fn confirm_add_channel(&mut self, cx: &mut Context<Self>) {
    // ... 现有验证代码
    
    // 如果有存储管理器，复制文件到本地
    if let Some(ref storage) = self.signal_storage {
        if let Some(ref library_id) = self.selected_library_id {
            if let Some(library) = self.library_manager.find_library(library_id) {
                // 复制文件
                match storage.copy_database(
                    &library.name,
                    &version_name,  // 需要获取当前版本名
                    Path::new(&self.new_channel_db_path)
                ) {
                    Ok(local_path) => {
                        // 使用本地路径
                        self.new_channel_db_path = local_path.to_string_lossy().to_string();
                    }
                    Err(e) => {
                        self.status_msg = format!("文件复制失败: {}", e).into();
                        return;
                    }
                }
            }
        }
    }
    
    // ... 继续原有的添加逻辑
}
```

### 步骤 3: 添加自动保存

在每个修改库的操作后调用保存：

```rust
// 添加库后
self.library_manager.create_library(...)?;
self.save_config()?;  // 自动保存

// 添加版本后
self.library_manager.add_version(...)?;
self.save_config()?;  // 自动保存

// 删除库后
self.library_manager.delete_library(...)?;
self.save_config()?;  // 自动保存
```

### 步骤 4: 添加启动加载

**文件**: `src/view/src/main.rs` 或应用初始化位置

```rust
// 在应用启动时
let mut app = CanViewApp::new(cx);

// 尝试加载配置
if let Err(e) = app.load_config() {
    eprintln!("加载配置失败: {}", e);
    // 使用默认配置继续
}
```

### 步骤 5: UI 显示类型标识

**文件**: `src/view/src/ui/views/library_management.rs`

在 `render_library_item` 函数中添加类型标识：

```rust
fn render_library_item(...) -> impl IntoElement {
    div()
        .child(
            // 类型标识徽章
            div()
                .px_1()
                .text_xs()
                .rounded_sm()
                .bg(match library.channel_type {
                    ChannelType::CAN => rgb(0x3b82f6),
                    ChannelType::LIN => rgb(0x10b981),
                })
                .text_color(rgb(0xffffff))
                .child(match library.channel_type {
                    ChannelType::CAN => "CAN",
                    ChannelType::LIN => "LIN",
                })
        )
        .child(library.name.clone())
        // ...
}
```

## 🎯 优先级建议

### 立即实现（高优先级）
1. ✅ 通道 ID 验证（已有）
2. 🔧 本地存储集成（步骤 1-2）
3. 🔧 自动保存配置（步骤 3）
4. 🔧 启动加载配置（步骤 4）

### 后续优化（中优先级）
5. 🔧 UI 类型标识（步骤 5）
6. 输入验证反馈
7. 错误提示优化

### 可选功能（低优先级）
8. 类型筛选
9. 批量导入
10. 版本对比

## 📂 相关文件

| 文件 | 用途 |
|------|------|
| `src/view/src/app/state.rs` | 应用状态定义 |
| `src/view/src/app/impls.rs` | 应用逻辑实现 |
| `src/view/src/library/storage.rs` | 本地存储管理 |
| `src/view/src/ui/views/library_management.rs` | UI 渲染 |
| `src/view/src/models/library.rs` | 数据模型 |

## 🚀 快速开始

如果您希望我立即实现某个功能，请告诉我优先级，我会按照以下顺序进行：

1. **本地存储集成** - 最重要，确保文件保存在软件目录
2. **自动保存/加载** - 确保下次打开时恢复状态
3. **UI 优化** - 提升用户体验

---

**创建日期**: 2026-01-25  
**当前状态**: ✅ 基础功能已实现，需要集成和优化  
**预计工作量**: 2-3 小时
