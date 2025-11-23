# auto-x-account

<div align="center">

# 🐦 X 账号自动注册系统

**自动化 X (Twitter) 账号注册工具 - 支持批量注册、代理访问、多平台**

[English](README_EN.md) | 简体中文

[![CI/CD](https://github.com/longzheng268/auto-x-account/actions/workflows/ci-cd.yml/badge.svg)](https://github.com/longzheng268/auto-x-account/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)

</div>

## ✨ 特性

- 🚀 **高性能** - 使用 Rust 开发，性能卓越，资源占用低
- 🎨 **现代化 GUI** - 中国风界面设计，使用小米 MiSans 字体
- 📧 **批量邮箱** - 支持批量创建和管理临时邮箱
- 👥 **批量注册** - 支持批量注册 X 账号，可配置并发数
- 🌐 **代理支持** - 三种代理模式：不使用、系统代理、手动配置
- 🤖 **智能验证** - 自动处理邮箱验证，支持人机验证
- 💾 **数据导出** - 支持导出账号信息为 JSON、CSV、TXT 格式
- 🌍 **多语言** - 支持中文和英文界面
- 📦 **跨平台** - 支持 Windows、macOS、Linux

## 📥 安装

### 方式一：下载预编译版本（推荐）

从 [Releases](https://github.com/longzheng268/auto-x-account/releases) 页面下载适合你系统的版本：

#### Windows
推荐选择：
- **MSVC 版本** (推荐): `auto-x-account-windows-x86_64-msvc.zip`
  - 使用 Visual Studio 工具链编译，兼容性最好
  - 适合大多数用户
  
- **GNU 版本**: `auto-x-account-windows-x86_64-gnu.zip`
  - 使用 MinGW-w64 编译，完全静态链接
  - 无需安装 Visual C++ 运行时库
  - 适合追求独立部署、绿色便携的用户

**解压后直接运行 `auto-x-account.exe`**

#### macOS
- Intel 芯片：`auto-x-account-macos-x86_64.tar.gz`
- Apple Silicon：`auto-x-account-macos-arm64.tar.gz`
- 解压后运行：`./auto-x-account`

#### Linux
- 下载 `auto-x-account-linux-x86_64.tar.gz`
- 解压后运行：`./auto-x-account`

### 方式二：从源码编译

#### 一键安装编译环境

**Windows (使用 Scoop):**
```powershell
powershell -ExecutionPolicy Bypass -File setup-windows.ps1
```

脚本会自动检测 MSVC 工具链，如果未安装，会提供两个选项：
1. **安装 Visual Studio Build Tools**（推荐，兼容性最好）
2. **使用 GNU 工具链**（通过 MSYS2，无需 Visual Studio）

**注意**: 如果遇到 `link.exe` 找不到的错误，请重新运行安装脚本并按提示操作。

**macOS:**
```bash
./setup-macos.sh
```

**Linux:**
```bash
./setup-linux.sh
```

#### 手动编译

**前提条件**：
- **Windows**: 
  - **MSVC 工具链**（推荐）: 安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
  - **GNU 工具链**（完全静态链接）: 安装 MinGW-w64（通过 MSYS2 或 Chocolatey）
- **macOS/Linux**: 需要基本的构建工具（gcc, make 等）

```bash
# 克隆项目
git clone https://github.com/longzheng268/auto-x-account.git
cd auto-x-account

# Windows MSVC 编译（默认，推荐）
cargo build --release

# Windows GNU 编译（静态链接，无需运行时）
cargo build --release --target x86_64-pc-windows-gnu

# macOS/Linux 编译
cargo build --release

# 运行
./target/release/auto-x-account
# Windows: .\target\release\auto-x-account.exe
```

**Windows GNU 工具链编译说明：**

使用 GNU 工具链编译可以获得完全静态链接的二进制文件，无需依赖 Visual C++ 运行时库。

1. **安装 MinGW-w64（通过 Chocolatey）：**
   ```powershell
   choco install mingw -y
   ```

2. **或通过 MSYS2 安装：**
   ```bash
   pacman -S mingw-w64-x86_64-toolchain
   ```

3. **配置 Rust 使用 GNU 工具链：**
   ```bash
   rustup target add x86_64-pc-windows-gnu
   rustup default stable-x86_64-pc-windows-gnu
   ```

4. **编译项目：**
   ```bash
   cargo build --release --target x86_64-pc-windows-gnu
   ```

项目已经配置了 `.cargo/config.toml` 文件，会自动使用静态链接配置。

## 🗑️ 卸载

### 完全清理开发环境

如果你不再需要此项目，可以使用一键卸载脚本完全清理所有安装的开发环境：

**Windows:**
```powershell
powershell -ExecutionPolicy Bypass -File uninstall-windows.ps1
```

**macOS:**
```bash
./uninstall-macos.sh
```

**Linux:**
```bash
./uninstall-linux.sh
```

卸载脚本会清理：
- ✅ 项目构建产物和数据文件
- ✅ Rust 工具链（可选）
- ✅ 通过包管理器安装的开发工具（可选）
- ✅ 应用程序数据和缓存
- ✅ 环境变量配置
- ✅ 临时文件

**注意**: 脚本会询问是否卸载 Rust 和其他开发工具，因为它们可能被其他项目使用。

### 仅删除项目文件

如果只想删除项目文件而保留开发环境：

```bash
# 删除构建产物
cargo clean

# 删除运行数据
rm -rf browser_data screenshots logs accounts.json config.json
```

## 🚀 快速开始

### GUI 模式（默认）

直接运行程序即可启动图形界面：

```bash
./auto-x-account
```

或显式启动 GUI：

```bash
./auto-x-account gui
```

### 命令行模式

#### 注册单个账号

```bash
./auto-x-account register --email your@email.com
```

使用代理：

```bash
./auto-x-account register --email your@email.com --proxy socks5://127.0.0.1:1080
```

#### 批量注册账号

```bash
# 注册 10 个账号，并发数为 3
./auto-x-account batch --count 10 --concurrent 3
```

使用现有邮箱列表：

```bash
./auto-x-account batch --count 10 --use-existing-emails
```

#### 批量创建邮箱

```bash
# 创建 20 个邮箱并导出
./auto-x-account create-emails --count 20 --output emails.json --verify
```

#### 导出账号

```bash
# 导出为 JSON 格式
./auto-x-account export --output accounts.json --format json

# 导出为 CSV 格式
./auto-x-account export --output accounts.csv --format csv
```

## ⚙️ 配置

配置文件：`config.json`

```json
{
  "language": "zh-CN",
  "smtp": {
    "host": "0.0.0.0",
    "port": 8025,
    "domain": "example.com",
    "enable": true
  },
  "proxy": {
    "mode": "none",
    "type": "socks5",
    "host": "127.0.0.1",
    "port": 1080,
    "username": "",
    "password": ""
  },
  "browser": {
    "headless": false,
    "timeout": 30000,
    "viewport": {
      "width": 1280,
      "height": 720
    },
    "user_data_dir": "browser_data"
  },
  "x_account": {
    "base_url": "https://twitter.com/i/flow/signup",
    "email_wait_timeout": 120,
    "retry_times": 3
  }
}
```

### 代理模式说明

- **`none`** - 不使用代理，直接连接
- **`system`** - 自动检测并使用系统代理设置
- **`manual`** - 使用手动配置的代理服务器

### 邮箱服务提供商

支持以下邮箱服务：

1. **mail.tm** - 免费临时邮箱 API（推荐）
2. **Guerrilla Mail** - 老牌临时邮箱服务
3. **自建 SMTP** - 完全自主控制（最稳定）
4. **自定义** - 使用自己的邮箱服务

## 🎨 界面预览

（待添加截图）

## 🛠️ 开发

### 项目结构

```
auto-x-account/
├── src/
│   ├── main.rs              # 主程序入口
│   ├── config.rs            # 配置管理
│   ├── gui.rs               # GUI 界面
│   ├── registration.rs      # 注册逻辑
│   ├── batch.rs             # 批量处理
│   ├── email.rs             # 邮件服务
│   ├── email_provider.rs    # 邮箱提供商
│   ├── captcha.rs           # 人机验证
│   └── i18n.rs              # 多语言
├── .github/
│   └── workflows/
│       └── ci-cd.yml        # CI/CD 工作流
├── setup-windows.ps1        # Windows 安装脚本
├── setup-macos.sh           # macOS 安装脚本
└── setup-linux.sh           # Linux 安装脚本
```

### 运行测试

```bash
cargo test
```

### 代码检查

```bash
cargo clippy
cargo fmt --check
```

## 📝 常见问题

### Q: Windows 编译时提示 "linker `link.exe` not found" 或 "dlltool.exe not found" 怎么办？

A: 这是因为缺少编译工具链。有两种解决方案：

**方案 1（推荐）**: 安装 Visual Studio Build Tools (MSVC)
1. 访问 https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. 下载并安装 "Build Tools for Visual Studio 2022"
3. 在安装程序中选择 "Desktop development with C++" 工作负载
4. 安装完成后重新编译：`cargo build --release`

**方案 2**: 使用 GNU 工具链（MinGW-w64）- 静态链接，无需运行时
1. 安装 MinGW-w64：
   - 通过 Chocolatey: `choco install mingw -y`
   - 或通过 MSYS2: `pacman -S mingw-w64-x86_64-toolchain`
   
2. 配置 Rust：
   ```bash
   rustup target add x86_64-pc-windows-gnu
   rustup default stable-x86_64-pc-windows-gnu
   ```

3. 编译项目：
   ```bash
   cargo build --release --target x86_64-pc-windows-gnu
   ```

**方案 3**: 使用一键安装脚本
1. 运行 `setup-windows.ps1` 脚本
2. 当提示选择时，选择 "2" 使用 GNU 工具链
3. 脚本会自动安装所有依赖

**GNU 工具链的优势：**
- ✅ 完全静态链接，生成的 exe 文件独立运行
- ✅ 无需安装 Visual C++ 运行时库
- ✅ 适合绿色便携部署
- ✅ 文件体积可能更小

**MSVC 工具链的优势：**
- ✅ 官方推荐，兼容性最好
- ✅ 与 Windows 系统集成更紧密
- ✅ 调试工具支持更完善

### Q: 如何处理人机验证？

A: 系统支持手动模式和第三方验证服务。手动模式下，程序会等待你在浏览器中完成验证。

### Q: 代理设置不生效？

A: 请检查：
1. 代理模式是否正确设置（`none`/`system`/`manual`）
2. 代理服务器是否正常运行
3. 防火墙是否允许连接

### Q: 邮箱验证码收不到？

A: 请确保：
1. SMTP 服务已启动
2. 邮箱服务提供商可用
3. 网络连接正常

### Q: 如何在中国大陆使用？

A: 推荐使用代理模式：
- 设置 `proxy.mode` 为 `manual`
- 配置 SOCKS5 或 HTTP 代理
- 或使用 `system` 模式自动检测系统代理

## 🤝 贡献

⚠️ **重要提示**：本软件是专有商业软件，不接受外部代码贡献。

如果您发现 Bug 或有功能建议，请通过以下方式反馈：
- 在 [Issues](https://github.com/longzheng268/auto-x-account/issues) 中提交问题报告
- 提供详细的问题描述和复现步骤
- 我们会评估并在后续版本中修复

## 📄 许可证

**本软件是专有商业软件，版权所有 © 2024 longzheng268 保留所有权利**

⚠️ **严格的版权保护声明**：

本软件受严格的商业许可协议保护，**不是开源软件**。查看 [LICENSE](LICENSE) 文件了解完整的法律条款。

**禁止以下行为**（违者将承担法律责任）：
- ❌ 未经授权复制、分发或传播本软件
- ❌ 反向工程、反编译或破解本软件
- ❌ 修改、改编或创建衍生作品
- ❌ 用于商业目的或盈利活动（需购买商业许可证）
- ❌ 删除或修改版权声明
- ❌ 分享、转售或转让许可证

**法律后果**：
- 民事赔偿：最低 **100 万元人民币** 或实际损失的 5 倍
- 刑事责任：最高 **7 年有期徒刑**并处罚金
- 全部法律费用：包括律师费、调查费、诉讼费等

**购买许可证**：
- 个人许可证：[联系购买]
- 企业许可证：[联系购买]
- 商业许可证：[联系购买]
- 联系方式：通过 GitHub Issues 咨询

**举报侵权**：
如发现侵权行为，请立即举报，经查证属实给予 **1000-10000 元人民币**奖励。

## ⚠️ 免责声明和法律声明

**重要法律声明**：

1. **软件性质**：本软件是专有商业软件，受中国和国际版权法保护。

2. **使用限制**：
   - 仅供合法授权用户使用
   - 必须遵守所在国家/地区的法律法规
   - 不得用于任何非法目的
   - 使用本软件的一切后果由用户自行承担

3. **隐私和数据**：
   - 本软件会收集使用数据用于防止滥用
   - 我们会严格保护用户隐私
   - 详见隐私政策（如适用）

4. **免责**：
   - 软件"按原样"提供，不提供任何保证
   - 作者不对使用本软件产生的任何后果负责
   - 不对数据丢失、业务中断等承担责任

5. **合规使用**：
   - 使用本工具时请遵守 X (Twitter) 服务条款
   - 请遵守反垃圾邮件法律法规
   - 不得用于欺诈、滥用或其他违法活动

**使用本软件即表示您已阅读、理解并同意遵守上述所有条款和 LICENSE 文件中的完整法律协议。**

## 📮 联系方式

- 项目地址：[https://github.com/longzheng268/auto-x-account](https://github.com/longzheng268/auto-x-account)
- 问题反馈：[Issues](https://github.com/longzheng268/auto-x-account/issues)

---

<div align="center">

**如果这个项目对你有帮助，请给一个 ⭐️**

Made with ❤️ by Auto X Account Team

</div>
