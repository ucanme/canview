# GitHub Actions 更新总结

## ✅ 已完成的更新

### 1. 移除 plotters 依赖

**文件**: `src/view/Cargo.toml`

**修改**:
```diff
- plotters = { version = "0.3", default-features = false }
```

**原因**: 使用 GPUI 原生绘图 API 替代 plotters

### 2. 更新 GitHub Actions

**文件**: 
- `.github/workflows/release.yml`
- `.github/workflows/ci.yml`

**修改**: 将所有 `actions/upload-artifact@v3` 和 `actions/download-artifact@v3` 更新为 `@v4`

**原因**: GitHub 已弃用 v3 版本

**更新的 actions**:
- ✅ `actions/upload-artifact@v3` → `@v4` (9 处)
- ✅ `actions/download-artifact@v3` → `@v4` (1 处)

## ⚠️ macOS 构建问题

### 问题描述

macOS 构建失败，错误信息：
```
error[E0308]: mismatched types
core-graphics-0.24.0 vs core-graphics-0.25.0
```

### 根本原因

这是 **GPUI 上游依赖问题**，不是我们的代码问题：
- `zed-font-kit` 依赖 `core-graphics` 0.24.0
- 其他依赖使用 `core-graphics` 0.25.0
- 导致类型不匹配

### 解决方案

#### 方案 1: 等待 GPUI 更新（推荐）

GPUI 团队需要更新 `font-kit` 依赖。这是上游问题。

#### 方案 2: 临时禁用 macOS 构建

在 `.github/workflows/release.yml` 中暂时注释掉 macOS 构建：

```yaml
jobs:
  build-windows:
    # ... Windows 构建正常

  # build-macos:  # 暂时禁用
  #   runs-on: macos-latest
  #   # ...

  build-linux:
    # ... Linux 构建正常
```

#### 方案 3: 使用 Cargo patch

在项目根目录的 `Cargo.toml` 中添加：

```toml
[patch.crates-io]
# 强制所有依赖使用相同版本的 core-graphics
core-graphics = { version = "0.24.0" }
```

**注意**: 这可能导致其他问题。

#### 方案 4: 锁定 GPUI 版本

使用特定的 GPUI commit 而不是最新版本：

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "特定commit" }
```

## 📊 当前状态

| 平台 | 构建状态 | 说明 |
|------|----------|------|
| Windows | ✅ 正常 | 无问题 |
| Linux | ✅ 正常 | 无问题 |
| macOS | ❌ 失败 | GPUI 依赖冲突 |

## 🔧 建议的临时解决方案

### 1. 禁用 macOS 自动构建

修改 `.github/workflows/release.yml`：

```yaml
jobs:
  build-windows:
    runs-on: windows-latest
    # ... 保持不变

  # 暂时注释掉 macOS 构建
  # build-macos:
  #   runs-on: macos-latest
  #   steps:
  #     # ...

  build-linux:
    runs-on: ubuntu-latest
    # ... 保持不变

  create-release:
    needs: [build-windows, build-linux]  # 移除 build-macos
    # ...
```

### 2. 本地 macOS 构建

在 macOS 机器上手动构建：

```bash
# 本地构建
cargo build --release -p view

# 手动打包
./package-macos.sh 1.0.0

# 手动上传到 GitHub Release
gh release upload v1.0.0 release-package/*.dmg
```

## 📝 后续跟进

### 监控 GPUI 更新

定期检查 GPUI 仓库：
- https://github.com/zed-industries/zed/issues
- 搜索 "core-graphics" 相关问题

### 测试修复

当 GPUI 更新后：
1. 更新 GPUI 依赖
2. 重新启用 macOS 构建
3. 测试构建是否成功

## ✅ 已验证的功能

### Windows 和 Linux

- ✅ 编译成功
- ✅ 打包成功
- ✅ GitHub Actions 正常
- ✅ Artifact 上传正常

### 功能完整性

- ✅ BLF 文件加载
- ✅ 信号库管理
- ✅ 配置自动保存/加载
- ✅ 通道配置
- ✅ 文件自动复制

## 🎯 推荐行动

### 立即执行

1. ✅ 已完成：移除 plotters
2. ✅ 已完成：更新 Actions 到 v4
3. ⏳ 待执行：禁用 macOS 自动构建

### 短期计划

1. 监控 GPUI 更新
2. 本地 macOS 构建和测试
3. 手动发布 macOS 版本

### 长期计划

1. 等待 GPUI 修复依赖问题
2. 重新启用 macOS 自动构建
3. 完整的 CI/CD 流程

---

**更新日期**: 2026-01-25  
**状态**: ✅ Windows/Linux 正常，⚠️ macOS 待修复  
**优先级**: P1 (macOS 可手动构建)
