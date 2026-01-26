# CI/CD Pipeline 说明文档

## 概述

全新的 CI/CD pipeline 配置，专注于 Windows 和 Linux 平台的自动化构建、测试和发布。

## 文件结构

```
.github/workflows/
├── main.yml          # 主 CI/CD pipeline（新）✨
├── release.yml       # 旧的发布配置（已弃用）
├── ci.yml            # 旧的 CI 配置（已弃用）
└── build.yml         # 旧的构建配置（已弃用）
```

**推荐**: 使用 `main.yml`，可以删除其他旧配置文件。

## Pipeline 结构

### 1. 代码检查和测试 (check)

**运行条件**: 所有推送、PR 和手动触发

**平台**: Linux + Windows

**步骤**:
- ✅ 代码格式检查 (`cargo fmt`)
- ✅ Clippy 静态分析 (`cargo clippy`)
- ✅ 单元测试 (`cargo test`)

### 2. 构建发布版本 (build)

**运行条件**: 代码检查通过后

**平台**: 
- Linux x86_64
- Windows x86_64

**输出**:
- `canview-{version}-linux-x86_64.tar.gz`
- `canview-{version}-windows-x86_64.zip`

### 3. 创建 Release (release)

**运行条件**: 推送 `v*` 标签时

**功能**:
- 自动创建 GitHub Release
- 上传所有平台的构建产物
- 生成发布说明

### 4. 构建状态通知 (notify)

**运行条件**: 总是运行

**功能**:
- 检查所有 job 的状态
- 报告构建成功或失败

## 触发方式

### 1. 自动触发

#### 推送到主分支
```bash
git push origin main
```
→ 运行代码检查和构建

#### 推送到开发分支
```bash
git push origin dev
```
→ 运行代码检查和构建

#### 创建 Pull Request
→ 运行代码检查和构建

#### 推送标签（发布）
```bash
git tag v1.0.0
git push origin v1.0.0
```
→ 运行完整流程 + 创建 Release

### 2. 手动触发

1. 访问 GitHub Actions 页面
2. 选择 "CI/CD Pipeline" workflow
3. 点击 "Run workflow"
4. 选择分支
5. （可选）输入版本号
6. 点击 "Run workflow"

## 版本号规则

| 触发方式 | 版本号格式 | 示例 |
|---------|-----------|------|
| 标签推送 | 标签名（去掉 v） | `v1.0.0` → `1.0.0` |
| 手动触发 | 用户输入 | `1.2.3` 或 `dev` |
| 其他推送 | `dev-{commit}` | `dev-abc1234` |

## 构建产物

### Linux

**文件名**: `canview-{version}-linux-x86_64.tar.gz`

**内容**:
```
canview              # 可执行文件
```

**使用**:
```bash
tar xzf canview-1.0.0-linux-x86_64.tar.gz
./canview
```

### Windows

**文件名**: `canview-{version}-windows-x86_64.zip`

**内容**:
```
canview.exe          # 可执行文件
```

**使用**:
```cmd
# 解压 ZIP 文件
canview.exe
```

## 缓存策略

使用 `Swatinem/rust-cache@v2` 缓存：
- Cargo 依赖
- 构建产物
- 按操作系统和目标平台分别缓存

**优势**:
- 加速构建（首次构建后）
- 减少网络流量
- 自动清理过期缓存

## 平台支持

| 平台 | 架构 | 状态 | 说明 |
|------|------|------|------|
| **Linux** | x86_64 | ✅ 支持 | 完全支持 |
| **Windows** | x86_64 | ✅ 支持 | 完全支持 |
| **macOS** | ARM64 | ❌ 禁用 | 依赖冲突 |
| **macOS** | x86_64 | ❌ 禁用 | 依赖冲突 |

### macOS 支持计划

macOS 构建因 GPUI 框架的 `core-graphics` 依赖冲突暂时禁用。

**重新启用条件**:
1. GPUI 修复依赖冲突
2. 或使用其他 UI 框架
3. 或 fork font-kit 并修复

**跟踪**: 参见 `MACOS_BUILD_ISSUE.md`

## 使用示例

### 开发流程

```bash
# 1. 开发功能
git checkout -b feature/new-feature
# ... 编写代码 ...

# 2. 提交代码
git add .
git commit -m "Add new feature"

# 3. 推送并创建 PR
git push origin feature/new-feature
# → GitHub Actions 自动运行检查

# 4. 合并到 main
git checkout main
git merge feature/new-feature
git push origin main
# → GitHub Actions 自动构建
```

### 发布流程

```bash
# 1. 更新版本号和文档
vim Cargo.toml
vim CHANGELOG.md

# 2. 提交更改
git add .
git commit -m "Release v1.0.0"
git push origin main

# 3. 创建并推送标签
git tag v1.0.0
git push origin v1.0.0

# → GitHub Actions 自动:
#   - 运行所有检查
#   - 构建所有平台
#   - 创建 GitHub Release
#   - 上传构建产物
```

## 环境变量

### 全局环境变量

```yaml
CARGO_TERM_COLOR: always    # 彩色输出
RUST_BACKTRACE: 1           # 启用回溯
```

### Job 特定变量

在 Release job 中：
```yaml
GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## 权限配置

### Release Job

```yaml
permissions:
  contents: write    # 允许创建 Release 和上传文件
```

## 故障排除

### 问题：检查失败

**Clippy 警告**:
```bash
# 本地运行 clippy
cargo clippy -p view -- -D warnings

# 修复自动修复的问题
cargo fix --allow-dirty
```

**格式问题**:
```bash
# 本地格式化代码
cargo fmt --all
```

### 问题：构建失败

**依赖问题**:
```bash
# 清理并重新构建
cargo clean
cargo build --release -p view
```

**Linux 依赖缺失**:
- 检查 workflow 中的 `apt-get install` 命令
- 确保所有必要的库都已列出

### 问题：Release 创建失败

**权限不足**:
- 检查仓库设置 > Actions > General
- 确保 "Workflow permissions" 设置为 "Read and write permissions"

**标签格式错误**:
- 标签必须以 `v` 开头，例如 `v1.0.0`
- 不要使用 `1.0.0` 这样的格式

### 问题：构建产物未上传

**检查步骤**:
1. 查看 Actions 日志
2. 确认 "Upload artifact" 步骤成功
3. 检查文件路径是否正确

## 优化建议

### 1. 加速构建

```yaml
# 使用 sccache
- name: Setup sccache
  uses: mozilla-actions/sccache-action@v0.0.3

- name: Build with sccache
  run: cargo build --release -p view
  env:
    RUSTC_WRAPPER: sccache
```

### 2. 并行测试

```yaml
- name: Run tests
  run: cargo test -p view --jobs 4
```

### 3. 增量构建

已通过 `rust-cache` 实现。

## 迁移指南

### 从旧配置迁移

1. **备份旧配置**:
```bash
mkdir .github/workflows/old
mv .github/workflows/{release,ci,build}.yml .github/workflows/old/
```

2. **使用新配置**:
```bash
# main.yml 已经创建
git add .github/workflows/main.yml
git commit -m "Add new CI/CD pipeline"
git push
```

3. **测试**:
```bash
# 推送一个测试分支
git checkout -b test/new-pipeline
git push origin test/new-pipeline
# 检查 Actions 是否正常运行
```

4. **清理**:
```bash
# 确认新配置工作正常后
rm -rf .github/workflows/old
```

## 监控和维护

### 查看构建状态

- GitHub 仓库页面的 Actions 标签
- README 中添加状态徽章:

```markdown
![CI/CD](https://github.com/your-username/canview/workflows/CI%2FCD%20Pipeline/badge.svg)
```

### 定期维护

- 每月检查依赖更新
- 更新 GitHub Actions 版本
- 检查缓存使用情况

## 相关文档

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Rust CI/CD 最佳实践](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [MACOS_BUILD_ISSUE.md](MACOS_BUILD_ISSUE.md) - macOS 构建问题说明

---

**创建日期**: 2026-01-25  
**版本**: 2.0  
**状态**: ✅ 生产就绪
