# 更新日志

所有重要的项目更改都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增
- 待添加的新功能

### 变更
- 待变更的功能

### 修复
- 待修复的问题

## [1.0.0] - 2026-01-25

### 新增
- BLF 文件解析和查看功能
- DBC/LDF 数据库支持
- 多通道配置管理
- 信号解码和显示
- 库管理界面（三栏布局）
- 版本管理功能
- 通道配置功能
- 跨平台支持（Windows、macOS、Linux）
- GitHub Actions 自动构建和发布

### 修复
- 修复新增通道时的崩溃问题
  - 移除事件回调中的 InputState 创建
  - 在 render 方法中延迟创建 InputState
  - 避免嵌套借用冲突

### 技术细节
- 使用 Rust + GPUI 框架
- 支持 CAN 和 LIN 总线
- 现代化的 Zed 风格 UI

## [0.1.0] - 2026-01-XX

### 新增
- 项目初始化
- 基础框架搭建

---

## 版本说明

### 版本号格式
版本号格式为 `主版本.次版本.修订号`，例如 `1.2.3`

- **主版本号**：当你做了不兼容的 API 修改
- **次版本号**：当你做了向下兼容的功能性新增
- **修订号**：当你做了向下兼容的问题修正

### 变更类型

- **新增** (Added)：新功能
- **变更** (Changed)：现有功能的变更
- **弃用** (Deprecated)：即将移除的功能
- **移除** (Removed)：已移除的功能
- **修复** (Fixed)：任何 bug 修复
- **安全** (Security)：安全相关的修复

### 发布流程

1. 更新此 CHANGELOG.md 文件
2. 更新 Cargo.toml 中的版本号
3. 提交更改：`git commit -am "Release v1.0.0"`
4. 创建标签：`git tag v1.0.0`
5. 推送标签：`git push origin v1.0.0`
6. GitHub Actions 将自动构建和发布

[未发布]: https://github.com/yourusername/canview/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/yourusername/canview/releases/tag/v1.0.0
[0.1.0]: https://github.com/yourusername/canview/releases/tag/v0.1.0
