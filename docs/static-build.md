# 静态编译指南

本文档提供如何在各个平台上构建完全静态链接的 Auto X Account 二进制文件，使程序可以在任何系统上直接运行，无需安装额外的运行时环境或依赖库。

## 📋 目录

- [为什么需要静态编译](#为什么需要静态编译)
- [Windows 静态编译](#windows-静态编译)
- [Linux 静态编译](#linux-静态编译)
- [macOS 静态编译](#macos-静态编译)
- [验证静态链接](#验证静态链接)
- [故障排查](#故障排查)

## 🎯 为什么需要静态编译

### 静态链接的优势

✅ **无依赖运行** - 程序包含所有必需的库，可在任何系统上直接运行  
✅ **绿色便携** - 单个可执行文件，无需安装，复制即用  
✅ **版本隔离** - 不受系统库版本影响，避免依赖冲突  
✅ **简化部署** - 无需在目标机器上安装运行时环境  
✅ **提高稳定性** - 避免系统更新导致的兼容性问题  

### 权衡考虑

⚠️ **文件体积较大** - 静态链接会增加可执行文件大小  
⚠️ **编译时间较长** - 需要编译和链接更多代码  
⚠️ **内存占用略高** - 每个进程都有独立的库副本  

## 🪟 Windows 静态编译

Windows 平台提供两种静态编译方案：

### 方案 1：MSVC 工具链（推荐，兼容性最好）

MSVC 编译的程序默认会静态链接 C 运行时库（通过 `/MT` 标志）。

#### 前提条件

安装 Visual Studio Build Tools 或 Visual Studio：
- 下载地址：https://visualstudio.microsoft.com/visual-cpp-build-tools/
- 选择 "C++ 生成工具" 工作负载
- 确保安装 Windows SDK

#### 编译步骤

```powershell
# 1. 安装 Rust（如果尚未安装）
# 下载：https://rustup.rs/
# 默认会安装 MSVC 工具链

# 2. 验证工具链
rustup show

# 3. 编译（默认使用 MSVC）
cargo build --release

# 4. 可执行文件位置
# target\release\auto-x-account.exe
```

项目已在 `.cargo/config.toml` 中配置静态链接：

```toml
[target.x86_64-pc-windows-msvc]
rustflags = [
    "-C", "target-feature=+crt-static",
]
```

### 方案 2：GNU 工具链（MinGW-w64，完全开源）

使用 MinGW-w64 可以获得完全开源的工具链，且能生成更小的可执行文件。

#### 安装 MinGW-w64

**通过 MSYS2（推荐）：**

```powershell
# 1. 安装 MSYS2
# 下载：https://www.msys2.org/
# 或通过 Scoop：scoop install msys2

# 2. 打开 MSYS2 终端，安装工具链
pacman -S mingw-w64-x86_64-toolchain

# 3. 添加到 PATH（临时）
$env:Path = "C:\msys64\mingw64\bin;$env:Path"

# 或添加到系统环境变量（永久）
[System.Environment]::SetEnvironmentVariable(
    "Path",
    "C:\msys64\mingw64\bin;" + [System.Environment]::GetEnvironmentVariable("Path", "Machine"),
    "Machine"
)
```

**通过 Chocolatey：**

```powershell
# 安装 Chocolatey（如果尚未安装）
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# 安装 MinGW-w64
choco install mingw -y

# 刷新环境变量
refreshenv
```

#### 安装 GNU 目标平台

```powershell
# 添加 GNU 工具链目标
rustup target add x86_64-pc-windows-gnu

# 验证安装
rustup target list --installed
```

#### 编译步骤

```powershell
# 编译
cargo build --release --target x86_64-pc-windows-gnu

# 可执行文件位置
# target\x86_64-pc-windows-gnu\release\auto-x-account.exe
```

项目已配置 GNU 工具链的静态链接选项：

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
rustflags = [
    "-C", "link-arg=-static-libgcc",
    "-C", "link-arg=-static-libstdc++",
    "-C", "link-arg=-static",
    "-C", "link-arg=-lwinpthread",
    "-C", "target-feature=+crt-static",
]
```

### Windows 编译问题修复

#### 问题 1：缺少 winapi 功能特性

**错误信息**：
```
error[E0432]: unresolved import `winapi::um::winuser`
error[E0433]: failed to resolve: could not find `windef` in `shared`
```

**解决方案**：

已在 `Cargo.toml` 中添加：

```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winuser", "windef"] }
```

#### 问题 2：找不到 core crate

**错误信息**：
```
error[E0463]: can't find crate for `core`
  = note: the `x86_64-pc-windows-gnu` target may not be installed
```

**解决方案**：

```powershell
# 安装目标平台
rustup target add x86_64-pc-windows-gnu
```

## 🐧 Linux 静态编译

Linux 静态编译推荐使用 **musl** 工具链，它专为静态链接设计。

### 使用 musl 目标（推荐）

#### 安装依赖

**Ubuntu/Debian：**

```bash
# 安装 musl 工具链和必要的构建工具
sudo apt update
sudo apt install -y musl-tools musl-dev

# 安装 Rust musl 目标
rustup target add x86_64-unknown-linux-musl
```

**CentOS/RHEL/Rocky Linux：**

```bash
# EPEL 仓库
sudo dnf install -y epel-release
sudo dnf install -y musl-gcc musl-devel

# 安装 Rust musl 目标
rustup target add x86_64-unknown-linux-musl
```

**Arch Linux：**

```bash
sudo pacman -S musl

# 安装 Rust musl 目标
rustup target add x86_64-unknown-linux-musl
```

#### 编译步骤

```bash
# 编译
cargo build --release --target x86_64-unknown-linux-musl

# 可执行文件位置
# target/x86_64-unknown-linux-musl/release/auto-x-account
```

### 处理 GUI 依赖（eframe/egui）

由于项目使用 `eframe` GUI 框架，需要处理一些系统库依赖。

#### 方法 1：使用 Docker 编译（推荐）

创建 `Dockerfile.static`:

```dockerfile
FROM rust:1.91-alpine AS builder

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    git

# 设置工作目录
WORKDIR /app

# 复制项目文件
COPY . .

# 编译
RUN cargo build --release --target x86_64-unknown-linux-musl

# 最终镜像
FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/auto-x-account /auto-x-account
ENTRYPOINT ["/auto-x-account"]
```

编译：

```bash
# 构建 Docker 镜像
docker build -f Dockerfile.static -t auto-x-account:static .

# 提取可执行文件
docker create --name temp auto-x-account:static
docker cp temp:/auto-x-account ./auto-x-account-static
docker rm temp

# 验证
./auto-x-account-static --version
```

#### 方法 2：在 Linux 上直接编译

如果遇到系统库依赖问题，可以暂时使用 gnu 目标：

```bash
# 使用默认的 gnu 目标
cargo build --release

# 检查依赖
ldd target/release/auto-x-account
```

虽然这不是完全静态的，但可以通过以下方法减少依赖：

```bash
# 在 .cargo/config.toml 中添加
[target.x86_64-unknown-linux-gnu]
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "link-arg=-static-libgcc",
]
```

### 使用 cargo-zigbuild（高级选项）

`cargo-zigbuild` 可以更容易地进行交叉编译和静态链接：

```bash
# 安装 Zig
curl https://ziglang.org/download/0.11.0/zig-linux-x86_64-0.11.0.tar.xz | tar -xJ
export PATH="$PWD/zig-linux-x86_64-0.11.0:$PATH"

# 安装 cargo-zigbuild
cargo install cargo-zigbuild

# 编译
cargo zigbuild --release --target x86_64-unknown-linux-musl
```

## 🍎 macOS 静态编译

macOS 的动态链接机制与 Linux/Windows 不同，但可以通过以下方法优化：

### 编译步骤

```bash
# 安装 Rust（如果尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Intel Mac
cargo build --release --target x86_64-apple-darwin

# Apple Silicon (M1/M2/M3)
cargo build --release --target aarch64-apple-darwin
```

### 静态链接 C 运行时

在 `.cargo/config.toml` 中添加：

```toml
[target.x86_64-apple-darwin]
rustflags = [
    "-C", "link-arg=-static-libgcc",
]

[target.aarch64-apple-darwin]
rustflags = [
    "-C", "link-arg=-static-libgcc",
]
```

### 创建通用二进制文件（Universal Binary）

```bash
# 分别编译两个架构
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# 合并成通用二进制
lipo -create \
    target/x86_64-apple-darwin/release/auto-x-account \
    target/aarch64-apple-darwin/release/auto-x-account \
    -output auto-x-account-universal

# 验证
lipo -info auto-x-account-universal
```

### macOS 应用程序打包

创建 `.app` 包：

```bash
# 创建应用结构
mkdir -p AutoXAccount.app/Contents/MacOS
mkdir -p AutoXAccount.app/Contents/Resources

# 复制可执行文件
cp target/release/auto-x-account AutoXAccount.app/Contents/MacOS/

# 创建 Info.plist
cat > AutoXAccount.app/Contents/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Auto X Account</string>
    <key>CFBundleDisplayName</key>
    <string>X账号自动注册系统</string>
    <key>CFBundleIdentifier</key>
    <string>com.autoxaccount.app</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>auto-x-account</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
</dict>
</plist>
EOF

# 签名（可选，需要 Apple Developer 账号）
codesign --force --deep --sign - AutoXAccount.app
```

## ✅ 验证静态链接

### Windows

```powershell
# 使用 dumpbin（需要 Visual Studio）
dumpbin /dependents target\release\auto-x-account.exe

# 或使用 Dependencies（第三方工具）
# 下载：https://github.com/lucasg/Dependencies
```

静态链接的程序应该只依赖系统核心 DLL：
- `KERNEL32.dll`
- `USER32.dll`
- `GDI32.dll`
- 等 Windows 系统 DLL

### Linux

```bash
# 查看动态库依赖
ldd target/release/auto-x-account

# 或使用 readelf
readelf -d target/release/auto-x-account

# musl 静态链接的程序应该输出：
# "not a dynamic executable"
```

### macOS

```bash
# 查看依赖库
otool -L target/release/auto-x-account

# 检查架构
file target/release/auto-x-account

# 应该只依赖系统库，如：
# /usr/lib/libSystem.B.dylib
# /usr/lib/libobjc.A.dylib
```

## 🔧 故障排查

### 常见问题

#### 1. 链接错误：找不到库

**问题**：
```
= note: /usr/bin/ld: cannot find -lxxx
```

**解决方案**：

确保安装了静态库版本：

```bash
# Ubuntu/Debian
sudo apt install libxxx-dev

# 对于 musl，使用 musl-tools
sudo apt install musl-tools
```

#### 2. OpenSSL 静态链接问题

**问题**：
```
error: failed to run custom build command for `openssl-sys`
```

**解决方案**：

```bash
# 安装 OpenSSL 静态库
sudo apt install libssl-dev pkg-config

# 或使用 vendored 特性
cargo build --release --features vendored
```

在 `Cargo.toml` 中添加：

```toml
[dependencies]
openssl = { version = "0.10", features = ["vendored"] }
```

#### 3. GUI 相关错误（Linux）

**问题**：
```
error: failed to run custom build command for `glib-sys`
```

**解决方案**：

```bash
# 安装 GUI 开发库
sudo apt install -y \
    libgtk-3-dev \
    libglib2.0-dev \
    libpango1.0-dev \
    libcairo2-dev \
    libgdk-pixbuf2.0-dev \
    libatk1.0-dev
```

对于纯静态编译，考虑使用 Docker 或交叉编译。

#### 4. 可执行文件体积过大

**解决方案**：

使用 `strip` 移除调试符号：

```bash
# Linux/macOS
strip target/release/auto-x-account

# Windows（需要 binutils）
strip.exe target\release\auto-x-account.exe
```

在 `Cargo.toml` 中启用 LTO 和 strip：

```toml
[profile.release]
opt-level = "z"      # 优化体积
lto = true           # 链接时优化
codegen-units = 1    # 更好的优化
strip = true         # 自动移除符号
panic = "abort"      # 减小体积
```

使用 `upx` 压缩（可选）：

```bash
# 安装 UPX
# Ubuntu: sudo apt install upx
# macOS: brew install upx
# Windows: choco install upx

# 压缩可执行文件
upx --best --lzma target/release/auto-x-account

# 注意：压缩后可能被某些杀毒软件误报
```

## 📦 自动化构建脚本

### 跨平台构建脚本

创建 `build-all.sh`:

```bash
#!/bin/bash

set -e

echo "🚀 开始构建所有平台..."

# Windows MSVC
echo "📦 构建 Windows (MSVC)..."
cargo build --release --target x86_64-pc-windows-msvc

# Windows GNU
echo "📦 构建 Windows (GNU)..."
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# Linux musl
echo "📦 构建 Linux (musl)..."
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl

# macOS
echo "📦 构建 macOS (x86_64)..."
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

echo "📦 构建 macOS (ARM64)..."
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

echo "✅ 所有平台构建完成！"
echo ""
echo "📁 输出文件："
ls -lh target/*/release/auto-x-account* | grep -v "\.d$"
```

使用：

```bash
chmod +x build-all.sh
./build-all.sh
```

## 📚 参考资料

- [Rust 官方文档 - 静态链接](https://doc.rust-lang.org/cargo/reference/config.html)
- [musl libc](https://musl.libc.org/)
- [MinGW-w64](https://www.mingw-w64.org/)
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)
- [UPX](https://upx.github.io/)

---

**版权所有 © 2024 PA733, longzheng268 及 Auto X Account Team**
