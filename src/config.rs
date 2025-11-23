//! 配置管理模块
//! Configuration management module

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub language: String,
    pub smtp: SmtpConfig,
    pub proxy: ProxyConfig,
    pub browser: BrowserConfig,
    pub x_account: XAccountConfig,
    pub output: OutputConfig,
    pub ui: UiConfig,
    /// 运行模式：production（生产）或 test（测试）
    /// Running mode: production or test
    #[serde(default = "default_mode")]
    pub mode: RunMode,
    /// 邮箱提供商配置
    /// Email provider configuration
    #[serde(default)]
    pub email_provider: EmailProviderSettings,
    /// 人机验证配置
    /// Captcha configuration
    #[serde(default)]
    pub captcha: CaptchaConfig,
}

/// 运行模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RunMode {
    /// 生产模式 - 只能使用自建域名邮箱
    Production,
    /// 测试模式 - 可以使用临时邮箱
    Test,
}

fn default_mode() -> RunMode {
    RunMode::Test
}

/// 邮箱提供商设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailProviderSettings {
    /// 强制使用自建域名（生产模式下自动启用）
    #[serde(default)]
    pub force_self_hosted: bool,
    /// 允许的临时邮箱提供商列表（仅测试模式）
    #[serde(default)]
    pub allowed_temp_providers: Vec<String>,
    /// 生产域名列表（必须配置）
    #[serde(default)]
    pub production_domains: Vec<String>,
}

/// 人机验证配置
/// Captcha configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaConfig {
    /// 验证方式模式
    /// Captcha solving mode
    #[serde(default = "default_captcha_mode")]
    pub mode: CaptchaMode,
    /// 是否启用手动模式作为后备
    /// Enable manual mode as fallback
    #[serde(default = "default_true")]
    pub manual_fallback: bool,
    /// 2Captcha API Key
    #[serde(default)]
    pub two_captcha_api_key: Option<String>,
    /// Anti-Captcha API Key
    #[serde(default)]
    pub anti_captcha_api_key: Option<String>,
    /// CapMonster API Key
    #[serde(default)]
    pub capmonster_api_key: Option<String>,
    /// LLM API 配置（测试功能）
    /// LLM API configuration (test feature)
    #[serde(default)]
    pub llm_api: Option<LlmApiConfig>,
}

/// 人机验证模式
/// Captcha solving mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CaptchaMode {
    /// 自动解决 - 使用自研算法（默认，免费）
    /// Automatic - Use custom algorithms (default, free)
    Auto,
    /// 手动模式 - 等待用户手动完成
    /// Manual - Wait for user to complete manually
    Manual,
    /// 第三方服务 - 使用付费服务（2Captcha等）
    /// Third-party - Use paid services (2Captcha, etc.)
    ThirdParty,
    /// LLM API - 使用大模型 API（测试功能）
    /// LLM API - Use LLM API (test feature)
    #[serde(rename = "llm")]
    Llm,
}

fn default_captcha_mode() -> CaptchaMode {
    CaptchaMode::Auto
}

/// LLM API 配置
/// LLM API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmApiConfig {
    /// API 提供商（openai, anthropic, etc.）
    pub provider: String,
    /// API Key
    pub api_key: String,
    /// API 端点 URL（可选）
    pub endpoint: Option<String>,
    /// 模型名称
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub domain: String,
    pub enable: bool,
}

/// 代理模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyMode {
    /// 不使用代理
    None,
    /// 使用系统代理
    System,
    /// 手动配置代理
    Manual,
}

/// 代理目标
/// Proxy target
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyTarget {
    /// 浏览器代理
    Browser,
    /// 邮箱代理
    Email,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 代理模式: none, system, manual
    pub mode: ProxyMode,
    /// 代理类型 (http, https, socks5)
    #[serde(rename = "type")]
    pub proxy_type: String,
    /// 代理主机
    pub host: String,
    /// 代理端口
    pub port: u16,
    /// 用户名（可选）
    pub username: Option<String>,
    /// 密码（可选）
    pub password: Option<String>,
    /// 浏览器是否使用代理
    #[serde(default = "default_true")]
    pub browser_enabled: bool,
    /// 邮箱是否使用代理
    #[serde(default = "default_false")]
    pub email_enabled: bool,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub headless: bool,
    pub timeout: u64,
    pub viewport: ViewportConfig,
    pub user_data_dir: String,
    /// Chrome/Chromium 可执行文件路径（可选，为空则使用系统安装的）
    /// Path to Chrome/Chromium executable (optional, uses system installation if empty)
    #[serde(default)]
    pub chrome_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportConfig {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XAccountConfig {
    pub base_url: String,
    pub email_wait_timeout: u64,
    pub retry_times: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub screenshots_dir: String,
    pub logs_dir: String,
    pub accounts_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub font: String,
    pub font_path: String,
}

impl Config {
    /// 从文件加载配置
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    /// Save configuration to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 获取代理 URL
    /// Get proxy URL based on proxy mode and target
    pub fn get_proxy_url(&self, target: ProxyTarget) -> Option<String> {
        // 检查目标是否启用代理
        match target {
            ProxyTarget::Browser if !self.proxy.browser_enabled => return None,
            ProxyTarget::Email if !self.proxy.email_enabled => return None,
            _ => {}
        }

        match self.proxy.mode {
            ProxyMode::None => None,
            ProxyMode::System => {
                // 检测系统代理
                self.detect_system_proxy()
            }
            ProxyMode::Manual => {
                // 使用手动配置的代理
                let auth = match (&self.proxy.username, &self.proxy.password) {
                    (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => {
                        format!("{}:{}@", u, p)
                    }
                    _ => String::new(),
                };

                Some(format!(
                    "{}://{}{}:{}",
                    self.proxy.proxy_type, auth, self.proxy.host, self.proxy.port
                ))
            }
        }
    }

    /// 检测系统代理设置
    /// Detect system proxy settings
    fn detect_system_proxy(&self) -> Option<String> {
        // 检查环境变量
        // 按优先级检查: https_proxy > http_proxy > all_proxy

        // 1. 检查 HTTPS_PROXY / https_proxy
        if let Ok(proxy) = std::env::var("HTTPS_PROXY") {
            if !proxy.is_empty() {
                return Some(proxy);
            }
        }
        if let Ok(proxy) = std::env::var("https_proxy") {
            if !proxy.is_empty() {
                return Some(proxy);
            }
        }

        // 2. 检查 HTTP_PROXY / http_proxy
        if let Ok(proxy) = std::env::var("HTTP_PROXY") {
            if !proxy.is_empty() {
                return Some(proxy);
            }
        }
        if let Ok(proxy) = std::env::var("http_proxy") {
            if !proxy.is_empty() {
                return Some(proxy);
            }
        }

        // 3. 检查 ALL_PROXY / all_proxy
        if let Ok(proxy) = std::env::var("ALL_PROXY") {
            if !proxy.is_empty() {
                return Some(proxy);
            }
        }
        if let Ok(proxy) = std::env::var("all_proxy") {
            if !proxy.is_empty() {
                return Some(proxy);
            }
        }

        // 4. 尝试从系统设置读取（特定平台）
        #[cfg(target_os = "windows")]
        {
            if let Some(proxy) = self.detect_windows_proxy() {
                return Some(proxy);
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(proxy) = self.detect_macos_proxy() {
                return Some(proxy);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(proxy) = self.detect_linux_proxy() {
                return Some(proxy);
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_proxy(&self) -> Option<String> {
        use std::process::Command;

        // 尝试从 Windows 注册表读取代理设置
        // 这里简化处理，实际可以使用 winreg crate
        if let Ok(output) = Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyServer",
            ])
            .output()
        {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // 解析输出查找代理服务器地址
                for line in stdout.lines() {
                    if line.contains("ProxyServer") && line.contains("REG_SZ") {
                        if let Some(proxy) = line.split_whitespace().last() {
                            if !proxy.is_empty() && proxy.contains(':') {
                                return Some(format!("http://{}", proxy));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    #[cfg(target_os = "macos")]
    fn detect_macos_proxy(&self) -> Option<String> {
        use std::process::Command;

        // 使用 networksetup 命令获取代理设置
        if let Ok(output) = Command::new("networksetup")
            .args(&["-getwebproxy", "Wi-Fi"])
            .output()
        {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                let mut host = String::new();
                let mut port = String::new();
                let mut enabled = false;

                for line in stdout.lines() {
                    if line.starts_with("Enabled:") {
                        enabled = line.contains("Yes");
                    } else if line.starts_with("Server:") {
                        host = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Port:") {
                        port = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    }
                }

                if enabled && !host.is_empty() && !port.is_empty() {
                    return Some(format!("http://{}:{}", host, port));
                }
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    fn detect_linux_proxy(&self) -> Option<String> {
        use std::process::Command;

        // 尝试从 gsettings 读取（GNOME）
        if let Ok(output) = Command::new("gsettings")
            .args(&["get", "org.gnome.system.proxy", "mode"])
            .output()
        {
            if let Ok(mode) = String::from_utf8(output.stdout) {
                if mode.trim().contains("manual") {
                    // 读取 HTTP 代理
                    if let Ok(host_output) = Command::new("gsettings")
                        .args(&["get", "org.gnome.system.proxy.http", "host"])
                        .output()
                    {
                        if let Ok(port_output) = Command::new("gsettings")
                            .args(&["get", "org.gnome.system.proxy.http", "port"])
                            .output()
                        {
                            let host = String::from_utf8_lossy(&host_output.stdout)
                                .trim()
                                .trim_matches('\'')
                                .to_string();
                            let port = String::from_utf8_lossy(&port_output.stdout)
                                .trim()
                                .to_string();

                            if !host.is_empty() && !port.is_empty() {
                                return Some(format!("http://{}:{}", host, port));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// 获取代理描述（用于显示）
    /// Get proxy description for display
    pub fn get_proxy_description(&self, target: ProxyTarget) -> String {
        let target_str = match target {
            ProxyTarget::Browser => "浏览器 / Browser",
            ProxyTarget::Email => "邮箱 / Email",
        };

        let enabled = match target {
            ProxyTarget::Browser => self.proxy.browser_enabled,
            ProxyTarget::Email => self.proxy.email_enabled,
        };

        if !enabled {
            return format!("{}: 未启用代理 / Proxy disabled", target_str);
        }

        match self.proxy.mode {
            ProxyMode::None => format!("{}: 不使用代理 / No Proxy", target_str),
            ProxyMode::System => {
                if let Some(proxy) = self.get_proxy_url(target) {
                    format!("{}: 系统代理 / System Proxy: {}", target_str, proxy)
                } else {
                    format!(
                        "{}: 系统代理（未检测到）/ System Proxy (Not Detected)",
                        target_str
                    )
                }
            }
            ProxyMode::Manual => {
                format!(
                    "{}: 手动代理 / Manual Proxy: {}://{}:{}",
                    target_str, self.proxy.proxy_type, self.proxy.host, self.proxy.port
                )
            }
        }
    }

    /// 验证邮箱提供商是否允许使用
    /// Validate if email provider is allowed
    pub fn validate_email_provider(&self, provider: &crate::email_provider::EmailProvider) -> Result<()> {
        use crate::email_provider::EmailProvider;

        // 在生产模式下，只能使用自建域名
        if self.mode == RunMode::Production || self.email_provider.force_self_hosted {
            match provider {
                EmailProvider::SelfHosted | EmailProvider::Custom => {
                    // 检查域名是否在生产域名列表中
                    if self.email_provider.production_domains.is_empty() {
                        anyhow::bail!(
                            "❌ 生产模式错误：未配置生产域名列表\n\
                            Production mode error: No production domains configured\n\
                            请在配置文件中设置 email_provider.production_domains\n\
                            Please set email_provider.production_domains in config file"
                        );
                    }
                    Ok(())
                }
                _ => {
                    anyhow::bail!(
                        "❌ 生产模式限制：不允许使用临时邮箱服务\n\
                        Production mode restriction: Temporary email services are not allowed\n\
                        \n\
                        当前模式 / Current mode: {:?}\n\
                        当前提供商 / Current provider: {:?}\n\
                        \n\
                        解决方案 / Solutions:\n\
                        1. 切换到测试模式（在配置中设置 mode = \"test\"）\n\
                           Switch to test mode (set mode = \"test\" in config)\n\
                        2. 使用自建域名邮箱（设置 provider = \"selfhosted\"）\n\
                           Use self-hosted email (set provider = \"selfhosted\")\n\
                        3. 配置自定义 IMAP/SMTP（设置 provider = \"custom\"）\n\
                           Configure custom IMAP/SMTP (set provider = \"custom\")\n\
                        \n\
                        ⚠️  重要提示：生产环境必须使用自己的域名邮箱，临时邮箱仅用于测试！\n\
                        ⚠️  Important: Production must use your own domain emails, temp emails are for testing only!",
                        self.mode, provider
                    );
                }
            }
        } else {
            // 测试模式下，检查是否在允许列表中
            let provider_name = format!("{:?}", provider);
            if !self.email_provider.allowed_temp_providers.is_empty() {
                let allowed = self.email_provider.allowed_temp_providers.iter()
                    .any(|p| p.to_lowercase() == provider_name.to_lowercase());
                
                if !allowed && !matches!(provider, EmailProvider::SelfHosted | EmailProvider::Custom) {
                    anyhow::bail!(
                        "⚠️  测试模式警告：提供商 {:?} 不在允许列表中\n\
                        Test mode warning: Provider {:?} is not in allowed list\n\
                        允许的提供商 / Allowed providers: {:?}",
                        provider, provider, self.email_provider.allowed_temp_providers
                    );
                }
            }
            Ok(())
        }
    }

    /// 检查是否为生产模式
    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        self.mode == RunMode::Production || self.email_provider.force_self_hosted
    }

    /// 获取推荐的邮箱配置信息
    /// Get recommended email configuration
    pub fn get_email_config_recommendation(&self) -> String {
        if self.is_production() {
            format!(
                "🏭 生产模式 / Production Mode\n\
                \n\
                必须使用自建域名邮箱：\n\
                Must use self-hosted domain emails:\n\
                \n\
                配置的生产域名 / Configured production domains:\n\
                {}\n\
                \n\
                推荐配置 / Recommended setup:\n\
                1. Postfix + Dovecot (完全自主控制)\n\
                2. MailCow (开源邮件服务器套件)\n\
                3. iRedMail (开源邮件服务器解决方案)\n\
                4. 商业邮箱服务 (Zoho, Google Workspace, Microsoft 365)\n\
                \n\
                ⚠️  请确保域名已配置 SPF、DKIM、DMARC 记录以提高送达率\n\
                ⚠️  Please ensure SPF, DKIM, DMARC records are configured for better deliverability",
                self.email_provider.production_domains.join(", ")
            )
        } else {
            format!(
                "🧪 测试模式 / Test Mode\n\
                \n\
                可以使用临时邮箱进行测试：\n\
                Can use temporary emails for testing:\n\
                \n\
                允许的提供商 / Allowed providers:\n\
                {}\n\
                \n\
                ⚠️  警告：临时邮箱仅用于测试，不要用于生产环境！\n\
                ⚠️  Warning: Temporary emails are for testing only, do not use in production!\n\
                \n\
                切换到生产模式 / Switch to production mode:\n\
                在配置文件中设置: mode = \"production\"\n\
                Set in config file: mode = \"production\"",
                self.email_provider.allowed_temp_providers.join(", ")
            )
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            language: "zh-CN".to_string(),
            smtp: SmtpConfig {
                host: "0.0.0.0".to_string(),
                port: 8025,
                domain: "example.com".to_string(),
                enable: true,
            },
            proxy: ProxyConfig {
                mode: ProxyMode::None,
                proxy_type: "socks5".to_string(),
                host: "127.0.0.1".to_string(),
                port: 1080,
                username: None,
                password: None,
                browser_enabled: true,
                email_enabled: false,
            },
            browser: BrowserConfig {
                headless: false,
                timeout: 30000,
                viewport: ViewportConfig {
                    width: 1280,
                    height: 720,
                },
                user_data_dir: "browser_data".to_string(),
                chrome_path: None,
            },
            x_account: XAccountConfig {
                base_url: "https://twitter.com/i/flow/signup".to_string(),
                email_wait_timeout: 120,
                retry_times: 3,
            },
            output: OutputConfig {
                screenshots_dir: "screenshots".to_string(),
                logs_dir: "logs".to_string(),
                accounts_file: "accounts.json".to_string(),
            },
            ui: UiConfig {
                font: "MiSans".to_string(),
                font_path: "fonts/MiSans-Regular.ttf".to_string(),
            },
            mode: RunMode::Test,
            email_provider: EmailProviderSettings::default(),
            captcha: CaptchaConfig::default(),
        }
    }
}

impl Default for EmailProviderSettings {
    fn default() -> Self {
        EmailProviderSettings {
            force_self_hosted: false,
            allowed_temp_providers: vec![
                "MailTm".to_string(),
                "GuerrillaMail".to_string(),
            ],
            production_domains: vec!["example.com".to_string()],
        }
    }
}

impl Default for CaptchaConfig {
    fn default() -> Self {
        CaptchaConfig {
            mode: CaptchaMode::Auto,
            manual_fallback: true,
            two_captcha_api_key: None,
            anti_captcha_api_key: None,
            capmonster_api_key: None,
            llm_api: None,
        }
    }
}
