#!/bin/bash
# CANVIEW Icon Conversion Script for macOS and Linux
# This script converts SVG icons to PNG and ICO formats

echo "============================================"
echo "CANVIEW Icon Conversion Script"
echo "============================================"
echo ""

# Check for ImageMagick
if command -v magick &> /dev/null; then
    CONVERT_CMD="magick"
elif command -v convert &> /dev/null; then
    CONVERT_CMD="convert"
else
    echo "ERROR: ImageMagick not found!"
    echo ""
    echo "Please install ImageMagick:"
    echo "  macOS:   brew install imagemagick"
    echo "  Ubuntu:  sudo apt-get install imagemagick"
    echo "  Fedora:  sudo dnf install imagemagick"
    echo ""
    echo "Or use online converters:"
    echo "  - SVG to PNG: https://cloudconvert.com/svg-to-png"
    echo "  - PNG to ICO: https://convertico.com/"
    echo ""
    exit 1
fi

# Check for rsvg-convert (optional, better quality)
if command -v rsvg-convert &> /dev/null; then
    RSVG_AVAILABLE=true
else
    RSVG_AVAILABLE=false
fi

echo "Using: $CONVERT_CMD"
if [ "$RSVG_AVAILABLE" = true ]; then
    echo "rsvg-convert also available (will use for better quality)"
fi
echo ""

# Create output directories
mkdir -p png ico

# Convert SVG to PNG
echo "Converting SVG icons to PNG format..."
echo ""

if [ "$RSVG_AVAILABLE" = true ]; then
    # Use rsvg-convert for better quality
    echo "[1/6] Converting to 512x512 PNG..."
    rsvg-convert -w 512 -h 512 icon_512.svg -o png/icon_512.png

    echo "[2/6] Converting to 256x256 PNG..."
    rsvg-convert -w 256 -h 256 icon_256.svg -o png/icon_256.png

    echo "[3/6] Converting to 128x128 PNG..."
    rsvg-convert -w 128 -h 128 icon_128.svg -o png/icon_128.png

    echo "[4/6] Converting to 64x64 PNG..."
    rsvg-convert -w 64 -h 64 icon_64.svg -o png/icon_64.png

    echo "[5/6] Converting to 48x48 PNG..."
    rsvg-convert -w 48 -h 48 icon_64.svg -o png/icon_48.png

    echo "[6/6] Converting to 32x32 PNG..."
    rsvg-convert -w 32 -h 32 icon_32.svg -o png/icon_32.png
else
    # Use ImageMagick convert
    echo "[1/6] Converting to 512x512 PNG..."
    $CONVERT_CMD -background none icon_512.svg png/icon_512.png

    echo "[2/6] Converting to 256x256 PNG..."
    $CONVERT_CMD -background none icon_256.svg png/icon_256.png

    echo "[3/6] Converting to 128x128 PNG..."
    $CONVERT_CMD -background none icon_128.svg png/icon_128.png

    echo "[4/6] Converting to 64x64 PNG..."
    $CONVERT_CMD -background none icon_64.svg png/icon_64.png

    echo "[5/6] Converting to 48x48 PNG..."
    $CONVERT_CMD -background none -resize 48x48 icon_64.svg png/icon_48.png

    echo "[6/6] Converting to 32x32 PNG..."
    $CONVERT_CMD -background none icon_32.svg png/icon_32.png
fi

echo ""
echo "Creating ICO file for Windows..."
$CONVERT_CMD \
    png/icon_256.png \
    png/icon_128.png \
    png/icon_64.png \
    png/icon_48.png \
    png/icon_32.png \
    ico/canview.ico

# For macOS: Create ICNS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo ""
    echo "Creating ICNS for macOS..."
    ICONSET=canview.iconset
    mkdir -p $ICONSET

    # Copy PNGs to iconset
    cp png/icon_16.png $ICONSET/icon_16x16.png 2>/dev/null || rsvg-convert -w 16 -h 16 icon_32.svg -o $ICONSET/icon_16x16.png
    cp png/icon_32.png $ICONSET/icon_16x16@2x.png
    cp png/icon_32.png $ICONSET/icon_32x32.png
    cp png/icon_64.png $ICONSET/icon_32x32@2x.png
    cp png/icon_128.png $ICONSET/icon_128x128.png
    cp png/icon_256.png $ICONSET/icon_128x128@2x.png
    cp png/icon_256.png $ICONSET/icon_256x256.png
    cp png/icon_512.png $ICONSET/icon_256x256@2x.png
    cp png/icon_512.png $ICONSET/icon_512x512.png

    # Create ICNS
    iconutil -c icns $ICONSET
    rm -rf $ICONSET

    echo "macOS ICNS created: canview.icns"
fi

echo ""
echo "============================================"
echo "Conversion complete!"
echo "============================================"
echo ""
echo "Output files:"
echo "  - PNG files: ./png/"
echo "  - ICO file: ./ico/canview.ico"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  - ICNS file: ./canview.icns"
fi
echo ""
echo "Next steps:"
echo "  Windows: Use ico/canview.ico for exe icon"
echo "  macOS:   Use canview.icns for .app bundle"
echo "  Linux:   Use png/icon_256.png or png/icon_512.png"
echo ""
