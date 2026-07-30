@echo off
REM CANVIEW Icon Conversion Script
REM This script converts the new logo.svg to PNG and ICO formats
REM
REQUIREMENTS: Install ImageMagick from https://imagemagick.org/script/download.php
REM Or use online tools: https://cloudconvert.com/svg-to-png

echo ============================================
echo CANVIEW Icon Conversion Script (New Logo)
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

echo Converting new logo.svg to PNG format...
echo Source: svg/logo.svg
echo.

REM Create output directories
if not exist "png" mkdir png
if not exist "ico" mkdir ico

REM Convert SVG to PNG at different sizes
echo [1/7] Converting to 512x512 PNG...
magick -background none svg/logo.svg -resize 512x512 png/icon_512.png

echo [2/7] Converting to 256x256 PNG...
magick -background none svg/logo.svg -resize 256x256 png/icon_256.png

echo [3/7] Converting to 128x128 PNG...
magick -background none svg/logo.svg -resize 128x128 png/icon_128.png

echo [4/7] Converting to 64x64 PNG...
magick -background none svg/logo.svg -resize 64x64 png/icon_64.png

echo [5/7] Converting to 48x48 PNG...
magick -background none svg/logo.svg -resize 48x48 png/icon_48.png

echo [6/7] Converting to 32x32 PNG...
magick -background none svg/logo.svg -resize 32x32 png/icon_32.png

echo [7/7] Converting to 16x16 PNG...
magick -background none svg/logo.svg -resize 16x16 png/icon_16.png

echo.
echo Creating ICO file for Windows...
magick ^
    png/icon_256.png ^
    png/icon_128.png ^
    png/icon_64.png ^
    png/icon_48.png ^
    png/icon_32.png ^
    png/icon_16.png ^
    ico/can-viewer.ico

echo.
echo ============================================
echo Conversion complete!
echo ============================================
echo.
echo Output files:
echo   - PNG files: .\png\
echo   - ICO file: .\ico\can-viewer.ico
echo.
echo Source logo: svg/logo.svg
echo.
echo Next steps:
echo   1. Run: cargo build --release
echo   2. The EXE file will have the new logo as icon
echo.
pause
