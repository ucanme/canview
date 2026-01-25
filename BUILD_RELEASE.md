# 快速开始：构建和发布

## 本地构建

### Windows
```cmd
package.bat
```

### Linux/macOS
```bash
chmod +x package.sh
./package.sh 1.0.0
```

## GitHub Actions 自动发布

### 创建新版本

```bash
# 1. 更新 CHANGELOG.md 和版本号
# 2. 提交更改
git add .
git commit -m "Release v1.0.0"

# 3. 创建并推送标签
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions 将自动：
- ✅ 在 Linux、macOS、Windows 上构建
- ✅ 创建 GitHub Release
- ✅ 上传所有平台的压缩包

## 输出文件

### Windows
- `CANVIEW-v1.0.0.zip`
  - 包含 `canview.exe` 和配置文件

### macOS
- `CANVIEW-v1.0.0-macos.zip`
  - 包含 `canview` 可执行文件和配置文件

### Linux
- `CANVIEW-v1.0.0-linux.tar.gz`
  - 包含 `canview` 可执行文件和配置文件

## 详细文档

- [完整发布指南](RELEASE_GUIDE.md)
- [打包指南](PACKAGING_GUIDE.md)
- [更新日志](CHANGELOG.md)

## 目录结构

```
发布包/
├── bin/
│   └── canview[.exe]    # 可执行文件
├── config/
│   ├── default_config.json
│   └── example_config.json
├── samples/
│   ├── sample.dbc
│   └── sample.blf
├── docs/
│   └── README.md
├── assets/
│   └── (资源文件)
└── start.[bat|sh]       # 启动脚本
```

## 系统要求

### Windows
- Windows 10 或更高版本
- 64 位系统

### macOS
- macOS 10.15 (Catalina) 或更高版本
- Intel 或 Apple Silicon

### Linux
- 现代 Linux 发行版（Ubuntu 20.04+, Fedora 35+, 等）
- X11 或 Wayland
- 64 位系统

## 故障排除

### 构建失败
```bash
# 清理并重新构建
cargo clean
cargo build --release -p view
```

### 权限错误 (Linux/macOS)
```bash
chmod +x package.sh
chmod +x bin/canview
```

### 依赖缺失 (Linux)
```bash
# Ubuntu/Debian
sudo apt-get install libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

---

**需要帮助？** 查看 [RELEASE_GUIDE.md](RELEASE_GUIDE.md) 获取详细说明。
