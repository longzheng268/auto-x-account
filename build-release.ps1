# 发布模式编译脚本 - 静态链接，完全优化
# Release build script - static linking, full optimization

Write-Host "🔨 发布模式编译 (静态链接) / Release Build (Static Linking)" -ForegroundColor Cyan
Write-Host ""

# 设置静态链接环境变量
$env:RUSTFLAGS = "-C target-feature=+crt-static"

# 编译
$jobs = if ($args.Count -gt 0) { $args[0] } else { "8" }
Write-Host "并发任务数 / Jobs: $jobs" -ForegroundColor Yellow
Write-Host "编译选项 / Options: 静态链接 + LTO + Strip" -ForegroundColor Yellow

cargo build --release --jobs $jobs

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ 编译成功 / Build succeeded!" -ForegroundColor Green
    Write-Host "📍 输出位置 / Output: target\release\auto-x-account.exe" -ForegroundColor Cyan
    
    $fileSize = (Get-Item "target\release\auto-x-account.exe").Length / 1MB
    Write-Host "📦 文件大小 / File size: $([math]::Round($fileSize, 2)) MB" -ForegroundColor Magenta
    
    Write-Host ""
    Write-Host "特点 / Features:" -ForegroundColor White
    Write-Host "  ✅ 完全静态链接 / Fully static linked" -ForegroundColor Green
    Write-Host "  ✅ 无需外部依赖 / No external dependencies" -ForegroundColor Green
    Write-Host "  ✅ 完全优化 / Fully optimized" -ForegroundColor Green
    Write-Host "  ✅ 可独立分发 / Ready for distribution" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "❌ 编译失败 / Build failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}
