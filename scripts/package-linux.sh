#!/bin/bash
# can-viewer Linux 打包脚本
# 创建 .deb, .rpm 和 .tar.gz 安装包

set -e

VERSION="${1:-1.0.0}"
APP_NAME="can-viewer"
OUTPUT_DIR="./release-package"
ARCH="amd64"  # 或 x86_64

echo "========================================"
echo "can-viewer Linux 打包脚本 v$VERSION"
echo "========================================"
echo ""

# 1. 编译 Release 版本
echo "📦 步骤 1: 编译 Release 版本..."
cargo build --release -p view
if [ $? -ne 0 ]; then
    echo "❌ 编译失败！"
    exit 1
fi
echo "✅ 编译成功！"
echo ""

# 2. 创建目录结构
echo "📁 步骤 2: 创建打包目录..."
PACKAGE_NAME="$APP_NAME-$VERSION"
PACKAGE_DIR="$OUTPUT_DIR/$PACKAGE_NAME"

# 清理旧的目录
rm -rf "$PACKAGE_DIR"

# 创建标准 Linux 目录结构
mkdir -p "$PACKAGE_DIR/usr/bin"
mkdir -p "$PACKAGE_DIR/usr/share/$APP_NAME/config/signal_library"
mkdir -p "$PACKAGE_DIR/usr/share/$APP_NAME/samples"
mkdir -p "$PACKAGE_DIR/usr/share/$APP_NAME/docs"
mkdir -p "$PACKAGE_DIR/usr/share/applications"
mkdir -p "$PACKAGE_DIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$PACKAGE_DIR/etc/$APP_NAME"

echo "✅ 目录结构创建完成！"
echo ""

# 3. 复制可执行文件
echo "📋 步骤 3: 复制文件..."
cp "./target/release/view" "$PACKAGE_DIR/usr/bin/$APP_NAME"
chmod +x "$PACKAGE_DIR/usr/bin/$APP_NAME"

# 4. 复制资源文件
if [ -f "sample.dbc" ]; then
    cp "sample.dbc" "$PACKAGE_DIR/usr/share/$APP_NAME/samples/"
fi
if [ -f "sample.blf" ]; then
    cp "sample.blf" "$PACKAGE_DIR/usr/share/$APP_NAME/samples/"
fi

# 复制文档
if [ -f "README.md" ]; then
    cp "README.md" "$PACKAGE_DIR/usr/share/$APP_NAME/docs/"
fi
if [ -f "BUILD.md" ]; then
    cp "BUILD.md" "$PACKAGE_DIR/usr/share/$APP_NAME/docs/"
fi

# 复制图标
if [ -f "assets/ico/can-viewer.png" ]; then
    cp "assets/ico/can-viewer.png" "$PACKAGE_DIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png"
elif [ -f "assets/png/icon_256.png" ]; then
    cp "assets/png/icon_256.png" "$PACKAGE_DIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png"
fi

echo "✅ 文件复制完成！"
echo ""

# 5. 创建配置文件
echo "📋 步骤 4: 创建配置文件..."
cat > "$PACKAGE_DIR/usr/share/$APP_NAME/config/default_config.json" << EOF
{
  "libraries": [],
  "mappings": [],
  "active_library_id": null,
  "active_version_name": null
}
EOF

# 创建信号库存储说明
cat > "$PACKAGE_DIR/usr/share/$APP_NAME/config/signal_library/README.txt" << EOF
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
- Linux 用户配置位置: ~/.config/can-viewer/

---
更新时间: $(date '+%Y-%m-%d %H:%M:%S')
EOF

echo "✅ 配置文件创建完成！"
echo ""

# 6. 创建 .desktop 文件
echo "📋 步骤 5: 创建桌面快捷方式..."
cat > "$PACKAGE_DIR/usr/share/applications/$APP_NAME.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=can-viewer
Comment=CAN/LIN Bus Analysis Tool
Exec=/usr/bin/$APP_NAME
Icon=$APP_NAME
Terminal=false
Categories=Development;
Keywords=CAN;LIN;Bus;Analysis;
EOF

echo "✅ 桌面快捷方式创建完成！"
echo ""

# 7. 创建 .deb 包
echo "📦 步骤 6: 创建 .deb 包..."
DEB_DIR="$OUTPUT_DIR/${APP_NAME}_${VERSION}_${ARCH}"
rm -rf "$DEB_DIR"
mkdir -p "$DEB_DIR/DEBIAN"

# 复制文件
cp -r "$PACKAGE_DIR"/* "$DEB_DIR/"

# 创建 control 文件
cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: $APP_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Maintainer: can-viewer Team <support@can-viewer.com>
Description: CAN/LIN Bus Analysis Tool
 can-viewer is a professional tool for analyzing CAN and LIN bus data.
 Features:
  - BLF file parsing and viewing
  - DBC/LDF database support
  - Multi-channel configuration
  - Signal decoding and display
  - Chart analysis
Homepage: https://github.com/cantool/can-viewer
EOF

# 创建 postinst 脚本
cat > "$DEB_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

# 创建用户配置目录
mkdir -p /etc/can-viewer
chmod 755 /etc/can-viewer

# 更新桌面数据库
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database -q
fi

# 更新图标缓存
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor
fi

echo "can-viewer 安装完成！"
echo "运行命令: can-viewer"
EOF

chmod 755 "$DEB_DIR/DEBIAN/postinst"

# 创建 prerm 脚本
cat > "$DEB_DIR/DEBIAN/prerm" << 'EOF'
#!/bin/bash
set -e
echo "正在卸载 can-viewer..."
EOF

chmod 755 "$DEB_DIR/DEBIAN/prerm"

# 构建 .deb 包
dpkg-deb --build "$DEB_DIR" "$OUTPUT_DIR/${APP_NAME}_${VERSION}_${ARCH}.deb" 2>/dev/null || {
    echo "⚠️  dpkg-deb 不可用，跳过 .deb 包创建"
}

if [ -f "$OUTPUT_DIR/${APP_NAME}_${VERSION}_${ARCH}.deb" ]; then
    echo "✅ .deb 包创建完成！"
else
    echo "⚠️  .deb 包创建失败（需要 dpkg-deb 工具）"
fi
echo ""

# 8. 创建 .rpm 包
echo "📦 步骤 7: 创建 .rpm 包..."
RPM_DIR="$(pwd)/$OUTPUT_DIR/rpm-build"
rm -rf "$RPM_DIR"
mkdir -p "$RPM_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# 创建 .spec 文件
cat > "$RPM_DIR/SPECS/$APP_NAME.spec" << EOF
Name:           $APP_NAME
Version:        $VERSION
Release:        1%{?dist}
Summary:        CAN/LIN Bus Analysis Tool

License:        MIT
URL:            https://github.com/cantool/can-viewer
Source0:        %{name}-%{version}.tar.gz

BuildArch:      x86_64
Requires:       gtk3

%description
can-viewer is a professional tool for analyzing CAN and LIN bus data.

%prep
%setup -q -c

%install
rm -rf \$RPM_BUILD_ROOT
mkdir -p \$RPM_BUILD_ROOT
cp -r * \$RPM_BUILD_ROOT/

%files
/usr/bin/%{name}
/usr/share/%{name}
/usr/share/applications/%{name}.desktop
/usr/share/icons/hicolor/256x256/apps/%{name}.png

%post
update-desktop-database &> /dev/null || :
gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor &> /dev/null || :

%changelog
* $(date '+%a %b %d %Y') can-viewer Team <support@can-viewer.com> - $VERSION-1
- Initial release
EOF

# 创建源码包
tar -czf "$RPM_DIR/SOURCES/${APP_NAME}-${VERSION}.tar.gz" -C "$PACKAGE_DIR" .

# 构建 RPM
rpmbuild --define "_topdir $RPM_DIR" -ba "$RPM_DIR/SPECS/$APP_NAME.spec" || {
    echo "⚠️  rpmbuild 不可用，跳过 .rpm 包创建"
}

if [ -f "$RPM_DIR/RPMS/x86_64/${APP_NAME}-${VERSION}-1.*.rpm" ]; then
    cp "$RPM_DIR/RPMS/x86_64/${APP_NAME}-${VERSION}-1.*.rpm" "$OUTPUT_DIR/"
    echo "✅ .rpm 包创建完成！"
else
    echo "⚠️  .rpm 包创建失败（需要 rpmbuild 工具）"
fi
echo ""

# 9. 创建 tar.gz 通用包
echo "📦 步骤 8: 创建 tar.gz 通用包..."
TAR_PATH="$OUTPUT_DIR/$APP_NAME-v$VERSION-linux-$ARCH.tar.gz"
tar -czf "$TAR_PATH" -C "$OUTPUT_DIR" "$PACKAGE_NAME"
echo "✅ tar.gz 通用包创建完成！"
echo ""

# 10. 创建 AppImage（可选）
if command -v appimagetool &> /dev/null; then
    echo "📦 步骤 9: 创建 AppImage..."
    APPDIR="$OUTPUT_DIR/$APP_NAME.AppDir"
    rm -rf "$APPDIR"
    mkdir -p "$APPDIR"
    
    cp -r "$PACKAGE_DIR/usr" "$APPDIR/"
    
    # 创建 AppRun
    cat > "$APPDIR/AppRun" << 'APPRUN_EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/can-viewer" "$@"
APPRUN_EOF
    chmod +x "$APPDIR/AppRun"
    
    # 创建 .desktop
    cp "$PACKAGE_DIR/usr/share/applications/$APP_NAME.desktop" "$APPDIR/"
    
    # 复制图标
    if [ -f "$PACKAGE_DIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png" ]; then
        cp "$PACKAGE_DIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png" "$APPDIR/$APP_NAME.png"
    fi
    
    # 构建 AppImage
    ARCH=x86_64 appimagetool "$APPDIR" "$OUTPUT_DIR/$APP_NAME-v$VERSION-x86_64.AppImage"
    echo "✅ AppImage 创建完成！"
else
    echo "⚠️  未安装 appimagetool，跳过 AppImage 创建"
    echo "   安装: https://github.com/AppImage/AppImageKit/releases"
fi
echo ""

# 完成
echo "========================================"
echo "✅ 打包完成！"
echo "========================================"
echo ""
echo "发布包位置:"
ls -lh "$OUTPUT_DIR"/*.{deb,rpm,tar.gz,AppImage} 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "安装方法:"
echo "  Debian/Ubuntu:"
echo "    sudo dpkg -i ${APP_NAME}_${VERSION}_${ARCH}.deb"
echo "    或"
echo "    sudo apt install ./${APP_NAME}_${VERSION}_${ARCH}.deb"
echo ""
echo "  Fedora/RHEL/CentOS:"
echo "    sudo rpm -i ${APP_NAME}-${VERSION}-1.*.rpm"
echo "    或"
echo "    sudo dnf install ${APP_NAME}-${VERSION}-1.*.rpm"
echo ""
echo "  通用方法:"
echo "    tar -xzf $APP_NAME-v$VERSION-linux-$ARCH.tar.gz"
echo "    cd $PACKAGE_NAME"
echo "    sudo cp -r usr/* /usr/"
echo ""
echo "  AppImage:"
echo "    chmod +x $APP_NAME-v$VERSION-x86_64.AppImage"
echo "    ./$APP_NAME-v$VERSION-x86_64.AppImage"
echo ""
