# GitHub Actions 工作流说明

本项目使用 GitHub Actions 进行持续集成和多平台构建。

## 工作流文件

### 1. CI Workflow (`.github/workflows/ci.yml`)

**用途**：快速代码质量检查，每次提交和 PR 都会触发

**检查项目**：
- ✅ Linux 快速检查（check + clippy + test）
- ✅ Windows 快速检查（check + test）
- ✅ macOS 快速检查（check + clippy + test）
- ✅ 代码格式检查（rustfmt）
- ✅ 安全审计（cargo audit）

**特点**：
- 快速反馈（5-10 分钟）
- 只检查不构建发布版本
- 并行运行多个检查任务

**触发条件**：
- Push 到 `main` 或 `dev` 分支
- 创建 Pull Request

---

### 2. Build Workflow (`.github/workflows/build.yml`)

**用途**：构建所有平台的发布版本二进制文件

**支持平台**：
- 🍎 macOS Apple Silicon (aarch64-apple-darwin)
- 🍎 macOS Intel (x86_64-apple-darwin)
- 🐧 Linux x86_64 (x86_64-unknown-linux-gnu)
- 🪟 Windows x86_64 (x86_64-pc-windows-msvc)
- 🍎 macOS Universal (自动合并 ARM64 和 x86_64)

**构建步骤**：
1. 安装平台特定的系统依赖
2. Rust 代码检查（check + clippy）
3. 构建优化的发布版本
4. 压缩二进制文件（strip 符号）
5. 打包为归档文件
6. 上传到 GitHub Artifacts
7. （可选）发布到 GitHub Releases

**触发条件**：
- Push 到 `main` 或 `dev` 分支
- 创建版本标签（如 `v1.0.0`）
- 手动触发（workflow_dispatch）

**产物**：
- `can-viewer-macos-aarch64.tar.gz`
- `can-viewer-macos-x86_64.tar.gz`
- `can-viewer-macos-universal.tar.gz`
- `can-viewer-linux-x86_64.tar.gz`
- `can-viewer-windows-x86_64.zip`

---

## 优化措施

### 1. 缓存优化

使用 `swatinem/rust-cache@v2` 缓存：
- Cargo 依赖
- 构建缓存
- 目标归档

**效果**：减少 50-70% 的构建时间

### 2. 减少外部依赖

**使用的 GitHub Actions（全部来自可信来源）**：

| Action | 用途 | 来源 |
|--------|------|------|
| `actions/checkout@v4` | 检出代码 | GitHub 官方 |
| `dtolnay/rust-toolchain@stable` | 安装 Rust | dtolnay（Rust 专家） |
| `swatinem/rust-cache@v2` | 缓存 | swatinem（社区维护） |
| `actions/upload-artifact@v4` | 上传产物 | GitHub 官方 |
| `actions/download-artifact@v4` | 下载产物 | GitHub 官方 |
| `softprops/action-gh-release@v1` | 发布版本 | softprops（社区维护） |

**特点**：
- ✅ 最小化外部 Actions 使用
- ✅ 优先使用 GitHub 官方 Actions
- ✅ 使用经过广泛验证的社区 Actions

### 3. 环境变量优化

```yaml
env:
  CARGO_REGISTRIES_CRATES_IO_PROTOCOL: sparse  # 使用稀疏索引，加速依赖解析
  RUST_BACKTRACE: 1                            # 启用详细错误追踪
```

### 4. 依赖安装优化

**Linux（Ubuntu）**：
```bash
sudo apt-get install -y --no-install-recommends \
  pkg-config \
  libxkbcommon-dev \
  libx11-dev \
  libegl1-mesa-dev \
  libfontconfig1-dev \
  libfreetype6-dev \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev
```

**特点**：
- ✅ 使用 `--no-install-recommends` 减少安装包大小
- ✅ 只安装必要的开发库
- ✅ 兼容 Ubuntu 20.04+（广泛使用的 LTS 版本）

**macOS 和 Windows**：
- ✅ 无需额外依赖
- ✅ 使用系统自带的框架和库

---

## 构建时间

| 平台 | 构建时间（首次） | 构建时间（缓存） |
|------|----------------|----------------|
| Linux | ~15-20 分钟 | ~5-8 分钟 |
| macOS (ARM) | ~20-25 分钟 | ~6-10 分钟 |
| macOS (x64) | ~20-25 分钟 | ~6-10 分钟 |
| Windows | ~15-20 分钟 | ~5-8 分钟 |

---

## 发布流程

### 1. 创建发布标签

```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

### 2. 自动触发构建

- 推送标签后自动触发 `build.yml`
- 构建所有平台的二进制文件
- 自动上传到 GitHub Releases

### 3. 下载产物

用户可以从 Releases 页面下载对应平台的二进制文件：
- https://github.com/<your-username>/can-viewer/releases

---

## 本地构建模拟

如果想在本地模拟 GitHub Actions 的构建环境：

### Linux（使用 Docker）

```bash
./test-linux-build.sh
```

### macOS

```bash
# Apple Silicon
cargo build --release -p view --target aarch64-apple-darwin

# Intel
cargo build --release -p view --target x86_64-apple-darwin

# Universal
lipo -create -output can-viewer-universal \
  target/aarch64-apple-darwin/release/view \
  target/x86_64-apple-darwin/release/view
```

### Windows

```bash
cargo build --release -p view
```

---

## 故障排查

### 构建失败

1. **检查依赖安装**：确保所有必要的系统库已安装
2. **查看日志**：点击 Actions 页面的失败任务查看详细日志
3. **本地复现**：使用相同的命令在本地运行

### 依赖冲突

如果遇到依赖冲突：
```bash
cargo clean
cargo update
cargo build --release -p view
```

### 缓存问题

如果缓存导致问题：
1. 进入 Actions 页面
2. 点击 "Caches" 删除缓存
3. 重新触发构建

---

## 最佳实践

### 开发流程

1. **创建功能分支**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **开发和测试**
   ```bash
   cargo check -p view
   cargo clippy -p view
   cargo test -p view
   ```

3. **提交并推送**
   ```bash
   git add .
   git commit -m "Add my feature"
   git push origin feature/my-feature
   ```

4. **创建 Pull Request**
   - CI 会自动运行
   - 确保所有检查通过
   - 请求代码审查

5. **合并到主分支**
   - 合并后自动触发完整构建
   - 构建产物可用于测试

### 发布流程

1. **更新版本号**（在 `Cargo.toml` 中）
2. **创建 CHANGELOG** 记录变更
3. **创建并推送标签**
4. **等待构建完成**
5. **验证并编辑 Release**

---

## 安全注意事项

- ✅ 使用官方 Actions 或广泛验证的社区 Actions
- ✅ 定期更新 Actions 版本
- ✅ 不在 Workflow 中硬编码敏感信息
- ✅ 使用 GitHub Secrets 存储敏感数据
- ✅ 定期运行安全审计（`cargo audit`）

---

## 性能优化建议

### 减少构建时间

1. **使用缓存**：已配置 `rust-cache`
2. **并行构建**：GitHub Actions 默认并行运行多个任务
3. **增量编译**：Rust 的增量编译已启用
4. **优化依赖**：减少不必要的依赖

### 减少二进制文件大小

1. **Strip 符号**：已配置
2. **使用 LTO**（Link Time Optimization）：
   ```toml
   [profile.release]
   lto = true
  codegen-units = 1
   opt-level = "z"
   ```
3. **使用 `cargo-bloat`** 检查大小区块：
   ```bash
   cargo install cargo-bloat
   cargo bloat --release -p view
   ```

---

## 总结

这套 GitHub Actions 配置提供了：

- ✅ **快速的 CI 反馈**：5-10 分钟
- ✅ **完整的多平台构建**：支持主流平台
- ✅ **自动化发布**：打标签即发布
- ✅ **最小化外部依赖**：只使用必要的 Actions
- ✅ **优化的构建时间**：使用缓存和增量编译
- ✅ **可靠的构建环境**：使用稳定的 Ubuntu 20.04

如有问题或建议，请提交 Issue 或 Pull Request！
