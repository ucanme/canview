# CanView 项目目录结构

```
canview/
├── src/
│   ├── view/                      # 主视图应用
│   │   ├── src/
│   │   │   ├── main.rs            # 主入口 (3400+ 行)
│   │   │   ├── main.rs.backup     # 原始备份
│   │   │   ├── playback.rs        # 播放控制模块
│   │   │   │
│   │   │   # 新创建的模块化结构
│   │   │   ├── models/            # 数据模型模块
│   │   │   │   └── mod.rs         # 数据结构定义
│   │   │   │       ├── ChannelType (enum)
│   │   │   │       ├── ChannelMapping (struct)
│   │   │   │       ├── AppConfig (struct)
│   │   │   │       ├── AppView (enum)
│   │   │   │       └── ScrollbarDragState (struct)
│   │   │   │
│   │   │   ├── config/            # 配置管理模块
│   │   │   │   └── mod.rs         # 配置加载/保存
│   │   │   │       ├── ConfigManager (trait)
│   │   │   │       ├── load_config_from_path()
│   │   │   │       ├── save_config_to_path()
│   │   │   │       └── DEFAULT_CONFIG_FILE
│   │   │   │
│   │   │   ├── filters/           # 过滤器模块
│   │   │   │   └── mod.rs         # 消息过滤逻辑
│   │   │   │       ├── filter_by_id()
│   │   │   │       ├── filter_by_channel()
│   │   │   │       ├── filter_by_id_and_channel()
│   │   │   │       ├── get_unique_ids()
│   │   │   │       ├── get_unique_channels()
│   │   │   │       └── format_id()
│   │   │   │
│   │   │   ├── ui/                # UI组件模块
│   │   │   │   ├── mod.rs         # UI渲染器
│   │   │   │   ├── components/    # UI组件
│   │   │   │   │   └── mod.rs     # 可重用组件
│   │   │   │   │       └── render_view_button()
│   │   │   │   └── views/         # 视图实现
│   │   │   │       └── mod.rs     # 视图模块声明
│   │   │   │           ├── log_view (待实现)
│   │   │   │           ├── config_view (待实现)
│   │   │   │           └── chart_view (待实现)
│   │   │   │
│   │   │   ├── app/               # 应用逻辑模块
│   │   │   │   └── mod.rs         # CanViewApp定义
│   │   │   │       ├── struct CanViewApp
│   │   │   │       ├── impl CanViewApp
│   │   │   │       ├── impl ConfigManager
│   │   │   │       └── 基础应用方法
│   │   │   │
│   │   │   # 文档文件
│   │   │   ├── MODULE_STRUCTURE.md           # 模块结构说明
│   │   │   ├── MODULE_USAGE_EXAMPLES.md      # 使用示例
│   │   │   ├── REFACTORING.md                # 重构计划
│   │   │   └── PROJECT_STRUCTURE.md          # 本文件
│   │   │
│   │   └── Cargo.toml
│   │
│   ├── parser/                    # DBC/LDF解析器
│   │   └── src/
│   │       ├── dbc.rs
│   │       └── ldf.rs
│   │
│   └── blf/                       # BLF文件解析器
│       └── (BLF解析逻辑)
│
├── multi_channel_config.json      # 配置文件
├── Cargo.toml
└── README.md
```

## 模块说明

### 已完全实现的模块

#### 1. models 模块 ✅
- **状态**: 可直接使用
- **功能**: 定义所有数据结构
- **依赖**: 无
- **使用**:
  ```rust
  use models::{ChannelType, ChannelMapping, AppConfig, AppView};
  ```

#### 2. config 模块 ✅
- **状态**: 可直接使用
- **功能**: 配置文件的读写
- **依赖**: models
- **使用**:
  ```rust
  use config::{load_config_from_path, save_config_to_path};
  ```

#### 3. filters 模块 ✅
- **状态**: 可直接使用
- **功能**: 消息过滤和格式化
- **依赖**: 无
- **使用**:
  ```rust
  use filters::{filter_by_id, get_unique_ids, format_id};
  ```

#### 4. ui 模块 ⚠️
- **状态**: 部分实现
- **功能**: UI组件和视图
- **依赖**: models
- **说明**: 框架已创建，具体视图需要从 main.rs 移动
- **使用**:
  ```rust
  use ui::components::render_view_button;
  ```

#### 5. app 模块 ⚠️
- **状态**: 部分实现
- **功能**: 应用核心逻辑
- **依赖**: models, config, filters
- **说明**: 基础结构已创建，但 main.rs 中仍有完整实现
- **使用**:
  ```rust
  use app::CanViewApp;
  ```

### 保留在 main.rs 的内容

当前以下内容仍在 main.rs 中（3400+ 行）：

1. **CanViewApp 完整实现**
   - 所有字段初始化
   - 所有方法实现
   - 所有渲染逻辑

2. **渲染方法**
   - `render_log_view()` (~1200 行)
   - `render_config_view()` (~200 行)
   - `render_chart_view()` (~20 行)
   - `render_message_row()` (~400 行)
   - 其他辅助渲染函数

3. **事件处理**
   - 鼠标事件
   - 键盘事件
   - 窗口事件

4. **主入口**
   - `main()` 函数
   - `Render for CanViewApp` 实现

## 模块化程度

| 模块 | 完成度 | 可用性 | 说明 |
|------|--------|--------|------|
| models | 100% | ✅ 可用 | 完全独立，可立即使用 |
| config | 100% | ✅ 可用 | 功能完整，可集成 |
| filters | 100% | ✅ 可用 | 纯函数，无副作用 |
| ui | 30% | ⚠️ 部分可用 | 框架存在，内容待迁移 |
| app | 40% | ⚠️ 部分可用 | 定义存在，与main.rs重复 |

## 使用建议

### 立即可用的模块

你可以立即在 main.rs 中使用这些模块：

```rust
// 在 main.rs 顶部
mod models;
mod config;
mod filters;

// 在代码中使用
use models::*;
use config::*;
use filters::*;
```

### 需要集成的模块

这些模块已创建但需要进一步集成：

1. **ui 模块** - 需要将渲染方法从 main.rs 移动到 ui/views/
2. **app 模块** - 需要解决与 main.rs 中 CanViewApp 的重复定义

## 迁移步骤

如果你想进一步模块化，可以按以下步骤操作：

### 步骤 1: 使用现有模块（无需迁移）
```rust
mod models;
mod config;
mod filters;

// 直接使用即可，无需修改现有代码
```

### 步骤 2: 逐步迁移工具函数
将小函数从 main.rs 移动到相应模块

### 步骤 3: 迁移渲染方法
将大型渲染函数移动到 ui/views/ 子模块

### 步骤 4: 清理重复定义
解决 app 模块和 main.rs 中的重复定义

## 文件大小对比

| 文件 | 行数 | 说明 |
|------|------|------|
| main.rs | ~3448 | 主入口，包含所有逻辑 |
| models/mod.rs | ~100 | 数据结构定义 |
| config/mod.rs | ~60 | 配置管理 |
| filters/mod.rs | ~110 | 过滤功能 |
| ui/mod.rs | ~70 | UI基础 |
| ui/components/mod.rs | ~60 | UI组件 |
| app/mod.rs | ~230 | 应用逻辑 |

**总计**: 新模块约 630 行，大幅提升代码组织性。

## 下一步行动

1. **阅读文档**:
   - 先读 `MODULE_STRUCTURE.md` 了解模块设计
   - 再读 `MODULE_USAGE_EXAMPLES.md` 学习使用方法

2. **尝试使用**:
   - 在 main.rs 中添加模块声明
   - 尝试使用模块中的函数

3. **逐步迁移** (可选):
   - 参考 `REFACTORING.md` 进行更深度的重构

4. **保持稳定**:
   - 保留 main.rs.backup 作为备份
   - 每次修改后测试编译
