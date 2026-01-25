# 项目状态更新 - macOS 修复与图表功能

## ✅ macOS 编译修复

我们已经解决了 obstructing macOS 编译的依赖冲突问题。

### 解决方案

在 `Cargo.toml` 中添加了 `font-kit` 的 patch，指向 Zed 官方维护的分支：

```toml
[patch.crates-io]
ashpd = { git = "https://github.com/bilelmoussaoui/ashpd", branch = "master" }
font-kit = { git = "https://github.com/zed-industries/font-kit", branch = "master" }
```

这确保了 macOS 构建使用正确的 `core-graphics` 版本，避免了类型不匹配错误。

同时，我们恢复了 `.github/workflows/release.yml` 中的 macOS 构建任务。

## 📊 新功能：信号图表 (Chart View)

我们已经完成了信号可视化图表的基础架构实现。

### 已完成组件

1.  **数据结构** (`src/view/src/chart/data.rs`)
    -   `TimeSeriesPoint`: 时间戳和值
    -   `SignalSeries`: 信号序列数据，包含颜色、名称和可见性

2.  **渲染器** (`src/view/src/chart/renderer.rs`)
    -   `ChartRenderer`: 基于 GPUI `canvas` API 的高性能渲染器
    -   实现了网格、坐标轴背景和折线绘制
    -   使用 GPU 加速，无 plotters 依赖

3.  **UI 集成**
    -   `AppView` 枚举新增 `ChartView` 状态
    -   导航栏新增 "Chart" 按钮
    -   主视图切换逻辑

### 下一步计划 (P0)

1.  **数据集成**
    -   实现从 BLF 消息提取信号数据到 `SignalSeries` 的逻辑
    -   添加信号选择器 UI，允许用户选择 DBC 中的信号

2.  **交互增强**
    -   缩放和平移功能
    -   鼠标悬停显示具体数值

3.  **性能优化**
    -   大量数据点的降采样渲染

## 📝 验证

您可以运行 `cargo build -p view --release` 来验证 Windows 上的构建。macOS 构建将在 GitHub Actions 上自动验证。

---

**更新时间**: 2026-01-25  
**状态**: ✅ macOS 修复完成，📈 图表功能 alpha 阶段
