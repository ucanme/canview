@echo off
echo Compiling canview...
cd /d "%~dp0"
cargo build --release -p view
if %ERRORLEVEL% EQU 0 (
    echo.
    echo ========================================
    echo Build successful!
    echo ========================================
    echo.
    echo Executable: target\release\view.exe
    echo.
    echo To run the application:
    echo   cargo run -p view --release
    echo.
    echo Or directly:
    echo   target\release\view.exe
    echo.
) else (
    echo.
    echo ========================================
    echo Build failed!
    echo ========================================
    echo.
    echo Please check the error messages above.
    echo.
)
pause
