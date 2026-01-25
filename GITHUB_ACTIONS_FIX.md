# GitHub Actions 依赖修复

## 问题描述

在 GitHub Actions 中构建时出现错误：
```
error: failed to load manifest for dependency `gpui-component`
Caused by:
  failed to read `/Users/runner/work/canview/gpui-component/crates/ui/Cargo.toml`
Caused by:
  No such file or directory (os error 2)
```

## 问题原因

`src/view/Cargo.toml` 中的 `gpui-component` 依赖使用了本地路径：
```toml
gpui-component = { path = "../../../gpui-component/crates/ui" }
```

这个路径在 GitHub Actions 环境中不存在，导致构建失败。

## 解决方案

### 修改依赖为 Git 仓库引用

**文件**: `src/view/Cargo.toml`

```toml
# 修改前
gpui-component = { path = "../../../gpui-component/crates/ui" }

# 修改后
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
```

### 更新 GitHub Actions 配置

**文件**: `.github/workflows/release.yml`

添加了以下改进：

1. **子模块检出**（以防将来需要）：
```yaml
- name: Checkout code
  uses: actions/checkout@v4
  with:
    submodules: recursive  # 检出所有子模块
```

2. **改进的缓存策略**：
```yaml
- name: Cache cargo registry
  uses: actions/cache@v3
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-registry-
```

## 验证

### 本地测试
```bash
# 更新依赖
cargo update -p gpui-component

# 检查编译
cargo check -p view
```

### GitHub Actions 测试
推送代码后，GitHub Actions 将自动运行构建。

## 依赖信息

- **仓库**: https://github.com/longbridge/gpui-component
- **版本**: 0.5.0
- **提交**: 35f55766

## 注意事项

1. **网络依赖**: 现在依赖于 GitHub 仓库，需要网络连接
2. **版本锁定**: 使用 `Cargo.lock` 锁定具体版本
3. **缓存**: GitHub Actions 会缓存依赖以加速构建

## 后续步骤

1. ✅ 修改依赖配置
2. ✅ 更新 GitHub Actions
3. ✅ 本地测试编译
4. ⏳ 推送到 GitHub 测试 CI

## 相关文件

- `src/view/Cargo.toml` - 依赖配置
- `.github/workflows/release.yml` - CI/CD 配置
- `Cargo.lock` - 依赖版本锁定

---

**修复日期**: 2026-01-25
**状态**: ✅ 已完成
