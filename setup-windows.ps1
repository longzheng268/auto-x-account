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

# ============================================
# 辅助函数 / Helper Functions
# ============================================

# 检查 MSVC 工具链是否已安装
function Test-MSVCInstalled {
    # 优先使用 vswhere.exe（更快更准确）
    $buildToolsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $buildToolsPath) {
        $output = & $buildToolsPath -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($output) {
            return $true
        }
    }
    
    # 回退到检查已知路径
    $vsPaths = @(
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\BuildTools\VC",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Community\VC",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Professional\VC",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Enterprise\VC",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2019\BuildTools\VC",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2017\BuildTools\VC"
    )
    
    foreach ($path in $vsPaths) {
        if (Test-Path $path) {
            return $true
        }
    }
    
    return $false
}

# 检查命令是否存在
function Test-CommandExists {
    param([string]$Command)
    return $null -ne (Get-Command $Command -ErrorAction SilentlyContinue)
}

# 安装 MSYS2 和 MinGW-w64
function Install-GNUToolchain {
    Write-Host ""
    Write-Host "正在安装 MSYS2 (GNU 工具链)..." -ForegroundColor Green
    Write-Host "Installing MSYS2 (GNU toolchain)..." -ForegroundColor Green
    
    # 检查 MSYS2 是否已通过 Scoop 安装
    $msys2Root = scoop prefix msys2 2>$null
    
    if (-not $msys2Root) {
        scoop install msys2
        if ($LASTEXITCODE -ne 0) {
            Write-Host "MSYS2 安装失败!" -ForegroundColor Red
            Write-Host "MSYS2 installation failed!" -ForegroundColor Red
            return $false
        }
        Write-Host "MSYS2 安装成功!" -ForegroundColor Green
        Write-Host "MSYS2 installed successfully!" -ForegroundColor Green
        
        # 重新获取路径
        $msys2Root = scoop prefix msys2
    } else {
        Write-Host "MSYS2 已安装" -ForegroundColor Green
        Write-Host "MSYS2 is already installed" -ForegroundColor Green
    }
    
    # 安装 mingw-w64 工具链
    Write-Host ""
    Write-Host "正在配置 MinGW-w64 工具链..." -ForegroundColor Green
    Write-Host "Configuring MinGW-w64 toolchain..." -ForegroundColor Green
    
    if ($msys2Root -and (Test-Path "$msys2Root\usr\bin\bash.exe")) {
        try {
            & "$msys2Root\usr\bin\bash.exe" -lc "pacman -S --noconfirm mingw-w64-x86_64-toolchain" 2>&1 | Out-Null
            if ($LASTEXITCODE -eq 0) {
                Write-Host "MinGW-w64 工具链配置完成" -ForegroundColor Green
                Write-Host "MinGW-w64 toolchain configured" -ForegroundColor Green
                return $true
            } else {
                Write-Host "MinGW-w64 工具链安装失败" -ForegroundColor Red
                Write-Host "MinGW-w64 toolchain installation failed" -ForegroundColor Red
                return $false
            }
        } catch {
            Write-Host "错误: 无法配置 MinGW-w64 工具链 - $_" -ForegroundColor Red
            Write-Host "Error: Failed to configure MinGW-w64 toolchain - $_" -ForegroundColor Red
            return $false
        }
    } else {
        Write-Host "错误: MSYS2 安装不完整" -ForegroundColor Red
        Write-Host "Error: MSYS2 installation incomplete" -ForegroundColor Red
        return $false
    }
}

# 安装 Rust 工具链
function Install-RustToolchain {
    param([bool]$UseGNU)
    
    Write-Host ""
    Write-Host "正在安装 Rust..." -ForegroundColor Green
    Write-Host "Installing Rust..." -ForegroundColor Green
    
    if (-not (Test-CommandExists "rustc")) {
        scoop install rustup
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Rust 安装失败!" -ForegroundColor Red
            Write-Host "Rust installation failed!" -ForegroundColor Red
            return $false
        }
        
        # 初始化 rustup
        if ($UseGNU) {
            rustup-init -y --default-toolchain stable --default-host x86_64-pc-windows-gnu
        } else {
            rustup-init -y --default-toolchain stable --default-host x86_64-pc-windows-msvc
        }
        
        # 更新环境变量
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + [System.Environment]::GetEnvironmentVariable("Path","Machine")
        
        Write-Host "Rust 安装成功!" -ForegroundColor Green
        Write-Host "Rust installed successfully!" -ForegroundColor Green
        return $true
    } else {
        Write-Host "Rust 已安装" -ForegroundColor Green
        Write-Host "Rust is already installed" -ForegroundColor Green
        
        # 如果选择了 GNU 工具链，添加 GNU target
        if ($UseGNU) {
            Write-Host "配置 GNU 工具链目标..." -ForegroundColor Green
            Write-Host "Configuring GNU toolchain target..." -ForegroundColor Green
            rustup target add x86_64-pc-windows-gnu
            rustup default stable-x86_64-pc-windows-gnu
        }
        return $true
    }
}

# ============================================
# 主程序 / Main Program
# ============================================

$hasMSVC = Test-MSVCInstalled

if (-not $hasMSVC) {
    Write-Host ""
    Write-Host "================================================" -ForegroundColor Yellow
    Write-Host "未检测到 MSVC 工具链 (Visual Studio Build Tools)" -ForegroundColor Yellow
    Write-Host "MSVC Toolchain (Visual Studio Build Tools) Not Detected" -ForegroundColor Yellow
    Write-Host "================================================" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Rust 在 Windows 上默认使用 MSVC 工具链编译。" -ForegroundColor White
    Write-Host "Rust on Windows uses MSVC toolchain by default for compilation." -ForegroundColor White
    Write-Host ""
    Write-Host "您有两个选择 / You have two options:" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "选项 1 (推荐): 安装 Visual Studio Build Tools" -ForegroundColor Green
    Write-Host "Option 1 (Recommended): Install Visual Studio Build Tools" -ForegroundColor Green
    Write-Host "  1. 访问 / Visit: https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Gray
    Write-Host "  2. 下载并安装 'Build Tools for Visual Studio 2022'" -ForegroundColor Gray
    Write-Host "  3. 在安装程序中选择 'Desktop development with C++' 工作负载" -ForegroundColor Gray
    Write-Host "     In the installer, select 'Desktop development with C++' workload" -ForegroundColor Gray
    Write-Host "  4. 安装完成后重新运行此脚本" -ForegroundColor Gray
    Write-Host "     After installation, run this script again" -ForegroundColor Gray
    Write-Host ""
    Write-Host "选项 2: 使用 GNU 工具链 (通过 MSYS2)" -ForegroundColor Yellow
    Write-Host "Option 2: Use GNU toolchain (via MSYS2)" -ForegroundColor Yellow
    Write-Host "  此选项将自动配置使用 GNU 工具链，无需 Visual Studio" -ForegroundColor Gray
    Write-Host "  This option will automatically configure GNU toolchain without Visual Studio" -ForegroundColor Gray
    Write-Host ""
    
    $choice = Read-Host "请选择 (1 或 2，按 Q 退出) / Please choose (1 or 2, press Q to quit)"
    
    if ($choice -eq "Q" -or $choice -eq "q") {
        Write-Host "退出安装 / Exiting installation" -ForegroundColor Yellow
        exit 0
    }
    elseif ($choice -eq "1") {
        Write-Host ""
        Write-Host "正在打开 Visual Studio Build Tools 下载页面..." -ForegroundColor Green
        Write-Host "Opening Visual Studio Build Tools download page..." -ForegroundColor Green
        Start-Process "https://visualstudio.microsoft.com/visual-cpp-build-tools/"
        Write-Host ""
        Write-Host "请按照以下步骤操作:" -ForegroundColor Cyan
        Write-Host "Please follow these steps:" -ForegroundColor Cyan
        Write-Host "1. 下载并运行 vs_BuildTools.exe" -ForegroundColor White
        Write-Host "2. 在 Visual Studio Installer 中选择 'Desktop development with C++'" -ForegroundColor White
        Write-Host "3. 点击 '安装' 按钮" -ForegroundColor White
        Write-Host "4. 安装完成后，重新运行此脚本" -ForegroundColor White
        Write-Host ""
        Write-Host "安装 Build Tools 后，请重新运行此脚本继续。" -ForegroundColor Yellow
        Write-Host "After installing Build Tools, please re-run this script to continue." -ForegroundColor Yellow
        exit 0
    }
    elseif ($choice -eq "2") {
        Write-Host ""
        Write-Host "选择使用 GNU 工具链..." -ForegroundColor Green
        Write-Host "Choosing GNU toolchain..." -ForegroundColor Green
        $useGNU = $true
    }
    else {
        Write-Host "无效选择，退出 / Invalid choice, exiting" -ForegroundColor Red
        exit 1
    }
}
else {
    Write-Host "检测到 MSVC 工具链已安装" -ForegroundColor Green
    Write-Host "MSVC toolchain detected" -ForegroundColor Green
    Write-Host ""
    $useGNU = $false
}

# 检查 Scoop 是否已安装
if (-not (Test-CommandExists "scoop")) {
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
scoop bucket add extras 2>&1 | Out-Null

Write-Host ""

# 如果使用 GNU 工具链，先安装 MSYS2
if ($useGNU) {
    $gnuSuccess = Install-GNUToolchain
    if (-not $gnuSuccess) {
        Write-Host ""
        Write-Host "GNU 工具链安装失败，无法继续" -ForegroundColor Red
        Write-Host "GNU toolchain installation failed, cannot continue" -ForegroundColor Red
        exit 1
    }
}

# 安装 Rust
$rustSuccess = Install-RustToolchain -UseGNU $useGNU
if (-not $rustSuccess) {
    Write-Host ""
    Write-Host "Rust 安装失败，无法继续" -ForegroundColor Red
    Write-Host "Rust installation failed, cannot continue" -ForegroundColor Red
    exit 1
}

Write-Host ""

# 安装 Git
Write-Host "正在安装 Git..." -ForegroundColor Green
Write-Host "Installing Git..." -ForegroundColor Green

if (-not (Test-CommandExists "git")) {
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

if (-not (Test-CommandExists "chrome") -and -not (Test-CommandExists "chromium")) {
    scoop install chromium 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Chromium 安装失败，尝试安装 Google Chrome..." -ForegroundColor Yellow
        Write-Host "Chromium installation failed, trying Google Chrome..." -ForegroundColor Yellow
        scoop install googlechrome 2>&1 | Out-Null
    }
} else {
    Write-Host "浏览器已安装" -ForegroundColor Green
    Write-Host "Browser is already installed" -ForegroundColor Green
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

if ($useGNU) {
    Write-Host "✓ 您已选择使用 GNU 工具链 (MinGW-w64)" -ForegroundColor Green
    Write-Host "✓ You have chosen to use GNU toolchain (MinGW-w64)" -ForegroundColor Green
    Write-Host ""
    Write-Host "注意: 使用 GNU 工具链编译时，请确保:" -ForegroundColor Yellow
    Write-Host "Note: When building with GNU toolchain, please ensure:" -ForegroundColor Yellow
    Write-Host "  - 某些 Windows 特定功能可能有差异" -ForegroundColor Gray
    Write-Host "    Some Windows-specific features may behave differently" -ForegroundColor Gray
    Write-Host "  - 如果遇到问题，可以重新运行此脚本选择安装 MSVC" -ForegroundColor Gray
    Write-Host "    If you encounter issues, re-run this script to install MSVC" -ForegroundColor Gray
    Write-Host ""
} else {
    Write-Host "✓ 您正在使用 MSVC 工具链（推荐）" -ForegroundColor Green
    Write-Host "✓ You are using MSVC toolchain (recommended)" -ForegroundColor Green
    Write-Host ""
}

Write-Host "1. 克隆项目 / Clone the project:" -ForegroundColor White
Write-Host "   git clone https://github.com/longzheng268/auto-x-account.git" -ForegroundColor Gray
Write-Host ""
Write-Host "2. 进入项目目录 / Enter project directory:" -ForegroundColor White
Write-Host "   cd auto-x-account" -ForegroundColor Gray
Write-Host ""
Write-Host "3. 编译项目 / Build the project:" -ForegroundColor White
if ($useGNU) {
    Write-Host "   cargo build --release --target x86_64-pc-windows-gnu" -ForegroundColor Gray
} else {
    Write-Host "   cargo build --release" -ForegroundColor Gray
}
Write-Host ""
Write-Host "4. 运行程序 / Run the program:" -ForegroundColor White
Write-Host "   .\target\release\auto-x-account.exe" -ForegroundColor Gray
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
        
        if ($useGNU) {
            cargo build --release --target x86_64-pc-windows-gnu
        } else {
            cargo build --release
        }
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host ""
            Write-Host "编译成功! 可执行文件位于: .\target\release\auto-x-account.exe" -ForegroundColor Green
            Write-Host "Build successful! Executable located at: .\target\release\auto-x-account.exe" -ForegroundColor Green
        } else {
            Write-Host ""
            Write-Host "编译失败，请检查错误信息" -ForegroundColor Red
            Write-Host "Build failed, please check error messages" -ForegroundColor Red
            Write-Host ""
            
            if (-not $useGNU -and -not (Test-MSVCInstalled)) {
                Write-Host "提示: 如果错误提示找不到 link.exe，请:" -ForegroundColor Yellow
                Write-Host "Hint: If error says link.exe not found, please:" -ForegroundColor Yellow
                Write-Host "  1. 安装 Visual Studio Build Tools" -ForegroundColor Gray
                Write-Host "     Install Visual Studio Build Tools" -ForegroundColor Gray
                Write-Host "  2. 或重新运行此脚本并选择 GNU 工具链" -ForegroundColor Gray
                Write-Host "     Or re-run this script and choose GNU toolchain" -ForegroundColor Gray
            }
        }
    }
}

Write-Host ""
Write-Host "脚本执行完成!" -ForegroundColor Green
Write-Host "Script execution completed!" -ForegroundColor Green
