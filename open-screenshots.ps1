# 打开截图文件夹
$screenshotDir = "$env:APPDATA\auto-x-account\screenshots"

Write-Host "📂 截图文件夹位置:" -ForegroundColor Cyan
Write-Host "   $screenshotDir" -ForegroundColor Yellow
Write-Host ""

if (Test-Path $screenshotDir) {
    Write-Host "📸 最新的截图文件:" -ForegroundColor Cyan
    Get-ChildItem $screenshotDir -Filter "*.png" | 
        Sort-Object LastWriteTime -Descending | 
        Select-Object -First 5 | 
        ForEach-Object {
            Write-Host "   ✓ $($_.Name)" -ForegroundColor Green
            Write-Host "     时间: $($_.LastWriteTime)" -ForegroundColor Gray
            Write-Host "     大小: $([math]::Round($_.Length / 1KB, 2)) KB" -ForegroundColor Gray
            Write-Host ""
        }
    
    Write-Host "🚀 正在打开截图文件夹..." -ForegroundColor Cyan
    Start-Process $screenshotDir
} else {
    Write-Host "❌ 截图文件夹不存在: $screenshotDir" -ForegroundColor Red
}
