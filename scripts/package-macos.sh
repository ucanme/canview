#!/bin/bash
# can-viewer macOS 打包脚本
# 创建 .app 应用包和 .dmg 安装镜像

set -e

VERSION="${1:-1.0.0}"
APP_NAME="can-viewer"
BUNDLE_ID="com.can-viewer.app"
OUTPUT_DIR="./release-package"

echo "========================================"
echo "can-viewer macOS 打包脚本 v$VERSION"
echo "========================================"
echo ""

# 1. 编译 Release 版本
echo "📦 步骤 1: 编译 Release 版本..."
cargo build --release -p viewer
if [ $? -ne 0 ]; then
    echo "❌ 编译失败！"
    exit 1
fi
echo "✅ 编译成功！"
echo ""

# 2. 创建 .app 包结构
echo "📁 步骤 2: 创建 .app 包..."
APP_DIR="$OUTPUT_DIR/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

# 清理旧的包
rm -rf "$APP_DIR"

# 创建目录结构
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR/config/signal_library"
mkdir -p "$RESOURCES_DIR/samples"
mkdir -p "$RESOURCES_DIR/docs"

echo "✅ 目录结构创建完成！"
echo ""

# 3. 复制可执行文件
echo "📋 步骤 3: 复制文件..."
cp "./target/release/viewer" "$MACOS_DIR/can-viewer"
chmod +x "$MACOS_DIR/can-viewer"

# 4. 复制资源文件
if [ -f "sample.dbc" ]; then
    cp "sample.dbc" "$RESOURCES_DIR/samples/"
fi
if [ -f "sample.blf" ]; then
    cp "sample.blf" "$RESOURCES_DIR/samples/"
fi

# 复制文档
if [ -f "README.md" ]; then
    cp "README.md" "$RESOURCES_DIR/docs/"
fi
if [ -f "BUILD.md" ]; then
    cp "BUILD.md" "$RESOURCES_DIR/docs/"
fi

# 复制图标（如果存在）
if [ -f "assets/ico/can-viewer.icns" ]; then
    cp "assets/ico/can-viewer.icns" "$RESOURCES_DIR/can-viewer.icns"
fi

echo "✅ 文件复制完成！"
echo ""

# 5. 创建 Info.plist
echo "📋 步骤 4: 创建 Info.plist..."
cat > "$CONTENTS_DIR/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>can-viewer</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2026 can-viewer. All rights reserved.</string>
    <key>CFBundleIconFile</key>
    <string>can-viewer</string>
</dict>
</plist>
EOF

echo "✅ Info.plist 创建完成！"
echo ""

# 6. 创建默认配置
echo "📋 步骤 5: 创建配置文件..."
cat > "$RESOURCES_DIR/config/default_config.json" << EOF
{
  "libraries": [],
  "mappings": [],
  "active_library_id": null,
  "active_version_name": null
}
EOF

# 创建信号库存储说明
cat > "$RESOURCES_DIR/config/signal_library/README.txt" << EOF
# 信号库本地存储目录

此目录用于存储信号库的数据库文件。

## 目录结构

signal_library/
└── {库名}/
    └── {版本}/
        └── database.{dbc|ldf}

## 说明

- 当您在软件中添加信号库和版本时，数据库文件会自动复制到此目录
- 配置文件中保存的是此目录下的路径，确保软件可移植性

---
更新时间: $(date '+%Y-%m-%d %H:%M:%S')
EOF

echo "✅ 配置文件创建完成！"
echo ""

# 6.5. 代码签名（Ad-hoc）
echo "🔐 步骤 5.5: 进行代码签名..."
echo "注意:这是 Ad-hoc 签名，用于消除 M1/M2 上的“已损坏”错误"
codesign --force --deep --sign - "$APP_DIR"
if [ $? -eq 0 ]; then
    echo "✅ 签名成功！"
else
    echo "⚠️  签名失败，这可能会导致在 Apple Silicon 上无法运行"
fi
echo ""

# 7. 创建 DMG 镜像（可选）
if command -v create-dmg &> /dev/null; then
    echo "📦 步骤 6: 创建 DMG 镜像..."
    DMG_PATH="$OUTPUT_DIR/$APP_NAME-v$VERSION.dmg"
    
    # 删除旧的 DMG
    rm -f "$DMG_PATH"
    
    # 创建 DMG
    # 构建参数
    ARGS=(
        --volname "$APP_NAME"
        --window-pos 200 120
        --window-size 800 400
        --icon-size 100
        --icon "$APP_NAME.app" 200 190
        --hide-extension "$APP_NAME.app"
        --app-drop-link 600 185
    )

    # 只有当图标存在时才添加图标参数
    if [ -f "$RESOURCES_DIR/can-viewer.icns" ]; then
        ARGS+=(--volicon "$RESOURCES_DIR/can-viewer.icns")
    fi

    create-dmg "${ARGS[@]}" "$DMG_PATH" "$APP_DIR"
    
    echo "✅ DMG 镜像创建完成！"
    echo ""
else
    echo "⚠️  未安装 create-dmg，跳过 DMG 创建"
    echo "   安装: brew install create-dmg"
    echo ""
fi

# 8. 创建 tar.gz 压缩包
echo "📦 步骤 7: 创建 tar.gz 压缩包..."
TAR_PATH="$OUTPUT_DIR/$APP_NAME-v$VERSION-macos.tar.gz"
tar -czf "$TAR_PATH" -C "$OUTPUT_DIR" "$APP_NAME.app"
echo "✅ tar.gz 压缩包创建完成！"
echo ""

# 完成
echo "========================================"
echo "✅ 打包完成！"
echo "========================================"
echo ""
echo "发布包位置:"
echo "  应用包: $APP_DIR"
if [ -f "$DMG_PATH" ]; then
    echo "  DMG 镜像: $DMG_PATH"
fi
echo "  tar.gz: $TAR_PATH"
echo ""
echo "安装方法:"
echo "  1. 双击 .dmg 文件"
echo "  2. 将 can-viewer.app 拖到 Applications 文件夹"
echo "  或者"
echo "  1. 解压 .tar.gz 文件"
echo "  2. 将 can-viewer.app 移动到 Applications 文件夹"
echo ""
