# 下载 Chromium 浏览器 / Download Chromium Browser
# 
# 这个脚本会下载适合 Windows 的 Chromium 浏览器
# This script downloads Chromium browser for Windows

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ChromiumDir = Join-Path $ScriptDir "chromium"

Write-Host "=== Chromium 浏览器下载工具 ===" -ForegroundColor Green
Write-Host "=== Chromium Browser Downloader ===" -ForegroundColor Green
Write-Host ""

# 检测架构
$IsARM64 = (Get-WmiObject -Class Win32_Processor).Architecture -eq 12
$Arch = if ($IsARM64) { 
    "ARM64" 
} elseif ([Environment]::Is64BitOperatingSystem) { 
    "x64" 
} else { 
    "x86" 
}
Write-Host "检测到架构 / Detected Architecture: " -NoNewline
Write-Host $Arch -ForegroundColor Yellow
Write-Host ""

# 检查是否已存在
if (Test-Path $ChromiumDir) {
    Write-Host "Chromium 目录已存在 / Chromium directory already exists" -ForegroundColor Yellow
    $Response = Read-Host "是否重新下载？/ Re-download? (y/N)"
    if ($Response -ne "y" -and $Response -ne "Y") {
        Write-Host "跳过下载 / Skipping download"
        exit 0
    }
    Remove-Item -Recurse -Force $ChromiumDir
}

New-Item -ItemType Directory -Force -Path $ChromiumDir | Out-Null

# Chromium 下载链接
$ChromiumBaseUrl = "https://commondatastorage.googleapis.com/chromium-browser-snapshots"
$Platform = if ($Arch -eq "ARM64") { 
    "Win_ARM64" 
} elseif ($Arch -eq "x64") { 
    "Win_x64" 
} else {
    Write-Host "错误: x86 架构不再受 Chromium 支持 / Error: x86 architecture is no longer supported by Chromium" -ForegroundColor Red
    exit 1
}
$LastChangeUrl = "$ChromiumBaseUrl/$Platform/LAST_CHANGE"

# 获取最新版本号
Write-Host "正在获取最新版本信息... / Fetching latest version..." -ForegroundColor Green

try {
    $Version = (Invoke-WebRequest -Uri $LastChangeUrl -UseBasicParsing).Content.Trim()
    $DownloadUrl = "$ChromiumBaseUrl/$Platform/$Version/chrome-win.zip"
    
    Write-Host "版本号 / Version: " -NoNewline
    Write-Host $Version -ForegroundColor Yellow
    Write-Host "下载地址 / Download URL: " -NoNewline
    Write-Host $DownloadUrl -ForegroundColor Yellow
    Write-Host ""
    
    # 下载 Chromium
    Write-Host "正在下载 Chromium... / Downloading Chromium..." -ForegroundColor Green
    $TempZip = Join-Path $env:TEMP "chromium_$([guid]::NewGuid()).zip"
    
    # 使用进度条下载
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip -UseBasicParsing
    $ProgressPreference = 'Continue'
    
    Write-Host "下载完成 / Download completed" -ForegroundColor Green
    
    # 解压
    Write-Host "正在解压... / Extracting..." -ForegroundColor Green
    Expand-Archive -Path $TempZip -DestinationPath $ChromiumDir -Force
    
    # 移动文件到正确位置
    $ExtractDir = Join-Path $ChromiumDir "chrome-win"
    if (Test-Path $ExtractDir) {
        Get-ChildItem -Path $ExtractDir | Move-Item -Destination $ChromiumDir -Force
        Remove-Item -Path $ExtractDir -Force
    }
    
    # 清理
    Remove-Item -Path $TempZip -Force
    
    Write-Host ""
    Write-Host "✓ Chromium 下载完成! / Chromium download completed!" -ForegroundColor Green
    Write-Host "安装位置 / Installation path: " -NoNewline
    Write-Host $ChromiumDir -ForegroundColor Yellow
    Write-Host ""
    
    # 显示版本信息
    $ChromeExec = Join-Path $ChromiumDir "chrome.exe"
    if (Test-Path $ChromeExec) {
        Write-Host "Chromium 可执行文件 / Chromium executable:" -ForegroundColor Green
        Write-Host "  $ChromeExec"
        
        # 尝试获取版本信息
        try {
            $VersionInfo = & $ChromeExec --version 2>$null
            Write-Host "版本 / Version: " -NoNewline -ForegroundColor Green
            Write-Host $VersionInfo
        } catch {
            Write-Host "无法获取版本信息 / Cannot get version"
        }
    }
    
    Write-Host ""
    Write-Host "现在可以运行程序了！/ You can now run the program!" -ForegroundColor Green
    
} catch {
    Write-Host "错误 / Error: $_" -ForegroundColor Red
    exit 1
}
