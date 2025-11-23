# 快速参考：邮箱功能更新
# Quick Reference: Email Functionality Updates

## 问题 / Issue
原始需求提到代码逻辑与 GUI 不匹配，缺少邮箱提供商选择和自定义配置选项。新需求增加了随机邮箱生成功能。

The original issue mentioned code logic not aligning with GUI, lacking email provider selection and custom configuration options. New requirement added random email generation.

## 解决方案概述 / Solution Overview

### 1. GUI 新增功能 / New GUI Features

#### 设置窗口 (Settings Window)
```
⚙ 设置 -> 📮 邮箱提供商设置
```

**新增内容:**
- 邮箱提供商下拉选择器（测试模式 8 个选项，生产模式 2 个选项）
- Custom 提供商完整配置界面：
  - SMTP 主机/端口
  - IMAP 主机/端口
  - 用户名/密码
  - API Key/Endpoint
- Mailinator API Key 配置
- SelfHosted 域名显示

#### 主注册面板 (Main Registration Panel)
```
📧 注册新账号
```

**新增内容:**
- 当前邮箱提供商显示
- 邮箱模式选择（手动输入 / 自动生成）
- 手动输入模式：文本输入框
- 自动生成模式：提示信息 + 生成的邮箱显示

### 2. 配置文件扩展 / Configuration Extension

**新增字段 (config.json):**
```json
{
  "email_provider": {
    "selected_provider": "MailTm",      // 当前选择的提供商
    "custom_smtp_host": null,           // 自定义 SMTP 主机
    "custom_smtp_port": null,           // 自定义 SMTP 端口
    "custom_imap_host": null,           // 自定义 IMAP 主机
    "custom_imap_port": null,           // 自定义 IMAP 端口
    "custom_username": null,            // 自定义用户名
    "custom_password": null,            // 自定义密码
    "custom_api_key": null,             // 自定义 API Key
    "custom_api_endpoint": null         // 自定义 API Endpoint
  }
}
```

### 3. 代码新增 API / New Code APIs

#### EmailProvider 枚举增强
```rust
// 从字符串解析提供商
EmailProvider::from_str("MailTm") -> Option<EmailProvider>

// 转换为字符串
provider.to_string() -> String
```

#### Config 新增方法
```rust
// 根据 GUI 配置创建后端邮箱配置
config.get_email_provider_config() -> EmailProviderConfig
```

## 支持的邮箱提供商 / Supported Email Providers

| 提供商 | 测试模式 | 生产模式 | 需要配置 |
|--------|---------|---------|----------|
| mail.tm | ✅ | ❌ | 无 |
| Guerrilla Mail | ✅ | ❌ | 无 |
| Temp-Mail.org | ✅ | ❌ | 无 |
| 10minutemail | ✅ | ❌ | 无 |
| DropMail | ✅ | ❌ | 无 |
| Mailinator | ✅ | ❌ | API Key |
| SelfHosted | ✅ | ✅ | SMTP 域名 |
| Custom | ✅ | ✅ | 完整配置 |

## 使用流程 / Usage Flow

### 方式 1: 使用临时邮箱（测试模式）

1. 确保配置文件中 `mode` 为 `"test"`
2. 打开 GUI 设置
3. 选择邮箱提供商（如 mail.tm）
4. 保存设置
5. 在主面板选择"自动生成"模式
6. 点击"开始注册"

### 方式 2: 使用自建邮箱（生产模式）

1. 设置 `mode` 为 `"production"`
2. 配置 SMTP 域名（如 example.com）
3. 在设置中选择 "SelfHosted"
4. 保存设置
5. 选择"自动生成"模式
6. 系统会使用配置的域名生成随机邮箱

### 方式 3: 使用自定义邮箱

1. 在设置中选择 "Custom"
2. 配置 SMTP 和 IMAP 服务器信息
3. 输入认证凭据
4. （可选）配置 API 信息
5. 保存设置
6. 使用"自动生成"或"手动输入"模式

## 文件改动清单 / File Changes

```
修改的文件 (Modified):
  src/config.rs           +61 行  - 扩展配置结构
  src/email_provider.rs   +43 行  - 增加枚举方法
  src/gui.rs              +268 行 - 更新 GUI 界面
  config.example.json     +11 行  - 更新配置示例

新增的文件 (New):
  docs/email-provider-gui-integration.md    - 用户文档
  IMPLEMENTATION_EMAIL_PROVIDER_GUI.md      - 技术实现文档
  docs/quick-reference-email-updates.md     - 快速参考（本文档）
```

## 下一步工作 / Next Steps

要使邮箱生成功能完全工作，需要：

1. **添加异步运行时到 GUI**
   ```rust
   // 在 AppState 中
   runtime: Arc<tokio::runtime::Runtime>
   email_manager: Arc<Mutex<BatchEmailManager>>
   ```

2. **实现"开始注册"按钮逻辑**
   ```rust
   if !state.email_manual_mode {
       // 使用后台线程或 runtime 生成邮箱
       // 更新 state.email 和 state.status
   }
   ```

3. **集成到批量注册流程**
   ```rust
   // 在 batch.rs 中使用
   let email_config = config.get_email_provider_config();
   ```

## 技术说明 / Technical Notes

### 为什么还需要异步运行时？

egui GUI 在主线程同步运行，但邮箱生成 API 是异步的。有两个解决方案：

**方案 A (简单但阻塞):**
```rust
runtime.block_on(async {
    email_manager.create_temp_email().await
})
```

**方案 B (推荐，不阻塞):**
```rust
// 使用 channel 在后台线程生成
let (tx, rx) = mpsc::channel();
thread::spawn(move || {
    let email = runtime.block_on(create_email());
    tx.send(email).unwrap();
});

// 在 GUI 更新时检查
if let Ok(email) = rx.try_recv() {
    state.email = email;
}
```

### 配置兼容性

所有新增字段都使用 `#[serde(default)]`，确保：
- 旧配置文件可以正常加载
- 缺失字段自动使用默认值
- 向后兼容

## 测试清单 / Testing Checklist

- [x] 配置文件正确序列化/反序列化
- [x] GUI 提供商选择正常显示
- [x] 自定义配置 UI 字段完整
- [x] 模式切换视觉效果正确
- [ ] 邮箱生成功能（待实现）
- [ ] 批量注册集成（待实现）
- [ ] 错误处理（待实现）

## 参考文档 / References

1. **用户指南**: `docs/email-provider-gui-integration.md`
   - 如何使用新功能
   - 配置不同提供商
   - 故障排查

2. **实现文档**: `IMPLEMENTATION_EMAIL_PROVIDER_GUI.md`
   - 技术实现细节
   - 待完成工作
   - 代码审查要点

3. **配置示例**: `config.example.json`
   - 完整的配置文件示例
   - 所有字段说明

## 常见问题 / FAQ

**Q: 为什么生产模式下看不到临时邮箱选项？**
A: 生产模式禁止使用临时邮箱服务，这是安全设计。请使用自建邮箱或切换到测试模式。

**Q: Custom 提供商需要配置哪些字段？**
A: 至少需要 SMTP 主机/端口和 IMAP 主机/端口，以及认证信息（用户名/密码）。API 配置是可选的。

**Q: 如何知道邮箱是否成功生成？**
A: 当前在状态栏会显示"正在生成临时邮箱..."，成功后会显示邮箱地址。完整的异步反馈待实现。

**Q: 可以同时使用多个邮箱提供商吗？**
A: 当前版本一次只能使用一个提供商。批量注册时所有账号使用相同提供商。

## 贡献者 / Contributors

- GUI 设计和实现
- 配置系统扩展
- 文档编写

---

**最后更新**: 2024-01-XX
**版本**: v1.1.0
