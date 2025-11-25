# build-dev.ps1
# 开发模式快速编译 / Fast development build
# Usage: .\build-dev.ps1

$ErrorActionPreference = "Stop"

# 启用详细错误信息
$env:RUST_BACKTRACE = "1"

# 获取 CPU 核心数
$jobs = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
Write-Host "🔨 开发模式编译 (动态链接) / Development Build (Dynamic Linking)"
Write-Host ""
Write-Host "并发任务数 / Jobs: $jobs"

# 快速编译（使用所有核心）
cargo build --jobs $jobs

if ($LASTEXITCODE -ne 0) {
    Write-Error "❌ 编译失败 / Build failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "✅ 编译成功 / Build succeeded!"
Write-Host "📍 输出位置 / Output: target\debug\auto-x-account.exe"
Write-Host "🚀 运行命令 / Run: .\target\debug\auto-x-account.exe"
Write-Host ""
Write-Host "特点 / Features:"
Write-Host "  ✅ 编译速度快 / Fast compilation"
Write-Host "  ✅ 包含调试信息 / Includes debug info"
