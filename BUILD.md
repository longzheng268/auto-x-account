# 编译说明 / Build Instructions

本项目支持两种编译模式，针对不同场景优化：

## 🚀 快速开始

### 方法 1：使用便捷脚本（推荐）

```powershell
# 开发模式（动态链接，快速编译）
.\build-dev.ps1

# 发布模式（静态链接，完全优化）
.\build-release.ps1

# 指定并发任务数（默认 8）
.\build-dev.ps1 16
.\build-release.ps1 16
```

### 方法 2：使用 Cargo 命令

## 1. 开发模式（推荐用于调试）

**动态链接**，编译速度快，适合日常开发和调试：

```powershell
# 编译开发版本（动态链接，编译快）
cargo build

# 使用多核并行编译（加快速度）
cargo build --jobs 8

# 运行
.\target\debug\auto-x-account.exe
```

**特点**：
- ✅ 编译速度快（不需要静态链接）
- ✅ 生成 EXE 可执行文件，可以直接运行
- ✅ 包含调试符号，方便调试
- ⚠️ 需要系统运行时库（MSVC Runtime）
- 📍 输出位置：`target/debug/auto-x-account.exe`

## 2. 发布模式（用于分发）

**静态链接**，完全优化，无需外部依赖：

```powershell
# 编译发布版本（静态链接，完全优化）
$env:RUSTFLAGS="-C target-feature=+crt-static"; cargo build --release

# 或使用脚本（推荐）
.\build-release.ps1

# 使用多核并行编译
$env:RUSTFLAGS="-C target-feature=+crt-static"; cargo build --release --jobs 8

# 运行
.\target\release\auto-x-account.exe
```

**特点**：
- ✅ 完全静态链接，无需任何外部 DLL
- ✅ 可以在任何 Windows 系统上独立运行
- ✅ 完全优化（LTO、strip 等）
- ⚠️ 编译时间较长
- 📍 输出位置：`target/release/auto-x-account.exe`

## 2. 编译为动态链接库

用于调试器或其他程序调用时使用：

```powershell
# 仅编译动态库（开发版本）
cargo build --lib

# 仅编译动态库（发布版本）
cargo build --lib --release
```

生成的动态库位置：
- 开发版：`target/debug/auto_x_account.dll` (Windows)
- 发布版：`target/release/auto_x_account.dll` (Windows)

**注意**：动态库版本**不包含**静态链接配置，会依赖系统的运行时库。

## 3. 同时编译两者

```powershell
# 同时编译可执行文件和动态库
cargo build --all-targets

# 发布版本
cargo build --all-targets --release
```

## 4. 仅编译二进制文件（跳过库）

如果你只想要可执行文件：

```powershell
cargo build --bin auto-x-account
cargo build --bin auto-x-account --release
```

## 静态链接 vs 动态链接

### 可执行文件（静态链接）
- ✅ 无需依赖外部 DLL
- ✅ 可以独立运行
- ✅ 文件体积较大
- ✅ 适合分发给最终用户

### 动态库（动态链接）
- ✅ 文件体积较小
- ✅ 编译速度更快
- ✅ 适合调试和开发
- ⚠️ 需要系统运行时库支持
- ⚠️ 不适合直接分发

## 使用场景

### 日常开发和调试
```powershell
# 快速编译，使用动态库
cargo build --lib
```

### 最终发布
```powershell
# 完整优化，静态链接
cargo build --release
```

### 供其他程序调用
```powershell
# 编译为动态库供 FFI 调用
cargo build --lib --release
```

## 清理构建产物

```powershell
# 清理所有构建产物
cargo clean
```
