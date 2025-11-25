# 浏览器检测诊断脚本
Write-Host "🔍 正在检测系统浏览器..." -ForegroundColor Cyan
Write-Host ""

$browsers = @(
    @{Name="Microsoft Edge"; Path="C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"},
    @{Name="Microsoft Edge (x64)"; Path="C:\Program Files\Microsoft\Edge\Application\msedge.exe"},
    @{Name="Google Chrome"; Path="C:\Program Files\Google\Chrome\Application\chrome.exe"},
    @{Name="Google Chrome (x86)"; Path="C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"},
    @{Name="Chromium"; Path="C:\Program Files\Chromium\Application\chrome.exe"},
    @{Name="Brave"; Path="C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe"}
)

$found = $false

foreach ($browser in $browsers) {
    if (Test-Path $browser.Path) {
        Write-Host "✅ 找到: $($browser.Name)" -ForegroundColor Green
        Write-Host "   路径: $($browser.Path)" -ForegroundColor Gray
        
        # 获取版本信息
        try {
            $version = (Get-Item $browser.Path).VersionInfo.FileVersion
            Write-Host "   版本: $version" -ForegroundColor Gray
        } catch {
            Write-Host "   版本: 无法获取" -ForegroundColor Yellow
        }
        
        Write-Host ""
        $found = $true
    }
}

if (-not $found) {
    Write-Host "❌ 未找到任何 Chromium 浏览器!" -ForegroundColor Red
    Write-Host ""
    Write-Host "建议:" -ForegroundColor Yellow
    Write-Host "1. 安装 Microsoft Edge（Windows 10/11 内置）" -ForegroundColor White
    Write-Host "2. 或安装 Google Chrome: https://www.google.com/chrome/" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host "✅ 检测完成！系统有可用的浏览器。" -ForegroundColor Green
}

# 检查注册表中的浏览器路径
Write-Host "📝 检查注册表..." -ForegroundColor Cyan
try {
    $edgeKey = Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\msedge.exe" -ErrorAction SilentlyContinue
    if ($edgeKey) {
        Write-Host "✅ 注册表中找到 Edge: $($edgeKey.'(default)')" -ForegroundColor Green
    }
} catch {
    Write-Host "⚠️  无法读取注册表" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "按任意键关闭..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
