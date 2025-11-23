# 邮箱提供商 GUI 集成文档
# Email Provider GUI Integration Documentation

## 概述 / Overview

本文档介绍了新增的邮箱提供商选择和自动生成功能的使用方法。
This document describes how to use the new email provider selection and auto-generation features.

## 功能特性 / Features

### 1. 邮箱提供商选择 / Email Provider Selection

在 GUI 设置面板中，您可以选择以下邮箱提供商：

**测试模式 (Test Mode) 可用提供商:**
- **mail.tm** - 免费临时邮箱服务，无需额外配置
- **Guerrilla Mail** - 老牌临时邮箱服务
- **Temp-Mail.org** - 简单易用的临时邮箱
- **10minutemail** - 10分钟有效期的临时邮箱
- **DropMail** - GraphQL API 驱动的临时邮箱
- **Mailinator** - 需要 API Key 的临时邮箱服务
- **SelfHosted** - 使用您自己的 SMTP 域名
- **Custom** - 自定义 IMAP/SMTP 配置

**生产模式 (Production Mode) 可用提供商:**
- **SelfHosted** - 使用您自己的 SMTP 域名（推荐）
- **Custom** - 自定义 IMAP/SMTP 配置

### 2. 邮箱输入模式 / Email Input Mode

在主注册面板中，您可以选择两种邮箱输入模式：

#### 手动输入模式 (Manual Input)
- 手动输入邮箱地址
- 适合使用已有邮箱进行注册
- 需要自己管理邮箱验证码接收

#### 自动生成模式 (Auto Generate)
- 使用选定的邮箱提供商自动生成临时邮箱
- 适合批量注册场景
- 自动处理邮箱验证码接收
- 无需手动输入邮箱地址

## 使用指南 / Usage Guide

### 配置邮箱提供商 / Configure Email Provider

1. 打开 GUI 界面
2. 点击右上角的"⚙ 设置"按钮
3. 滚动到"📮 邮箱提供商设置"部分
4. 从下拉菜单中选择您想使用的邮箱提供商
5. 根据提示配置相应的参数（如有需要）

#### 自定义提供商配置 (Custom Provider Configuration)

如果选择"Custom"自定义提供商，需要配置以下信息：

**SMTP 配置:**
- 主机 (Host): SMTP 服务器地址
- 端口 (Port): SMTP 服务器端口（通常是 587 或 465）

**IMAP 配置:**
- 主机 (Host): IMAP 服务器地址
- 端口 (Port): IMAP 服务器端口（通常是 993）

**认证信息:**
- 用户名 (Username): 邮箱账号
- 密码 (Password): 邮箱密码

**API 配置（可选）:**
- API Key: 某些服务需要的 API 密钥
- API Endpoint: API 端点 URL

#### Mailinator 配置

如果选择 Mailinator，需要配置：
- **API Key**: 从 [Mailinator 官网](https://www.mailinator.com/) 获取

### 使用自动生成邮箱 / Use Auto-Generated Email

1. 在主注册面板中，选择"自动生成"模式
2. 系统会显示"✨ 将使用选定的邮箱提供商自动生成临时邮箱"
3. 点击"🚀 开始注册"按钮
4. 系统会自动：
   - 使用选定的提供商创建临时邮箱
   - 显示生成的邮箱地址
   - 使用该邮箱进行 X 账号注册
   - 自动接收并处理验证码

### 配置文件格式 / Configuration File Format

新的邮箱提供商配置已添加到 `config.json` 中：

```json
{
  "email_provider": {
    "force_self_hosted": false,
    "allowed_temp_providers": ["MailTm", "GuerrillaMail"],
    "production_domains": ["example.com"],
    "selected_provider": "MailTm",
    "custom_smtp_host": null,
    "custom_smtp_port": null,
    "custom_imap_host": null,
    "custom_imap_port": null,
    "custom_username": null,
    "custom_password": null,
    "custom_api_key": null,
    "custom_api_endpoint": null
  }
}
```

## 最佳实践 / Best Practices

### 测试环境 (Test Environment)

1. 使用免费的临时邮箱服务（如 mail.tm 或 Guerrilla Mail）
2. 不要在测试中使用生产邮箱
3. 定期清理测试账号

### 生产环境 (Production Environment)

1. **强烈推荐使用自建邮箱系统**
   - MailCow（推荐）
   - Postfix + Dovecot
   - Maddy Mail Server

2. **配置 DNS 记录**
   - SPF 记录：防止邮件被标记为垃圾邮件
   - DKIM 记录：验证邮件来源
   - DMARC 记录：邮件安全策略

3. **使用独立域名**
   - 不要使用主业务域名
   - 准备多个备用域名
   - 使用二级域名（如 mail.yourdomain.com）

4. **限制速率**
   - 避免短时间内发送大量邮件
   - 实施延迟机制
   - 监控邮件队列

## 故障排查 / Troubleshooting

### 问题 1: 无法生成邮箱

**可能原因:**
- 邮箱提供商服务不可用
- 网络连接问题
- API 配置错误

**解决方案:**
1. 检查网络连接
2. 验证邮箱提供商服务状态
3. 检查 API Key 是否正确
4. 尝试切换到其他邮箱提供商

### 问题 2: 无法接收验证码

**可能原因:**
- IMAP 配置错误
- 邮箱服务商限制
- 验证码邮件被拦截

**解决方案:**
1. 验证 IMAP 配置是否正确
2. 检查邮箱是否有新邮件
3. 查看垃圾邮件文件夹
4. 增加等待超时时间

### 问题 3: 生产模式下无法使用临时邮箱

**原因:**
- 生产模式下禁止使用临时邮箱服务

**解决方案:**
1. 切换到测试模式（如果是测试环境）
2. 配置自建邮箱系统
3. 使用 Custom 提供商配置您的邮箱服务

## 安全建议 / Security Recommendations

1. **不要在配置文件中明文存储密码**
   - 使用环境变量
   - 使用密钥管理服务

2. **定期更换 API Key**
   - 特别是在生产环境中
   - 监控 API 使用情况

3. **限制访问权限**
   - 使用最小权限原则
   - 定期审查访问日志

4. **加密敏感数据**
   - 使用 HTTPS/TLS 传输
   - 加密存储的凭证

## API 参考 / API Reference

### EmailProvider 枚举

```rust
pub enum EmailProvider {
    SelfHosted,    // 自建邮箱
    MailTm,        // mail.tm
    GuerrillaMail, // Guerrilla Mail
    TempMail,      // Temp-Mail.org
    TenMinuteMail, // 10minutemail
    DropMail,      // DropMail
    Mailinator,    // Mailinator
    Custom,        // 自定义
}
```

### 关键方法

```rust
// 从字符串解析提供商
EmailProvider::from_str("MailTm") -> Option<EmailProvider>

// 获取邮箱提供商配置
config.get_email_provider_config() -> EmailProviderConfig

// 创建临时邮箱
email_manager.create_temp_email().await -> Result<TempEmail>
```

## 更新日志 / Changelog

### v1.1.0 (2024-01-XX)
- ✨ 新增邮箱提供商选择功能
- ✨ 新增自动生成邮箱模式
- ✨ 新增自定义邮箱提供商配置
- 🎨 改进 GUI 设置面板布局
- 📝 新增邮箱提供商集成文档

## 相关文档 / Related Documentation

- [MailCow 部署指南](./mailcow-setup.md)
- [数据持久化和代理配置](./persistence-and-proxy.md)
- [使用示例](./usage-examples.md)

## 支持 / Support

如果您遇到问题或有建议，请：
1. 查看本文档的故障排查部分
2. 在 [GitHub Issues](https://github.com/longzheng268/auto-x-account/issues) 提交问题
3. 参考其他文档获取更多信息
