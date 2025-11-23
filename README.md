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
- 下载 `auto-x-account-windows-x86_64.zip`
- 解压后直接运行 `auto-x-account.exe`

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
- **Windows**: 需要安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 或使用 GNU 工具链
- **macOS/Linux**: 需要基本的构建工具（gcc, make 等）

```bash
# 克隆项目
git clone https://github.com/longzheng268/auto-x-account.git
cd auto-x-account

# 编译发布版本
cargo build --release

# Windows 使用 GNU 工具链编译（可选）
cargo build --release --target x86_64-pc-windows-gnu

# 运行
./target/release/auto-x-account
```

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

### Q: Windows 编译时提示 "linker `link.exe` not found" 怎么办？

A: 这是因为缺少 MSVC 工具链。有两种解决方案：

**方案 1（推荐）**: 安装 Visual Studio Build Tools
1. 访问 https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. 下载并安装 "Build Tools for Visual Studio 2022"
3. 在安装程序中选择 "Desktop development with C++" 工作负载
4. 安装完成后重新编译

**方案 2**: 使用 GNU 工具链
1. 重新运行 `setup-windows.ps1` 脚本
2. 当提示选择时，选择 "2" 使用 GNU 工具链
3. 脚本会自动安装 MSYS2 和 MinGW-w64
4. 使用命令编译：`cargo build --release --target x86_64-pc-windows-gnu`

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

欢迎贡献代码、报告问题或提出建议！

1. Fork 本项目
2. 创建你的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交你的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启一个 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## ⚠️ 免责声明

本工具仅供学习和研究使用。使用本工具时请遵守相关法律法规和服务条款。作者不对使用本工具产生的任何后果负责。

## 📮 联系方式

- 项目地址：[https://github.com/longzheng268/auto-x-account](https://github.com/longzheng268/auto-x-account)
- 问题反馈：[Issues](https://github.com/longzheng268/auto-x-account/issues)

---

<div align="center">

**如果这个项目对你有帮助，请给一个 ⭐️**

Made with ❤️ by Auto X Account Team

</div>
