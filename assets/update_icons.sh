#!/bin/bash
# can-viewer Icon Conversion Script for macOS and Linux
# This script converts the new logo.svg to PNG and ICO formats

echo "============================================"
echo "can-viewer Icon Conversion Script"
echo "Using new logo from assets/svg/logo.svg"
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

# Source SVG file
LOGO_SVG="svg/logo.svg"

# Check if logo.svg exists
if [ ! -f "$LOGO_SVG" ]; then
    echo "ERROR: $LOGO_SVG not found!"
    echo "Please ensure the logo file exists at: $LOGO_SVG"
    exit 1
fi

# Create output directories
mkdir -p png ico

# Convert SVG to PNG at different sizes
echo "Converting $LOGO_SVG to PNG format..."
echo ""

if [ "$RSVG_AVAILABLE" = true ]; then
    # Use rsvg-convert for better quality
    echo "[1/7] Converting to 512x512 PNG..."
    rsvg-convert -w 512 -h 512 "$LOGO_SVG" -o png/logo_512.png

    echo "[2/7] Converting to 256x256 PNG..."
    rsvg-convert -w 256 -h 256 "$LOGO_SVG" -o png/logo_256.png

    echo "[3/7] Converting to 128x128 PNG..."
    rsvg-convert -w 128 -h 128 "$LOGO_SVG" -o png/logo_128.png

    echo "[4/7] Converting to 64x64 PNG..."
    rsvg-convert -w 64 -h 64 "$LOGO_SVG" -o png/logo_64.png

    echo "[5/7] Converting to 48x48 PNG..."
    rsvg-convert -w 48 -h 48 "$LOGO_SVG" -o png/logo_48.png

    echo "[6/7] Converting to 32x32 PNG..."
    rsvg-convert -w 32 -h 32 "$LOGO_SVG" -o png/logo_32.png

    echo "[7/7] Converting to 16x16 PNG..."
    rsvg-convert -w 16 -h 16 "$LOGO_SVG" -o png/logo_16.png
else
    # Use ImageMagick convert
    echo "[1/7] Converting to 512x512 PNG..."
    $CONVERT_CMD -background none "$LOGO_SVG" -resize 512x512 png/logo_512.png

    echo "[2/7] Converting to 256x256 PNG..."
    $CONVERT_CMD -background none "$LOGO_SVG" -resize 256x256 png/logo_256.png

    echo "[3/7] Converting to 128x128 PNG..."
    $CONVERT_CMD -background none "$LOGO_SVG" -resize 128x128 png/logo_128.png

    echo "[4/7] Converting to 64x64 PNG..."
    $CONVERT_CMD -background none "$LOGO_SVG" -resize 64x64 png/logo_64.png

    echo "[5/7] Converting to 48x48 PNG..."
    $CONVERT_CMD -background none "$LOGO_SVG" -resize 48x48 png/logo_48.png

    echo "[6/7] Converting to 32x32 PNG..."
    $CONVERT_CMD -background none "$LOGO_SVG" -resize 32x32 png/logo_32.png

    echo "[7/7] Converting to 16x16 PNG..."
    $CONVERT_CMD -background none "$LOGO_SVG" -resize 16x16 png/logo_16.png
fi

echo ""
echo "Creating ICO file for Windows..."
$CONVERT_CMD \
    png/logo_256.png \
    png/logo_128.png \
    png/logo_64.png \
    png/logo_48.png \
    png/logo_32.png \
    png/logo_16.png \
    ico/can-viewer.ico

# For macOS: Create ICNS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo ""
    echo "Creating ICNS for macOS..."
    ICONSET=can-viewer.iconset
    rm -rf $ICONSET
    mkdir -p $ICONSET

    # Copy PNGs to iconset with proper naming
    cp png/logo_16.png $ICONSET/icon_16x16.png
    cp png/logo_32.png $ICONSET/icon_16x16@2x.png
    cp png/logo_32.png $ICONSET/icon_32x32.png
    cp png/logo_64.png $ICONSET/icon_32x32@2x.png
    cp png/logo_128.png $ICONSET/icon_128x128.png
    cp png/logo_256.png $ICONSET/icon_128x128@2x.png
    cp png/logo_256.png $ICONSET/icon_256x256.png
    cp png/logo_512.png $ICONSET/icon_256x256@2x.png
    cp png/logo_512.png $ICONSET/icon_512x512.png

    # Create ICNS
    iconutil -c icns $ICONSET
    rm -rf $ICONSET

    echo "macOS ICNS created: can-viewer.icns"
fi

echo ""
echo "============================================"
echo "Conversion complete!"
echo "============================================"
echo ""
echo "Output files:"
echo "  - PNG files: ./png/"
echo "  - ICO file: ./ico/can-viewer.ico"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  - ICNS file: ./can-viewer.icns"
fi
echo ""
echo "Next steps:"
echo "  Windows: Run 'cargo build --release' (build.rs will use ico/can-viewer.ico)"
echo "  macOS:   Use can-viewer.icns for .app bundle"
echo "  Linux:   Install png/logo_256.png or png/logo_512.png as desktop icon"
echo ""
