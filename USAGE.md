# X 账号自动注册系统 - 使用文档

## 系统功能

### ✅ 核心功能
1. **完整的注册流程** - 支持从邮箱切换到密码设置的全流程自动化
2. **数据持久化** - 所有配置和数据保存在系统标准目录
3. **导入导出** - 支持 JSON/CSV/Excel 格式
4. **批量注册** - 支持暂停/恢复/停止控制
5. **BitBrowser集成** - 支持指纹浏览器，每账号独立配置
6. **完整日志** - 详细的步骤日志，便于调试
7. **GUI配置** - 所有选项可通过界面配置
8. **多语言支持** - 中文/英文切换

## 数据目录

配置和数据保存位置：
- **Windows**: `%APPDATA%\auto-x-account\`
- **macOS**: `~/Library/Application Support/auto-x-account/`
- **Linux**: `~/.local/share/auto-x-account/`

目录结构：
```
auto-x-account/
├── config.json           # 配置文件
├── accounts.json         # 账号数据
├── emails.json          # 邮箱数据
├── tasks/               # 批量任务
├── screenshots/         # 截图
├── browser_data/        # 浏览器数据
└── logs/                # 日志文件
```

## 配置说明

### BitBrowser 配置
```json
{
  "browser": {
    "browser_type": "bitbrowser",
    "bitbrowser": {
      "api_url": "http://127.0.0.1",
      "api_port": 54345,
      "profile_ids": [],
      "auto_create_profile": true,
      "separate_profile_per_account": true
    }
  }
}
```

### 代理配置
支持三种模式：
- **None** - 不使用代理
- **System** - 自动检测系统代理
- **Manual** - 手动配置（HTTP/HTTPS/SOCKS5）

可以为浏览器和邮箱分别配置代理。

### 邮箱提供商
- **MailTm** - 免费临时邮箱（测试用）
- **GuerrillaMail** - 免费临时邮箱（测试用）
- **SelfHosted** - 自建邮箱服务器（生产环境）
- **Custom** - 自定义IMAP/SMTP配置

## 使用方法

### GUI 模式（推荐）
```bash
./auto-x-account
# 或
./auto-x-account gui
```

在GUI中可以：
- 配置所有选项
- 切换语言（右上角🌐按钮）
- 查看注册进度
- 管理账号列表

### 命令行模式
```bash
# 单个注册
./auto-x-account register --email test@example.com

# 批量注册
./auto-x-account batch --count 10 --concurrent 3

# 导出账号
./auto-x-account export -o accounts.xlsx

# 导入账号
./auto-x-account import -i accounts.xlsx
```

## 注册流程

1. 切换到邮箱注册模式
2. 填写姓名、邮箱、出生日期
3. 处理人机验证（如出现，需手动完成）
4. 输入邮箱验证码
5. 设置密码
6. 点击 Get Started
7. 自动跳过可选步骤

整个流程会自动截图记录，保存在 `screenshots/` 目录。

## 注意事项

1. **生产环境**：必须使用自建邮箱，不能用临时邮箱
2. **BitBrowser**：使用前确保已启动并运行在配置的端口
3. **代理**：大批量注册建议配置代理，避免IP被封
4. **人机验证**：当前需要手动完成，建议使用干净IP减少出现频率
5. **日志**：定期检查日志目录，自动清理30天前的日志

## 技术细节

### 依赖
- Rust 2021 edition
- egui - GUI界面
- chromiumoxide - 浏览器自动化
- tokio - 异步运行时

### 构建
```bash
cargo build --release
```

### 测试
```bash
cargo test
```

---

**版本**: v0.2.0  
**更新日期**: 2024-11-24
