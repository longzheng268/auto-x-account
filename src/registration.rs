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
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::email::EmailHandler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub email: String,
    pub name: String,
    pub username: String,
    pub password: String,
    pub phone: Option<String>,
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
                // Microsoft Edge (优先，Windows 内置)
                PathBuf::from(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
                PathBuf::from(r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
                // Google Chrome
                PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
                // Chromium
                PathBuf::from(r"C:\Program Files\Chromium\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files (x86)\Chromium\Application\chrome.exe"),
                // Brave
                PathBuf::from(r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe"),
                PathBuf::from(r"C:\Program Files (x86)\BraveSoftware\Brave-Browser\Application\brave.exe"),
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
            phone: None,
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

        info!("🚀 开始注册账号 / Starting account registration");
        info!("   邮箱 / Email: {}", account_info.email);
        info!("   用户名 / Username: {}", account_info.username);
        info!("   生日 / Birthday: {}/{}/{}", 
            account_info.birth_date.month, 
            account_info.birth_date.day, 
            account_info.birth_date.year
        );

        // 配置浏览器
        info!("⚙️  配置浏览器 / Configuring browser...");
        let mut builder = BrowserConfig::builder();

        if !self.config.browser.headless {
            builder = builder.with_head();
            info!("   模式 / Mode: 有界面 / With UI");
        } else {
            info!("   模式 / Mode: 无界面 / Headless");
        }

        // 设置 Chrome/Chromium 可执行文件路径
        // Set Chrome/Chromium executable path
        let chrome_path = self.get_chrome_executable_path()?;
        if let Some(path) = chrome_path {
            info!("   浏览器路径 / Browser path: {}", path.display());
            builder = builder.chrome_executable(&path);
        } else {
            info!("   浏览器路径 / Browser path: 使用系统默认 / Using system default");
        }

        // 配置代理
        if let Some(proxy_url) = self.config.get_proxy_url(crate::config::ProxyTarget::Browser) {
            info!("   代理 / Proxy: {}", proxy_url);
            builder = builder.arg(format!("--proxy-server={}", proxy_url));
        } else {
            info!("   代理 / Proxy: 不使用 / Not using");
        }

        // 强制使用中文环境
        info!("   语言 / Language: 简体中文 / Simplified Chinese (zh-CN)");
        builder = builder
            .arg("--lang=zh-CN")
            .arg("--accept-lang=zh-CN,zh")
            .arg("--disable-blink-features=AutomationControlled")
            // 增加稳定性参数
            .arg("--no-sandbox")
            .arg("--disable-gpu")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-software-rasterizer")
            .arg("--disable-software-rasterizer");

        // 设置固定的用户数据目录，避免权限问题
        let temp_dir = std::env::temp_dir().join("auto-x-account-browser-data");
        // 每次启动前清理旧数据，确保干净的环境
        if temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
        }
        std::fs::create_dir_all(&temp_dir)?;
        builder = builder.user_data_dir(&temp_dir);
        info!("   数据目录 / Data directory: {}", temp_dir.display());

        // 启动浏览器 / Launch browser
        info!("🌐 启动浏览器 / Launching browser...");
        
        // 构建配置
        let config = builder.build()
            .map_err(|e| anyhow::anyhow!("Browser configuration error / 浏览器配置错误: {}", e))?;
            
        let (mut browser, mut handler) = Browser::launch(config).await?;
        info!("✅ 浏览器已启动 / Browser launched successfully");

        // 处理浏览器事件
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    error!("浏览器事件错误 / Browser event error: {}", e);
                }
            }
        });

        // 创建新页面
        info!("📄 创建新页面 / Creating new page...");
        let page = browser.new_page("about:blank").await?;

        // 注意: chromiumoxide 0.6 的 set_viewport API 可能不同
        // 如果需要设置视口大小，可以使用 CDP 协议直接设置
        // 这里暂时跳过，使用默认视口

        // 执行注册流程
        info!("📝 执行注册流程 / Performing registration...");
        let result = self.perform_registration(&page, &mut account_info).await;

        // 清理
        info!("🧹 清理资源 / Cleaning up resources...");
        if let Err(e) = browser.close().await {
            warn!("关闭浏览器时出错 / Error closing browser: {}", e);
        }

        match &result {
            Ok(acc) => {
                info!("✅ 账号注册成功 / Account registered successfully: {}", acc.username);
            }
            Err(e) => {
                error!("❌ 账号注册失败 / Account registration failed: {}", e);
            }
        }

        result
    }

    /// 检查并处理 403 错误 (循环检查直到恢复)
    async fn check_and_handle_403(&self, page: &chromiumoxide::Page) -> Result<()> {
        let max_retries = 10; // 增加重试次数
        for i in 0..max_retries {
            let check_result = page.evaluate(r#"
                (function() {
                    // 1. 检查标题
                    if (document.title.includes('403') || document.title.includes('Forbidden')) return true;
                    
                    // 2. 检查页面内容的关键短语
                    const text = document.body.innerText;
                    if (text.includes('403 Forbidden') || 
                        text.includes('Access Denied') || 
                        text.includes('Please reload') ||
                        text.includes('Something went wrong, but don’t fret') ||
                        text.includes('出错啦，但别担心')) return true;

                    // 3. 检查重试按钮 (通常是蓝色的 "Retry" 或 "重试")
                    const buttons = document.querySelectorAll('div[role="button"], button');
                    for (let btn of buttons) {
                        const btnText = (btn.innerText || '').trim();
                        if (btnText === 'Retry' || btnText === '重试') {
                            return true;
                        }
                    }
                        
                    return false;
                })()
            "#).await;

            if let Ok(val) = check_result {
                if val.into_value::<bool>().unwrap_or(false) {
                    warn!("⚠️  检测到 403/错误页面 (尝试 {}/{})，正在处理... / 403/Error detected, handling...", i + 1, max_retries);
                    
                    // 尝试点击重试按钮
                    let clicked_retry = page.evaluate(r#"
                        (function() {
                            const buttons = document.querySelectorAll('div[role="button"], button');
                            for (let btn of buttons) {
                                const btnText = (btn.innerText || '').trim();
                                if (btnText === 'Retry' || btnText === '重试') {
                                    btn.click();
                                    return true;
                                }
                            }
                            return false;
                        })()
                    "#).await;
                    
                    if let Ok(clicked) = clicked_retry {
                        if clicked.into_value::<bool>().unwrap_or(false) {
                            info!("   👉 已点击重试按钮 / Clicked Retry button");
                            sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                    }

                    // 如果没找到按钮或点击无效，则刷新页面
                    info!("   🔄 刷新页面 / Refreshing page");
                    page.reload().await?;
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }
            
            // 如果没有检测到错误，且不是第一次检查（说明之前可能有错误但已恢复），或者第一次就没错误
            if i > 0 {
                info!("✅ 页面已恢复正常 / Page recovered");
            }
            return Ok(());
        }
        
        warn!("⚠️  多次尝试后页面仍可能有问题 / Page might still be broken after retries");
        Ok(())
    }

    async fn perform_registration(
        &self,
        page: &chromiumoxide::Page,
        account_info: &mut AccountInfo,
    ) -> Result<AccountInfo> {
        // 访问注册页面 (增加重试机制)
        info!("🔗 访问注册页面 / Navigating to registration page");
        info!("   URL: {}", self.config.x_account.base_url);
        
        let mut attempts = 0;
        let max_attempts = 3;
        
        while attempts < max_attempts {
            match page.goto(&self.config.x_account.base_url).await {
                Ok(_) => {
                    info!("✅ 页面加载指令已发送 / Navigation command sent");
                    break;
                },
                Err(e) => {
                    attempts += 1;
                    warn!("⚠️  页面跳转失败 (尝试 {}/{}) / Navigation failed: {}", attempts, max_attempts, e);
                    if attempts == max_attempts {
                        anyhow::bail!("无法访问注册页面 / Could not navigate to registration page: {}", e);
                    }
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
        
        // 等待页面加载完成
        sleep(Duration::from_secs(5)).await;
        
        // 检查 403
        self.check_and_handle_403(page).await?;
        
        // 检查当前URL确认是否跳转成功
        let current_url = page.url().await.unwrap_or(None).unwrap_or_default();
        info!("   当前 URL / Current URL: {}", current_url);
        
        if !current_url.contains("twitter.com") && !current_url.contains("x.com") {
             warn!("⚠️  似乎未成功跳转到 X/Twitter，尝试重新跳转...");
             page.goto(&self.config.x_account.base_url).await?;
             sleep(Duration::from_secs(5)).await;
        }

        info!("✅ 页面加载完成 / Page loaded");
        self.take_screenshot(page, "01_registration_page").await?;

        // 步骤1: 使用图像识别点击"创建账号"按钮
        info!("🔘 步骤1: 点击创建账号按钮 / Step 1: Click Sign up button");
        if let Err(e) = self.click_signup_button_by_image(page).await {
            warn!("图像识别点击失败，尝试选择器方式 / Image recognition failed, trying selectors: {}", e);
        }
        sleep(Duration::from_secs(2)).await;

        // 步骤2: 切换到邮箱注册
        info!("📧 步骤2: 切换到邮箱注册 / Step 2: Switch to email registration");
        if let Err(e) = self.switch_to_email_registration(page).await {
            warn!("切换到邮箱注册失败，尝试继续 / Failed to switch to email: {}", e);
        }
        sleep(Duration::from_secs(2)).await;

        // 步骤2: 填写姓名
        info!("✏️  步骤2: 填写姓名 / Step 2: Fill in name");
        info!("   姓名 / Name: {}", account_info.name);
        self.fill_name(page, &account_info.name).await?;
        sleep(Duration::from_secs(1)).await;

        // 步骤3: 填写邮箱
        info!("✏️  步骤3: 填写邮箱 / Step 3: Fill in email");
        info!("   邮箱 / Email: {}", account_info.email);
        self.fill_email(page, &account_info.email).await?;
        sleep(Duration::from_secs(1)).await;

        // 步骤4: 填写出生日期
        info!("✏️  步骤4: 填写出生日期 / Step 4: Fill in date of birth");
        info!("   出生日期 / DOB: {}/{}/{}", 
            account_info.birth_date.month, 
            account_info.birth_date.day, 
            account_info.birth_date.year
        );
        self.fill_birth_date(page, &account_info.birth_date).await?;
        sleep(Duration::from_secs(1)).await;
        self.take_screenshot(page, "03_basic_info_filled").await?;

        // 点击下一步/继续按钮
        info!("➡️  点击下一步 / Click Next");
        self.click_next_button(page).await?;
        sleep(Duration::from_secs(3)).await;
        self.take_screenshot(page, "04_after_basic_info").await?;

        // 步骤5: 处理人机验证（如果出现）
        info!("🤖 步骤5: 检查人机验证 / Step 5: Check for captcha");
        if self.has_captcha(page).await {
            info!("⚠️  检测到人机验证 / Captcha detected");
            self.take_screenshot(page, "05_captcha_detected").await?;
            
            info!("⏳ 等待手动完成人机验证 / Waiting for manual captcha completion");
            
            // 等待验证完成（最多3分钟）
            let captcha_timeout = Duration::from_secs(180);
            let start = tokio::time::Instant::now();
            while start.elapsed() < captcha_timeout && self.has_captcha(page).await {
                sleep(Duration::from_secs(2)).await;
            }
            
            if self.has_captcha(page).await {
                error!("❌ 人机验证超时 / Captcha timeout");
                anyhow::bail!("人机验证超时 / Captcha verification timeout");
            }
            
            info!("✅ 人机验证完成 / Captcha completed");
            sleep(Duration::from_secs(2)).await;
        } else {
            info!("✅ 未检测到人机验证，IP环境良好 / No captcha detected, clean IP");
        }
        
        self.take_screenshot(page, "06_after_captcha").await?;

        // 可能需要再次点击下一步
        if let Ok(_) = self.click_next_button(page).await {
            sleep(Duration::from_secs(3)).await;
        }

        // 步骤6: 等待并输入邮箱验证码
        info!("📬 步骤6: 等待邮箱验证码 / Step 6: Waiting for email verification code");
        let email_timeout = Duration::from_secs(self.config.x_account.email_wait_timeout);
        info!("   超时时间 / Timeout: {} 秒 / seconds", self.config.x_account.email_wait_timeout);
        
        if let Some(code) = self
            .email_handler
            .wait_for_verification_code(&account_info.email, email_timeout)
            .await
        {
            info!("✅ 收到验证码 / Received verification code: {}", code);
            self.take_screenshot(page, "07_verification_code_page").await?;
            
            info!("🔢 输入验证码 / Entering verification code");
            self.fill_verification_code(page, &code).await?;
            sleep(Duration::from_secs(2)).await;
            
            info!("➡️  点击 Next / Click Next");
            self.click_next_button(page).await?;
            sleep(Duration::from_secs(3)).await;
            info!("✅ 验证码已确认 / Verification code confirmed");
            self.take_screenshot(page, "08_after_verification").await?;
        } else {
            error!("❌ 未收到验证码 / Verification code not received");
            self.take_screenshot(page, "07_verification_timeout").await?;
            anyhow::bail!("未收到验证码 / Verification code not received");
        }

        // 步骤7: 设置密码
        info!("🔐 步骤7: 设置密码 / Step 7: Set password");
        info!("   密码 / Password: {}", account_info.password);
        self.fill_password(page, &account_info.password).await?;
        sleep(Duration::from_secs(2)).await;
        self.take_screenshot(page, "09_password_set").await?;

        // 点击下一步
        info!("➡️  点击 Next / Click Next");
        self.click_next_button(page).await?;
        sleep(Duration::from_secs(3)).await;
        self.take_screenshot(page, "10_after_password").await?;

        // 步骤8: 点击 Get Started
        info!("🚀 步骤8: 点击 Get Started / Step 8: Click Get Started");
        if let Ok(_) = self.click_get_started(page).await {
            sleep(Duration::from_secs(3)).await;
            info!("✅ Get Started 已点击 / Get Started clicked");
            self.take_screenshot(page, "11_get_started").await?;
        } else {
            warn!("⚠️  未找到 Get Started 按钮，可能已进入下一步");
        }

        // 步骤9: 跳过所有可选步骤
        info!("⏭️  步骤9: 跳过可选步骤 / Step 9: Skip optional steps");
        for i in 1..=5 {
            if self.click_skip_button(page).await.is_ok() {
                info!("   跳过步骤 {} / Skipped step {}", i, i);
                sleep(Duration::from_secs(2)).await;
                self.take_screenshot(page, &format!("12_skip_{}", i)).await?;
            } else {
                info!("   没有更多跳过选项 / No more skip options");
                break;
            }
        }

        account_info.status = "registered".to_string();
        info!("🎉 账号注册完成 / Account registration completed");
        info!("   用户名 / Username: {}", account_info.username);
        info!("   邮箱 / Email: {}", account_info.email);
        self.take_screenshot(page, "13_registration_complete").await?;

        Ok(account_info.clone())
    }

    /// 切换到邮箱注册模式
    async fn switch_to_email_registration(&self, page: &chromiumoxide::Page) -> Result<()> {
        info!("   查找'改用电子邮件'按钮 / Looking for 'Use email' button");
        
        let js_script = r#"
        (function() {
            // 查找包含特定文本的元素
            const targets = ['改用电子邮件', 'Use email instead', '改用电子邮箱'];
            // 扩大查找范围
            const elements = document.querySelectorAll('span, div[role="button"], button, a');
            
            for (let el of elements) {
                const text = (el.innerText || '').trim();
                if (targets.includes(text)) {
                    console.log('找到切换按钮:', text);
                    el.click();
                    return true;
                }
            }
            return false;
        })()
        "#;
        
        for i in 0..5 {
            if i > 0 {
                sleep(Duration::from_secs(1)).await;
            }
            
            match page.evaluate(js_script).await {
                Ok(val) => {
                    if val.into_value::<bool>()? {
                        info!("   ✅ 通过 JS 点击了切换按钮 / Clicked switch button via JS");
                        return Ok(());
                    }
                },
                Err(e) => warn!("   JS 切换失败: {} / JS switch failed: {}", e, e),
            }
        }
        
        warn!("   未找到切换按钮 (可能已是邮箱模式) / Switch button not found");
        Ok(())
    }

    /// 填写姓名
    async fn fill_name(&self, page: &chromiumoxide::Page, name: &str) -> Result<()> {
        info!("   正在查找姓名输入框 / Looking for name input...");

        // 方法1: 通过 JS 智能查找并聚焦
        let found = page.evaluate(r#"
            (function() {
                const nameInput = document.querySelector('input[name="name"]');
                if (nameInput) {
                    nameInput.focus();
                    return true;
                }
                const autoInput = document.querySelector('input[autocomplete="name"]');
                if (autoInput) {
                    autoInput.focus();
                    return true;
                }
                const inputs = document.querySelectorAll('input');
                for (const input of inputs) {
                    const attr = (input.getAttribute('name') || input.getAttribute('autocomplete') || input.placeholder || '').toLowerCase();
                    if (attr.includes('name') || attr.includes('名字') || attr.includes('姓名')) {
                        input.focus();
                        return true;
                    }
                }
                return false;
            })()
        "#).await;

        match found {
            Ok(val) => {
                if val.into_value::<bool>()? {
                    info!("   ✅ 通过 JS 找到并聚焦姓名输入框 / Found and focused name input via JS");
                    sleep(Duration::from_millis(500)).await;
                    
                    let js_value = serde_json::to_string(name)?;
                    let script = format!(r#"
                        (function() {{
                            const element = document.activeElement;
                            if (!element) return false;
                            const value = {};
                            
                            const valueSetter = Object.getOwnPropertyDescriptor(element, 'value').set;
                            const prototype = Object.getPrototypeOf(element);
                            const prototypeValueSetter = Object.getOwnPropertyDescriptor(prototype, 'value').set;
                            
                            if (valueSetter && valueSetter !== prototypeValueSetter) {{
                                prototypeValueSetter.call(element, value);
                            }} else {{
                                valueSetter.call(element, value);
                            }}
                            
                            element.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            element.dispatchEvent(new Event('change', {{ bubbles: true }}));
                            return true;
                        }})()
                    "#, js_value);
                    
                    page.evaluate(script).await?;
                    info!("   ✅ 姓名已填写 / Name filled");
                    return Ok(());
                }
            },
            Err(e) => warn!("   JS 查找失败: {} / JS search failed: {}", e, e),
        }

        // 方法2: 传统的选择器回退
        let selectors = vec![
            "input[name='name']",
            "input[autocomplete='name']",
            "input[placeholder*='Name']",
            "input[placeholder*='名字']",
        ];

        for selector in selectors {
            if let Ok(element) = page.find_element(selector).await {
                element.click().await?;
                element.type_str(name).await?;
                info!("   ✅ 姓名已填写 (选择器) / Name filled (selector)");
                return Ok(());
            }
        }

        anyhow::bail!("未找到姓名输入框 / Name input not found")
    }

    /// 使用图像识别点击创建账号按钮
    async fn click_signup_button_by_image(&self, page: &chromiumoxide::Page) -> Result<()> {
        info!("   方法1: 尝试通过JavaScript查找按钮 (文本+样式) / Method 1: Trying JavaScript search (text+style)");
        
        let js_script = r#"
        (function() {
            const buttons = document.querySelectorAll('button, div[role="button"], a[role="button"]');
            for (let btn of buttons) {
                const text = (btn.innerText || btn.textContent || '').trim();
                if (text.includes('创建账号') || text.includes('创建帐号') || text.includes('创建帳號')) {
                    const style = window.getComputedStyle(btn);
                    const bgColor = style.backgroundColor;
                    if (bgColor.includes('0, 0, 0') || bgColor === 'black' || bgColor === '#000000') {
                         console.log('找到黑色创建账号按钮!', text);
                         btn.click();
                         return true;
                    }
                    console.log('找到创建账号按钮(非纯黑):', text);
                    btn.click();
                    return true;
                }
            }
            return false;
        })()
        "#;
        
        // 增加重试机制
        for i in 0..5 {
            if i > 0 {
                // 在重试前检查是否是 403
                if let Err(e) = self.check_and_handle_403(page).await {
                    warn!("   检查 403 失败: {} / Failed to check 403: {}", e, e);
                }

                sleep(Duration::from_secs(2)).await;
                info!("   重试查找按钮 ({}/5) / Retrying button search ({}/5)", i + 1, 5);
            }
            
            match page.evaluate(js_script).await {
                Ok(val) => {
                    if val.into_value::<bool>()? {
                        info!("   ✅ 通过JavaScript成功点击按钮 / Successfully clicked via JavaScript");
                        return Ok(());
                    }
                },
                Err(e) => {
                     debug!("   JS 执行出错 (可能页面正在加载): {} / JS execution error: {}", e, e);
                }
            }
        }
        
        info!("   方法2: 使用图像识别 / Method 2: Using image recognition");
        
        let screenshot = page
            .screenshot(chromiumoxide::page::ScreenshotParams::builder().build())
            .await?;
        
        let screenshots_dir = crate::data_dir::get_screenshots_dir();
        let debug_screenshot_path = screenshots_dir.join("debug_before_click.png");
        std::fs::write(&debug_screenshot_path, &screenshot)?;
        info!("   📸 调试截图已保存 / Debug screenshot saved: {}", debug_screenshot_path.display());
        
        let template_path = std::env::current_dir()?
            .join("src")
            .join("resources")
            .join("images")
            .join("create_account_btn.png");
        
        if !template_path.exists() {
            anyhow::bail!("模板图片不存在 / Template image not found: {}", template_path.display());
        }
        
        if let Some((x, y)) = crate::visual::find_button_by_template(
            &screenshot, 
            template_path.to_str().unwrap(), 
            Some(0.5)
        )? {
            info!("   ✅ 通过图像识别找到按钮 / Found button via image recognition");
            crate::visual::click_at_coordinates(page, x, y).await?;
            info!("   ✅ 已点击创建账号按钮 / Clicked sign up button");
            return Ok(());
        }
        
        anyhow::bail!("所有方法都失败 / All methods failed")
    }

    /// 填写邮箱
    async fn fill_email(&self, page: &chromiumoxide::Page, email: &str) -> Result<()> {
        info!("   正在查找邮箱输入框 / Looking for email input...");

        // 方法1: 通过 JS 智能查找并聚焦
        let found = page.evaluate(r#"
            (function() {
                // 1. 优先查找明确的 name/type 属性
                const emailInput = document.querySelector('input[name="email"]') || document.querySelector('input[type="email"]');
                if (emailInput) {
                    emailInput.focus();
                    return true;
                }

                // 2. 遍历输入框
                const inputs = document.querySelectorAll('input');
                for (const input of inputs) {
                    const attr = (input.getAttribute('name') || input.getAttribute('autocomplete') || input.placeholder || '').toLowerCase();
                    if (attr.includes('email') || attr.includes('邮箱') || attr.includes('邮件')) {
                        input.focus();
                        return true;
                    }
                }
                return false;
            })()
        "#).await;

        match found {
            Ok(val) => {
                if val.into_value::<bool>()? {
                    info!("   ✅ 通过 JS 找到并聚焦邮箱输入框 / Found and focused email input via JS");
                    sleep(Duration::from_millis(500)).await;
                    
                    // 使用 JS 设置值 (React 兼容)
                    let js_value = serde_json::to_string(email)?;
                    let script = format!(r#"
                        (function() {{
                            const element = document.activeElement;
                            if (!element) return false;
                            const value = {};
                            
                            const valueSetter = Object.getOwnPropertyDescriptor(element, 'value').set;
                            const prototype = Object.getPrototypeOf(element);
                            const prototypeValueSetter = Object.getOwnPropertyDescriptor(prototype, 'value').set;
                            
                            if (valueSetter && valueSetter !== prototypeValueSetter) {{
                                prototypeValueSetter.call(element, value);
                            }} else {{
                                valueSetter.call(element, value);
                            }}
                            
                            element.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            element.dispatchEvent(new Event('change', {{ bubbles: true }}));
                            return true;
                        }})()
                    "#, js_value);
                    
                    page.evaluate(script).await?;
                    info!("   ✅ 邮箱已填写 / Email filled");
                    return Ok(());
                }
            },
            Err(e) => warn!("   JS 查找失败: {} / JS search failed: {}", e, e),
        }

        // 方法2: 传统的选择器回退
        let selectors = vec![
            "input[name='email']",
            "input[autocomplete='email']",
            "input[type='email']",
            "input[placeholder*='Email']",
            "input[placeholder*='邮箱']",
        ];

        for selector in selectors {
            if let Ok(element) = page.find_element(selector).await {
                element.click().await?;
                element.type_str(email).await?;
                info!("   ✅ 邮箱已填写 (选择器) / Email filled (selector)");
                return Ok(());
            }
        }

        anyhow::bail!("未找到邮箱输入框 / Email input not found")
    }


    /// 填写出生日期
    async fn fill_birth_date(&self, page: &chromiumoxide::Page, birth_date: &BirthDate) -> Result<()> {
        // 填写月份
        if let Ok(month_select) = page.find_element("select[id*='MONTH'], select[aria-label*='Month']").await {
            month_select.click().await?;
            sleep(Duration::from_millis(500)).await;
            // 选择月份选项
            let month_option = format!("option:has-text('{}')", birth_date.month);
            if let Ok(option) = page.find_element(&month_option).await {
                option.click().await?;
            }
        }

        // 填写日期
        if let Ok(day_input) = page.find_element("input[id*='DAY'], input[aria-label*='Day']").await {
            day_input.click().await?;
            day_input.type_str(&birth_date.day).await?;
        }

        // 填写年份
        if let Ok(year_input) = page.find_element("input[id*='YEAR'], input[aria-label*='Year']").await {
            year_input.click().await?;
            year_input.type_str(&birth_date.year).await?;
        }

        info!("   ✅ 出生日期已填写 / Birth date filled");
        Ok(())
    }

    /// 点击下一步按钮
    async fn click_next_button(&self, page: &chromiumoxide::Page) -> Result<()> {
        info!("   查找'下一步'按钮 / Looking for 'Next' button");
        
        // 方法1: JS 查找 (文本匹配)
        let js_script = r#"
        (function() {
            const targets = ['Next', '下一步', '继续'];
            const buttons = document.querySelectorAll('button, div[role="button"]');
            
            for (let btn of buttons) {
                const text = (btn.innerText || '').trim();
                // 精确匹配或包含匹配
                if (targets.includes(text) || (text.length < 10 && (text.includes('Next') || text.includes('下一步')))) {
                    // 检查是否可见
                    const style = window.getComputedStyle(btn);
                    if (style.display !== 'none' && style.visibility !== 'hidden') {
                        if (btn.disabled || btn.getAttribute('aria-disabled') === 'true') {
                            console.log('找到下一步按钮，但是被禁用了:', text);
                            return 'disabled';
                        }
                        console.log('找到下一步按钮:', text);
                        btn.click();
                        return 'clicked';
                    }
                }
            }
            
            // 尝试通过 data-testid
            const testIdBtn = document.querySelector('[data-testid="ocfEnterTextNextButton"]');
            if (testIdBtn) {
                if (testIdBtn.disabled || testIdBtn.getAttribute('aria-disabled') === 'true') {
                     return 'disabled';
                }
                testIdBtn.click();
                return 'clicked';
            }
            
            return 'not_found';
        })()
        "#;
        
        // 增加重试
        for i in 0..5 {
            if i > 0 {
                sleep(Duration::from_secs(1)).await;
            }
            
            match page.evaluate(js_script).await {
                Ok(val) => {
                    let result: String = val.into_value()?;
                    if result == "clicked" {
                        info!("   ✅ 通过 JS 点击了 Next 按钮 / Clicked Next button via JS");
                        return Ok(());
                    } else if result == "disabled" {
                        warn!("   ⚠️  Next 按钮存在但被禁用 (可能是表单未完成或有错误) / Next button disabled");
                        // 检查是否有错误提示
                        self.check_form_errors(page).await?;
                    }
                },
                Err(e) => warn!("   JS 点击失败: {} / JS click failed: {}", e, e),
            }
        }

        // 方法2: 尝试特定的 CSS 选择器 (作为备选)
        let selectors = vec![
            "[data-testid='ocfEnterTextNextButton']",
            "button[type='submit']",
        ];

        for selector in selectors {
            if let Ok(element) = page.find_element(selector).await {
                element.click().await?;
                info!("   ✅ Next 按钮已点击 (选择器) / Next button clicked (selector)");
                return Ok(());
            }
        }

        anyhow::bail!("未找到 Next 按钮 / Next button not found")
    }

    /// 检查表单错误
    async fn check_form_errors(&self, page: &chromiumoxide::Page) -> Result<()> {
        let js_script = r#"
        (function() {
            // 查找常见的错误提示元素
            // X 的错误提示通常在 input 下方，或者有特定的 role="alert"
            const errors = document.querySelectorAll('[data-testid="app-bar-close"], div[role="alert"], span[style*="color: rgb(244, 33, 46)"], span[style*="color: rgb(220, 30, 41)"]');
            for (let err of errors) {
                const text = err.innerText;
                if (text && (text.includes('taken') || text.includes('registered') || text.includes('已被注册') || text.includes('占用') || text.includes('valid'))) {
                    return text;
                }
            }
            return null;
        })()
        "#;
        
        if let Ok(val) = page.evaluate(js_script).await {
            if let Some(err_msg) = val.into_value::<Option<String>>()? {
                warn!("⚠️  检测到表单错误: {} / Form error detected: {}", err_msg, err_msg);
                // 这里我们不 bail，只是记录警告，让上层决定是否继续（虽然通常意味着无法继续）
            }
        }
        Ok(())
    }

    /// 检查是否有人机验证
    async fn has_captcha(&self, page: &chromiumoxide::Page) -> bool {
        let captcha_selectors = vec![
            "iframe[src*='captcha']",
            "iframe[src*='recaptcha']",
            "iframe[src*='hcaptcha']",
            "iframe[src*='arkose']",
            "iframe[src*='funcaptcha']",
            "[id*='captcha']",
            "[class*='captcha']",
        ];

        for selector in captcha_selectors {
            if page.find_element(selector).await.is_ok() {
                return true;
            }
        }

        false
    }

    /// 填写验证码
    async fn fill_verification_code(&self, page: &chromiumoxide::Page, code: &str) -> Result<()> {
        let selectors = vec![
            "input[name='verfication_code']",
            "input[name='code']",
            "input[autocomplete='one-time-code']",
            "input[placeholder*='code']",
            "input[placeholder*='验证码']",
            "input[type='text'][maxlength='6']",
        ];

        for selector in selectors {
            if let Ok(element) = page.find_element(selector).await {
                element.click().await?;
                element.type_str(code).await?;
                info!("   ✅ 验证码已填写 / Verification code filled");
                return Ok(());
            }
        }

        anyhow::bail!("未找到验证码输入框 / Verification code input not found")
    }

    /// 填写密码
    async fn fill_password(&self, page: &chromiumoxide::Page, password: &str) -> Result<()> {
        let selectors = vec![
            "input[name='password']",
            "input[type='password']",
            "input[autocomplete='new-password']",
            "input[placeholder*='Password']",
            "input[placeholder*='密码']",
        ];

        for selector in selectors {
            if let Ok(element) = page.find_element(selector).await {
                element.click().await?;
                element.type_str(password).await?;
                info!("   ✅ 密码已填写 / Password filled");
                return Ok(());
            }
        }

        anyhow::bail!("未找到密码输入框 / Password input not found")
    }

    /// 点击 Get Started 按钮
    async fn click_get_started(&self, page: &chromiumoxide::Page) -> Result<()> {
        let selectors = vec![
            "button:has-text('Get started')",
            "button:has-text('开始')",
            "div[role='button']:has-text('Get started')",
            "[data-testid='get-started-btn']",
        ];

        for selector in selectors {
            if let Ok(element) = page.find_element(selector).await {
                element.click().await?;
                info!("   ✅ Get Started 已点击 / Get Started clicked");
                return Ok(());
            }
        }

        anyhow::bail!("未找到 Get Started 按钮 / Get Started button not found")
    }

    /// 点击跳过按钮
    async fn click_skip_button(&self, page: &chromiumoxide::Page) -> Result<()> {
        let selectors = vec![
            "button:has-text('Skip')",
            "button:has-text('跳过')",
            "span:has-text('Skip for now')",
            "span:has-text('暂时跳过')",
            "a:has-text('Skip')",
            "[data-testid='skip']",
        ];

        for selector in selectors {
            if let Ok(element) = page.find_element(selector).await {
                element.click().await?;
                return Ok(());
            }
        }

        anyhow::bail!("未找到 Skip 按钮 / Skip button not found")
    }

    async fn take_screenshot(&self, page: &chromiumoxide::Page, name: &str) -> Result<()> {
        let dir = crate::data_dir::get_screenshots_dir();
        std::fs::create_dir_all(&dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.png", name, timestamp);
        let filepath = dir.join(filename);

        info!("   保存截图到 / Saving screenshot to: {}", filepath.display());
        let screenshot = page
            .screenshot(chromiumoxide::page::ScreenshotParams::builder().build())
            .await?;
        std::fs::write(&filepath, screenshot)?;

        info!("   ✅ 截图已保存 / Screenshot saved");
        Ok(())
    }
}
