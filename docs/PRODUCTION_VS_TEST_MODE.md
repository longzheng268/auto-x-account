# 生产模式 vs 测试模式说明
# Production Mode vs Test Mode Guide

## 📋 概述 / Overview

本系统支持两种运行模式，确保生产环境的安全性和合规性：

This system supports two running modes to ensure production environment security and compliance:

1. **生产模式 (Production Mode)** - 仅允许使用自建域名邮箱
2. **测试模式 (Test Mode)** - 允许使用临时邮箱进行功能测试

---

## 🏭 生产模式 (Production Mode)

### 特点 / Features

- ✅ **强制使用自建域名邮箱** - 只能使用您自己的域名
- ✅ **安全可控** - 完全掌握邮箱数据和隐私
- ✅ **长期稳定** - 不依赖第三方临时邮箱服务
- ✅ **合规性** - 符合企业级应用要求

### 允许的邮箱提供商 / Allowed Email Providers

在生产模式下，**只能**使用以下提供商：

In production mode, **only** the following providers are allowed:

- `SelfHosted` - 自建邮件服务器
- `Custom` - 自定义 IMAP/SMTP 配置

### 配置要求 / Configuration Requirements

#### 1. 设置运行模式

```json
{
  "mode": "production"
}
```

#### 2. 配置生产域名

```json
{
  "email_provider": {
    "force_self_hosted": true,
    "production_domains": [
      "yourdomain.com",
      "mail.yourdomain.com"
    ]
  }
}
```

#### 3. 配置 SMTP 服务器

```json
{
  "smtp": {
    "host": "mail.yourdomain.com",
    "port": 25,
    "domain": "yourdomain.com",
    "enable": true
  }
}
```

### 推荐的邮件服务器方案 / Recommended Email Server Solutions

#### 1. **Postfix + Dovecot** (推荐 / Recommended)
- ✅ 完全开源免费
- ✅ 功能强大，可扩展性强
- ✅ 社区支持完善
- 📖 适合有一定 Linux 经验的用户

**安装命令 / Installation:**
```bash
# Ubuntu/Debian
sudo apt install postfix dovecot-core dovecot-imapd dovecot-lmtpd

# CentOS/RHEL
sudo yum install postfix dovecot
```

#### 2. **MailCow** (推荐新手 / Recommended for Beginners)
- ✅ 基于 Docker，部署简单
- ✅ Web 管理界面
- ✅ 集成反垃圾邮件、防病毒
- 📖 适合快速部署

**安装命令 / Installation:**
```bash
cd /opt
git clone https://github.com/mailcow/mailcow-dockerized
cd mailcow-dockerized
./generate_config.sh
docker-compose up -d
```

官网: https://mailcow.email/

#### 3. **iRedMail** (一键安装 / One-click Installation)
- ✅ 自动化安装脚本
- ✅ 支持多个发行版
- ✅ 包含 Web 管理
- 📖 适合希望快速搭建的用户

**安装命令 / Installation:**
```bash
wget https://github.com/iredmail/iRedMail/archive/1.6.3.tar.gz
tar xzf 1.6.3.tar.gz
cd iRedMail-1.6.3
bash iRedMail.sh
```

官网: https://www.iredmail.org/

#### 4. **商业邮箱服务 / Commercial Email Services**

如果不想自建服务器，可以使用商业服务：

If you don't want to self-host, use commercial services:

- **Google Workspace** - $6/用户/月
- **Microsoft 365** - $5/用户/月
- **Zoho Mail** - 免费版可用，付费版 $1/用户/月
- **ProtonMail** - 注重隐私，€3.99/月起

### DNS 配置要求 / DNS Configuration Requirements

为确保邮件送达率，必须配置以下 DNS 记录：

To ensure email deliverability, configure these DNS records:

#### 1. **MX 记录** (Mail Exchange)
```
yourdomain.com.    MX    10    mail.yourdomain.com.
```

#### 2. **SPF 记录** (Sender Policy Framework)
```
yourdomain.com.    TXT   "v=spf1 ip4:YOUR_SERVER_IP ~all"
```

#### 3. **DKIM 记录** (DomainKeys Identified Mail)
```
default._domainkey.yourdomain.com.    TXT    "v=DKIM1; k=rsa; p=YOUR_PUBLIC_KEY"
```

#### 4. **DMARC 记录** (Domain-based Message Authentication)
```
_dmarc.yourdomain.com.    TXT    "v=DMARC1; p=quarantine; rua=mailto:postmaster@yourdomain.com"
```

### 验证配置 / Verify Configuration

使用以下工具验证邮件服务器配置：

Use these tools to verify email server configuration:

1. **MXToolbox**: https://mxtoolbox.com/
2. **Mail-Tester**: https://www.mail-tester.com/
3. **DMARC Analyzer**: https://www.dmarcanalyzer.com/

---

## 🧪 测试模式 (Test Mode)

### 特点 / Features

- ✅ **允许临时邮箱** - 可以使用临时邮箱服务进行测试
- ✅ **快速测试** - 无需配置真实邮件服务器
- ✅ **成本低** - 大多数临时邮箱服务免费
- ⚠️  **仅用于测试** - 不应在生产环境使用

### 允许的邮箱提供商 / Allowed Email Providers

测试模式下，可以使用以下提供商：

In test mode, you can use these providers:

#### 免费临时邮箱 / Free Temporary Email Services:

1. **Mail.tm**
   - API: https://docs.mail.tm/
   - 特点: 稳定，支持 API
   - 无需注册

2. **Guerrilla Mail**
   - API: https://www.guerrillamail.com/GuerrillaMailAPI.html
   - 特点: 老牌服务，速度快
   - 无需注册

3. **Temp-Mail.org**
   - API: https://temp-mail.org/en/api/
   - 特点: 简单易用
   - 无需注册

4. **10minutemail**
   - API: https://10minutemail.com/
   - 特点: 10分钟有效期
   - 适合快速测试

5. **DropMail**
   - API: GraphQL
   - 特点: 现代化 API
   - 无需注册

6. **Mailinator**
   - API: https://www.mailinator.com/v3/
   - 特点: 功能强大，企业级
   - 需要 API key（付费）

### 配置示例 / Configuration Example

```json
{
  "mode": "test",
  "email_provider": {
    "force_self_hosted": false,
    "allowed_temp_providers": [
      "MailTm",
      "GuerrillaMail",
      "TempMail"
    ]
  }
}
```

### ⚠️  重要警告 / Important Warnings

1. **不要在生产环境使用临时邮箱！**
   - 临时邮箱不可靠
   - 可能随时失效
   - 邮件可能被他人访问

2. **仅用于功能测试**
   - 测试注册流程
   - 测试验证码接收
   - 调试程序逻辑

3. **切换到生产前务必修改配置**
   - 将 `mode` 改为 `"production"`
   - 配置自己的域名邮箱

---

## 🔄 模式切换 / Mode Switching

### 从测试切换到生产 / Switch from Test to Production

1. 修改配置文件 / Modify config file:
```json
{
  "mode": "production",  // 改为 production
  "email_provider": {
    "force_self_hosted": true,  // 启用强制自建
    "production_domains": ["yourdomain.com"]  // 配置域名
  }
}
```

2. 配置邮件服务器 / Configure email server
3. 测试邮件发送和接收 / Test email sending and receiving
4. 验证 DNS 配置 / Verify DNS configuration

### 从生产切换到测试 / Switch from Production to Test

⚠️  **不建议** 在生产环境切换到测试模式！

Not recommended to switch from production to test mode!

如果需要测试，请使用单独的测试配置文件：

If testing is needed, use a separate test config file:

```bash
# 使用测试配置
./auto-x-account --config config.test.json

# 使用生产配置
./auto-x-account --config config.production.json
```

---

## 🛡️ 安全检查 / Security Checks

系统会自动执行以下检查：

The system automatically performs these checks:

### 生产模式检查 / Production Mode Checks

✅ 验证是否配置了生产域名
✅ 禁止使用临时邮箱提供商
✅ 检查邮箱地址是否属于生产域名
✅ 强制使用 SelfHosted 或 Custom 提供商

如果违反检查，会显示错误：

If checks are violated, an error will be shown:

```
❌ 生产模式限制：不允许使用临时邮箱服务
Production mode restriction: Temporary email services are not allowed

当前模式：Production
当前提供商：MailTm

解决方案 / Solutions:
1. 切换到测试模式（在配置中设置 mode = "test"）
2. 使用自建域名邮箱（设置 provider = "selfhosted"）
3. 配置自定义 IMAP/SMTP（设置 provider = "custom"）
```

---

## 📊 对比总结 / Comparison Summary

| 特性 / Feature | 生产模式 / Production | 测试模式 / Test |
|---------------|----------------------|----------------|
| 临时邮箱 | ❌ 禁止 | ✅ 允许 |
| 自建域名 | ✅ 必须 | ✅ 可选 |
| 成本 | 💰 需要服务器 | 🆓 免费 |
| 稳定性 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| 可控性 | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| 安全性 | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| 适用场景 | 正式运营 | 功能测试 |

---

## ❓ 常见问题 / FAQ

### Q1: 为什么生产模式不允许使用临时邮箱？

A: 临时邮箱存在以下问题：
- 不稳定，可能随时失效
- 不安全，邮件可能被他人访问
- 不可靠，影响账号的长期使用
- 不合规，不符合企业级应用要求

### Q2: 我没有自己的域名，能使用生产模式吗？

A: 不能。生产模式要求使用自己的域名。建议：
1. 购买域名（约 $10-15/年）
2. 使用商业邮箱服务（如 Zoho 免费版）
3. 使用测试模式进行功能测试

### Q3: 配置自建邮件服务器很复杂吗？

A: 使用 MailCow 或 iRedMail 可以大大简化配置：
- MailCow: 基于 Docker，一键部署
- iRedMail: 自动化脚本，10分钟完成

### Q4: 可以在测试模式下进行小规模注册吗？

A: 不建议。测试模式仅用于：
- 功能测试
- 流程调试
- 开发验证

实际注册请切换到生产模式。

### Q5: 如何验证我的邮件服务器配置正确？

A: 使用以下工具：
1. 发送测试邮件到 Gmail/Outlook
2. 使用 mail-tester.com 检查评分
3. 检查 SPF、DKIM、DMARC 记录

---

## 📚 更多资源 / Additional Resources

### 邮件服务器搭建教程 / Email Server Setup Tutorials

- [MailCow 官方文档](https://mailcow.github.io/mailcow-dockerized-docs/)
- [iRedMail 快速安装指南](https://docs.iredmail.org/install.iredmail.on.debian.ubuntu.html)
- [Postfix 完整指南](https://www.postfix.org/documentation.html)

### 邮件安全配置 / Email Security Configuration

- [SPF 配置指南](https://www.cloudflare.com/learning/dns/dns-records/dns-spf-record/)
- [DKIM 配置指南](https://www.cloudflare.com/learning/dns/dns-records/dns-dkim-record/)
- [DMARC 配置指南](https://dmarc.org/overview/)

### DNS 配置工具 / DNS Configuration Tools

- [Cloudflare DNS](https://www.cloudflare.com/dns/)
- [DNSimple](https://dnsimple.com/)
- [Route 53 (AWS)](https://aws.amazon.com/route53/)

---

## 🎯 最佳实践建议 / Best Practice Recommendations

### 开发阶段 / Development Phase
```
✅ 使用测试模式
✅ 使用临时邮箱测试功能
✅ 快速迭代开发
```

### 上线前准备 / Pre-Production
```
✅ 购买域名
✅ 配置邮件服务器
✅ 设置 DNS 记录
✅ 测试邮件送达率
```

### 生产运营 / Production Operation
```
✅ 切换到生产模式
✅ 使用自建域名邮箱
✅ 监控邮件送达率
✅ 定期检查 DNS 配置
```

---

**最后提醒 / Final Reminder:**

⚠️  **生产环境必须使用自己的域名邮箱！**
⚠️  **Production environment MUST use your own domain emails!**

临时邮箱仅供测试使用，违规使用可能导致：
- 账号不稳定
- 数据丢失
- 安全风险

Temporary emails are for testing only. Improper use may lead to:
- Account instability
- Data loss
- Security risks
