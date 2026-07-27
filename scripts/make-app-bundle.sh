#!/bin/bash
# Quick dev wrapper: build the view binary and wrap it in a .app bundle
# next to the binary so Dock and Cmd-Tab show the canview icon during
# local development. Release packaging uses scripts/package-macos.sh.

set -e

cd "$(dirname "$0")/.."

echo "🛠  Building view (debug)..."
cargo +nightly build -p view

APP_DIR="./target/debug/canview.app"
CONTENTS="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS/MacOS"
RESOURCES_DIR="$CONTENTS/Resources"

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

cp ./target/debug/view "$MACOS_DIR/canview"
chmod +x "$MACOS_DIR/canview"

if [ -f assets/ico/canview.icns ]; then
    cp assets/ico/canview.icns "$RESOURCES_DIR/canview.icns"
fi

cat > "$CONTENTS/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key><string>en</string>
    <key>CFBundleExecutable</key><string>canview</string>
    <key>CFBundleIdentifier</key><string>com.canview.app</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleName</key><string>canview</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>0.1.0</string>
    <key>CFBundleVersion</key><string>0.1.0</string>
    <key>LSMinimumSystemVersion</key><string>10.13</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>CFBundleIconFile</key><string>canview</string>
</dict>
</plist>
EOF

echo "✅ Built $APP_DIR"
echo "   Open with: open $APP_DIR"
