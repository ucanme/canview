@echo off
REM CANVIEW 快速打包脚本
echo ========================================
echo CANVIEW 打包工具
echo ========================================
echo.

REM 调用 PowerShell 脚本
powershell -ExecutionPolicy Bypass -File "%~dp0package.ps1" -Version "1.0.0"

echo.
echo 按任意键退出...
pause >nul
