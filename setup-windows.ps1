# Windows 一键搭建编译环境脚本 (使用 Scoop)
# Windows One-Click Build Environment Setup Script (using Scoop)

Write-Host "================================================" -ForegroundColor Cyan
Write-Host "X 账号自动注册系统 - Windows 编译环境安装" -ForegroundColor Cyan
Write-Host "X Account Auto Registration - Windows Build Setup" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# 检查是否以管理员权限运行
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "警告: 建议以管理员权限运行此脚本" -ForegroundColor Yellow
    Write-Host "Warning: It's recommended to run this script as Administrator" -ForegroundColor Yellow
    Write-Host ""
}

# 检查 Scoop 是否已安装
if (!(Get-Command scoop -ErrorAction SilentlyContinue)) {
    Write-Host "正在安装 Scoop..." -ForegroundColor Green
    Write-Host "Installing Scoop..." -ForegroundColor Green
    
    Set-ExecutionPolicy RemoteSigned -Scope CurrentUser -Force
    Invoke-RestMethod get.scoop.sh | Invoke-Expression
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Scoop 安装失败!" -ForegroundColor Red
        Write-Host "Scoop installation failed!" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "Scoop 安装成功!" -ForegroundColor Green
    Write-Host "Scoop installed successfully!" -ForegroundColor Green
} else {
    Write-Host "Scoop 已安装" -ForegroundColor Green
    Write-Host "Scoop is already installed" -ForegroundColor Green
}

Write-Host ""

# 添加必要的 bucket
Write-Host "添加 extras bucket..." -ForegroundColor Green
Write-Host "Adding extras bucket..." -ForegroundColor Green
scoop bucket add extras

Write-Host ""

# 安装 Rust
Write-Host "正在安装 Rust..." -ForegroundColor Green
Write-Host "Installing Rust..." -ForegroundColor Green

if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
    scoop install rustup
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Rust 安装失败!" -ForegroundColor Red
        Write-Host "Rust installation failed!" -ForegroundColor Red
        exit 1
    }
    
    # 初始化 rustup
    rustup-init -y
    
    # 更新环境变量
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + [System.Environment]::GetEnvironmentVariable("Path","Machine")
    
    Write-Host "Rust 安装成功!" -ForegroundColor Green
    Write-Host "Rust installed successfully!" -ForegroundColor Green
} else {
    Write-Host "Rust 已安装" -ForegroundColor Green
    Write-Host "Rust is already installed" -ForegroundColor Green
}

Write-Host ""

# 安装 Git
Write-Host "正在安装 Git..." -ForegroundColor Green
Write-Host "Installing Git..." -ForegroundColor Green

if (!(Get-Command git -ErrorAction SilentlyContinue)) {
    scoop install git
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Git 安装失败!" -ForegroundColor Red
        Write-Host "Git installation failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "Git 安装成功!" -ForegroundColor Green
    Write-Host "Git installed successfully!" -ForegroundColor Green
} else {
    Write-Host "Git 已安装" -ForegroundColor Green
    Write-Host "Git is already installed" -ForegroundColor Green
}

Write-Host ""

# 安装 Chrome/Chromium (用于浏览器自动化)
Write-Host "正在安装 Chromium..." -ForegroundColor Green
Write-Host "Installing Chromium..." -ForegroundColor Green

if (!(Get-Command chrome -ErrorAction SilentlyContinue)) {
    scoop install chromium
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Chromium 安装失败，尝试安装 Google Chrome..." -ForegroundColor Yellow
        Write-Host "Chromium installation failed, trying Google Chrome..." -ForegroundColor Yellow
        scoop install googlechrome
    }
}

Write-Host ""

# 显示版本信息
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "安装完成! 当前版本信息:" -ForegroundColor Green
Write-Host "Installation complete! Current versions:" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "Rust:" -ForegroundColor Yellow
rustc --version
cargo --version

Write-Host ""
Write-Host "Git:" -ForegroundColor Yellow
git --version

Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "下一步 / Next Steps:" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. 克隆项目 / Clone the project:" -ForegroundColor White
Write-Host "   git clone https://github.com/longzheng268/auto-x-account.git" -ForegroundColor Gray
Write-Host ""
Write-Host "2. 进入项目目录 / Enter project directory:" -ForegroundColor White
Write-Host "   cd auto-x-account" -ForegroundColor Gray
Write-Host ""
Write-Host "3. 编译项目 / Build the project:" -ForegroundColor White
Write-Host "   cargo build --release" -ForegroundColor Gray
Write-Host ""
Write-Host "4. 运行程序 / Run the program:" -ForegroundColor White
Write-Host "   .\target\release\auto-x-account.exe --email your@email.com" -ForegroundColor Gray
Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan

# 询问是否立即编译项目
if (Test-Path "Cargo.toml") {
    Write-Host ""
    $compile = Read-Host "检测到 Cargo.toml，是否立即编译项目？(Y/N) / Cargo.toml detected, compile now? (Y/N)"
    if ($compile -eq "Y" -or $compile -eq "y") {
        Write-Host ""
        Write-Host "正在编译项目..." -ForegroundColor Green
        Write-Host "Compiling project..." -ForegroundColor Green
        cargo build --release
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host ""
            Write-Host "编译成功! 可执行文件位于: .\target\release\auto-x-account.exe" -ForegroundColor Green
            Write-Host "Build successful! Executable located at: .\target\release\auto-x-account.exe" -ForegroundColor Green
        } else {
            Write-Host ""
            Write-Host "编译失败，请检查错误信息" -ForegroundColor Red
            Write-Host "Build failed, please check error messages" -ForegroundColor Red
        }
    }
}

Write-Host ""
Write-Host "脚本执行完成!" -ForegroundColor Green
Write-Host "Script execution completed!" -ForegroundColor Green
