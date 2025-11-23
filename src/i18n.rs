//! 多语言支持模块
//! Multi-language support module

use std::collections::HashMap;

pub struct I18n {
    language: String,
    messages: HashMap<String, HashMap<String, String>>,
}

impl I18n {
    pub fn new(language: &str) -> Self {
        let mut messages = HashMap::new();
        
        // 中文
        let mut zh_cn = HashMap::new();
        zh_cn.insert("app_name".to_string(), "X账号自动注册系统".to_string());
        zh_cn.insert("starting".to_string(), "正在启动...".to_string());
        zh_cn.insert("smtp_starting".to_string(), "正在启动 SMTP 服务器...".to_string());
        zh_cn.insert("smtp_started".to_string(), "SMTP 服务器已启动".to_string());
        zh_cn.insert("smtp_stopped".to_string(), "SMTP 服务器已停止".to_string());
        zh_cn.insert("browser_init".to_string(), "正在初始化浏览器...".to_string());
        zh_cn.insert("browser_ready".to_string(), "浏览器已准备就绪".to_string());
        zh_cn.insert("browser_closed".to_string(), "浏览器已关闭".to_string());
        zh_cn.insert("registration_start".to_string(), "开始注册账号".to_string());
        zh_cn.insert("registration_success".to_string(), "账号注册成功".to_string());
        zh_cn.insert("registration_failed".to_string(), "账号注册失败".to_string());
        zh_cn.insert("waiting_verification".to_string(), "等待验证码...".to_string());
        zh_cn.insert("verification_received".to_string(), "已收到验证码".to_string());
        zh_cn.insert("proxy_enabled".to_string(), "代理已启用".to_string());
        zh_cn.insert("proxy_disabled".to_string(), "不使用代理".to_string());
        zh_cn.insert("proxy_system".to_string(), "使用系统代理".to_string());
        zh_cn.insert("proxy_manual".to_string(), "使用手动代理".to_string());
        zh_cn.insert("error".to_string(), "错误".to_string());
        zh_cn.insert("success".to_string(), "成功".to_string());
        messages.insert("zh-CN".to_string(), zh_cn);
        
        // 英文
        let mut en_us = HashMap::new();
        en_us.insert("app_name".to_string(), "X Account Auto Registration System".to_string());
        en_us.insert("starting".to_string(), "Starting...".to_string());
        en_us.insert("smtp_starting".to_string(), "Starting SMTP server...".to_string());
        en_us.insert("smtp_started".to_string(), "SMTP server started".to_string());
        en_us.insert("smtp_stopped".to_string(), "SMTP server stopped".to_string());
        en_us.insert("browser_init".to_string(), "Initializing browser...".to_string());
        en_us.insert("browser_ready".to_string(), "Browser is ready".to_string());
        en_us.insert("browser_closed".to_string(), "Browser closed".to_string());
        en_us.insert("registration_start".to_string(), "Starting account registration".to_string());
        en_us.insert("registration_success".to_string(), "Account registered successfully".to_string());
        en_us.insert("registration_failed".to_string(), "Account registration failed".to_string());
        en_us.insert("waiting_verification".to_string(), "Waiting for verification code...".to_string());
        en_us.insert("verification_received".to_string(), "Verification code received".to_string());
        en_us.insert("proxy_enabled".to_string(), "Proxy enabled".to_string());
        en_us.insert("proxy_disabled".to_string(), "No proxy".to_string());
        en_us.insert("proxy_system".to_string(), "Using system proxy".to_string());
        en_us.insert("proxy_manual".to_string(), "Using manual proxy".to_string());
        en_us.insert("error".to_string(), "Error".to_string());
        en_us.insert("success".to_string(), "Success".to_string());
        messages.insert("en-US".to_string(), en_us);
        
        I18n {
            language: language.to_string(),
            messages,
        }
    }

    pub fn t(&self, key: &str) -> String {
        self.messages
            .get(&self.language)
            .and_then(|lang_map| lang_map.get(key))
            .cloned()
            .unwrap_or_else(|| {
                // 回退到英文
                self.messages
                    .get("en-US")
                    .and_then(|lang_map| lang_map.get(key))
                    .cloned()
                    .unwrap_or_else(|| key.to_string())
            })
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }
}
