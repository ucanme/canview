# CANVIEW 交叉编译指南

## 🚀 GitHub Actions 自动编译

最简单的方式是使用 GitHub Actions 自动编译所有平台的版本。

### 使用方法

#### 1. 推送代码到 GitHub

确保你的代码已经推送到 GitHub 仓库：

```bash
git add .
git commit -m "Add cross-compilation support"
git push
```

#### 2. 触发 GitHub Actions

GitHub Actions 会在以下情况自动运行：
- 推送代码到 `main` 或 `dev` 分支
- 创建 Pull Request
- 创建 tag（格式：`v*`）
- 手动触发（在 GitHub Actions 页面点击 "Run workflow"）

#### 3. 下载编译好的二进制文件

1. 访问你仓库的 GitHub Actions 页面
2. 点击最近的 workflow run
3. 在 "Artifacts" 部分下载你需要的平台版本

### 支持的平台

| 平台 | 文件名 | 架构 |
|------|--------|------|
| macOS ARM (Apple Silicon) | `canview-macos-aarch64.tar.gz` | aarch64 |
| macOS Intel | `canview-macos-x86_64.tar.gz` | x86_64 |
| macOS Universal (二合一) | `canview-macos-universal.tar.gz` | aarch64 + x86_64 |
| Linux | `canview-linux-x86_64.tar.gz` | x86_64 |
| Windows | `canview-windows-x86_64.zip` | x86_64 |

### 自动发布

当创建 tag 时（如 `v1.0.0`），GitHub Actions 会自动创建 Release 并上传所有平台的二进制文件。

```bash
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

---

## 💻 本地交叉编译

如果你需要在本地交叉编译，可以使用以下方法：

### 方法 1: 使用 cargo-zigbuild（推荐）

```bash
# 安装 cargo-zigbuild
cargo install cargo-zigbuild

# 下载 Zig (https://ziglang.org/download/)

# 编译 macOS ARM
cargo zigbuild --release --bin view --target aarch64-apple-darwin

# 编译 Linux ARM64
cargo zigbuild --release --bin view --target aarch64-unknown-linux-gnu
```

### 方法 2: 使用 osxcross

编译 macOS 需要 Apple SDK。可以使用 osxcross：

```bash
# macOS ARM
cargo build --release --bin view --target aarch64-apple-darwin
```

**注意**: 在 Windows/Linux 上交叉编译 macOS 需要 macOS SDK，这通常比较复杂。

### 方法 3: 使用 Docker

```bash
# Linux 交叉编译
docker run --rm -v $(pwd):/app -w /app rust:latest cargo build --release
```

---

## 📦 各平台详细说明

### macOS

#### 在 Mac 上编译（推荐）
如果你有 Mac 电脑，这是最简单的方式：

```bash
# Apple Silicon (M1/M2/M3)
cargo build --release --target aarch64-apple-darwin

# Intel
cargo build --release --target x86_64-apple-darwin

# Universal Binary (同时支持两种架构)
lipo -create \
  target/aarch64-apple-darwin/release/view \
  target/x86_64-apple-darwin/release/view \
  -output target/view-universal
```

#### 创建 .app bundle

```bash
mkdir -p CanView.app/Contents/{MacOS,Resources}
cat > CanView.app/Contents/Info.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>view</string>
    <key>CFBundleIdentifier</key>
    <string>com.canview.app</string>
    <key>CFBundleName</key>
    <string>CANVIEW</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
</dict>
</plist>
EOF

cp target/aarch64-apple-darwin/release/view CanView.app/Contents/MacOS/
cp assets/png/icon_512.png CanView.app/Contents/Resources/AppIcon.icns
```

### Linux

```bash
# x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# ARM64 (需要交叉编译工具链)
cargo build --release --target aarch64-unknown-linux-gnu
```

依赖安装：
```bash
sudo apt-get install libxkbcommon-dev libx11-dev libegl1-mesa-dev libfontconfig1-dev
```

### Windows

```bash
# x86_64 (在 Windows 上)
cargo build --release --target x86_64-pc-windows-msvc

# 或在 Linux 上交叉编译
cargo build --release --target x86_64-pc-windows-msvc --target x86_64-pc-windows-gnu
```

---

## 🎯 推荐工作流

1. **日常开发**: 在本地平台直接编译 (`cargo build --release`)
2. **发布多平台版本**: 使用 GitHub Actions
3. **快速测试特定平台**: 使用 cargo-zigbuild 或在对应平台的机器上编译

---

## 🔧 故障排查

### GitHub Actions 失败

1. 检查 `.github/workflows/build.yml` 语法
2. 查看 Actions 日志中的具体错误信息
3. 确保 `Cargo.toml` 配置正确

### 交叉编译依赖问题

某些依赖（如 `gpui`）可能需要特定平台的库：
- macOS: 需要 Xcode 命令行工具
- Linux: 需要 X11、Wayland 等图形库
- Windows: 需要 MSVC 或 MinGW

### 图标和资源

Windows 图标嵌入需要 `winres`，已在 `src/view/build.rs` 中配置。
其他平台的图标和应用图标需要额外的打包工具。

---

## 📚 相关资源

- [Rust Cross-Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)
- [osxcross](https://github.com/macports/osxcross)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)

---

更新时间: 2026-01-13
版本: 1.0.0
