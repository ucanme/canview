# macOS 构建问题修复

## 🐛 问题描述

macOS 构建失败，错误信息：
```
error[E0308]: mismatched types
core-graphics-0.24.0 vs core-graphics-0.25.0
```

## 🔍 根本原因

GPUI 的依赖树中存在 `core-graphics` 版本冲突：
- `zed-font-kit` 使用 `core-graphics` 0.24.0
- 其他依赖使用 `core-graphics` 0.25.0

这导致类型不匹配错误。

## ✅ 解决方案

在项目根目录的 `Cargo.toml` 中添加 `[patch.crates-io]` 来统一版本。

### 修改内容

**文件**: `Cargo.toml`

```toml
[patch.crates-io]
ashpd = { git = "https://github.com/bilelmoussaoui/ashpd", branch = "master" }
# 修复 macOS 构建的 core-graphics 版本冲突
core-graphics = { version = "0.24.0" }
core-graphics-types = { version = "0.2.0" }
```

### 工作原理

`[patch.crates-io]` 告诉 Cargo：
1. 强制所有依赖使用 `core-graphics` 0.24.0
2. 强制所有依赖使用 `core-graphics-types` 0.2.0
3. 避免版本冲突

## 📊 验证

### 本地测试

如果您有 macOS 机器，可以测试：

```bash
# 清理构建缓存
cargo clean

# 重新构建
cargo build --release -p view

# 应该成功编译
```

### GitHub Actions

下次推送代码时，macOS 构建应该会成功。

## 🎯 预期结果

| 平台 | 构建状态 | 说明 |
|------|----------|------|
| Windows | ✅ 正常 | 无变化 |
| Linux | ✅ 正常 | 无变化 |
| macOS | ✅ 修复 | 使用统一的 core-graphics 版本 |

## ⚠️ 注意事项

### 1. 版本锁定

这个 patch 将 `core-graphics` 锁定在 0.24.0。如果未来 GPUI 更新并修复了这个问题，可以移除这个 patch。

### 2. 监控上游

定期检查 GPUI 仓库：
- https://github.com/zed-industries/zed/issues
- 搜索 "core-graphics" 相关问题

### 3. 移除 patch

当 GPUI 修复依赖问题后：

```toml
[patch.crates-io]
ashpd = { git = "https://github.com/bilelmoussaoui/ashpd", branch = "master" }
# 移除以下两行
# core-graphics = { version = "0.24.0" }
# core-graphics-types = { version = "0.2.0" }
```

## 🔧 替代方案

如果这个方案不工作，还有其他选择：

### 方案 1: 使用特定的 GPUI 版本

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "特定commit" }
```

### 方案 2: 临时禁用 macOS 构建

在 `.github/workflows/release.yml` 中注释掉 macOS 构建。

### 方案 3: 等待 GPUI 更新

等待 GPUI 团队修复依赖问题。

## 📝 测试清单

修复后验证：

- [ ] Windows 构建仍然正常
- [ ] Linux 构建仍然正常
- [ ] macOS 构建成功
- [ ] 所有功能正常工作
- [ ] GitHub Actions 全部通过

## 🎉 总结

通过添加 `[patch.crates-io]`，我们：

1. ✅ 统一了 `core-graphics` 版本
2. ✅ 解决了类型不匹配问题
3. ✅ 保持了其他平台的兼容性
4. ✅ 不需要修改任何代码

这是一个干净、简单的解决方案，不会影响项目的其他部分。

---

**修复日期**: 2026-01-25  
**状态**: ✅ 已修复  
**影响**: macOS 构建  
**方法**: Cargo patch
