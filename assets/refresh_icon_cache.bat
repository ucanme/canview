@echo off
REM ============================================
REM CANVIEW 图标缓存清除工具
REM 用于强制Windows重新加载应用图标
REM ============================================

echo.
echo ============================================
echo CANVIEW 图标缓存清除工具
echo ============================================
echo.
echo 此工具将：
echo 1. 停止 Windows 资源管理器
echo 2. 删除图标缓存文件
echo 3. 重启 Windows 资源管理器
echo.
echo 注意：所有打开的文件资源管理器窗口将被关闭
echo ============================================
echo.

pause

echo.
echo [1/3] 正在停止 Windows 资源管理器...
taskkill /f /im explorer.exe

echo.
echo [2/3] 正在删除图标缓存文件...

REM 删除用户图标缓存
del /f /s /q /a "%userprofile%\AppData\Local\IconCache.db" 2>nul
del /f /s /q /a "%userprofile%\AppData\Local\Microsoft\Windows\Explorer\*.db" 2>nul
del /f /s /q /a "%userprofile%\AppData\Local\Microsoft\Windows\Explorer\iconcache_*.db" 2>nul

REM 删除系统图标缓存
del /f /s /q /a "%localappdata%\IconCache.db" 2>nul
del /f /s /q /a "%localappdata%\Microsoft\Windows\Explorer\*.db" 2>nul

echo.
echo [3/3] 正在重启 Windows 资源管理器...
start explorer.exe

echo.
echo ============================================
echo ✅ 图标缓存已清除！
echo ============================================
echo.
echo 现在请执行以下步骤验证图标：
echo.
echo 1. 打开文件夹：
echo    C:\Users\Administrator\RustroverProjects\canview\target\release\
echo.
echo 2. 找到 view.exe 文件
echo.
echo 3. 查看文件图标是否已更新为示波器风格的Logo
echo.
echo 如果仍然看不到新图标：
echo   - 重启电脑
echo   - 或使用 F5 刷新文件资源管理器
echo.
pause
