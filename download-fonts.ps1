# 下载 MiSans 字体 / Download MiSans Font
# PowerShell Script

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$FontDir = Join-Path $ScriptDir "fonts"
$FontUrl = "https://hyperos.mi.com/font-download/MiSans.zip"
$ZipPath = Join-Path $FontDir "MiSans.zip"
$ExtractPath = Join-Path $FontDir "MiSans"

Write-Host "正在下载 MiSans 字体... / Downloading MiSans font..." -ForegroundColor Cyan

# 检查是否已存在字体文件
if (Test-Path $ExtractPath) {
    Write-Host "MiSans 字体已存在 / MiSans font already exists" -ForegroundColor Yellow
    $response = Read-Host "是否重新下载？(y/N) / Re-download? (y/N)"
    if ($response -ne "y" -and $response -ne "Y") {
        Write-Host "跳过下载 / Skipping download" -ForegroundColor Yellow
        exit 0
    }
    Remove-Item -Path $ExtractPath -Recurse -Force
}

# 下载字体
try {
    Write-Host "正在从 $FontUrl 下载... / Downloading from $FontUrl..." -ForegroundColor Green
    Invoke-WebRequest -Uri $FontUrl -OutFile $ZipPath -UseBasicParsing
    Write-Host "下载完成 / Download completed" -ForegroundColor Green
} catch {
    Write-Host "下载失败 / Download failed: $_" -ForegroundColor Red
    exit 1
}

# 解压
try {
    Write-Host "正在解压... / Extracting..." -ForegroundColor Green
    Expand-Archive -Path $ZipPath -DestinationPath $FontDir -Force
    Remove-Item -Path $ZipPath -Force
    Write-Host "✓ MiSans 字体下载完成 / MiSans font download completed" -ForegroundColor Green
} catch {
    Write-Host "解压失败 / Extraction failed: $_" -ForegroundColor Red
    Write-Host "请手动解压 MiSans.zip / Please manually extract MiSans.zip" -ForegroundColor Yellow
    exit 1
}
