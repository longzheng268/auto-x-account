# Email Plus 模式 / Email Plus Mode

## 概述 / Overview

Email Plus 模式是一种智能的测试方法，允许您使用现有的 Gmail、Outlook 或其他主流邮箱服务的 "+" 后缀功能进行 X 账号注册测试，而无需创建新的邮箱账号。

Email Plus Mode is an intelligent testing method that allows you to use the "+" suffix feature of existing Gmail, Outlook, or other mainstream email services for X account registration testing without creating new email accounts.

## 工作原理 / How It Works

### Plus 地址别名 / Plus Address Aliasing

大多数现代邮箱服务（如 Gmail、Outlook）支持在邮箱用户名中使用 "+" 符号创建别名：

- **基础邮箱**: `yourname@gmail.com`
- **Plus 地址**: `yourname+test01@gmail.com`
- **Plus 地址**: `yourname+x2@gmail.com`

所有发送到这些 Plus 地址的邮件都会被路由到您的基础邮箱 `yourname@gmail.com`。

Most modern email services (like Gmail, Outlook) support creating aliases using the "+" symbol in the username:

- **Base email**: `yourname@gmail.com`
- **Plus address**: `yourname+test01@gmail.com`
- **Plus address**: `yourname+x2@gmail.com`

All emails sent to these Plus addresses will be routed to your base email `yourname@gmail.com`.

## 优势 / Advantages

### 1. 无需创建新邮箱 / No Need to Create New Emails
✅ 使用现有的 Gmail/Outlook 账号  
✅ Use existing Gmail/Outlook accounts

### 2. 高域名权重 / High Domain Trust
✅ Gmail/Outlook 等域名被 X 高度信任  
✅ Gmail/Outlook domains are highly trusted by X

### 3. 便捷的验证码接收 / Convenient Verification Code Reception
✅ 所有验证码都发到同一个基础邮箱  
✅ All verification codes sent to the same base email  
✅ 无需切换多个网页或邮箱  
✅ No need to switch between multiple pages or email accounts

### 4. 自动化测试友好 / Automation-Friendly
✅ 程序自动生成类人后缀，避免看起来像机器  
✅ Automatically generates human-like suffixes to avoid looking like a bot  
✅ 支持批量注册  
✅ Supports batch registration

## 配置说明 / Configuration

### 通过 GUI 配置 / Configure via GUI

1. 启动应用并点击 "⚙ 设置"
2. 滚动到 "➕ Email Plus 模式" 部分
3. 勾选 "启用 Plus 模式"
4. 输入您的基础邮箱地址（例如：`yourname@gmail.com`）
5. 选择后缀模式：
   - **自动生成人名**: 自动生成类似 `john123`, `emily456` 的后缀
   - **手动指定**: 手动输入后缀，如 `test01`

### 通过配置文件 / Configure via Config File

编辑 `config.json` 文件：

```json
{
  "email_provider": {
    "plus_mode": {
      "enabled": true,
      "base_email": "yourname@gmail.com",
      "suffix_mode": "auto",
      "manual_suffix": null
    }
  }
}
```

#### 配置项说明 / Configuration Options

- **`enabled`** (boolean): 是否启用 Plus 模式 / Enable Plus mode
- **`base_email`** (string): 基础邮箱地址 / Base email address
- **`suffix_mode`** (string): 后缀生成模式 / Suffix generation mode
  - `"auto"`: 自动生成人名后缀 / Auto-generate human-like suffixes
  - `"manual"`: 手动指定后缀 / Manually specify suffix
- **`manual_suffix`** (string): 手动指定的后缀（仅在 `suffix_mode = "manual"` 时使用）

## 使用方法 / Usage

### 单个注册 / Single Registration

```bash
# Plus 模式会自动使用配置的基础邮箱生成测试地址
./auto-x-account register
```

生成的邮箱地址示例 / Generated email example:
- `yourname+john123@gmail.com`
- `yourname+emily456@gmail.com`

### 批量注册 / Batch Registration

```bash
# 注册 10 个账号，每个使用不同的 Plus 地址
./auto-x-account batch --count 10 --concurrent 3
```

生成的邮箱地址示例 / Generated email examples:
- `yourname+john1@gmail.com`
- `yourname+emily2@gmail.com`
- `yourname+michael3@gmail.com`
- ...

## 支持的邮箱服务 / Supported Email Services

### ✅ 完全支持 / Fully Supported

- **Gmail** (`@gmail.com`, `@googlemail.com`)
- **Outlook** (`@outlook.com`, `@hotmail.com`, `@live.com`)
- **Yahoo** (`@yahoo.com`, `@yahoo.co.uk`)
- **ProtonMail** (`@protonmail.com`, `@pm.me`)
- **Zoho Mail** (`@zoho.com`)

### ⚠️ 可能支持 / May Support

其他邮箱服务可能也支持 Plus 地址功能，但需要自行测试验证。

Other email services may also support Plus addressing, but require testing to verify.

## 最佳实践 / Best Practices

### 1. 使用主流邮箱服务 / Use Mainstream Email Services
推荐使用 Gmail 或 Outlook，这些服务具有最高的域名信任度。

Recommend using Gmail or Outlook, which have the highest domain trust.

### 2. 自动生成模式 / Auto-Generation Mode
自动生成模式会创建类似真人的名字后缀（如 `john123`, `emily456`），避免被识别为机器行为。

Auto-generation mode creates human-like name suffixes (like `john123`, `emily456`) to avoid being identified as bot behavior.

### 3. 邮箱过滤规则 / Email Filter Rules
建议在您的邮箱中设置过滤规则，自动将带有特定 Plus 后缀的邮件归类到特定文件夹。

It's recommended to set up filter rules in your email to automatically categorize emails with specific Plus suffixes into designated folders.

#### Gmail 过滤规则示例 / Gmail Filter Example:
1. 打开 Gmail 设置 → 过滤器和阻止的地址
2. 创建新过滤器
3. 收件人: `yourname+*@gmail.com`
4. 应用标签: "X Registration"
5. 跳过收件箱（可选）

### 4. 定期清理 / Regular Cleanup
Plus 模式测试完成后，可以通过邮箱过滤器批量删除相关邮件。

After Plus mode testing, you can batch delete related emails through email filters.

## 注意事项 / Notes

### ⚠️ 测试目的 / Testing Purpose
Plus 模式主要用于测试和开发目的，不建议用于大规模生产环境。

Plus mode is primarily for testing and development purposes, not recommended for large-scale production use.

### ⚠️ 邮箱提供商限制 / Email Provider Limits
某些邮箱提供商可能对发送频率有限制，请注意不要过度使用。

Some email providers may have rate limits on sending frequency, be careful not to overuse.

### ⚠️ X 平台政策 / X Platform Policies
使用本功能时请遵守 X (Twitter) 的服务条款和使用政策。

When using this feature, please comply with X (Twitter)'s Terms of Service and usage policies.

## 故障排查 / Troubleshooting

### 问题：验证码未收到 / Issue: Verification Code Not Received

**检查项 / Checklist:**
1. ✅ 确认基础邮箱地址正确无误
2. ✅ 检查邮箱的垃圾邮件文件夹
3. ✅ 确认邮箱服务支持 Plus 地址
4. ✅ 检查邮箱过滤规则是否误删了邮件

### 问题：邮箱被识别为无效 / Issue: Email Recognized as Invalid

**解决方案 / Solutions:**
1. 尝试更换其他主流邮箱服务（Gmail/Outlook）
2. 使用自动生成模式，避免使用明显测试特征的后缀
3. 检查 X 平台是否对该域名有特殊限制

### 问题：批量注册失败率高 / Issue: High Failure Rate in Batch Registration

**优化建议 / Optimization Suggestions:**
1. 降低并发数（`--concurrent`）
2. 增加每次注册之间的延迟
3. 使用自动生成模式，确保每个后缀都唯一且自然

## 示例场景 / Example Scenarios

### 场景 1: 功能测试 / Scenario 1: Feature Testing
```json
{
  "plus_mode": {
    "enabled": true,
    "base_email": "developer@gmail.com",
    "suffix_mode": "auto",
    "manual_suffix": null
  }
}
```

用于开发者测试新功能，自动生成多个测试账号。

For developers to test new features with automatically generated test accounts.

### 场景 2: 演示环境 / Scenario 2: Demo Environment
```json
{
  "plus_mode": {
    "enabled": true,
    "base_email": "demo@outlook.com",
    "suffix_mode": "manual",
    "manual_suffix": "demo"
  }
}
```

用于演示环境，使用固定后缀 `demo`，便于识别。

For demo environments, using a fixed suffix `demo` for easy identification.

### 场景 3: 批量测试 / Scenario 3: Batch Testing
```json
{
  "plus_mode": {
    "enabled": true,
    "base_email": "testing@gmail.com",
    "suffix_mode": "auto",
    "manual_suffix": null
  }
}
```

用于批量测试，自动生成唯一的人名后缀。

For batch testing with automatically generated unique name suffixes.

## 技术细节 / Technical Details

### 后缀生成算法 / Suffix Generation Algorithm

自动模式使用以下算法生成后缀：

Auto mode uses the following algorithm to generate suffixes:

1. 从常见英文名字库中随机选择一个名字（如 John, Emily, Michael）
2. 添加一个随机数字（批量模式下使用递增索引）
3. 转换为小写
4. 结果示例：`john123`, `emily2`, `michael456`

### Plus 地址规范 / Plus Address Specification

Plus 地址遵循 RFC 5233 (Sieve Email Filtering: Subaddress Extension) 规范：

Plus addresses follow the RFC 5233 (Sieve Email Filtering: Subaddress Extension) specification:

- 用户名和后缀之间使用 "+" 分隔
- 后缀可以包含字母、数字、破折号、下划线
- 邮件服务器会忽略 "+" 及其后的所有字符进行邮件投递

## 相关资源 / Related Resources

- [Gmail Plus Addressing](https://gmail.googleblog.com/2008/03/2-hidden-ways-to-get-more-from-your.html)
- [RFC 5233 - Subaddress Extension](https://tools.ietf.org/html/rfc5233)
- [X (Twitter) Account Registration Best Practices](https://help.twitter.com/en/using-twitter/create-twitter-account)

## 反馈和支持 / Feedback and Support

如果您在使用 Plus 模式时遇到问题或有改进建议，请通过以下方式反馈：

If you encounter issues or have suggestions for improvements when using Plus mode, please provide feedback through:

- [GitHub Issues](https://github.com/longzheng268/auto-x-account/issues)
- 项目文档 / Project Documentation

---

**最后更新 / Last Updated**: 2024-11-23
