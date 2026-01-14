# Linux 构建说明

## 问题分析

原始需求是为 Linux 编译时添加静态链接。在尝试过程中遇到了以下问题：

1. **GUI 应用与系统库的耦合**：X11、Wayland、OpenGL 等图形库必须动态链接
2. **依赖冲突**：
   - font-kit 的 fontconfig 支持在静态链接时失败
   - ashpd 同时启用了 tokio 和 async-std 特性导致冲突
   - gpui 依赖的 zed-fork font-kit 与标准 font-kit 版本冲突

3. **musl 静态链接的局限性**：
   - C 运行时可以静态链接
   - 但 GUI 库仍然需要动态链接
   - 交叉编译复杂且容易出错

## 最终解决方案

采用 **标准 glibc 动态链接** 而非完全静态链接，原因：

### 为什么动态链接更适合 GUI 应用？

- ✅ **更好的系统集成**：可以使用系统字体、主题、输入法
- ✅ **更小的二进制文件**：不需要打包所有系统库
- ✅ **更简单的构建过程**：避免复杂的交叉编译问题
- ✅ **更好的兼容性**：自动适配不同发行版的系统库版本

### 配置说明

**Cargo.toml 关键配置**：

```toml
[dependencies]
rfd = { version = "0.14", default-features = false, features = ["tokio"] }

# 强制所有 ashpd 版本使用 tokio 特性（避免 async-std 冲突）
[patch.crates-io]
ashpd = { git = "https://github.com/bilelmoussaoui/ashpd", branch = "main", version = "0.8" }
```

**构建脚本 (`test-linux-build.sh`)**：

```bash
# 安装必要的开发库
apt-get install -y \
    build-essential pkg-config \
    libxkbcommon-dev libx11-dev libegl1-mesa-dev \
    libfontconfig1-dev libfreetype6-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# 标准构建
cargo build --release -p view
```

## 使用方法

### 在 Linux 上直接构建

```bash
# Ubuntu/Debian
sudo apt-get install -y build-essential pkg-config \
    libxkbcommon-dev libx11-dev libegl1-mesa-dev \
    libfontconfig1-dev libfreetype6-dev

cargo build --release -p view
```

### 使用 Docker 构建（推荐）

```bash
./test-linux-build.sh
```

这会：
1. 在 Docker 容器中模拟 Ubuntu 环境
2. 安装所有必要的依赖
3. 执行构建
4. 使用 `ldd` 检查二进制依赖

### 依赖说明

| 库 | 用途 |
|---|---|
| libxkbcommon-dev | 键盘布局支持 |
| libx11-dev | X11 窗口系统 |
| libegl1-mesa-dev | OpenGL 渲染 |
| libfontconfig1-dev | 字体配置 |
| libfreetype6-dev | 字体渲染 |
| libxcb-* | X11 协议支持 |

## 检查二进制文件

构建完成后可以使用 `ldd` 检查依赖：

```bash
ldd target/release/view
```

你会看到类似这样的输出：
```
	libX11.so.6 => /usr/lib/x86_64-linux-gnu/libX11.so.6
	libfontconfig.so.1 => /usr/lib/x86_64-linux-gnu/libfontconfig.so.1
	...
```

这些是正常的系统库动态链接。

## 故障排查

### 如果遇到 fontconfig 错误

确保安装了 `libfontconfig1-dev`：
```bash
sudo apt-get install libfontconfig1-dev
```

### 如果遇到 X11 错误

确保安装了 X11 开发库：
```bash
sudo apt-get install libx11-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### 如果遇到 ashpd 错误

确保 `Cargo.toml` 中的 patch 配置正确，并且只启用了 tokio 特性。

## 部署

生成的二进制文件位于 `target/release/view`。

由于使用了动态链接，部署时需要确保目标系统有必要的库：
- Ubuntu 20.04+ / Debian 11+ 通常已经包含所有需要的库
- 对于其他发行版，可能需要安装上述依赖的运行时版本（不带 -dev 后缀）

## 总结

对于这个 GUI 应用：
- ✅ 使用标准 glibc 构建（而非 musl）
- ✅ 动态链接 GUI 相关库
- ✅ 通过 patch 解决 ashpd 特性冲突
- ✅ 使用 Docker 保证构建环境一致性

这是最实用、最可靠的 Linux 构建方案。
