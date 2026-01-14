@echo off
REM CANVIEW Icon Conversion Script
REM This script converts SVG icons to PNG and ICO formats
REM
REQUIREMENTS: Install ImageMagick from https://imagemagick.org/script/download.php
REM Or use online tools: https://cloudconvert.com/svg-to-png

echo ============================================
echo CANVIEW Icon Conversion Script
echo ============================================
echo.

REM Check if ImageMagick is installed
where magick >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: ImageMagick not found!
    echo.
    echo Please install ImageMagick from: https://imagemagick.org/script/download.php
    echo.
    echo Or use online converters:
    echo   - SVG to PNG: https://cloudconvert.com/svg-to-png
    echo   - PNG to ICO: https://convertico.com/
    echo.
    pause
    exit /b 1
)

echo Converting SVG icons to PNG format...
echo.

REM Create output directories
if not exist "png" mkdir png
if not exist "ico" mkdir ico

REM Convert SVG to PNG at different sizes
echo [1/6] Converting to 512x512 PNG...
magick -background none icon_512.svg png/icon_512.png

echo [2/6] Converting to 256x256 PNG...
magick -background none icon_256.svg png/icon_256.png

echo [3/6] Converting to 128x128 PNG...
magick -background none icon_128.svg png/icon_128.png

echo [4/6] Converting to 64x64 PNG...
magick -background none icon_64.svg png/icon_64.png

echo [5/6] Converting to 48x48 PNG...
magick -background none -resize 48x48 icon_64.svg png/icon_48.png

echo [6/6] Converting to 32x32 PNG...
magick -background none icon_32.svg png/icon_32.png

echo.
echo Creating ICO file for Windows...
magick ^
    png/icon_256.png ^
    png/icon_128.png ^
    png/icon_64.png ^
    png/icon_48.png ^
    png/icon_32.png ^
    ico/canview.ico

echo.
echo ============================================
echo Conversion complete!
echo ============================================
echo.
echo Output files:
echo   - PNG files: .\png\
echo   - ICO file: .\ico\canview.ico
echo.
echo Next steps:
echo   Windows: Use canview.ico for exe icon
echo   macOS:   See README.md for ICNS creation
echo   Linux:   Use png/icon_256.png or png/icon_512.png
echo.
pause
