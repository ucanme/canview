#!/bin/bash
# CANVIEW 跨平台打包脚本 (Linux/macOS)

set -e

VERSION="${1:-1.0.0}"
OUTPUT_DIR="${2:-./release-package}"

echo "========================================"
echo "CANVIEW 打包脚本 v$VERSION"
echo "平台: $(uname -s)"
echo "========================================"
echo ""

# 检测操作系统
OS_TYPE=$(uname -s)
case "$OS_TYPE" in
    Linux*)     PLATFORM="linux";;
    Darwin*)    PLATFORM="macos";;
    *)          PLATFORM="unknown";;
esac

echo "📦 步骤 1: 编译 Release 版本..."
cargo build --release -p view
echo "✅ 编译成功！"
echo ""

# 创建发布目录
echo "📁 步骤 2: 创建发布目录..."
PACKAGE_NAME="CANVIEW-v${VERSION}-${PLATFORM}"
PACKAGE_DIR="${OUTPUT_DIR}/${PACKAGE_NAME}"

rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"/{bin,config,docs,samples,assets}

echo "✅ 目录结构创建完成！"
echo ""

# 复制可执行文件
echo "📋 步骤 3: 复制可执行文件..."
if [ "$PLATFORM" = "macos" ]; then
    cp ./target/release/view "$PACKAGE_DIR/bin/canview"
else
    cp ./target/release/view "$PACKAGE_DIR/bin/canview"
fi
chmod +x "$PACKAGE_DIR/bin/canview"
echo "✅ 可执行文件已复制"
echo ""

# 创建默认配置
echo "📋 步骤 4: 创建配置文件..."
cat > "$PACKAGE_DIR/config/default_config.json" << 'EOF'
{
  "libraries": [],
  "mappings": [],
  "active_library_id": null,
  "active_version_name": null
}
EOF

# 复制示例配置
if [ -f "./multi_channel_config.json" ]; then
    cp ./multi_channel_config.json "$PACKAGE_DIR/config/example_config.json"
fi

echo "✅ 配置文件已创建"
echo ""

# 复制示例文件
echo "📋 步骤 5: 复制示例文件..."
[ -f "./sample.dbc" ] && cp ./sample.dbc "$PACKAGE_DIR/samples/"
[ -f "./sample.blf" ] && cp ./sample.blf "$PACKAGE_DIR/samples/"
echo "✅ 示例文件已复制"
echo ""

# 复制资源文件
echo "📋 步骤 6: 复制资源文件..."
if [ -d "./assets" ]; then
    cp -r ./assets/* "$PACKAGE_DIR/assets/" 2>/dev/null || true
fi
echo "✅ 资源文件已复制"
echo ""

# 复制文档
echo "📋 步骤 7: 复制文档..."
[ -f "./README.md" ] && cp ./README.md "$PACKAGE_DIR/docs/"
[ -f "./BUILD.md" ] && cp ./BUILD.md "$PACKAGE_DIR/docs/"
[ -f "./ADD_CHANNEL_CRASH_FIX.md" ] && cp ./ADD_CHANNEL_CRASH_FIX.md "$PACKAGE_DIR/docs/"
echo "✅ 文档已复制"
echo ""

# 创建启动脚本
echo "📋 步骤 8: 创建启动脚本..."
cat > "$PACKAGE_DIR/start.sh" << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
./bin/canview
EOF
chmod +x "$PACKAGE_DIR/start.sh"
echo "✅ 启动脚本已创建"
echo ""

# 创建 README
echo "📋 步骤 9: 创建发布说明..."
cat > "$PACKAGE_DIR/README.txt" << EOF
CANVIEW v${VERSION} - ${PLATFORM}

目录结构:
  bin/        - 可执行文件
  config/     - 配置文件
  samples/    - 示例文件
  assets/     - 资源文件
  docs/       - 文档

快速开始:
  运行: ./start.sh
  或: ./bin/canview

配置文件位置:
  1. ./multi_channel_config.json (优先)
  2. ./config/default_config.json (默认)

构建时间: $(date '+%Y-%m-%d %H:%M:%S')
平台: ${PLATFORM}
版本: ${VERSION}
EOF
echo "✅ 发布说明已创建"
echo ""

# 创建压缩包
echo "📦 步骤 10: 创建压缩包..."
cd "$OUTPUT_DIR"
if [ "$PLATFORM" = "macos" ]; then
    # macOS 使用 zip
    zip -r "${PACKAGE_NAME}.zip" "${PACKAGE_NAME}" > /dev/null
    echo "✅ 压缩包已创建: ${PACKAGE_NAME}.zip"
else
    # Linux 使用 tar.gz
    tar -czf "${PACKAGE_NAME}.tar.gz" "${PACKAGE_NAME}"
    echo "✅ 压缩包已创建: ${PACKAGE_NAME}.tar.gz"
fi
cd - > /dev/null

echo ""
echo "========================================"
echo "✅ 打包完成！"
echo "========================================"
echo ""
echo "发布包位置:"
echo "  文件夹: $PACKAGE_DIR"
if [ "$PLATFORM" = "macos" ]; then
    echo "  压缩包: ${OUTPUT_DIR}/${PACKAGE_NAME}.zip"
else
    echo "  压缩包: ${OUTPUT_DIR}/${PACKAGE_NAME}.tar.gz"
fi
echo ""
