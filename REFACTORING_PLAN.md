# CAN View 渐进式重构计划

## 当前架构分析

### 代码规模统计
```
app/impls.rs:        4161 行 (巨型文件，需要拆分)
app/state.rs:         ~600 行
controllers/:        ~1270 行
  - ui_controller:     811 行 (过大)
  - library_controller: 378 行
domain/:             ~1967 行
  - config_manager:    696 行
  - log_processor:     609 行
  - signal_decoder:    382 行
  - time_handler:      262 行
```

### 当前架构层次
```
┌─────────────────────────────────────────────┐
│  UI层 (ui/, rendering/)                     │  ← 用户交互、视觉呈现
├─────────────────────────────────────────────┤
│  逻辑层 (controllers/, handlers/)           │  ← 状态管理、事件处理
├─────────────────────────────────────────────┤
│  应用层 (app/)                              │  ← 应用编排、生命周期
├─────────────────────────────────────────────┤
│  领域层 (domain/, models/)                  │  ← 业务逻辑、数据处理
├─────────────────────────────────────────────┤
│  工具层 (config/, filters/, gpui_input/)    │  ← 基础设施、通用工具
└─────────────────────────────────────────────┘
```

## 重构目标

### 1. 分层清晰
- 每层职责明确，不越界
- 依赖方向：UI → 逻辑 → 应用 → 领域 → 工具

### 2. 单一职责
- 每个模块只负责一件事
- 文件大小控制在 500 行以内

### 3. 易于测试
- 领域层完全独立，可单独测试
- 逻辑层可通过接口mock测试

### 4. 可维护性
- 代码结构清晰，易于定位
- 修改影响范围可控

## 渐进式重构计划

### 📅 Phase 1: 领域层强化 (1-2周)
**目标**: 建立稳固的业务逻辑基础

#### 1.1 完善领域模型
**文件**: `domain/models/` (新建)
```
domain/
├── models/
│   ├── mod.rs
│   ├── message.rs           # LogMessage, MessageData
│   ├── channel.rs           # Channel, ChannelConfig
│   ├── library.rs           # Library, Version, Signal
│   ├── filter.rs            # FilterCriteria, FilterType
│   └── statistics.rs        # LogStatistics, Metrics
├── services/                # 业务服务
│   ├── mod.rs
│   ├── message_service.rs   # 消息处理逻辑
│   ├── filter_service.rs    # 过滤逻辑
│   └── decode_service.rs    # 信号解码逻辑
└── repositories/            # 数据访问抽象
    ├── mod.rs
    ├── message_repository.rs
    └── library_repository.rs
```

**任务**:
- [ ] 从 `log_processor.rs` 提取 `Message` 和 `MessageData` 到 `models/message.rs`
- [ ] 从 `config_manager.rs` 提取 `ChannelConfig` 到 `models/channel.rs`
- [ ] 创建 `filter.rs` 统一过滤逻辑
- [ ] 创建单元测试框架（仅依赖domain，无GPUI）

#### 1.2 服务层拆分
**任务**:
- [ ] 将 `log_processor.rs` (609行) 拆分为:
  - `services/message_service.rs` - 消息加载、解析
  - `services/filter_service.rs` - 消息过滤
  - `services/decode_service.rs` - 信号解码
- [ ] 提取纯函数到独立模块
- [ ] 添加 `#[cfg(test)]` 测试模块

**验收标准**:
- domain/ 层无任何 GPUI 依赖
- 所有业务逻辑有单元测试覆盖
- 模块间通过 trait 交互，不直接依赖实现

---

### 📅 Phase 2: 应用层瘦身 (2-3周)
**目标**: 拆分巨型 `app/impls.rs` (4161行)

#### 2.1 应用状态分离
**文件**: `app/state.rs`
**当前**: ~600行，包含所有状态
**重构为**:
```rust
app/
├── state/
│   ├── mod.rs
│   ├── app_state.rs         # 主状态结构
│   ├── view_state.rs        # 视图切换状态
│   ├── ui_state.rs          # UI交互状态 (dropdown, filter等)
│   ├── scroll_state.rs      # 滚动状态
│   └── input_state.rs       # 输入状态
```

**任务**:
- [ ] 拆分 `state.rs` 为多个子模块
- [ ] 每个子模块独立管理自己的状态
- [ ] 使用 `pub struct` 和 `impl` 分离

#### 2.2 应用逻辑拆分
**文件**: `app/` 目录
**重构为**:
```rust
app/
├── mod.rs
├── state/                  # 状态模块
├── commands/                # 命令模式 (新建)
│   ├── mod.rs
│   ├── load_command.rs     # 文件加载
│   ├── filter_command.rs   # 过滤操作
│   ├── navigation_command.rs # 视图导航
│   └── library_command.rs  # 库管理
├── events/                  # 事件处理 (新建)
│   ├── mod.rs
│   ├── keyboard_events.rs  # 键盘事件
│   ├── mouse_events.rs     # 鼠标事件
│   └── scroll_events.rs    # 滚动事件
└── impls.rs                # 保留核心渲染逻辑，目标 <1500行
```

**任务**:
- [ ] 提取文件加载逻辑到 `commands/load_command.rs`
- [ ] 提取过滤逻辑到 `commands/filter_command.rs`
- [ ] 提取键盘事件处理到 `events/keyboard_events.rs`
- [ ] 提取滚动逻辑到 `events/scroll_events.rs`
- [ ] `impls.rs` 只保留 `render()` 和视图渲染

**验收标准**:
- `impls.rs` < 1500 行
- 每个命令/事件模块 < 300 行
- 命令可独立测试

---

### 📅 Phase 3: 逻辑层重组 (2-3周)
**目标**: 重构 controllers，建立清晰的协调层

#### 3.1 UI控制器拆分
**文件**: `controllers/ui_controller.rs` (811行 → 多个文件)
```rust
controllers/
├── ui/
│   ├── mod.rs
│   ├── log_controller.rs       # 日志视图UI (300行)
│   ├── filter_controller.rs    # 过滤器UI (200行)
│   ├── scroll_controller.rs    # 滚动控制 (200行)
│   └── layout_controller.rs    # 布局计算 (100行)
├── config_controller.rs        # 保持不变
├── library_controller.rs       # 保持不变
└── window_controller.rs        # 保持不变
```

**任务**:
- [ ] 拆分 `ui_controller.rs` 为 4 个子控制器
- [ ] 每个控制器职责单一
- [ ] 控制器通过 trait 与 domain 交互

#### 3.2 事件总线系统
**目标**: 解耦事件发送和接收

```rust
app/
├── event_bus/
│   ├── mod.rs
│   ├── event.rs              # 事件定义
│   ├── handler.rs            # 处理器trait
│   └── bus.rs               # 事件总线
```

**事件类型**:
```rust
pub enum AppEvent {
    MessageLoaded(Vec<LogMessage>),
    FilterChanged(FilterCriteria),
    ViewChanged(AppView),
    LibraryUpdated(Library),
    ScrollRequested(ScrollPosition),
}
```

**任务**:
- [ ] 定义事件类型
- [ ] 实现简单的发布-订阅机制
- [ ] 重构现有的事件处理使用事件总线

**验收标准**:
- controllers/ 平均文件大小 < 300行
- 事件处理逻辑清晰，易追踪
- 可添加日志记录所有事件

---

### 📅 Phase 4: UI层组件化 (2-3周)
**目标**: 提升UI组件复用性，统一视觉风格

#### 4.1 基础组件库
**文件**: `ui/components/`
```rust
ui/components/
├── mod.rs
├── button/
│   ├── mod.rs
│   ├── button.rs
│   └── variants.rs          # IconButton, TextButton等
├── input/
│   ├── mod.rs
│   ├── text_input.rs        # 已存在，需优化
│   └── number_input.rs
├── dropdown/
│   ├── mod.rs
│   ├── dropdown.rs
│   └── dropdown_item.rs
├── table/
│   ├── mod.rs
│   ├── table.rs
│   ├── header.rs
│   └── row.rs
└── scroll/
    ├── mod.rs
    ├── scrollbar.rs         # 提取当前滚动条逻辑
    └── scroll_handle.rs
```

**任务**:
- [ ] 从 `app/impls.rs` 提取滚动条逻辑到 `scroll/scrollbar.rs`
- [ ] 从 `app/impls.rs` 提取下拉菜单到 `dropdown/`
- [ ] 创建表格组件系统
- [ ] 统一组件样式

#### 4.2 视图模板化
**文件**: `ui/views/`
```rust
ui/views/
├── mod.rs
├── templates/
│   ├── mod.rs
│   ├── list_view.rs         # 列表视图模板
│   ├── detail_view.rs       # 详情视图模板
│   └── split_view.rs        # 分屏视图模板
├── log_view.rs              # 优化现有实现
└── library_view.rs          # 优化现有实现
```

**任务**:
- [ ] 创建可复用的列表视图模板
- [ ] 统一视图间切换逻辑
- [ ] 提取公共布局模式

**验收标准**:
- UI组件有完整的文档和示例
- 组件可独立测试和使用
- 新增视图可快速组装

---

### 📅 Phase 5: 工具层完善 (1-2周)
**目标**: 提升基础设施质量

#### 5.1 配置系统改进
**文件**: `config/`
```rust
config/
├── mod.rs
├── constants.rs            # 已有
├── app_config.rs           # 应用配置结构
├── config_loader.rs        # 配置加载
├── config_saver.rs         # 配置保存
└── config_validator.rs     # 配置验证
```

**任务**:
- [ ] 增强配置验证（类型安全）
- [ ] 支持配置迁移（版本升级）
- [ ] 添加配置默认值和文档

#### 5.2 错误处理
**文件**: `error.rs` (新建)
```rust
error/
├── mod.rs
├── app_error.rs            # 应用级错误
├── io_error.rs             # IO错误
└── parse_error.rs          # 解析错误
```

**任务**:
- [ ] 定义清晰的错误类型
- [ ] 实现友好的错误展示
- [ ] 添加错误日志记录

**验收标准**:
- 配置系统健壮，有错误恢复
- 所有错误有用户友好的提示
- 可通过配置调整详细程度

---

### 📅 Phase 6: 性能优化 (1-2周)
**目标**: 提升大数据量场景性能

#### 6.1 虚拟滚动优化
**当前**: uniform_list 已使用
**优化**:
- [ ] 实现智能预加载（提前加载可见范围外3-5行）
- [ ] 优化滚动节流（减少重绘）
- [ ] 实现滚动位置缓存

#### 6.2 过滤性能
**当前**: 50万条数据过滤卡顿
**优化**:
- [ ] 实现增量过滤（过滤结果缓存）
- [ ] 使用索引加速ID查找
- [ ] 后台线程预处理（可选）

#### 6.3 内存优化
**任务**:
- [ ] 实现 message 数据的分页加载
- [ ] 优化字符串存储（如使用 `Cow<str>`）
- [ ] 及时释放不需要的数据

**验收标准**:
- 50万条数据流畅滚动
- 过滤响应 < 100ms
- 内存占用合理

---

### 📅 Phase 7: 测试与文档 (持续)
**目标**: 提升代码质量和可维护性

#### 7.1 测试体系
```
tests/
├── unit/                   # 单元测试
│   ├── domain/            # 领域层测试
│   ├── services/          # 服务层测试
│   └── utils/             # 工具层测试
├── integration/           # 集成测试
│   ├── commands/          # 命令测试
│   └── controllers/       # 控制器测试
└── e2e/                   # 端到端测试
    └── scenarios/         # 用户场景测试
```

**任务**:
- [ ] Domain层测试覆盖率 > 80%
- [ ] 关键路径有集成测试
- [ ] 性能基准测试（如50万条数据加载时间）

#### 7.2 文档完善
**文档结构**:
```
docs/
├── architecture/          # 架构文档
│   ├── overview.md        # 系统架构
│   ├── layers.md          # 分层说明
│   └── data-flow.md       # 数据流
├── api/                   # API文档
│   ├── domain.md          # 领域层API
│   ├── controllers.md     # 控制器API
│   └── components.md      # 组件API
├── guides/                # 使用指南
│   ├── adding-view.md     # 如何添加视图
│   ├── custom-filter.md   # 如何自定义过滤
│   └── theme.md           # 主题定制
└── development/           # 开发指南
    ├── setup.md           # 环境搭建
    ├── contributing.md    # 贡献指南
    └── debugging.md       # 调试技巧
```

**任务**:
- [ ] API文档自动生成（使用 rustdoc）
- [ ] 架构决策记录（ADR）
- [ ] 代码示例和教程

**验收标准**:
- 新开发者可在30分钟内搭建环境
- 核心功能有完整文档
- 文档与代码保持同步

---

## 实施原则

### 1. 渐进式，不中断开发
- 每个Phase可独立完成
- 每次改动保持功能可用
- 使用特性分支逐步合并

### 2. 向后兼容
- 不改变外部API
- 配置文件格式兼容
- 数据文件格式兼容

### 3. 测试驱动
- 先写测试，再重构
- 重构前后测试通过
- 性能基准不下降

### 4. 代码审查
- 每个PR需要审查
- 重要重构集体讨论
- 记录架构决策

## 成功指标

### 代码质量
- [ ] 最大文件 < 500行
- [ ] 圈复杂度 < 10
- [ ] 测试覆盖率 > 70%

### 性能
- [ ] 50万条数据加载 < 2s
- [ ] 过滤响应 < 100ms
- [ ] 滚动流畅（60fps）

### 可维护性
- [ ] 新增功能时间减少50%
- [ ] Bug定位时间减少60%
- [ ] 代码审查时间减少40%

## 风险与应对

### 风险1: 重构引入新Bug
**应对**:
- 完善的测试覆盖
- 灰度发布策略
- 快速回滚机制

### 风险2: 开发周期延长
**应对**:
- 优先级管理
- 并行开发（新功能 + 重构）
- 定期回顾调整计划

### 风险3: 团队协作成本
**应对**:
- 清晰的接口定义
- 完善的文档
- 定期技术分享

## 下一步行动

### 立即开始（本周）
1. 创建重构分支 `refactor/domain-layer`
2. 完成 Phase 1.1：创建 `domain/models/`
3. 提取第一个模型（message.rs）

### 短期计划（本月）
1. 完成 Phase 1：领域层重构
2. 开始 Phase 2.1：状态拆分

### 中期计划（本季度）
1. 完成 Phase 2-3：应用层和逻辑层
2. 完成基础重构

### 长期计划（本年度）
1. 完成 Phase 4-7：优化和完善
2. 建立持续改进机制

---

**文档版本**: v1.0
**创建日期**: 2025-02-09
**负责人**: Architecture Team
**审查周期**: 每月回顾更新
