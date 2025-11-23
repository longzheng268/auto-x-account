# Windows 卸载和清理脚本
# Windows Uninstall and Cleanup Script

Write-Host "================================================" -ForegroundColor Red
Write-Host "X 账号自动注册系统 - Windows 环境清理" -ForegroundColor Red
Write-Host "X Account Auto Registration - Windows Cleanup" -ForegroundColor Red
Write-Host "================================================" -ForegroundColor Red
Write-Host ""

# 检查是否以管理员权限运行
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "警告: 建议以管理员权限运行此脚本以完全清理" -ForegroundColor Yellow
    Write-Host "Warning: Run as Administrator for complete cleanup" -ForegroundColor Yellow
    Write-Host ""
}

Write-Host "此脚本将清理本项目安装的所有开发环境" -ForegroundColor Yellow
Write-Host "This script will clean up all development environments installed by this project" -ForegroundColor Yellow
Write-Host ""
$confirm = Read-Host "确认继续？(Y/N) / Continue? (Y/N)"

if ($confirm -ne "Y" -and $confirm -ne "y") {
    Write-Host "已取消清理" -ForegroundColor Yellow
    Write-Host "Cleanup cancelled" -ForegroundColor Yellow
    exit 0
}

Write-Host ""
Write-Host "开始清理..." -ForegroundColor Green
Write-Host "Starting cleanup..." -ForegroundColor Green
Write-Host ""

# 1. 清理本项目的构建产物和数据
Write-Host "[1/6] 清理项目构建产物和数据..." -ForegroundColor Cyan
Write-Host "[1/6] Cleaning project build artifacts and data..." -ForegroundColor Cyan

$itemsToRemove = @(
    "target",
    "browser_data",
    "screenshots",
    "logs",
    "accounts.json",
    "config.json",
    "Cargo.lock"
)

foreach ($item in $itemsToRemove) {
    if (Test-Path $item) {
        Remove-Item -Recurse -Force $item
        Write-Host "  ✓ 已删除: $item" -ForegroundColor Green
    }
}

Write-Host ""

# 2. 询问是否卸载 Rust
Write-Host "[2/6] Rust 卸载..." -ForegroundColor Cyan
Write-Host "[2/6] Rust uninstall..." -ForegroundColor Cyan
$uninstallRust = Read-Host "是否卸载 Rust？(Y/N) / Uninstall Rust? (Y/N)"

if ($uninstallRust -eq "Y" -or $uninstallRust -eq "y") {
    if (Get-Command rustup -ErrorAction SilentlyContinue) {
        Write-Host "  正在卸载 Rust..." -ForegroundColor Yellow
        rustup self uninstall -y
        
        # 清理 Rust 环境变量
        $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
        $cargoPath = "$env:USERPROFILE\.cargo\bin"
        if ($userPath -like "*$cargoPath*") {
            $newPath = $userPath -replace [regex]::Escape($cargoPath), ""
            $newPath = $newPath -replace ";;", ";"
            [System.Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            Write-Host "  ✓ 已清理 Rust 环境变量" -ForegroundColor Green
        }
        
        # 删除 .cargo 和 .rustup 目录
        $cargoDir = "$env:USERPROFILE\.cargo"
        $rustupDir = "$env:USERPROFILE\.rustup"
        
        if (Test-Path $cargoDir) {
            Remove-Item -Recurse -Force $cargoDir
            Write-Host "  ✓ 已删除: $cargoDir" -ForegroundColor Green
        }
        
        if (Test-Path $rustupDir) {
            Remove-Item -Recurse -Force $rustupDir
            Write-Host "  ✓ 已删除: $rustupDir" -ForegroundColor Green
        }
        
        Write-Host "  ✓ Rust 已卸载" -ForegroundColor Green
    } else {
        Write-Host "  Rust 未安装，跳过" -ForegroundColor Gray
    }
} else {
    Write-Host "  保留 Rust" -ForegroundColor Gray
}

Write-Host ""

# 3. 询问是否通过 Scoop 卸载相关工具
Write-Host "[3/6] Scoop 包管理器清理..." -ForegroundColor Cyan
Write-Host "[3/6] Scoop package manager cleanup..." -ForegroundColor Cyan

if (Get-Command scoop -ErrorAction SilentlyContinue) {
    $uninstallScoop = Read-Host "是否卸载通过 Scoop 安装的工具？(Y/N) / Uninstall Scoop packages? (Y/N)"
    
    if ($uninstallScoop -eq "Y" -or $uninstallScoop -eq "y") {
        # 包含 MSYS2（用于 GNU 工具链）
        $packages = @("rustup", "git", "chromium", "googlechrome", "msys2")
        
        foreach ($pkg in $packages) {
            if (scoop list | Select-String -Pattern $pkg -Quiet) {
                Write-Host "  正在卸载: $pkg" -ForegroundColor Yellow
                scoop uninstall $pkg
                Write-Host "  ✓ 已卸载: $pkg" -ForegroundColor Green
            }
        }
        
        # 清理 MSYS2 目录（如果存在）
        $msys2Dir = "$env:USERPROFILE\scoop\apps\msys2"
        if (Test-Path $msys2Dir) {
            Remove-Item -Recurse -Force $msys2Dir -ErrorAction SilentlyContinue
            Write-Host "  ✓ 已删除 MSYS2 目录" -ForegroundColor Green
        }
        
        # 询问是否卸载 Scoop 本身
        $uninstallScoopItself = Read-Host "是否卸载 Scoop 本身？(Y/N) / Uninstall Scoop itself? (Y/N)"
        if ($uninstallScoopItself -eq "Y" -or $uninstallScoopItself -eq "y") {
            scoop uninstall scoop
            
            # 清理 Scoop 目录
            $scoopDir = "$env:USERPROFILE\scoop"
            if (Test-Path $scoopDir) {
                Remove-Item -Recurse -Force $scoopDir
                Write-Host "  ✓ 已删除 Scoop 目录" -ForegroundColor Green
            }
            
            # 清理环境变量
            $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
            $scoopPath = "$env:USERPROFILE\scoop\shims"
            if ($userPath -like "*$scoopPath*") {
                $newPath = $userPath -replace [regex]::Escape($scoopPath), ""
                $newPath = $newPath -replace ";;", ";"
                [System.Environment]::SetEnvironmentVariable("Path", $newPath, "User")
                Write-Host "  ✓ 已清理 Scoop 环境变量" -ForegroundColor Green
            }
        }
    }
} else {
    Write-Host "  Scoop 未安装，跳过" -ForegroundColor Gray
}

Write-Host ""

# 4. 清理注册表（如果有）
Write-Host "[4/6] 清理注册表..." -ForegroundColor Cyan
Write-Host "[4/6] Cleaning registry..." -ForegroundColor Cyan

if ($isAdmin) {
    # 这里可以添加特定的注册表清理逻辑
    # 目前项目不需要特殊的注册表项
    Write-Host "  本项目无需清理注册表" -ForegroundColor Gray
} else {
    Write-Host "  需要管理员权限才能清理注册表" -ForegroundColor Yellow
    Write-Host "  Administrator rights required for registry cleanup" -ForegroundColor Yellow
}

Write-Host ""

# 5. 清理临时文件
Write-Host "[5/6] 清理临时文件..." -ForegroundColor Cyan
Write-Host "[5/6] Cleaning temporary files..." -ForegroundColor Cyan

$tempDirs = @(
    "$env:TEMP\auto-x-account*",
    "$env:LOCALAPPDATA\auto-x-account"
)

foreach ($dir in $tempDirs) {
    if (Test-Path $dir) {
        Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
        Write-Host "  ✓ 已删除: $dir" -ForegroundColor Green
    }
}

Write-Host ""

# 6. 清理缓存
Write-Host "[6/6] 清理缓存..." -ForegroundColor Cyan
Write-Host "[6/6] Cleaning cache..." -ForegroundColor Cyan

$cacheDirs = @(
    "$env:LOCALAPPDATA\Temp\chromiumoxide*",
    "$env:LOCALAPPDATA\Temp\auto-x-account*"
)

foreach ($dir in $cacheDirs) {
    if (Test-Path $dir) {
        Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
        Write-Host "  ✓ 已删除缓存: $dir" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "================================================" -ForegroundColor Green
Write-Host "清理完成!" -ForegroundColor Green
Write-Host "Cleanup completed!" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Green
Write-Host ""
Write-Host "注意事项 / Notes:" -ForegroundColor Yellow
Write-Host "1. 请重启终端以使环境变量更改生效" -ForegroundColor White
Write-Host "   Please restart terminal for environment variable changes to take effect" -ForegroundColor White
Write-Host ""
Write-Host "2. 如果卸载了 Rust 或 Git，它们可能被其他项目使用" -ForegroundColor White
Write-Host "   If you uninstalled Rust or Git, they might be used by other projects" -ForegroundColor White
Write-Host ""
Write-Host "3. Scoop 和其包可能被其他项目使用，请谨慎卸载" -ForegroundColor White
Write-Host "   Scoop and its packages might be used by other projects" -ForegroundColor White
Write-Host ""

Read-Host "按 Enter 键退出 / Press Enter to exit"
