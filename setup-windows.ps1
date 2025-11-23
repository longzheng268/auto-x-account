# Windows 一键搭建编译环境脚本 (使用 Scoop)
# Windows One-Click Build Environment Setup Script (using Scoop)

Write-Host "================================================" -ForegroundColor Cyan
Write-Host "X 账号自动注册系统 - Windows 环境管理" -ForegroundColor Cyan
Write-Host "X Account Auto Registration - Windows Environment Manager" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# 检查是否以管理员权限运行
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "警告: 建议以管理员权限运行此脚本" -ForegroundColor Yellow
    Write-Host "Warning: It's recommended to run this script as Administrator" -ForegroundColor Yellow
    Write-Host ""
}

# 全局变量
$script:hasCompleteEnvironment = $false
$script:needSetup = $false
$script:projectName = "auto-x-account"  # 项目名称，从 Cargo.toml 读取更好，但为简化先硬编码

# ============================================
# 辅助函数 / Helper Functions
# ============================================

# 检查是否已有完整的编译环境
function Test-CompleteEnvironment {
    Write-Host "正在检测编译环境..." -ForegroundColor Cyan
    Write-Host "Detecting build environment..." -ForegroundColor Cyan
    Write-Host ""
    
    $hasRust = $false
    $hasToolchain = $false
    $hasGit = $false
    $toolchainType = "无"
    
    # 检查 Rust
    if (Get-Command rustc -ErrorAction SilentlyContinue) {
        $rustVersion = rustc --version
        Write-Host "✓ Rust 已安装: $rustVersion" -ForegroundColor Green
        $hasRust = $true
        
        # 检查工具链配置
        $toolchainInfo = rustup show 2>&1 | Out-String
        if ($toolchainInfo -match "x86_64-pc-windows-gnu") {
            $toolchainType = "GNU"
            Write-Host "✓ 检测到 GNU 工具链 (MinGW-w64)" -ForegroundColor Green
            $hasToolchain = $true
        } elseif ($toolchainInfo -match "x86_64-pc-windows-msvc") {
            $toolchainType = "MSVC"
            Write-Host "✓ 检测到 MSVC 工具链" -ForegroundColor Green
            $hasToolchain = $true
        }
    } else {
        Write-Host "✗ Rust 未安装" -ForegroundColor Yellow
    }
    
    # 检查 Git
    if (Get-Command git -ErrorAction SilentlyContinue) {
        $gitVersion = git --version
        Write-Host "✓ Git 已安装: $gitVersion" -ForegroundColor Green
        $hasGit = $true
    } else {
        Write-Host "✗ Git 未安装" -ForegroundColor Yellow
    }
    
    Write-Host ""
    
    if ($hasRust -and $hasToolchain -and $hasGit) {
        Write-Host "================================================" -ForegroundColor Green
        Write-Host "检测到完整的编译环境!" -ForegroundColor Green
        Write-Host "Complete build environment detected!" -ForegroundColor Green
        Write-Host "================================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "当前配置:" -ForegroundColor Cyan
        Write-Host "Current configuration:" -ForegroundColor Cyan
        Write-Host "  - Rust 工具链: $toolchainType" -ForegroundColor White
        Write-Host "  - Rust toolchain: $toolchainType" -ForegroundColor White
        Write-Host ""
        
        return @{
            HasEnvironment = $true
            ToolchainType = $toolchainType
        }
    } else {
        Write-Host "编译环境不完整，需要安装" -ForegroundColor Yellow
        Write-Host "Build environment incomplete, setup required" -ForegroundColor Yellow
        Write-Host ""
        
        return @{
            HasEnvironment = $false
            ToolchainType = $null
        }
    }
}

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
            Write-Host "  安装 MinGW-w64 工具包..." -ForegroundColor Gray
            $output = & "$msys2Root\usr\bin\bash.exe" -lc "pacman -S --noconfirm mingw-w64-x86_64-toolchain 2>&1"
            if ($LASTEXITCODE -eq 0) {
                Write-Host "MinGW-w64 工具链配置完成" -ForegroundColor Green
                Write-Host "MinGW-w64 toolchain configured" -ForegroundColor Green
                return $true
            } else {
                Write-Host "MinGW-w64 工具链安装失败" -ForegroundColor Red
                Write-Host "MinGW-w64 toolchain installation failed" -ForegroundColor Red
                Write-Host "输出: $output" -ForegroundColor Gray
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
        Write-Host "  初始化 Rust 工具链..." -ForegroundColor Gray
        try {
            if ($UseGNU) {
                $output = rustup-init -y --default-toolchain stable --default-host x86_64-pc-windows-gnu 2>&1
            } else {
                $output = rustup-init -y --default-toolchain stable --default-host x86_64-pc-windows-msvc 2>&1
            }
            
            if ($LASTEXITCODE -ne 0) {
                Write-Host "Rust 初始化失败!" -ForegroundColor Red
                Write-Host "Rust initialization failed!" -ForegroundColor Red
                Write-Host "输出: $output" -ForegroundColor Gray
                return $false
            }
        } catch {
            Write-Host "Rust 初始化错误: $_" -ForegroundColor Red
            Write-Host "Rust initialization error: $_" -ForegroundColor Red
            return $false
        }
        
        # 更新环境变量
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + [System.Environment]::GetEnvironmentVariable("Path","Machine")
        
        Write-Host "Rust 安装成功!" -ForegroundColor Green
        Write-Host "Rust installed successfully!" -ForegroundColor Green
        return $true
    } else {
        Write-Host "Rust 已安装" -ForegroundColor Green
        Write-Host "Rust is already installed" -ForegroundColor Green
        
        # 如果选择了 GNU 工具链，添加 GNU target 并设置为默认
        if ($UseGNU) {
            Write-Host ""
            Write-Host "配置 GNU 工具链..." -ForegroundColor Green
            Write-Host "Configuring GNU toolchain..." -ForegroundColor Green
            
            # 添加 GNU 目标
            Write-Host "  添加 x86_64-pc-windows-gnu 目标..." -ForegroundColor Gray
            rustup target add x86_64-pc-windows-gnu 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0) {
                Write-Host "警告: 添加 GNU 目标失败" -ForegroundColor Yellow
                Write-Host "Warning: Failed to add GNU target" -ForegroundColor Yellow
            }
            
            # 关键：设置默认主机为 GNU，避免 cargo 反复同步 MSVC 工具链
            Write-Host "  设置默认主机为 x86_64-pc-windows-gnu..." -ForegroundColor Gray
            rustup set default-host x86_64-pc-windows-gnu 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0) {
                Write-Host "警告: 设置默认主机失败" -ForegroundColor Yellow
                Write-Host "Warning: Failed to set default host" -ForegroundColor Yellow
            }
            
            # 安装并设置 GNU 工具链为默认
            Write-Host "  安装并设置 GNU 工具链为默认..." -ForegroundColor Gray
            rustup toolchain install stable-x86_64-pc-windows-gnu 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0) {
                Write-Host "警告: 安装 GNU 工具链失败" -ForegroundColor Yellow
                Write-Host "Warning: Failed to install GNU toolchain" -ForegroundColor Yellow
            }
            
            rustup default stable-x86_64-pc-windows-gnu 2>&1 | Out-Null
            if ($LASTEXITCODE -ne 0) {
                Write-Host "警告: 设置默认工具链失败" -ForegroundColor Yellow
                Write-Host "Warning: Failed to set default toolchain" -ForegroundColor Yellow
            }
            
            Write-Host "GNU 工具链配置完成!" -ForegroundColor Green
            Write-Host "GNU toolchain configured!" -ForegroundColor Green
        }
        return $true
    }
}

# 配置 MinGW 环境变量
function Configure-MinGWPath {
    Write-Host ""
    Write-Host "配置 MinGW 环境变量..." -ForegroundColor Green
    Write-Host "Configuring MinGW environment..." -ForegroundColor Green
    
    # 检查 Scoop 是否可用
    if (-not (Get-Command scoop -ErrorAction SilentlyContinue)) {
        Write-Host "警告: Scoop 未安装或不在 PATH 中" -ForegroundColor Yellow
        Write-Host "Warning: Scoop is not installed or not in PATH" -ForegroundColor Yellow
        return $false
    }
    
    $msys2Root = $null
    try {
        $msys2Root = scoop prefix msys2 -ErrorAction SilentlyContinue
    } catch {
        # 忽略错误，继续检查
    }
    
    if ($msys2Root -and (Test-Path "$msys2Root\mingw64\bin")) {
        $mingwBin = "$msys2Root\mingw64\bin"
        
        # 检查是否已在 PATH 中
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($currentPath -notlike "*$mingwBin*") {
            Write-Host "  添加 MinGW bin 目录到 PATH: $mingwBin" -ForegroundColor Gray
            [Environment]::SetEnvironmentVariable("Path", "$mingwBin;$currentPath", "User")
            $env:Path = "$mingwBin;$env:Path"
            Write-Host "MinGW 环境变量配置完成!" -ForegroundColor Green
            Write-Host "MinGW environment configured!" -ForegroundColor Green
        } else {
            Write-Host "MinGW 已在 PATH 中" -ForegroundColor Green
            Write-Host "MinGW already in PATH" -ForegroundColor Green
        }
        return $true
    } else {
        Write-Host "警告: 未找到 MSYS2 MinGW 目录" -ForegroundColor Yellow
        Write-Host "Warning: MSYS2 MinGW directory not found" -ForegroundColor Yellow
        return $false
    }
}

# ============================================
# 主程序 / Main Program
# ============================================

# 首先检测是否已有完整环境
$envCheck = Test-CompleteEnvironment

if ($envCheck.HasEnvironment) {
    # 已有完整环境，询问用户意图
    Write-Host "您想要做什么？" -ForegroundColor Cyan
    Write-Host "What would you like to do?" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "1. 编译并运行项目 (推荐)" -ForegroundColor Green
    Write-Host "   Build and run the project (Recommended)" -ForegroundColor Green
    Write-Host ""
    Write-Host "2. 重新配置编译环境" -ForegroundColor Yellow
    Write-Host "   Reconfigure build environment" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "3. 退出" -ForegroundColor Gray
    Write-Host "   Exit" -ForegroundColor Gray
    Write-Host ""
    
    $userChoice = Read-Host "请选择 (1/2/3)"
    
    if ($userChoice -eq "1") {
        # 编译并运行
        Write-Host ""
        Write-Host "================================================" -ForegroundColor Cyan
        Write-Host "开始编译项目..." -ForegroundColor Green
        Write-Host "Starting project build..." -ForegroundColor Green
        Write-Host "================================================" -ForegroundColor Cyan
        Write-Host ""
        
        if (Test-Path "Cargo.toml") {
            Write-Host "正在编译 (release 模式)..." -ForegroundColor Green
            Write-Host "Building (release mode)..." -ForegroundColor Green
            cargo build --release
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host ""
                Write-Host "编译成功!" -ForegroundColor Green
                Write-Host "Build successful!" -ForegroundColor Green
                Write-Host ""
                
                $exePath = Join-Path -Path ".\target\release" -ChildPath "$script:projectName.exe"
                if (Test-Path $exePath) {
                    Write-Host "是否立即运行程序？(Y/N)" -ForegroundColor Cyan
                    Write-Host "Run the program now? (Y/N)" -ForegroundColor Cyan
                    $runNow = Read-Host
                    
                    if ($runNow -eq "Y" -or $runNow -eq "y") {
                        Write-Host ""
                        Write-Host "正在启动程序..." -ForegroundColor Green
                        Write-Host "Starting program..." -ForegroundColor Green
                        & $exePath
                    } else {
                        Write-Host ""
                        Write-Host "可执行文件位于: $exePath" -ForegroundColor Cyan
                        Write-Host "Executable located at: $exePath" -ForegroundColor Cyan
                    }
                }
            } else {
                Write-Host ""
                Write-Host "编译失败，请检查错误信息" -ForegroundColor Red
                Write-Host "Build failed, please check error messages" -ForegroundColor Red
            }
        } else {
            Write-Host "错误: 未找到 Cargo.toml 文件" -ForegroundColor Red
            Write-Host "Error: Cargo.toml not found" -ForegroundColor Red
            Write-Host "请确保在项目根目录运行此脚本" -ForegroundColor Yellow
            Write-Host "Please run this script from project root directory" -ForegroundColor Yellow
        }
        
        Write-Host ""
        Write-Host "按任意键退出..." -ForegroundColor Gray
        Write-Host "Press any key to exit..." -ForegroundColor Gray
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 0
    }
    elseif ($userChoice -eq "2") {
        Write-Host ""
        Write-Host "将重新配置编译环境..." -ForegroundColor Yellow
        Write-Host "Reconfiguring build environment..." -ForegroundColor Yellow
        Write-Host ""
        $script:needSetup = $true
    }
    else {
        Write-Host "退出" -ForegroundColor Gray
        Write-Host "Exiting" -ForegroundColor Gray
        exit 0
    }
} else {
    # 没有完整环境，需要安装
    Write-Host "编译环境不完整，将进行配置..." -ForegroundColor Yellow
    Write-Host "Build environment incomplete, will configure..." -ForegroundColor Yellow
    Write-Host ""
    $script:needSetup = $true
}

# 如果需要配置环境，继续执行安装流程
if (-not $script:needSetup) {
    exit 0
}

Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "开始配置编译环境..." -ForegroundColor Green
Write-Host "Starting environment setup..." -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

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
    Write-Host "您有两个选择:" -ForegroundColor Cyan
    Write-Host "You have two options:" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "选项 1 (推荐): 安装 Visual Studio Build Tools" -ForegroundColor Green
    Write-Host "Option 1 (Recommended): Install Visual Studio Build Tools" -ForegroundColor Green
    Write-Host "  1. 访问: https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Gray
    Write-Host "     Visit: https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Gray
    Write-Host "  2. 下载并安装 'Build Tools for Visual Studio 2022'" -ForegroundColor Gray
    Write-Host "     Download and install 'Build Tools for Visual Studio 2022'" -ForegroundColor Gray
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
    
    $choice = Read-Host "请选择 (1 或 2，按 Q 退出)"
    
    if ($choice -eq "Q" -or $choice -eq "q") {
        Write-Host "退出安装" -ForegroundColor Yellow
        Write-Host "Exiting installation" -ForegroundColor Yellow
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
        Write-Host "无效选择，退出" -ForegroundColor Red
        Write-Host "Invalid choice, exiting" -ForegroundColor Red
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
$bucketOutput = scoop bucket add extras 2>&1
if ($LASTEXITCODE -ne 0 -and $bucketOutput -notmatch "already added") {
    Write-Host "警告: extras bucket 添加可能失败" -ForegroundColor Yellow
    Write-Host "Warning: extras bucket add may have failed" -ForegroundColor Yellow
}

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
    
    # 配置 MinGW 环境变量
    Configure-MinGWPath | Out-Null
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
Write-Host "当前 Rust 工具链配置:" -ForegroundColor Yellow
Write-Host "Current Rust Toolchain Configuration:" -ForegroundColor Yellow
rustup show

Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "下一步 / Next Steps:" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

if ($useGNU) {
    Write-Host "✓ 您已选择使用 GNU 工具链 (MinGW-w64)" -ForegroundColor Green
    Write-Host "✓ You have chosen to use GNU toolchain (MinGW-w64)" -ForegroundColor Green
    Write-Host ""
    Write-Host "GNU 工具链优势:" -ForegroundColor Cyan
    Write-Host "GNU Toolchain Benefits:" -ForegroundColor Cyan
    Write-Host "  ✅ 完全静态链接，生成独立 exe 文件" -ForegroundColor Gray
    Write-Host "     Fully static linking, generates standalone exe" -ForegroundColor Gray
    Write-Host "  ✅ 无需 Visual C++ 运行时库" -ForegroundColor Gray
    Write-Host "     No Visual C++ runtime required" -ForegroundColor Gray
    Write-Host "  ✅ 适合绿色便携部署" -ForegroundColor Gray
    Write-Host "     Perfect for portable deployment" -ForegroundColor Gray
    Write-Host "  ✅ 已自动配置默认工具链，避免 MSVC 工具链反复同步" -ForegroundColor Gray
    Write-Host "     Default toolchain configured to prevent MSVC syncing issues" -ForegroundColor Gray
    Write-Host ""
    Write-Host "重要提示:" -ForegroundColor Yellow
    Write-Host "Important Notes:" -ForegroundColor Yellow
    Write-Host "  - 如果看到 'syncing channel updates for MSVC'，请打开新的 PowerShell 窗口" -ForegroundColor Gray
    Write-Host "    If you see 'syncing channel updates for MSVC', open a new PowerShell window" -ForegroundColor Gray
    Write-Host "  - MinGW 路径已添加到 PATH 环境变量" -ForegroundColor Gray
    Write-Host "    MinGW path has been added to PATH environment variable" -ForegroundColor Gray
    Write-Host ""
} else {
    Write-Host "✓ 您正在使用 MSVC 工具链（官方推荐）" -ForegroundColor Green
    Write-Host "✓ You are using MSVC toolchain (officially recommended)" -ForegroundColor Green
    Write-Host ""
    Write-Host "MSVC 工具链优势:" -ForegroundColor Cyan
    Write-Host "MSVC Toolchain Benefits:" -ForegroundColor Cyan
    Write-Host "  ✅ Rust 官方推荐，兼容性最好" -ForegroundColor Gray
    Write-Host "     Officially recommended by Rust, best compatibility" -ForegroundColor Gray
    Write-Host "  ✅ 与 Windows 系统集成更紧密" -ForegroundColor Gray
    Write-Host "     Better Windows system integration" -ForegroundColor Gray
    Write-Host "  ✅ Visual Studio 调试器支持完善" -ForegroundColor Gray
    Write-Host "     Excellent Visual Studio debugger support" -ForegroundColor Gray
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
    Write-Host "   cargo build --release" -ForegroundColor Gray
    Write-Host "   # 注意: 已设置 GNU 为默认工具链，无需指定 --target" -ForegroundColor DarkGray
    Write-Host "   # Note: GNU is now default, no need for --target flag" -ForegroundColor DarkGray
} else {
    Write-Host "   cargo build --release" -ForegroundColor Gray
}
Write-Host ""
Write-Host "4. 运行程序 / Run the program:" -ForegroundColor White
Write-Host "   .\target\release\auto-x-account.exe" -ForegroundColor Gray
Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan

Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "环境配置完成!" -ForegroundColor Green
Write-Host "Environment setup completed!" -ForegroundColor Green
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "提示: 编译项目请在新的 PowerShell 窗口中执行，以确保环境变量生效" -ForegroundColor Yellow
Write-Host "Tip: Please compile in a new PowerShell window to ensure environment variables are loaded" -ForegroundColor Yellow
Write-Host ""
