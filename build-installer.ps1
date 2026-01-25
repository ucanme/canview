# 构建 CANVIEW 安装程序
# 此脚本会编译程序并使用 Inno Setup 创建安装包

param(
    [string]$Version = "1.0.0",
    [string]$InnoSetupPath = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "CANVIEW 安装程序构建脚本 v$Version" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. 检查 Inno Setup 是否安装
Write-Host "📋 步骤 1: 检查 Inno Setup..." -ForegroundColor Green
if (-not (Test-Path $InnoSetupPath)) {
    Write-Host "❌ 未找到 Inno Setup！" -ForegroundColor Red
    Write-Host ""
    Write-Host "请下载并安装 Inno Setup:" -ForegroundColor Yellow
    Write-Host "  https://jrsoftware.org/isdl.php" -ForegroundColor White
    Write-Host ""
    Write-Host "或者指定 Inno Setup 路径:" -ForegroundColor Yellow
    Write-Host "  .\build-installer.ps1 -InnoSetupPath 'C:\Path\To\ISCC.exe'" -ForegroundColor White
    Write-Host ""
    exit 1
}
Write-Host "✅ Inno Setup 已安装: $InnoSetupPath" -ForegroundColor Green
Write-Host ""

# 2. 编译 Release 版本
Write-Host "📦 步骤 2: 编译 Release 版本..." -ForegroundColor Green
cargo build --release -p view
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 编译失败！" -ForegroundColor Red
    exit 1
}
Write-Host "✅ 编译成功！" -ForegroundColor Green
Write-Host ""

# 3. 创建必要的目录
Write-Host "📁 步骤 3: 准备文件..." -ForegroundColor Green

# 确保 config 目录存在
if (-not (Test-Path "config")) {
    New-Item -ItemType Directory -Path "config" -Force | Out-Null
}
if (-not (Test-Path "config\signal_library")) {
    New-Item -ItemType Directory -Path "config\signal_library" -Force | Out-Null
}

# 创建默认配置文件
$defaultConfig = @"
{
  "libraries": [],
  "mappings": [],
  "active_library_id": null,
  "active_version_name": null
}
"@
$defaultConfig | Out-File -FilePath "config\default_config.json" -Encoding UTF8 -Force

Write-Host "✅ 文件准备完成！" -ForegroundColor Green
Write-Host ""

# 4. 构建安装程序
Write-Host "🔨 步骤 4: 构建安装程序..." -ForegroundColor Green

# 更新版本号
$issContent = Get-Content "installer.iss" -Raw
$issContent = $issContent -replace '#define MyAppVersion ".*"', "#define MyAppVersion `"$Version`""
$issContent | Out-File -FilePath "installer.iss" -Encoding UTF8 -Force

# 运行 Inno Setup 编译器
& $InnoSetupPath "installer.iss"
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 安装程序构建失败！" -ForegroundColor Red
    exit 1
}

Write-Host "✅ 安装程序构建成功！" -ForegroundColor Green
Write-Host ""

# 5. 完成
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "✅ 构建完成！" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "安装程序位置:" -ForegroundColor Yellow
Write-Host "  installer-output\CANVIEW-Setup-v$Version.exe" -ForegroundColor White
Write-Host ""
Write-Host "您可以分发此安装程序给用户。" -ForegroundColor Cyan
Write-Host ""

# 显示文件信息
if (Test-Path "installer-output\CANVIEW-Setup-v$Version.exe") {
    $fileInfo = Get-Item "installer-output\CANVIEW-Setup-v$Version.exe"
    $fileSizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
    Write-Host "文件大小: $fileSizeMB MB" -ForegroundColor White
    Write-Host "创建时间: $($fileInfo.CreationTime)" -ForegroundColor White
}
