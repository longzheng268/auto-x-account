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
    /// Get proxy URL based on proxy mode
    pub fn get_proxy_url(&self) -> Option<String> {
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
    pub fn get_proxy_description(&self) -> String {
        match self.proxy.mode {
            ProxyMode::None => "不使用代理 / No Proxy".to_string(),
            ProxyMode::System => {
                if let Some(proxy) = self.get_proxy_url() {
                    format!("系统代理 / System Proxy: {}", proxy)
                } else {
                    "系统代理（未检测到）/ System Proxy (Not Detected)".to_string()
                }
            }
            ProxyMode::Manual => {
                format!(
                    "手动代理 / Manual Proxy: {}://{}:{}",
                    self.proxy.proxy_type, self.proxy.host, self.proxy.port
                )
            }
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
        }
    }
}
