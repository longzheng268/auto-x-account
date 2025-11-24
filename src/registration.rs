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

        // 设置用户数据目录
        let user_data_dir = crate::data_dir::get_browser_data_dir();
        std::fs::create_dir_all(&user_data_dir)?;
        builder = builder.user_data_dir(&user_data_dir);
        info!("   数据目录 / Data directory: {}", user_data_dir.display());

        // 启动浏览器 / Launch browser
        info!("🌐 启动浏览器 / Launching browser...");
        let (mut browser, mut handler) = Browser::launch(
            builder.build().map_err(|e| anyhow::anyhow!("Browser configuration error / 浏览器配置错误: {}", e))?
        ).await?;
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

    async fn perform_registration(
        &self,
        page: &chromiumoxide::Page,
        account_info: &mut AccountInfo,
    ) -> Result<AccountInfo> {
        // 访问注册页面
        info!("🔗 访问注册页面 / Navigating to registration page");
        info!("   URL: {}", self.config.x_account.base_url);
        page.goto(&self.config.x_account.base_url).await?;
        info!("✅ 页面加载完成 / Page loaded");
        sleep(Duration::from_secs(3)).await;

        // 截图
        info!("📸 截图: 注册页面 / Screenshot: Registration page");
        self.take_screenshot(page, "01_signup_page").await?;

        // 步骤1: 切换到邮箱注册
        info!("📧 步骤1: 切换到邮箱注册 / Step 1: Switch to email registration");
        if let Err(e) = self.switch_to_email_registration(page).await {
            warn!("切换到邮箱注册失败，尝试继续 / Failed to switch to email: {}", e);
            // 可能已经是邮箱模式，继续
        }
        sleep(Duration::from_secs(2)).await;
        self.take_screenshot(page, "02_email_mode").await?;

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
            
            // 这里可以尝试自动识别，或者等待手动完成
            info!("⏳ 等待手动完成人机验证 / Waiting for manual captcha completion");
            info!("   请在浏览器中完成左侧数字与右侧图片匹配的验证");
            info!("   Please complete the captcha matching numbers with images");
            
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
        info!("   查找'使用邮箱'按钮 / Looking for 'Use email' button");
        
        // 尝试多个可能的选择器
        let selectors = vec![
            "span:has-text('Use email instead')",
            "span:has-text('使用电子邮件')",
            "a:has-text('Use email')",
            "[data-testid='switch-to-email']",
            "span[dir='ltr']:has-text('email')",
        ];

        for selector in selectors {
            if let Ok(element) = page.find_element(selector).await {
                info!("   找到切换按钮，点击 / Found switch button, clicking");
                element.click().await?;
                return Ok(());
            }
        }

        // 如果没找到，可能已经是邮箱模式
        warn!("   未找到切换按钮，可能已是邮箱模式 / Switch button not found, may already be in email mode");
        Ok(())
    }

    /// 填写姓名
    async fn fill_name(&self, page: &chromiumoxide::Page, name: &str) -> Result<()> {
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
                info!("   ✅ 姓名已填写 / Name filled");
                return Ok(());
            }
        }

        anyhow::bail!("未找到姓名输入框 / Name input not found")
    }

    /// 填写邮箱
    async fn fill_email(&self, page: &chromiumoxide::Page, email: &str) -> Result<()> {
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
                info!("   ✅ 邮箱已填写 / Email filled");
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
        let selectors = vec![
            "button:has-text('Next')",
            "button:has-text('下一步')",
            "div[role='button']:has-text('Next')",
            "[data-testid='ocfEnterTextNextButton']",
            "button[type='submit']",
        ];

        for selector in selectors {
            if let Ok(element) = page.find_element(selector).await {
                element.click().await?;
                info!("   ✅ Next 按钮已点击 / Next button clicked");
                return Ok(());
            }
        }

        anyhow::bail!("未找到 Next 按钮 / Next button not found")
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
