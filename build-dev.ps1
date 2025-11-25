# 开发模式编译脚本 - 动态链接，快速编译
# Development build script - dynamic linking, fast compilation

Write-Host "🔨 开发模式编译 (动态链接) / Development Build (Dynamic Linking)" -ForegroundColor Cyan
Write-Host ""

# 编译
$jobs = if ($args.Count -gt 0) { $args[0] } else { "8" }
Write-Host "并发任务数 / Jobs: $jobs" -ForegroundColor Yellow

cargo build --jobs $jobs

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ 编译成功 / Build succeeded!" -ForegroundColor Green
    Write-Host "📍 输出位置 / Output: target\debug\auto-x-account.exe" -ForegroundColor Cyan
    Write-Host "🚀 运行命令 / Run: .\target\debug\auto-x-account.exe" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "特点 / Features:" -ForegroundColor White
    Write-Host "  ✅ 编译速度快 / Fast compilation" -ForegroundColor Green
    Write-Host "  ✅ 包含调试信息 / Includes debug info" -ForegroundColor Green
    Write-Host "  ⚠️  需要 MSVC Runtime / Requires MSVC Runtime" -ForegroundColor Yellow
} else {
    Write-Host ""
    Write-Host "❌ 编译失败 / Build failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}
