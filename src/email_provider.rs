//! 邮箱服务提供商模块
//! Email service provider module

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

/// 邮箱服务提供商类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailProvider {
    /// 自建 SMTP 服务器
    SelfHosted,
    /// mail.tm（临时邮箱）
    MailTm,
    /// Guerrilla Mail
    GuerrillaMail,
    /// Temp-Mail.org (临时邮箱)
    TempMail,
    /// 10minutemail.com (临时邮箱)
    TenMinuteMail,
    /// DropMail.me (临时邮箱)
    DropMail,
    /// Mailinator (临时邮箱)
    Mailinator,
    /// Outlook.com (Microsoft)
    Outlook,
    /// Gmail (Google)
    Gmail,
    /// Yahoo Mail
    Yahoo,
    /// ProtonMail
    ProtonMail,
    /// Zoho Mail
    ZohoMail,
    /// 自定义 IMAP/SMTP
    Custom,
}

impl EmailProvider {
    /// 从字符串解析邮箱提供商
    /// Parse email provider from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SelfHosted" => Some(EmailProvider::SelfHosted),
            "MailTm" => Some(EmailProvider::MailTm),
            "GuerrillaMail" => Some(EmailProvider::GuerrillaMail),
            "TempMail" => Some(EmailProvider::TempMail),
            "TenMinuteMail" => Some(EmailProvider::TenMinuteMail),
            "DropMail" => Some(EmailProvider::DropMail),
            "Mailinator" => Some(EmailProvider::Mailinator),
            "Outlook" => Some(EmailProvider::Outlook),
            "Gmail" => Some(EmailProvider::Gmail),
            "Yahoo" => Some(EmailProvider::Yahoo),
            "ProtonMail" => Some(EmailProvider::ProtonMail),
            "ZohoMail" => Some(EmailProvider::ZohoMail),
            "Custom" => Some(EmailProvider::Custom),
            _ => None,
        }
    }

    /// 转换为字符串
    /// Convert to string
    pub fn to_string(&self) -> String {
        match self {
            EmailProvider::SelfHosted => "SelfHosted".to_string(),
            EmailProvider::MailTm => "MailTm".to_string(),
            EmailProvider::GuerrillaMail => "GuerrillaMail".to_string(),
            EmailProvider::TempMail => "TempMail".to_string(),
            EmailProvider::TenMinuteMail => "TenMinuteMail".to_string(),
            EmailProvider::DropMail => "DropMail".to_string(),
            EmailProvider::Mailinator => "Mailinator".to_string(),
            EmailProvider::Outlook => "Outlook".to_string(),
            EmailProvider::Gmail => "Gmail".to_string(),
            EmailProvider::Yahoo => "Yahoo".to_string(),
            EmailProvider::ProtonMail => "ProtonMail".to_string(),
            EmailProvider::ZohoMail => "ZohoMail".to_string(),
            EmailProvider::Custom => "Custom".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailProviderConfig {
    pub provider: EmailProvider,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub imap_host: Option<String>,
    pub imap_port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub domain: String,
    /// API密钥（某些服务需要）/ API key (required by some services)
    pub api_key: Option<String>,
    /// API端点URL（某些服务需要）/ API endpoint URL (required by some services)
    pub api_endpoint: Option<String>,
}

/// 邮箱服务管理器
pub struct EmailProviderManager {
    config: EmailProviderConfig,
    /// 用于验证的运行模式配置（可选）
    validation_config: Option<crate::config::Config>,
}

impl EmailProviderManager {
    pub fn new(config: EmailProviderConfig) -> Self {
        EmailProviderManager { 
            config,
            validation_config: None,
        }
    }

    /// 创建带验证的管理器
    /// Create manager with validation config
    pub fn new_with_validation(config: EmailProviderConfig, validation_config: crate::config::Config) -> Self {
        EmailProviderManager {
            config,
            validation_config: Some(validation_config),
        }
    }

    /// 验证当前提供商配置
    /// Validate current provider configuration
    fn validate_provider(&self) -> Result<()> {
        if let Some(validation_cfg) = &self.validation_config {
            validation_cfg.validate_email_provider(&self.config.provider)?;
        }
        Ok(())
    }

    /// 批量创建临时邮箱（带生产模式检查）
    /// Batch create temporary emails (with production mode check)
    pub async fn batch_create_emails(&self, count: usize) -> Result<Vec<TempEmail>> {
        // 验证提供商
        self.validate_provider()?;

        info!("开始批量创建 {} 个邮箱", count);

        let mut emails = Vec::new();
        let mut failed_count = 0;

        for i in 0..count {
            match self.create_temp_email().await {
                Ok(email) => {
                    info!("成功创建邮箱 {}/{}: {}", i + 1, count, email.address);
                    emails.push(email);

                    // 添加延迟避免请求过快
                    if i < count - 1 {
                        sleep(Duration::from_millis(500)).await;
                    }
                }
                Err(e) => {
                    error!("创建邮箱 {}/{} 失败: {}", i + 1, count, e);
                    failed_count += 1;

                    // 如果连续失败太多次，停止批量创建
                    if failed_count > 5 {
                        warn!("连续失败次数过多，停止批量创建");
                        break;
                    }

                    // 失败后等待更长时间再重试
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }

        info!(
            "批量创建邮箱完成: 成功 {}, 失败 {}",
            emails.len(),
            failed_count
        );

        Ok(emails)
    }

    /// 批量创建并验证邮箱（确保邮箱可用）
    /// Batch create and verify emails
    pub async fn batch_create_verified_emails(
        &self,
        count: usize,
        verify: bool,
    ) -> Result<Vec<TempEmail>> {
        let emails = self.batch_create_emails(count).await?;

        if verify {
            info!("验证邮箱可用性...");
            let mut verified_emails = Vec::new();

            for email in emails {
                if self.verify_email(&email).await {
                    verified_emails.push(email);
                } else {
                    warn!("邮箱验证失败: {}", email.address);
                }
            }

            info!("验证完成: {}/{} 邮箱可用", verified_emails.len(), count);
            Ok(verified_emails)
        } else {
            Ok(emails)
        }
    }

    /// 验证邮箱是否可用
    async fn verify_email(&self, email: &TempEmail) -> bool {
        match email.provider {
            EmailProvider::MailTm => {
                // 尝试获取邮件列表来验证
                self.check_mail_tm_emails(email).await.is_ok()
            }
            EmailProvider::GuerrillaMail => self.check_guerrilla_emails(email).await.is_ok(),
            EmailProvider::TempMail | EmailProvider::TenMinuteMail | 
            EmailProvider::DropMail | EmailProvider::Mailinator => {
                // 这些服务创建即可用，无需额外验证
                true
            }
            _ => true, // 自建服务器和其他服务默认认为可用
        }
    }

    /// 创建临时邮箱（带生产模式检查）
    /// Create temporary email (with production mode check)
    pub async fn create_temp_email(&self) -> Result<TempEmail> {
        // 验证提供商
        self.validate_provider()?;

        match self.config.provider {
            EmailProvider::MailTm => self.create_mail_tm_account().await,
            EmailProvider::GuerrillaMail => self.create_guerrilla_account().await,
            EmailProvider::TempMail => self.create_tempmail_account().await,
            EmailProvider::TenMinuteMail => self.create_10minutemail_account().await,
            EmailProvider::DropMail => self.create_dropmail_account().await,
            EmailProvider::Mailinator => self.create_mailinator_account().await,
            EmailProvider::SelfHosted => {
                // 使用自己的域名生成随机邮箱
                let random_name = format!("user_{}", chrono::Utc::now().timestamp_millis());
                Ok(TempEmail {
                    address: format!("{}@{}", random_name, self.config.domain),
                    password: None,
                    token: None,
                    provider: EmailProvider::SelfHosted,
                })
            }
            EmailProvider::Outlook | EmailProvider::Gmail | EmailProvider::Yahoo 
            | EmailProvider::ProtonMail | EmailProvider::ZohoMail => {
                // 这些服务需要真实账号，不支持自动创建
                anyhow::bail!("{}不支持自动创建账号，请使用已有账号或配置IMAP/SMTP", 
                    format!("{:?}", self.config.provider))
            }
            EmailProvider::Custom => {
                // 使用配置的账号
                Ok(TempEmail {
                    address: format!(
                        "user_{}@{}",
                        chrono::Utc::now().timestamp_millis(),
                        self.config.domain
                    ),
                    password: self.config.password.clone(),
                    token: None,
                    provider: EmailProvider::Custom,
                })
            }
        }
    }

    /// 导出邮箱列表到文件
    /// Export email list to file
    pub fn export_emails_to_file(
        &self,
        emails: &[TempEmail],
        filename: &str,
        format: ExportFormat,
    ) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(filename)?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(emails)?;
                file.write_all(json.as_bytes())?;
            }
            ExportFormat::Csv => {
                // CSV 格式
                writeln!(file, "地址,密码,Token,提供商")?;
                for email in emails {
                    writeln!(
                        file,
                        "{},{},{},{:?}",
                        email.address,
                        email.password.as_deref().unwrap_or(""),
                        email.token.as_deref().unwrap_or(""),
                        email.provider
                    )?;
                }
            }
            ExportFormat::Txt => {
                // 纯文本格式，每行一个邮箱地址
                for email in emails {
                    writeln!(file, "{}", email.address)?;
                }
            }
        }

        info!("已导出 {} 个邮箱到文件: {}", emails.len(), filename);
        Ok(())
    }

    /// 从文件导入邮箱列表
    /// Import email list from file
    pub fn import_emails_from_file(filename: &str) -> Result<Vec<TempEmail>> {
        use std::fs;

        let content = fs::read_to_string(filename)?;

        // 尝试 JSON 格式
        if let Ok(emails) = serde_json::from_str::<Vec<TempEmail>>(&content) {
            info!("从文件导入了 {} 个邮箱 (JSON 格式)", emails.len());
            return Ok(emails);
        }

        // 尝试纯文本格式（每行一个邮箱）
        let mut emails = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && line.contains('@') {
                emails.push(TempEmail {
                    address: line.to_string(),
                    password: None,
                    token: None,
                    provider: EmailProvider::Custom,
                });
            }
        }

        info!("从文件导入了 {} 个邮箱 (文本格式)", emails.len());
        Ok(emails)
    }

    /// 创建 mail.tm 账号
    async fn create_mail_tm_account(&self) -> Result<TempEmail> {
        info!("正在创建 mail.tm 临时邮箱...");

        let client = reqwest::Client::new();

        // 获取可用域名
        let domains_response = client.get("https://api.mail.tm/domains").send().await?;

        let domains: serde_json::Value = domains_response.json().await?;
        let domain = domains["hydra:member"][0]["domain"]
            .as_str()
            .context("无法获取 mail.tm 域名")?;

        // 生成随机账号
        let username = format!("user_{}", chrono::Utc::now().timestamp_millis());
        let password = format!("Pass{}!@", rand::random::<u32>());
        let address = format!("{}@{}", username, domain);

        // 创建账号
        let create_response = client
            .post("https://api.mail.tm/accounts")
            .json(&serde_json::json!({
                "address": address,
                "password": password
            }))
            .send()
            .await?;

        if !create_response.status().is_success() {
            anyhow::bail!("创建 mail.tm 账号失败: {}", create_response.status());
        }

        info!("mail.tm 邮箱创建成功: {}", address);

        // 获取 token
        let token_response = client
            .post("https://api.mail.tm/token")
            .json(&serde_json::json!({
                "address": address,
                "password": password
            }))
            .send()
            .await?;

        let token_data: serde_json::Value = token_response.json().await?;
        let token = token_data["token"]
            .as_str()
            .context("无法获取 token")?
            .to_string();

        Ok(TempEmail {
            address,
            password: Some(password),
            token: Some(token),
            provider: EmailProvider::MailTm,
        })
    }

    /// 创建 Guerrilla Mail 账号
    async fn create_guerrilla_account(&self) -> Result<TempEmail> {
        info!("正在创建 Guerrilla Mail 临时邮箱...");

        let client = reqwest::Client::new();

        let response = client
            .get("https://api.guerrillamail.com/ajax.php")
            .query(&[("f", "get_email_address")])
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        let address = data["email_addr"]
            .as_str()
            .context("无法获取 Guerrilla Mail 地址")?
            .to_string();

        let sid = data["sid_token"].as_str().map(|s| s.to_string());

        info!("Guerrilla Mail 邮箱创建成功: {}", address);

        Ok(TempEmail {
            address,
            password: None,
            token: sid,
            provider: EmailProvider::GuerrillaMail,
        })
    }

    /// 创建 Temp-Mail.org 账号
    async fn create_tempmail_account(&self) -> Result<TempEmail> {
        info!("正在创建 Temp-Mail.org 临时邮箱...");

        let client = reqwest::Client::new();

        // Temp-Mail API (https://temp-mail.org/en/api/)
        let response = client
            .get("https://api.temp-mail.org/request/mail/id/1/")
            .send()
            .await?;

        let address = response.text().await?;

        info!("Temp-Mail.org 邮箱创建成功: {}", address);

        Ok(TempEmail {
            address,
            password: None,
            token: None,
            provider: EmailProvider::TempMail,
        })
    }

    /// 创建 10minutemail 账号
    async fn create_10minutemail_account(&self) -> Result<TempEmail> {
        info!("正在创建 10minutemail 临时邮箱...");

        let client = reqwest::Client::new();

        // 访问主页获取邮箱地址
        let response = client
            .get("https://10minutemail.com/session/address")
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        let address = data["address"]
            .as_str()
            .context("无法获取 10minutemail 地址")?
            .to_string();

        info!("10minutemail 邮箱创建成功: {}", address);

        Ok(TempEmail {
            address,
            password: None,
            token: None,
            provider: EmailProvider::TenMinuteMail,
        })
    }

    /// 创建 DropMail 账号  
    async fn create_dropmail_account(&self) -> Result<TempEmail> {
        info!("正在创建 DropMail 临时邮箱...");

        let client = reqwest::Client::new();

        // DropMail API
        let response = client
            .post("https://dropmail.me/api/graphql/web-test-wgq6m5i?query=mutation%20%7B%0A%20%20introduceSession%20%7B%0A%20%20%20%20id%0A%20%20%20%20expiresAt%0A%20%20%20%20addresses%20%7B%0A%20%20%20%20%20%20address%0A%20%20%20%20%7D%0A%20%20%7D%0A%7D")
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        let session_id = data["data"]["introduceSession"]["id"]
            .as_str()
            .context("无法获取 DropMail session ID")?
            .to_string();
        
        let address = data["data"]["introduceSession"]["addresses"][0]["address"]
            .as_str()
            .context("无法获取 DropMail 地址")?
            .to_string();

        info!("DropMail 邮箱创建成功: {}", address);

        Ok(TempEmail {
            address,
            password: None,
            token: Some(session_id),
            provider: EmailProvider::DropMail,
        })
    }

    /// 创建 Mailinator 账号
    async fn create_mailinator_account(&self) -> Result<TempEmail> {
        info!("正在创建 Mailinator 临时邮箱...");

        // Mailinator 不需要注册，直接生成一个随机邮箱地址即可
        let random_name = format!("user_{}", chrono::Utc::now().timestamp_millis());
        let address = format!("{}@mailinator.com", random_name);

        info!("Mailinator 邮箱创建成功: {}", address);

        Ok(TempEmail {
            address,
            password: None,
            token: None,
            provider: EmailProvider::Mailinator,
        })
    }

    /// 检查邮件
    pub async fn check_emails(&self, temp_email: &TempEmail) -> Result<Vec<EmailMessage>> {
        match temp_email.provider {
            EmailProvider::MailTm => self.check_mail_tm_emails(temp_email).await,
            EmailProvider::GuerrillaMail => self.check_guerrilla_emails(temp_email).await,
            EmailProvider::TempMail => self.check_tempmail_emails(temp_email).await,
            EmailProvider::TenMinuteMail => self.check_10minutemail_emails(temp_email).await,
            EmailProvider::DropMail => self.check_dropmail_emails(temp_email).await,
            EmailProvider::Mailinator => self.check_mailinator_emails(temp_email).await,
            EmailProvider::SelfHosted | EmailProvider::Custom | 
            EmailProvider::Outlook | EmailProvider::Gmail | EmailProvider::Yahoo |
            EmailProvider::ProtonMail | EmailProvider::ZohoMail => {
                // 这些由 SMTP/IMAP 服务器处理，或需要配置
                Ok(Vec::new())
            }
        }
    }

    async fn check_mail_tm_emails(&self, temp_email: &TempEmail) -> Result<Vec<EmailMessage>> {
        let token = temp_email.token.as_ref().context("缺少 mail.tm token")?;

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.mail.tm/messages")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        let messages = data["hydra:member"]
            .as_array()
            .context("无法解析邮件列表")?;

        let mut result = Vec::new();
        for msg in messages {
            if let Ok(email) = self.parse_mail_tm_message(msg) {
                result.push(email);
            }
        }

        Ok(result)
    }

    fn parse_mail_tm_message(&self, data: &serde_json::Value) -> Result<EmailMessage> {
        Ok(EmailMessage {
            id: data["id"].as_str().unwrap_or("").to_string(),
            from: data["from"]["address"].as_str().unwrap_or("").to_string(),
            subject: data["subject"].as_str().unwrap_or("").to_string(),
            body: data["intro"].as_str().unwrap_or("").to_string(),
            received_at: data["createdAt"].as_str().unwrap_or("").to_string(),
        })
    }

    async fn check_guerrilla_emails(&self, temp_email: &TempEmail) -> Result<Vec<EmailMessage>> {
        let sid = temp_email
            .token
            .as_ref()
            .context("缺少 Guerrilla Mail sid")?;

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.guerrillamail.com/ajax.php")
            .query(&[("f", "get_email_list"), ("sid_token", sid)])
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        let emails = data["list"].as_array().context("无法解析邮件列表")?;

        let mut result = Vec::new();
        for email in emails {
            if let Ok(msg) = self.parse_guerrilla_message(email) {
                result.push(msg);
            }
        }

        Ok(result)
    }

    fn parse_guerrilla_message(&self, data: &serde_json::Value) -> Result<EmailMessage> {
        Ok(EmailMessage {
            id: data["mail_id"].as_str().unwrap_or("").to_string(),
            from: data["mail_from"].as_str().unwrap_or("").to_string(),
            subject: data["mail_subject"].as_str().unwrap_or("").to_string(),
            body: data["mail_excerpt"].as_str().unwrap_or("").to_string(),
            received_at: data["mail_timestamp"].as_str().unwrap_or("").to_string(),
        })
    }

    async fn check_tempmail_emails(&self, temp_email: &TempEmail) -> Result<Vec<EmailMessage>> {
        let client = reqwest::Client::new();
        
        // 使用邮箱地址的哈希作为ID
        let email_hash = format!("{:x}", md5::compute(temp_email.address.as_bytes()));
        
        let response = client
            .get(&format!("https://api.temp-mail.org/request/mail/id/{}/", email_hash))
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        
        let mut result = Vec::new();
        if let Some(emails) = data.as_array() {
            for email in emails {
                if let Ok(msg) = self.parse_tempmail_message(email) {
                    result.push(msg);
                }
            }
        }

        Ok(result)
    }

    fn parse_tempmail_message(&self, data: &serde_json::Value) -> Result<EmailMessage> {
        Ok(EmailMessage {
            id: data["mail_id"].as_str().unwrap_or("").to_string(),
            from: data["mail_from"].as_str().unwrap_or("").to_string(),
            subject: data["mail_subject"].as_str().unwrap_or("").to_string(),
            body: data["mail_text"].as_str().unwrap_or("").to_string(),
            received_at: data["mail_timestamp"].as_str().unwrap_or("").to_string(),
        })
    }

    async fn check_10minutemail_emails(&self, _temp_email: &TempEmail) -> Result<Vec<EmailMessage>> {
        let client = reqwest::Client::new();
        
        let response = client
            .get("https://10minutemail.com/messages/messagesAfter/0")
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        
        let mut result = Vec::new();
        if let Some(emails) = data.as_array() {
            for email in emails {
                if let Ok(msg) = self.parse_10minutemail_message(email) {
                    result.push(msg);
                }
            }
        }

        Ok(result)
    }

    fn parse_10minutemail_message(&self, data: &serde_json::Value) -> Result<EmailMessage> {
        Ok(EmailMessage {
            id: data["id"].as_str().unwrap_or("").to_string(),
            from: data["from"].as_str().unwrap_or("").to_string(),
            subject: data["subject"].as_str().unwrap_or("").to_string(),
            body: data["bodyPlainText"].as_str().unwrap_or("").to_string(),
            received_at: data["receivedAt"].as_str().unwrap_or("").to_string(),
        })
    }

    async fn check_dropmail_emails(&self, temp_email: &TempEmail) -> Result<Vec<EmailMessage>> {
        let session_id = temp_email.token.as_ref().context("缺少 DropMail session ID")?;
        
        let client = reqwest::Client::new();
        
        let query = format!(
            "query {{session(id: \"{}\") {{mails {{edges {{node {{id from {{address}} subject text createdAt}}}}}}}}}}", 
            session_id
        );
        
        let response = client
            .post("https://dropmail.me/api/graphql/web-test-wgq6m5i")
            .json(&serde_json::json!({"query": query}))
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        
        let mut result = Vec::new();
        if let Some(edges) = data["data"]["session"]["mails"]["edges"].as_array() {
            for edge in edges {
                let node = &edge["node"];
                result.push(EmailMessage {
                    id: node["id"].as_str().unwrap_or("").to_string(),
                    from: node["from"]["address"].as_str().unwrap_or("").to_string(),
                    subject: node["subject"].as_str().unwrap_or("").to_string(),
                    body: node["text"].as_str().unwrap_or("").to_string(),
                    received_at: node["createdAt"].as_str().unwrap_or("").to_string(),
                });
            }
        }

        Ok(result)
    }

    async fn check_mailinator_emails(&self, temp_email: &TempEmail) -> Result<Vec<EmailMessage>> {
        // Mailinator 需要 API key 才能查询邮件
        // 如果配置了 API key，使用它；否则返回空列表
        let api_key = self.config.api_key.as_ref().context("Mailinator 需要 API key")?;
        
        let inbox_name = temp_email.address.split('@').next().unwrap_or("");
        
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("https://api.mailinator.com/v2/domains/public/inboxes/{}", inbox_name))
            .header("Authorization", api_key)
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        
        let mut result = Vec::new();
        if let Some(msgs) = data["msgs"].as_array() {
            for msg in msgs {
                // Safely extract body from parts array
                let body = msg["parts"]
                    .as_array()
                    .and_then(|parts| parts.get(0))
                    .and_then(|part| part["body"].as_str())
                    .unwrap_or("")
                    .to_string();
                
                result.push(EmailMessage {
                    id: msg["id"].as_str().unwrap_or("").to_string(),
                    from: msg["from"].as_str().unwrap_or("").to_string(),
                    subject: msg["subject"].as_str().unwrap_or("").to_string(),
                    body,
                    received_at: msg["time"].as_str().unwrap_or("").to_string(),
                });
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempEmail {
    pub address: String,
    pub password: Option<String>,
    pub token: Option<String>,
    pub provider: EmailProvider,
}

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub body: String,
    pub received_at: String,
}

/// 导出格式
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Txt,
}

/// 推荐的邮箱服务配置
pub fn get_recommended_providers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("mail.tm", "免费临时邮箱 API，稳定可靠，支持批量创建"),
        ("Guerrilla Mail", "老牌临时邮箱服务，速度快"),
        ("Temp-Mail.org", "简单易用的临时邮箱，无需注册"),
        ("10minutemail", "10分钟有效期临时邮箱，适合快速验证"),
        ("DropMail", "GraphQL API，功能强大"),
        ("Mailinator", "需要API key，但功能全面，适合企业使用"),
        ("自建 Postfix", "完全自主控制，最稳定，适合大规模批量"),
        ("MailCow", "开源邮件服务器套件，易于部署"),
        ("Zoho Mail", "商业邮箱，有免费版，适合长期使用"),
        ("Outlook", "Microsoft邮箱，需要配置IMAP/SMTP"),
        ("Gmail", "Google邮箱，需要配置IMAP/SMTP和应用密码"),
        ("Yahoo Mail", "雅虎邮箱，需要配置IMAP/SMTP"),
        ("ProtonMail", "注重隐私的邮箱，需要配置Bridge"),
    ]
}

/// 批量邮箱管理器
pub struct BatchEmailManager {
    provider_manager: EmailProviderManager,
    emails: Vec<TempEmail>,
}

impl BatchEmailManager {
    pub fn new(config: EmailProviderConfig) -> Self {
        BatchEmailManager {
            provider_manager: EmailProviderManager::new(config),
            emails: Vec::new(),
        }
    }

    /// 批量创建邮箱
    pub async fn create_batch(&mut self, count: usize, verify: bool) -> Result<usize> {
        let new_emails = self
            .provider_manager
            .batch_create_verified_emails(count, verify)
            .await?;

        let created_count = new_emails.len();
        self.emails.extend(new_emails);

        Ok(created_count)
    }

    /// 获取所有邮箱
    pub fn get_all_emails(&self) -> &[TempEmail] {
        &self.emails
    }

    /// 获取未使用的邮箱
    pub fn get_unused_email(&mut self) -> Option<TempEmail> {
        if self.emails.is_empty() {
            None
        } else {
            Some(self.emails.remove(0))
        }
    }

    /// 导出到文件
    pub fn export(&self, filename: &str, format: ExportFormat) -> Result<()> {
        self.provider_manager
            .export_emails_to_file(&self.emails, filename, format)
    }

    /// 从文件导入
    pub fn import(&mut self, filename: &str) -> Result<usize> {
        let imported = EmailProviderManager::import_emails_from_file(filename)?;
        let count = imported.len();
        self.emails.extend(imported);
        Ok(count)
    }

    /// 获取邮箱数量
    pub fn count(&self) -> usize {
        self.emails.len()
    }
}
