//! SMTP 邮件服务模块
//! SMTP email service module

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration, Instant};
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct Email {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct EmailHandler {
    emails: Arc<Mutex<Vec<Email>>>,
    verification_codes: Arc<Mutex<HashMap<String, String>>>,
}

impl EmailHandler {
    pub fn new() -> Self {
        EmailHandler {
            emails: Arc::new(Mutex::new(Vec::new())),
            verification_codes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn handle_email(&self, from: String, to: String, subject: String, body: String) {
        let email = Email {
            from: from.clone(),
            to: to.clone(),
            subject: subject.clone(),
            body: body.clone(),
            timestamp: chrono::Utc::now(),
        };

        info!("收到邮件 from: {}, to: {}, subject: {}", from, to, subject);

        // 提取验证码
        if let Some(code) = extract_verification_code(&subject, &body) {
            info!("提取到验证码: {} for {}", code, to);
            let mut codes = self.verification_codes.lock().unwrap();
            codes.insert(to.clone(), code);
        }

        let mut emails = self.emails.lock().unwrap();
        emails.push(email);
    }

    pub async fn wait_for_verification_code(
        &self,
        email_address: &str,
        timeout: Duration,
    ) -> Option<String> {
        let start = Instant::now();

        loop {
            {
                let codes = self.verification_codes.lock().unwrap();
                if let Some(code) = codes.get(email_address) {
                    return Some(code.clone());
                }
            }

            if start.elapsed() > timeout {
                warn!("等待验证码超时: {}", email_address);
                return None;
            }

            sleep(Duration::from_secs(2)).await;
        }
    }

    pub fn clear(&self) {
        let mut emails = self.emails.lock().unwrap();
        emails.clear();
        let mut codes = self.verification_codes.lock().unwrap();
        codes.clear();
    }
}

/// 从邮件内容中提取验证码
/// Extract verification code from email content
fn extract_verification_code(subject: &str, body: &str) -> Option<String> {
    let text = format!("{} {}", subject, body);

    // 常见的验证码模式
    let patterns = [
        regex::Regex::new(r"verification code[:\s]+([A-Z0-9]{6,8})").ok()?,
        regex::Regex::new(r"code[:\s]+([A-Z0-9]{6,8})").ok()?,
        regex::Regex::new(r"\b([A-Z0-9]{6,8})\b.*verify").ok()?,
        regex::Regex::new(r"confirm.*code[:\s]+([A-Z0-9]{6,8})").ok()?,
        regex::Regex::new(r"验证码[:\s：]+([A-Z0-9]{6,8})").ok()?,
        regex::Regex::new(r"([0-9]{6,8})").ok()?,
    ];

    for pattern in &patterns {
        if let Some(caps) = pattern.captures(&text) {
            if let Some(code) = caps.get(1) {
                let code_str = code.as_str();
                if code_str.len() >= 6 && code_str.len() <= 8 {
                    return Some(code_str.to_string());
                }
            }
        }
    }

    None
}

pub struct EmailService {
    handler: EmailHandler,
    config: crate::config::SmtpConfig,
}

impl EmailService {
    pub fn new(config: crate::config::SmtpConfig) -> Self {
        EmailService {
            handler: EmailHandler::new(),
            config,
        }
    }

    pub async fn start(&self) -> Result<()> {
        if !self.config.enable {
            info!("SMTP 服务未启用");
            return Ok(());
        }

        info!(
            "SMTP 服务器启动在 {}:{}",
            self.config.host, self.config.port
        );

        // 注意：这里简化了 SMTP 服务器的实现
        // 实际部署时需要使用完整的 SMTP 服务器库
        // 或者使用外部的 SMTP 服务配合 IMAP/POP3 来接收邮件

        Ok(())
    }

    pub fn get_handler(&self) -> EmailHandler {
        self.handler.clone()
    }
}
