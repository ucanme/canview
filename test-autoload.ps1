# 测试信号库自动加载

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "测试信号库自动加载功能" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. 检查配置文件
Write-Host "📋 步骤 1: 检查配置文件..." -ForegroundColor Green
$configPath = ".\target\release\multi_channel_config.json"
if (Test-Path $configPath) {
    Write-Host "✅ 配置文件存在: $configPath" -ForegroundColor Green
    
    # 读取并显示配置
    $config = Get-Content $configPath | ConvertFrom-Json
    $libCount = $config.libraries.Count
    Write-Host "   库数量: $libCount" -ForegroundColor White
    
    foreach ($lib in $config.libraries) {
        Write-Host "   📦 $($lib.name): $($lib.versions.Count) 个版本" -ForegroundColor White
        foreach ($ver in $lib.versions) {
            $chCount = $ver.channel_databases.Count
            Write-Host "      📁 $($ver.name): $chCount 个通道" -ForegroundColor White
        }
    }
} else {
    Write-Host "❌ 配置文件不存在" -ForegroundColor Red
    exit 1
}
Write-Host ""

# 2. 运行程序并捕获输出
Write-Host "📦 步骤 2: 启动程序..." -ForegroundColor Green
Write-Host "   查看控制台输出，应该看到加载信息" -ForegroundColor Yellow
Write-Host ""

# 切换到 release 目录运行
Push-Location .\target\release
.\view.exe
Pop-Location
