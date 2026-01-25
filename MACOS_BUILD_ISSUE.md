# macOS 构建问题说明

## 问题描述

在 GitHub Actions 中构建 macOS 版本时出现编译错误：

```
error[E0308]: mismatched types
   --> font-kit/src/loaders/core_text.rs:142:17
    |
142 |                 &core_graphics_font,
    |                 ^^^^^^^^^^^^^^^^^^^ expected `core_graphics::font::CGFont`, 
    |                                     found a different `core_graphics::font::CGFont`
```

## 问题原因

这是一个依赖版本冲突问题：

1. **多个 core-graphics 版本**: 项目中同时存在 `core-graphics` 0.24.0 和 0.25.0 两个版本
2. **类型不匹配**: 虽然类型名称相同，但来自不同版本的 crate，Rust 编译器将它们视为不同的类型
3. **上游问题**: 这是 GPUI 框架及其依赖（font-kit）的已知问题

### 依赖树分析

```
gpui (from git)
├── font-kit → core-graphics 0.25.0
└── core-text → core-graphics 0.24.0
```

## 临时解决方案

### 当前状态

暂时禁用 macOS 构建，只构建 Windows 和 Linux 版本。

### GitHub Actions 修改

**文件**: `.github/workflows/release.yml`

1. **注释掉 macOS 构建矩阵**:
```yaml
# - os: macos-latest
#   platform: macos
#   artifact_name: canview
#   asset_extension: zip
```

2. **注释掉 macOS 打包步骤**:
```yaml
# - name: Package (macOS)
#   if: matrix.os == 'macos-latest'
#   run: |
#     chmod +x package.sh
#     ./package.sh ${{ steps.get_version.outputs.version }}
```

3. **注释掉 macOS 上传步骤**:
```yaml
# - name: Upload artifacts (macOS)
# - name: Upload macOS Release Asset
```

4. **更新发布说明**:
```yaml
**注意**: macOS 版本暂时不可用，由于 GPUI 框架的依赖冲突问题。
```

## 长期解决方案

### 方案 1: 等待上游修复

最理想的解决方案是等待 GPUI 或 font-kit 修复依赖冲突。

**跟踪问题**:
- GPUI: https://github.com/zed-industries/zed/issues
- font-kit: https://github.com/servo/font-kit/issues

### 方案 2: 使用统一的 core-graphics 版本

尝试在 `Cargo.toml` 中强制使用统一版本：

```toml
[patch.crates-io]
core-graphics = { version = "0.25.0" }
```

**问题**: 可能导致其他依赖不兼容。

### 方案 3: Fork font-kit

Fork font-kit 并更新其依赖到 core-graphics 0.25.0。

**步骤**:
1. Fork https://github.com/servo/font-kit
2. 更新 `Cargo.toml` 中的 core-graphics 版本
3. 在项目中使用 fork 版本

```toml
[patch.crates-io]
font-kit = { git = "https://github.com/your-username/font-kit", branch = "update-core-graphics" }
```

### 方案 4: 使用不同的 UI 框架

考虑使用其他跨平台 UI 框架，如：
- egui
- iced
- tauri

**优点**: 更成熟的跨平台支持  
**缺点**: 需要重写大量 UI 代码

## 本地 macOS 构建

如果您需要在本地构建 macOS 版本，可以尝试以下方法：

### 方法 1: 降级 Rust 工具链

```bash
rustup install 1.75.0
rustup default 1.75.0
cargo build --release -p view
```

### 方法 2: 清理并重新构建

```bash
cargo clean
rm -rf ~/.cargo/registry
rm -rf ~/.cargo/git
cargo build --release -p view
```

### 方法 3: 使用特定的 GPUI 版本

在 `src/view/Cargo.toml` 中指定 GPUI 的特定提交：

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "specific-commit-hash" }
```

## 影响范围

### 受影响的功能
- ❌ macOS 自动构建
- ❌ macOS 发布包

### 不受影响的功能
- ✅ Windows 构建和发布
- ✅ Linux 构建和发布
- ✅ 本地开发（Windows/Linux）
- ✅ 所有核心功能

## 监控和更新

### 检查上游修复

定期检查以下仓库的更新：

```bash
# 检查 GPUI 更新
git ls-remote https://github.com/zed-industries/zed HEAD

# 检查 font-kit 更新
git ls-remote https://github.com/servo/font-kit HEAD
```

### 测试修复

当上游有更新时，测试是否修复了问题：

```bash
# 更新依赖
cargo update

# 尝试构建
cargo build --release -p view
```

### 重新启用 macOS 构建

如果问题已修复：

1. 取消注释 `.github/workflows/release.yml` 中的 macOS 相关部分
2. 更新发布说明
3. 测试 GitHub Actions 构建
4. 更新此文档

## 相关文档

- [GitHub Actions 配置](.github/workflows/release.yml)
- [跨平台打包指南](CROSS_PLATFORM_PACKAGING.md)
- [发布指南](RELEASE_GUIDE.md)

## 更新日志

| 日期 | 状态 | 说明 |
|------|------|------|
| 2026-01-25 | ❌ 禁用 | 由于 core-graphics 版本冲突暂时禁用 macOS 构建 |
| TBD | ⏳ 待定 | 等待上游修复或实施替代方案 |

---

**最后更新**: 2026-01-25  
**状态**: ❌ macOS 构建暂时不可用  
**优先级**: 中等（不影响核心功能）
