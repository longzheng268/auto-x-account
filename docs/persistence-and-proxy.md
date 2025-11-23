# 数据持久化和代理配置说明

## 数据持久化 (Data Persistence)

### 功能说明

系统现在支持自动数据持久化，所有任务和账号信息会自动保存到本地缓存目录。

### 缓存目录位置

缓存目录根据操作系统自动选择：

| 操作系统 | 缓存目录路径 |
|---------|------------|
| **Windows** | `%APPDATA%\auto-x-account\` |
| **macOS** | `~/Library/Application Support/auto-x-account/` |
| **Linux** | `~/.local/share/auto-x-account/` |

### 缓存文件

- `tasks.json` - 批量注册任务信息
- `accounts.json` - 已注册的账号信息

### 自动保存时机

系统会在以下操作后自动保存数据：

1. **任务更新时**
   - 任务进度更新（成功/失败计数）
   - 任务状态变更（运行中、已完成、已暂停等）

2. **账号创建时**
   - 每成功注册一个账号后立即保存

3. **启动时加载**
   - 程序启动时自动从缓存加载历史数据

### 使用示例

```rust
// 创建批量注册管理器
let batch_manager = BatchRegistrationManager::new(config, email_handler, email_manager);

// 从缓存加载历史数据
batch_manager.load_from_cache().await?;

// 开始批量注册 - 数据会自动保存
batch_manager.start_batch_registration(10, 3, false).await?;
```

### 手动导出

除了自动缓存，你还可以手动导出账号到指定位置：

```bash
# 导出为 JSON 格式
./auto-x-account export --output my_accounts.json --format json

# 导出为 CSV 格式
./auto-x-account export --output my_accounts.csv --format csv

# 导出为 TXT 格式
./auto-x-account export --output my_accounts.txt --format txt
```

或在代码中：

```rust
// 导出账号
batch_manager.export_accounts("accounts_backup.json", ExportFormat::Json).await?;
```

## 日志系统 (Logging System)

### 功能说明

系统现在同时输出日志到文件和控制台，方便调试和审计。

### 日志目录位置

| 操作系统 | 日志目录路径 |
|---------|------------|
| **Windows** | `%APPDATA%\auto-x-account\logs\` |
| **macOS** | `~/Library/Logs/auto-x-account/` |
| **Linux** | `~/.local/share/auto-x-account/logs/` |

### 日志文件命名

日志文件按日期命名：`auto-x-account_YYYYMMDD.log`

例如：
- `auto-x-account_20241123.log`
- `auto-x-account_20241124.log`

### 日志级别

默认日志级别为 `INFO`，可通过环境变量调整：

```bash
# Linux/macOS
export RUST_LOG=debug
./auto-x-account

# Windows PowerShell
$env:RUST_LOG="debug"
.\auto-x-account.exe
```

支持的级别：
- `ERROR` - 仅错误
- `WARN` - 警告及以上
- `INFO` - 信息及以上（默认）
- `DEBUG` - 调试信息
- `TRACE` - 详细追踪

### 自动清理

系统会自动清理超过 30 天的旧日志文件，避免占用过多磁盘空间。

### 查看日志

```bash
# Linux/macOS
tail -f ~/.local/share/auto-x-account/logs/auto-x-account_20241123.log

# Windows PowerShell
Get-Content "$env:APPDATA\auto-x-account\logs\auto-x-account_20241123.log" -Wait
```

## 代理配置分离 (Proxy Separation)

### 功能说明

系统现在支持为浏览器和邮箱分别配置代理，提供更灵活的网络控制。

### 配置选项

在 `config.json` 中新增两个字段：

```json
{
  "proxy": {
    "mode": "manual",
    "type": "socks5",
    "host": "127.0.0.1",
    "port": 1080,
    "username": "",
    "password": "",
    "browser_enabled": true,    // 浏览器是否使用代理
    "email_enabled": false      // 邮箱是否使用代理
  }
}
```

### 使用场景

#### 场景 1：仅浏览器使用代理

适用于：访问 X (Twitter) 需要代理，但邮件服务器在本地网络

```json
{
  "proxy": {
    "mode": "manual",
    "type": "socks5",
    "host": "127.0.0.1",
    "port": 1080,
    "browser_enabled": true,
    "email_enabled": false
  }
}
```

#### 场景 2：浏览器和邮箱都使用代理

适用于：所有网络访问都需要通过代理

```json
{
  "proxy": {
    "mode": "manual",
    "type": "socks5",
    "host": "127.0.0.1",
    "port": 1080,
    "browser_enabled": true,
    "email_enabled": true
  }
}
```

#### 场景 3：都不使用代理

适用于：本地开发或直连网络环境

```json
{
  "proxy": {
    "mode": "none",
    "browser_enabled": false,
    "email_enabled": false
  }
}
```

### 代理模式说明

#### none - 不使用代理
```json
{
  "mode": "none"
}
```

#### system - 使用系统代理

自动检测并使用系统配置的代理：

```json
{
  "mode": "system",
  "browser_enabled": true,
  "email_enabled": false
}
```

系统会按以下顺序检测：
1. 环境变量：`HTTPS_PROXY`, `HTTP_PROXY`, `ALL_PROXY`
2. Windows 注册表（Windows 系统）
3. networksetup 命令（macOS 系统）
4. gsettings（Linux GNOME 系统）

#### manual - 手动配置代理

```json
{
  "mode": "manual",
  "type": "socks5",           // 支持: http, https, socks5
  "host": "127.0.0.1",
  "port": 1080,
  "username": "user",         // 可选：代理认证用户名
  "password": "pass",         // 可选：代理认证密码
  "browser_enabled": true,
  "email_enabled": false
}
```

### 代码中使用

```rust
use crate::config::ProxyTarget;

// 获取浏览器代理
if let Some(proxy_url) = config.get_proxy_url(ProxyTarget::Browser) {
    println!("浏览器代理: {}", proxy_url);
}

// 获取邮箱代理
if let Some(proxy_url) = config.get_proxy_url(ProxyTarget::Email) {
    println!("邮箱代理: {}", proxy_url);
}

// 获取代理描述（用于显示）
println!("{}", config.get_proxy_description(ProxyTarget::Browser));
println!("{}", config.get_proxy_description(ProxyTarget::Email));
```

### 验证配置

启动程序时会显示代理配置信息：

```
[INFO] 浏览器 手动代理 / Manual Proxy: socks5://127.0.0.1:1080
[INFO] 邮箱代理: 已禁用 / Disabled
```

## 人机验证说明 (Captcha Verification)

详细说明请参考 `src/captcha.rs` 文件开头的文档注释。

### 简要说明

系统支持两种人机验证方式：

1. **手动模式**（推荐）
   - 程序检测到验证码后暂停
   - 用户手动完成验证
   - 验证完成后程序自动继续
   - 成功率高，无额外成本

2. **第三方服务**（可选）
   - 集成 2Captcha、Anti-Captcha 等服务
   - 全自动处理验证码
   - 适合大批量注册
   - 需要付费（约 $1-3 / 1000次）

### 配置手动模式

在 `config.json` 中设置：

```json
{
  "browser": {
    "headless": false  // 必须设置为 false 才能手动操作
  }
}
```

### 使用建议

- **小规模注册**（<100个/天）：使用手动模式
- **中等规模注册**（100-1000个/天）：手动模式 + 多个实例
- **大规模注册**（>1000个/天）：集成第三方验证服务

## 常见问题 (FAQ)

### Q1: 缓存文件太大怎么办？

A: 可以定期清理旧的任务和账号数据：

```bash
# 删除缓存目录
# Windows
rmdir /s "%APPDATA%\auto-x-account"

# Linux/macOS
rm -rf ~/.local/share/auto-x-account
```

### Q2: 如何备份数据？

A: 复制缓存目录到安全位置：

```bash
# Linux/macOS
cp -r ~/.local/share/auto-x-account ~/backup/

# Windows PowerShell
Copy-Item "$env:APPDATA\auto-x-account" -Destination "D:\backup\" -Recurse
```

### Q3: 代理设置不生效？

A: 检查以下几点：

1. 确认代理服务器正在运行
2. 检查 `browser_enabled` 或 `email_enabled` 是否设置正确
3. 验证代理地址和端口
4. 查看日志文件中的代理相关信息

### Q4: 如何同时使用不同的代理？

A: 当前版本浏览器和邮箱使用相同的代理配置，只能控制是否启用。如需使用不同代理，建议：

1. 浏览器使用手动配置的代理
2. 邮箱直连或通过系统代理

### Q5: 日志文件在哪里？

A: 根据操作系统：
- Windows: `%APPDATA%\auto-x-account\logs\`
- macOS: `~/Library/Logs/auto-x-account/`
- Linux: `~/.local/share/auto-x-account/logs/`

## 更新日志

### v0.1.1 (2024-11-23)

- ✨ 新增：自动数据持久化功能
  - 任务和账号数据自动保存到缓存目录
  - 支持从缓存加载历史数据
  
- ✨ 新增：完整的日志系统
  - 同时输出到文件和控制台
  - 自动清理30天前的旧日志
  - 根据操作系统选择合适的日志路径

- ✨ 新增：代理设置分离
  - 浏览器和邮箱可以分别配置是否使用代理
  - 新增 `browser_enabled` 和 `email_enabled` 配置项
  
- 📝 新增：人机验证处理文档
  - 详细说明手动模式和第三方服务模式
  - 提供不同规模的使用建议
  - 说明 X 平台常见验证码类型

- 🔧 改进：配置文件结构
  - 更新示例配置文件
  - 添加向后兼容的默认值
