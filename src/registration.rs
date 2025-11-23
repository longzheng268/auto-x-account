//! X (Twitter) 账号注册模块
//! X (Twitter) account registration module

use anyhow::Result;
use chromiumoxide::browser::{Browser, BrowserConfig};
use fake::faker::name::raw::*;
use fake::locales::*;
use fake::{Fake, Faker};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::email::EmailHandler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub email: String,
    pub name: String,
    pub username: String,
    pub password: String,
    pub birth_date: BirthDate,
    pub created_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthDate {
    pub month: String,
    pub day: String,
    pub year: String,
}

pub struct XRegistration {
    config: Config,
    email_handler: EmailHandler,
}

impl XRegistration {
    pub fn new(config: Config, email_handler: EmailHandler) -> Self {
        XRegistration {
            config,
            email_handler,
        }
    }

    /// 获取 Chrome/Chromium 可执行文件路径
    /// Get Chrome/Chromium executable path
    /// 
    /// 优先级 / Priority:
    /// 1. 配置文件指定的路径 / Config specified path
    /// 2. 打包在程序旁边的 chromium 目录 / Bundled chromium directory
    /// 3. 系统安装的 Chromium/Chrome / System installed Chromium/Chrome
    fn get_chrome_executable_path(&self) -> Result<Option<PathBuf>> {
        // 1. 检查配置文件中指定的路径
        if let Some(config_path) = &self.config.browser.chrome_path {
            let path = PathBuf::from(config_path);
            if path.exists() {
                return Ok(Some(path));
            } else {
                warn!("配置的 Chrome 路径不存在: {}", config_path);
            }
        }

        // 2. 检查打包在程序旁边的 chromium 目录
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // 尝试不同的可能路径
                let bundled_paths = vec![
                    exe_dir.join("chromium").join("chrome"),           // Linux
                    exe_dir.join("chromium").join("chrome.exe"),       // Windows
                    exe_dir.join("chromium").join("Chromium.app").join("Contents").join("MacOS").join("Chromium"), // macOS
                    exe_dir.join("chrome-linux").join("chrome"),       // Linux (另一种命名)
                    exe_dir.join("chrome-win").join("chrome.exe"),     // Windows (另一种命名)
                ];

                for path in bundled_paths {
                    if path.exists() {
                        info!("找到打包的 Chromium: {}", path.display());
                        return Ok(Some(path));
                    }
                }
            }
        }

        // 3. 检查系统安装的 Chromium/Chrome
        let system_paths = if cfg!(target_os = "windows") {
            vec![
                PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files\Chromium\Application\chrome.exe"),
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
                PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium"),
            ]
        } else {
            // Linux
            vec![
                PathBuf::from("/usr/bin/chromium"),
                PathBuf::from("/usr/bin/chromium-browser"),
                PathBuf::from("/usr/bin/google-chrome"),
                PathBuf::from("/usr/bin/google-chrome-stable"),
                PathBuf::from("/snap/bin/chromium"),
            ]
        };

        for path in system_paths {
            if path.exists() {
                info!("使用系统安装的浏览器: {}", path.display());
                return Ok(Some(path));
            }
        }

        // 如果都没找到，返回 None，让 chromiumoxide 使用默认查找逻辑
        warn!("未找到 Chrome/Chromium 可执行文件，将使用默认查找方式");
        Ok(None)
    }

    /// 生成账号信息
    /// Generate account information
    fn generate_account_info(&self, email: String) -> AccountInfo {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let name: String = if self.config.language.starts_with("zh") {
            Name(ZH_CN).fake()
        } else {
            Name(EN).fake()
        };

        let username = format!(
            "user_{}_{}",
            Faker
                .fake::<String>()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .take(8)
                .collect::<String>(),
            chrono::Utc::now().timestamp()
        );

        let password = format!(
            "{}{}{}{}",
            Faker
                .fake::<String>()
                .chars()
                .filter(|c| c.is_alphabetic())
                .take(4)
                .collect::<String>(),
            rng.gen_range(1000..9999),
            Faker
                .fake::<String>()
                .chars()
                .filter(|c| c.is_uppercase())
                .take(2)
                .collect::<String>(),
            "!@"
        );

        let months = vec![
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];

        AccountInfo {
            email,
            name,
            username,
            password,
            birth_date: BirthDate {
                month: months[rng.gen_range(0..12)].to_string(),
                day: rng.gen_range(1..29).to_string(),
                year: rng.gen_range(1985..2003).to_string(),
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            status: "pending".to_string(),
        }
    }

    /// 注册账号
    /// Register account
    pub async fn register_account(&self, email: String) -> Result<AccountInfo> {
        let mut account_info = self.generate_account_info(email.clone());

        info!("开始注册账号: {}", account_info.email);

        // 配置浏览器
        let mut builder = BrowserConfig::builder();

        if !self.config.browser.headless {
            builder = builder.with_head();
        }

        // 设置 Chrome/Chromium 可执行文件路径
        // Set Chrome/Chromium executable path
        let chrome_path = self.get_chrome_executable_path()?;
        if let Some(path) = chrome_path {
            info!("使用 Chromium 路径: {}", path.display());
            builder = builder.chrome_executable(&path);
        }

        // 配置代理
        if let Some(proxy_url) = self.config.get_proxy_url(crate::config::ProxyTarget::Browser) {
            info!("浏览器使用代理 / Browser using proxy: {}", proxy_url);
            builder = builder.arg(format!("--proxy-server={}", proxy_url));
        }

        // 设置用户数据目录
        let user_data_dir = PathBuf::from(&self.config.browser.user_data_dir);
        std::fs::create_dir_all(&user_data_dir)?;
        builder = builder.user_data_dir(&user_data_dir);

        // 启动浏览器 / Launch browser
        let (mut browser, mut handler) = Browser::launch(
            builder.build().map_err(|e| anyhow::anyhow!("Browser configuration error / 浏览器配置错误: {}", e))?
        ).await?;

        // 处理浏览器事件
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    error!("浏览器事件错误: {}", e);
                }
            }
        });

        // 创建新页面
        let page = browser.new_page("about:blank").await?;

        // 注意: chromiumoxide 0.6 的 set_viewport API 可能不同
        // 如果需要设置视口大小，可以使用 CDP 协议直接设置
        // 这里暂时跳过，使用默认视口

        // 执行注册流程
        let result = self.perform_registration(&page, &mut account_info).await;

        // 清理
        if let Err(e) = browser.close().await {
            warn!("关闭浏览器时出错: {}", e);
        }

        result
    }

    async fn perform_registration(
        &self,
        page: &chromiumoxide::Page,
        account_info: &mut AccountInfo,
    ) -> Result<AccountInfo> {
        // 访问注册页面
        info!("访问注册页面: {}", self.config.x_account.base_url);
        page.goto(&self.config.x_account.base_url).await?;
        sleep(Duration::from_secs(3)).await;

        // 截图
        self.take_screenshot(page, "01_signup_page").await?;

        // 这里需要实现具体的注册步骤
        // 由于 X/Twitter 的注册流程可能会变化，这里提供一个框架

        info!("填写注册信息...");
        // TODO: 实现具体的表单填写逻辑

        info!("等待验证码...");
        let timeout = Duration::from_secs(self.config.x_account.email_wait_timeout);
        if let Some(code) = self
            .email_handler
            .wait_for_verification_code(&account_info.email, timeout)
            .await
        {
            info!("收到验证码: {}", code);
            // TODO: 输入验证码
        } else {
            anyhow::bail!("未收到验证码");
        }

        account_info.status = "registered".to_string();
        info!("账号注册成功: {}", account_info.username);

        Ok(account_info.clone())
    }

    async fn take_screenshot(&self, page: &chromiumoxide::Page, name: &str) -> Result<()> {
        let dir = PathBuf::from(&self.config.output.screenshots_dir);
        std::fs::create_dir_all(&dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.png", name, timestamp);
        let filepath = dir.join(filename);

        let screenshot = page
            .screenshot(chromiumoxide::page::ScreenshotParams::builder().build())
            .await?;
        std::fs::write(&filepath, screenshot)?;

        info!("已保存截图: {}", filepath.display());
        Ok(())
    }
}
