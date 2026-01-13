# CANVIEW 图标使用说明

## ✅ 已完成的配置

### Windows 可执行文件图标

图标已经成功配置到项目中！

#### 配置内容：

1. **Cargo.toml** - 添加了 winres 依赖
```toml
[target.'cfg(windows)'.build-dependencies]
winres = "0.1"
```

2. **build.rs** - 创建了资源编译脚本
- 自动将图标嵌入到 exe 文件中
- 仅在 Windows 平台编译

3. **图标文件**
- 位置: `assets/ico/canview.ico`
- 包含多种尺寸: 16x16 到 256x256

---

## 🚀 编译和使用

### 编译带图标的可执行文件

```cmd
cd C:\Users\Administrator\RustroverProjects\canview
cargo build --release
```

编译完成后，可执行文件位于：
```
target/release/canview.exe  (Windows)
target/release/canview      (Linux/macOS)
```

### 验证图标

1. 打开文件资源管理器
2. 导航到 `target/release/`
3. 查看 `canview.exe` 文件
4. 应该看到 CANVIEW 的图标（深色背景，5个彩色节点）

---

## 📋 当前图标资源

### 生成的文件：

```
assets/
├── ico/
│   └── canview.ico          ← Windows exe 图标
└── png/
    ├── icon_512.png         ← 高质量 PNG
    ├── icon_256.png
    ├── icon_128.png
    ├── icon_64.png
    ├── icon_48.png
    └── icon_32.png
```

### 源文件：

```
assets/
├── icon_512.svg             ← SVG 源文件
├── icon_256.svg
├── icon_128.svg
├── icon_64.svg
└── icon_32.svg
```

---

## 🎨 图标设计

### 视觉特点：
- **背景**: 深蓝灰色 (#1e293b)
- **节点**: 5个圆形节点代表 CAN 总线设备
  - 外侧: 绿色 (#34d399)
  - 中间: 蓝色 (#60a5fa)
  - 中心: 靛蓝色 (#818cf8)，稍大
- **圆角**: 56px (256尺寸)

### 设计理念：
- 简洁现代
- 适合小尺寸显示
- 易于识别
- 符合 CAN 总线技术特征

---

## 🔧 重新生成图标

如果需要重新生成图标：

### 方法1: 使用 Python 脚本（已配置）

```cmd
cd assets
python convert_icons.py
```

### 方法2: 使用在线工具

1. **SVG 转 PNG**: https://cloudconvert.com/svg-to-png
2. **PNG 转 ICO**: https://convertico.com/

### 方法3: 使用 ImageMagick

```cmd
cd assets
convert_icons.bat
```

---

## 🌐 其他平台使用

### macOS (.app 图标)

如果需要为 macOS 创建 .app bundle：

1. 转换 SVG 到 ICNS:
   ```bash
   # 使用在线工具: https://cloudconvert.com/svg-to-icns
   # 或在 macOS 上运行:
   ./convert_icons.sh
   ```

2. 创建 .app 结构:
   ```bash
   mkdir -p CanView.app/Contents/{MacOS,Resources}
   cp target/release/canview CanView.app/Contents/MacOS/
   cp canview.icns CanView.app/Contents/Resources/
   ```

### Linux (桌面图标)

安装 PNG 图标到系统：

```bash
# 用户级安装
mkdir -p ~/.local/share/icons/hicolor/256x256/apps
mkdir -p ~/.local/share/icons/hicolor/512x512/apps
cp assets/png/icon_256.png ~/.local/share/icons/hicolor/256x256/apps/canview.png
cp assets/png/icon_512.png ~/.local/share/icons/hicolor/512x512/apps/canview.png

# 创建 .desktop 文件
cat > ~/.local/share/applications/canview.desktop << EOF
[Desktop Entry]
Name=CANVIEW
Comment=Bus Data Analyzer
Exec=/path/to/canview
Icon=canview
Terminal=false
Type=Application
Categories=Development;Electronics;
EOF
```

---

## 📝 项目文件结构

```
canview/
├── build.rs              ← 资源编译脚本（新建）
├── Cargo.toml            ← 添加了 winres 依赖
├── assets/
│   ├── ico/
│   │   └── canview.ico  ← Windows 图标
│   ├── png/
│   │   └── icon_*.png   ← PNG 图标
│   ├── icon_*.svg       ← SVG 源文件
│   ├── convert_icons.py ← Python 转换脚本
│   └── ICON_GUIDE.md    ← 详细指南
├── src/
│   └── view/
│       └── src/
│           └── main.rs  ← 应用内已集成 logo
└── target/
    └── release/
        └── canview.exe  ← 编译后的可执行文件（带图标）
```

---

## ✅ 检查清单

- [x] 图标 SVG 文件已创建
- [x] PNG 图标已生成
- [x] ICO 文件已生成
- [x] Cargo.toml 已配置
- [x] build.rs 已创建
- [x] 应用内 logo 已更新
- [ ] 编译带图标的 exe（正在进行）
- [ ] 验证图标显示正确

---

## 🎯 下一步

1. 等待编译完成
2. 在文件管理器中查看 `target/release/canview.exe`
3. 验证图标显示正确
4. 如果需要，可以分发带图标的可执行文件

---

## 📞 问题排查

### 问题: exe 没有显示图标

**解决方法:**
1. 清除 Windows 图标缓存:
   ```cmd
   del %localappdata%\IconCache.db /a
   ```

2. 重新启动电脑

3. 确认编译时没有错误:
   ```cmd
   cargo clean
   cargo build --release
   ```

### 问题: 编译失败

**解决方法:**
1. 确认 winres 依赖已安装:
   ```cmd
   cargo build --release
   ```

2. 检查 ICO 文件路径是否正确

3. 查看编译错误信息

---

## 📚 相关文档

- `assets/ICON_GUIDE.md` - 详细的平台图标设置指南
- `assets/README.md` - 图标资源说明
- `assets/convert_icons.py` - Python 转换脚本

---

生成日期: 2026-01-11
版本: 1.0.0
