# CanView 代码重构计划

## 当前状态

已创建以下模块结构：

```
src/view/src/
├── models/          # 数据模型
│   └── mod.rs      # 数据结构定义 (ChannelType, ChannelMapping, AppConfig, etc.)
├── config/          # 配置管理
│   └── mod.rs      # 配置加载/保存逻辑
├── filters/         # 过滤器功能
│   └── mod.rs      # 消息过滤逻辑 (按ID、通道过滤等)
├── ui/              # UI组件
│   ├── mod.rs      # UI渲染器基础
│   ├── views/      # 视图实现
│   │   └── mod.rs
│   └── components/ # UI组件
│       └── mod.rs  # 按钮组件等
├── app/             # 应用逻辑
│   └── mod.rs      # CanViewApp 核心结构和方法
└── main.rs         # 主入口 (当前3400+行)
```

## 重构策略

由于main.rs文件较大（3400+行），建议采用渐进式重构：

### 阶段1: 数据结构分离 ✅
- [x] 创建 models 模块
- [x] 移动数据结构定义到 models/mod.rs

### 阶段2: 配置管理分离 ✅
- [x] 创建 config 模块
- [x] 移动配置相关方法到 config/mod.rs

### 阶段3: 过滤器逻辑分离 ✅
- [x] 创建 filters 模块
- [x] 移动过滤逻辑到 filters/mod.rs

### 阶段4: UI组件分离 (部分完成)
- [x] 创建 UI 模块结构
- [ ] 需要将大型渲染函数移动到独立文件：
  - render_log_view() -> ui/views/log_view.rs
  - render_config_view() -> ui/views/config_view.rs
  - render_chart_view() -> ui/views/chart_view.rs

### 阶段5: 应用状态管理
- [x] 创建 app 模块
- [ ] 需要将CanViewApp的状态管理方法移到app模块

## 推荐的重构顺序

1. **先让新模块可用**
   ```rust
   // 在 main.rs 中添加模块声明
   mod models;
   mod config;
   mod filters;
   ```

2. **逐步移动方法**
   - 从小的方法开始
   - 每次移动后测试编译
   - 保持功能不变

3. **最后处理大型渲染函数**
   - 这些函数最复杂，需要最后处理
   - 可以考虑进一步拆分

## 当前可用的模块

### models
提供以下类型：
- `ChannelType`
- `ChannelMapping`
- `AppConfig`
- `AppView`
- `ScrollbarDragState`

### config
提供以下功能：
- `ConfigManager` trait
- `load_config_from_path()`
- `save_config_to_path()`

### filters
提供以下函数：
- `filter_by_id()`
- `filter_by_channel()`
- `filter_by_id_and_channel()`
- `get_unique_ids()`
- `get_unique_channels()`
- `format_id()`

### ui
提供以下组件：
- `UiRenderer` 结构
- `render_view_button()` 函数

### app
提供：
- `CanViewApp` 结构
- 基础应用逻辑
- `ConfigManager` trait 实现

## 使用新模块

在main.rs中添加：

```rust
// 导入模块
mod app;
mod config;
mod filters;
mod models;
mod ui;

// 使用 models 中的类型
use models::{ChannelType, AppConfig, AppView};

// 使用 filters 中的函数
use filters::{filter_by_id, get_unique_ids};

// 使用 config 中的trait
use config::ConfigManager;
```

## 下一步

继续重构时建议：
1. 先让现有模块集成到main.rs中
2. 测试编译通过
3. 然后逐步移动render方法
4. 最后移动大型UI逻辑

这样可以保证应用始终可用，同时逐步改进代码结构。
