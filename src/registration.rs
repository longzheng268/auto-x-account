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

#[derive(Debug, Clone, Default)]
pub struct RegistrationRequest {
    pub email: String,
    pub name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub phone: Option<String>,
    pub birth_date: Option<BirthDate>,
}

impl RegistrationRequest {
    pub fn new(email: String) -> Self {
        Self {
            email,
            ..Default::default()
        }
    }
}

pub struct XRegistration {
    config: Config,
    email_handler: EmailHandler,
}

impl XRegistration {
    pub fn new(config: Config, email_handler: EmailHandler) -> Self {
        XRegistration { config, email_handler }
    }

    // ------------------------------------------------------------
    // 浏览器路径检测（加入 Edge 支持）
    // ------------------------------------------------------------
    fn get_chrome_executable_path(&self) -> Result<Option<PathBuf>> {
        // 1. 配置文件指定路径
        if let Some(config_path) = &self.config.browser.chrome_path {
            let path = PathBuf::from(config_path);
            if path.exists() {
                return Ok(Some(path));
            } else {
                warn!("配置的 Chrome 路径不存在: {}", config_path);
            }
        }
        // 2. 打包的 Chromium 目录
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let bundled_paths = vec![
                    exe_dir.join("chromium").join("chrome"),           // Linux
                    exe_dir.join("chromium").join("chrome.exe"),       // Windows
                    exe_dir.join("chromium").join("Chromium.app").join("Contents").join("MacOS").join("Chromium"), // macOS
                    exe_dir.join("chrome-linux").join("chrome"),       // Linux alternative
                    exe_dir.join("chrome-win").join("chrome.exe"),     // Windows alternative
                ];
                for path in bundled_paths {
                    if path.exists() {
                        info!("找到打包的 Chromium: {}", path.display());
                        return Ok(Some(path));
                    }
                }
            }
        }
        // 3. 系统已安装的浏览器（加入 Edge）
        let system_paths = if cfg!(target_os = "windows") {
            vec![
                PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
                PathBuf::from(r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
                PathBuf::from(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
                PathBuf::from(r"C:\Program Files\Chromium\Application\chrome.exe"),
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
                PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium"),
                PathBuf::from("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
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
        warn!("未找到 Chrome/Chromium 可执行文件，将使用默认查找方式");
        Ok(None)
    }

    // ------------------------------------------------------------
    // 账号信息生成
    // ------------------------------------------------------------
    fn generate_account_info(&self, email: String) -> AccountInfo {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let name = if self.config.language.starts_with("zh") {
            Name(ZH_CN).fake()
        } else {
            Name(EN).fake()
        };
        let username = format!(
            "user_{}_{}",
            Faker.fake::<String>().chars().filter(|c| c.is_alphanumeric()).take(8).collect::<String>(),
            chrono::Utc::now().timestamp()
        );
        let password = format!(
            "{}{}{}{}",
            Faker.fake::<String>().chars().filter(|c| c.is_alphabetic()).take(4).collect::<String>(),
            rng.gen_range(1000..9999),
            Faker.fake::<String>().chars().filter(|c| c.is_uppercase()).take(2).collect::<String>(),
            "!@"
        );
        let months = vec![
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December",
        ];
        let birth_year = rng.gen_range(1965..=2007).to_string();
        let birth_day = rng.gen_range(1..29).to_string();
        AccountInfo {
            email,
            name,
            username,
            password,
            phone: None,
            birth_date: BirthDate {
                month: months[rng.gen_range(0..12)].to_string(),
                day: birth_day,
                year: birth_year,
            },
            created_at: chrono::Utc::now().to_rfc3339(),
            status: "pending".to_string(),
        }
    }

    // ------------------------------------------------------------
    // 主入口
    // ------------------------------------------------------------
    pub async fn register_account(&self, request: RegistrationRequest) -> Result<AccountInfo> {
        let mut account_info = self.generate_account_info(request.email.clone());
        if let Some(name) = request.name {
            account_info.name = name;
        }
        if let Some(username) = request.username {
            account_info.username = username;
        }
        if let Some(password) = request.password {
            account_info.password = password;
        }
        if let Some(phone) = request.phone {
            account_info.phone = Some(phone);
        }
        if let Some(birth) = request.birth_date {
            account_info.birth_date = birth;
        }
        info!("🚀 开始注册账号 / Starting account registration");
        info!("   邮箱 / Email: {}", account_info.email);
        info!("   用户名 / Username: {}", account_info.username);
        info!("   生日 / Birthday: {}/{}/{}", account_info.birth_date.month, account_info.birth_date.day, account_info.birth_date.year);

        // 配置浏览器
        info!("⚙️  配置浏览器 / Configuring browser...");
        let mut builder = BrowserConfig::builder();
        if !self.config.browser.headless {
            builder = builder.with_head();
            info!("   模式 / Mode: 有界面 / With UI");
        } else {
            info!("   模式 / Mode: 无界面 / Headless");
        }
        if let Some(path) = self.get_chrome_executable_path()? {
            info!("   浏览器路径 / Browser path: {}", path.display());
            builder = builder.chrome_executable(&path);
        } else {
            info!("   浏览器路径 / Browser path: 使用系统默认");
        }
        if let Some(proxy_url) = self.config.get_proxy_url(crate::config::ProxyTarget::Browser) {
            info!("   代理 / Proxy: {}", proxy_url);
            builder = builder.arg(format!("--proxy-server={}", proxy_url));
        } else {
            info!("   代理 / Proxy: 不使用");
        }

        // 设置中文环境和隐私模式
        info!("   语言 / Language: 简体中文 (zh-CN)");
        info!("   模式 / Mode: InPrivate（隐私模式）");
        builder = builder
            .arg("--lang=zh-CN")
            .arg("--accept-lang=zh-CN,zh;q=0.9,en;q=0.8")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--remote-debugging-port=0")
            .arg("--inprivate")  // 启用 InPrivate 隐私模式
            .arg("--new-window"); // 强制新窗口

        // 每个任务使用独立的临时配置目录，确保 DevTools 端口可用
        let user_data_dir = crate::data_dir::create_temporary_browser_dir()?;
        info!("   数据目录 / Data directory: {}", user_data_dir.display());
        builder = builder.user_data_dir(&user_data_dir);

        // 启动浏览器
        info!("🌐 启动浏览器 / Launching browser...");
        let (mut browser, mut handler) = Browser::launch(
            builder.build().map_err(|e| anyhow::anyhow!("Browser configuration error: {}", e))?
        ).await?;
        info!("✅ 浏览器已启动 / Browser launched successfully");
        
        // 处理事件
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    error!("浏览器事件错误 / Browser event error: {}", e);
                }
            }
        });

        // 等待浏览器完全启动
        sleep(Duration::from_secs(2)).await;

        // 在新标签页打开注册页面（而不是 about:blank）
        info!("📄 在新标签页打开注册页面 / Opening registration page in new tab...");
        let page = browser.new_page(&self.config.x_account.base_url).await?;
        info!("✅ 注册页面已打开 / Registration page opened");
        // 执行注册流程
        info!("📝 执行注册流程 / Performing registration...");
        let result = self.perform_registration(&page, &mut account_info).await;
        // 清理
        info!("🧹 清理资源 / Cleaning up resources...");
        if let Err(e) = browser.close().await {
            warn!("关闭浏览器时出错 / Error closing browser: {}", e);
        }
        match &result {
            Ok(acc) => info!("✅ 账号注册成功 / Account registered successfully: {}", acc.username),
            Err(e) => error!("❌ 账号注册失败 / Account registration failed: {}", e),
        }
        result
    }

    // ------------------------------------------------------------
    // 注册主流程
    // ------------------------------------------------------------
    async fn perform_registration(&self, page: &chromiumoxide::Page, account_info: &mut AccountInfo) -> Result<AccountInfo> {
        // 1. 等待页面加载（页面已在 new_page 时打开）
        info!("⏳ 等待页面加载完成 / Waiting for page to load...");
        sleep(Duration::from_secs(3)).await;
        
        // 检查当前 URL
        if let Some(current_url) = page.url().await.ok().flatten() {
            info!("   当前 URL / Current URL: {}", current_url);
        }
        
        self.take_screenshot(page, "01_signup_page").await?;

        // 2. 点击 "创建账号"（动态 JS + 图像识别）
        info!("🔘 步骤1: 点击创建账号按钮 / Click Sign up button");
        self.click_signup_button(page).await?;
        sleep(Duration::from_secs(2)).await;
        self.take_screenshot(page, "02_after_signup_click").await?;

        // 3. 切换到邮箱注册（如果需要）
        info!("📧 步骤2: 切换到邮箱注册 / Switch to email registration");
        if let Err(e) = self.switch_to_email_registration(page).await {
            warn!("切换到邮箱注册失败，可能已是邮箱模式: {}", e);
        }
        sleep(Duration::from_secs(2)).await;
        self.take_screenshot(page, "03_email_mode").await?;

        // 4. 填写姓名、邮箱、出生日期
        info!("✏️ 步骤3: 填写姓名 / Fill in name");
        self.fill_name(page, &account_info.name).await?;
        sleep(Duration::from_secs(1)).await;
        info!("✏️ 步骤4: 填写邮箱 / Fill in email");
        self.fill_email(page, &account_info.email).await?;
        sleep(Duration::from_secs(1)).await;
        info!("✏️ 步骤5: 填写出生日期 / Fill in birth date");
        self.fill_birth_date(page, &account_info.birth_date).await?;
        sleep(Duration::from_secs(1)).await;
        self.take_screenshot(page, "04_basic_info_filled").await?;

        // 5. 点击下一步
        info!("➡️ 步骤6: 点击下一步 / Click Next");
        if let Err(e) = self.click_next_button(page).await {
            if let Some(err_msg) = self.check_form_errors(page).await? {
                if Self::is_email_taken_error(&err_msg) {
                    account_info.status = "email_used".to_string();
                    warn!("⚠️ 邮箱已被使用: {}", err_msg);
                    anyhow::bail!("邮箱已被使用: {}", err_msg);
                } else {
                    warn!("⚠️ 表单错误: {}", err_msg);
                }
            }
            return Err(e);
        }
        sleep(Duration::from_secs(3)).await;
        self.take_screenshot(page, "05_after_basic_info").await?;

        // 6. 人机验证（如有）
        info!("🤖 步骤7: 检查人机验证 / Check captcha");
        if self.has_captcha(page).await {
            info!("⚠️ 检测到验证码 / Captcha detected");
            self.take_screenshot(page, "06_captcha_detected").await?;
            // 等待手动完成
            info!("⏳ 等待手动完成人机验证");
            let timeout = Duration::from_secs(180);
            let start = tokio::time::Instant::now();
            while start.elapsed() < timeout && self.has_captcha(page).await {
                sleep(Duration::from_secs(2)).await;
            }
            if self.has_captcha(page).await {
                error!("❌ 人机验证超时");
                anyhow::bail!("Captcha timeout");
            }
            info!("✅ 人机验证完成");
            sleep(Duration::from_secs(2)).await;
        } else {
            info!("✅ 未检测到人机验证");
        }
        self.take_screenshot(page, "07_after_captcha").await?;

        // 7. 如有需要再次点击下一步
        if let Ok(_) = self.click_next_button(page).await {
            sleep(Duration::from_secs(3)).await;
        }

        // 8. 等待并填写邮箱验证码
        info!("📬 步骤8: 等待邮箱验证码");
        let email_timeout = Duration::from_secs(self.config.x_account.email_wait_timeout);
        if let Some(code) = self.email_handler.wait_for_verification_code(&account_info.email, email_timeout).await {
            info!("✅ 收到验证码: {}", code);
            self.take_screenshot(page, "08_verification_code_page").await?;
            self.fill_verification_code(page, &code).await?;
            sleep(Duration::from_secs(2)).await;
            self.click_next_button(page).await?;
            sleep(Duration::from_secs(3)).await;
            self.take_screenshot(page, "09_after_verification").await?;
        } else {
            error!("❌ 未收到验证码");
            self.take_screenshot(page, "08_verification_timeout").await?;
            anyhow::bail!("Verification code not received");
        }

        // 9. 设置密码
        info!("🔐 步骤9: 设置密码");
        self.fill_password(page, &account_info.password).await?;
        sleep(Duration::from_secs(2)).await;
        self.take_screenshot(page, "10_password_set").await?;
        self.click_next_button(page).await?;
        sleep(Duration::from_secs(3)).await;
        self.take_screenshot(page, "11_after_password").await?;

        // 10. 点击 Get Started
        info!("🚀 步骤10: 点击 Get Started");
        if let Ok(_) = self.click_get_started(page).await {
            sleep(Duration::from_secs(3)).await;
            self.take_screenshot(page, "12_get_started").await?;
        } else {
            warn!("⚠️ 未找到 Get Started 按钮");
        }

        // 11. 跳过可选步骤
        info!("⏭️ 步骤11: 跳过可选步骤");
        for i in 1..=5 {
            if self.click_skip_button(page).await.is_ok() {
                info!("   跳过步骤 {}", i);
                sleep(Duration::from_secs(2)).await;
                self.take_screenshot(page, &format!("13_skip_{}", i)).await?;
            } else {
                info!("   没有更多跳过选项");
                break;
            }
        }

        account_info.status = "registered".to_string();
        info!("🎉 账号注册完成");
        self.take_screenshot(page, "14_registration_complete").await?;
        Ok(account_info.clone())
    }

    // ------------------------------------------------------------
    // 动态点击创建账号按钮（JS + 图像识别回退）
    // ------------------------------------------------------------
    async fn click_signup_button(&self, page: &chromiumoxide::Page) -> Result<()> {
        // JS 动态遍历所有按钮文本
        let js = r#"
        (function() {
            const btns = document.querySelectorAll('button, div[role="button"], a[role="button"]');
            for (let b of btns) {
                const txt = (b.innerText || b.textContent || '').trim();
                if (txt.includes('创建账号') || txt.includes('创建帐号') || txt.includes('创建帳號')) {
                    b.click();
                    return true;
                }
            }
            return false;
        })()
        "#;
        if page.evaluate(js).await?.into_value::<bool>()? {
            info!("✅ 通过 JavaScript 成功点击 创建账号 按钮");
            return Ok(());
        }
        // 回退到图像识别（已实现的函数）
        info!("⚠️ JavaScript 未找到按钮，尝试图像识别");
        // Fallback to image recognition removed – if JS fails, return an error
anyhow::bail!("未找到创建账号按钮 / Sign‑up button not found");
    }

    // ------------------------------------------------------------
    // 选择器集合（创建账号、下一步、Get Started）
    // ------------------------------------------------------------
    fn getSignUpButtonSelectors() -> Vec<&'static str> {
        vec![
            "button:has-text('创建账号')",
            "button:has-text('创建帐号')",
            "button:has-text('创建帳號')",
            "div[role='button']:has-text('创建账号')",
        ]
    }

    fn getNextButtonSelectors() -> Vec<&'static str> {
        vec![
            "button:has-text('Next')",
            "button:has-text('下一步')",
            "div[role='button']:has-text('Next')",
            "[data-testid='ocfEnterTextNextButton']",
            "button[type='submit']",
        ]
    }

    fn getGetStartedButtonSelectors() -> Vec<&'static str> {
        vec![
            "button:has-text('Get started')",
            "button:has-text('开始')",
            "div[role='button']:has-text('Get started')",
            "[data-testid='get-started-btn']",
        ]
    }

    fn is_email_taken_error(msg: &str) -> bool {
        let msg_lower = msg.to_lowercase();
        let keywords = [
            "already taken",
            "already registered",
            "already in use",
            "email is taken",
            "邮箱已被使用",
            "电子邮件已被使用",
            "已被注册",
            "已存在",
            "占用",
            "使用中",
        ];
        keywords.iter().any(|k| msg_lower.contains(k))
    }

    // ------------------------------------------------------------
    // 切换到邮箱注册模式
    // ------------------------------------------------------------
    async fn switch_to_email_registration(&self, page: &chromiumoxide::Page) -> Result<()> {
        info!("   查找“使用邮箱”入口 / Searching for \"Use email\" toggle");
        let js_script = r#"
        (function() {
            const candidates = [
                "Use email instead",
                "Use email",
                "使用电子邮件",
                "改用电子邮件",
                "使用邮箱",
                "改用邮箱",
                "使用邮箱注册",
                "使用邮件注册",
                "改用电子邮件而不是电话号码"
            ];
            const elements = document.querySelectorAll('button, div[role="button"], span, a');
            for (const el of elements) {
                const text = (el.innerText || el.textContent || '').trim();
                if (!text) continue;
                for (const target of candidates) {
                    if (text.includes(target)) {
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none' || style.visibility === 'hidden') continue;
                        el.click();
                        return true;
                    }
                }
            }
            return false;
        })()
        "#;

        for attempt in 1..=3 {
            if let Ok(eval_result) = page.evaluate(js_script).await {
                if eval_result.into_value::<bool>()? {
                    info!("   ✅ 通过 JavaScript 切换到邮箱方式（尝试 {}）", attempt);
                    return Ok(());
                }
            }
            sleep(Duration::from_secs(1)).await;
        }

        let selectors = vec![
            "span:has-text('Use email instead')",
            "button:has-text('Use email instead')",
            "span:has-text('Use email')",
            "button:has-text('Use email')",
            "span:has-text('使用电子邮件')",
            "button:has-text('使用电子邮件')",
            "span:has-text('改用电子邮件')",
            "button:has-text('改用电子邮件')",
            "span:has-text('使用邮箱')",
            "button:has-text('使用邮箱')",
            "div[role='button']:has-text('使用邮箱')",
            "a:has-text('Use email')",
            "[data-testid='switch-to-email']",
            "span[dir='ltr']:has-text('email')",
        ];

        for attempt in 1..=3 {
            for selector in &selectors {
                if let Ok(el) = page.find_element(*selector).await {
                    el.click().await?;
                    info!("   ✅ 通过选择器切换到邮箱方式（尝试 {}）", attempt);
                    return Ok(());
                }
            }
            sleep(Duration::from_secs(1)).await;
        }

        warn!("   ⚠️ 未找到“使用邮箱”切换入口，可能已默认使用邮箱注册");
        Ok(())
    }

    // ------------------------------------------------------------
    // 填写姓名（多层定位）
    // ------------------------------------------------------------
    async fn fill_name(&self, page: &chromiumoxide::Page, name: &str) -> Result<()> {
        // 1. 常规选择器
        let selectors = vec![
            "input[name='name']",
            "input[autocomplete='name']",
            "input[placeholder*='Name']",
            "input[placeholder*='名字']",
        ];
        for sel in &selectors {
            if let Ok(el) = page.find_element(*sel).await {
                el.click().await?;
                el.type_str(name).await?;
                info!("✅ 姓名已填写（selector）");
                return Ok(());
            }
        }
        // 2. XPath 备选
        let xpaths = vec![
            "//input[contains(@aria-label,'name') or contains(@placeholder,'Name') or contains(@placeholder,'名字')]",
            "//input[contains(@type,'text') and (contains(@placeholder,'Name') or contains(@placeholder,'名字'))]",
        ];
        for xp in &xpaths {
            if let Ok(el) = page.find_element(*xp).await {
                el.click().await?;
                el.type_str(name).await?;
                info!("✅ 姓名已填写（XPath）");
                return Ok(());
            }
        }
        anyhow::bail!("未找到姓名输入框 / Name input not found")
    }

    // ------------------------------------------------------------
    // 填写邮箱（多层定位）
    // ------------------------------------------------------------
    async fn fill_email(&self, page: &chromiumoxide::Page, email: &str) -> Result<()> {
        let selectors = vec![
            "input[name='email']",
            "input[autocomplete='email']",
            "input[type='email']",
            "input[placeholder*='Email']",
            "input[placeholder*='邮箱']",
        ];
        for sel in &selectors {
            if let Ok(el) = page.find_element(*sel).await {
                el.click().await?;
                el.type_str(email).await?;
                info!("✅ 邮箱已填写（selector）");
                return Ok(());
            }
        }
        anyhow::bail!("未找到邮箱输入框 / Email input not found")
    }

    // ------------------------------------------------------------
    // 填写出生日期
    // ------------------------------------------------------------
    async fn fill_birth_date(&self, page: &chromiumoxide::Page, birth_date: &BirthDate) -> Result<()> {
        let month_value = Self::month_to_number(&birth_date.month);
        let js = format!(
            r#"
            (function() {{
                const month = document.querySelector("select[id*='MONTH'], select[aria-label*='Month'], select[aria-label*='月'], select[aria-label*='月份'], select[name*='month']");
                const day = document.querySelector("input[id*='DAY'], input[aria-label*='Day'], input[aria-label*='日'], input[aria-label*='日期'], input[name*='day']");
                const year = document.querySelector("input[id*='YEAR'], input[aria-label*='Year'], input[aria-label*='年'], input[name*='year']");

                if (month) {{
                    month.value = "{month}";
                    month.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    month.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}
                if (day) {{
                    day.value = "{day}";
                    day.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    day.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}
                if (year) {{
                    year.value = "{year}";
                    year.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    year.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }}
                return Boolean(month && day && year);
            }})()
            "#,
            month = month_value,
            day = birth_date.day,
            year = birth_date.year
        );

        if let Ok(res) = page.evaluate(js).await {
            if res.into_value::<bool>()? {
                info!("✅ 出生日期已填写（JS）");
                return Ok(());
            }
        }

        // Fallback to direct element typing if JS failed
        if let Ok(month_select) = page
            .find_element("select[id*='MONTH'], select[aria-label*='Month'], select[aria-label*='月'], select[aria-label*='月份'], select[name*='month']")
            .await
        {
            month_select.click().await?;
            sleep(Duration::from_millis(200)).await;
            let month_option = format!("option[value='{}'], option:has-text('{}')", month_value, birth_date.month);
            if let Ok(option) = page.find_element(&month_option).await {
                option.click().await?;
            }
        }
        if let Ok(day_input) = page
            .find_element("input[id*='DAY'], input[aria-label*='Day'], input[aria-label*='日'], input[aria-label*='日期'], input[name*='day']")
            .await
        {
            day_input.click().await?;
            day_input.type_str(&birth_date.day).await?;
        }
        if let Ok(year_input) = page
            .find_element("input[id*='YEAR'], input[aria-label*='Year'], input[aria-label*='年'], input[name*='year']")
            .await
        {
            year_input.click().await?;
            year_input.type_str(&birth_date.year).await?;
        }
        info!("✅ 出生日期已填写");
        Ok(())
    }

    fn month_to_number(month: &str) -> &'static str {
        match month.trim().to_lowercase().as_str() {
            "january" | "jan" | "1" | "01" | "一月" => "1",
            "february" | "feb" | "2" | "02" | "二月" => "2",
            "march" | "mar" | "3" | "03" | "三月" => "3",
            "april" | "apr" | "4" | "04" | "四月" => "4",
            "may" | "5" | "05" | "五月" => "5",
            "june" | "jun" | "6" | "06" | "六月" => "6",
            "july" | "jul" | "7" | "07" | "七月" => "7",
            "august" | "aug" | "8" | "08" | "八月" => "8",
            "september" | "sep" | "9" | "09" | "九月" => "9",
            "october" | "oct" | "10" | "十月" => "10",
            "november" | "nov" | "11" | "十一月" => "11",
            "december" | "dec" | "12" | "十二月" => "12",
            _ => "1",
        }
    }

    // ------------------------------------------------------------
    // 点击下一步按钮（增强版）
    // ------------------------------------------------------------
    async fn click_next_button(&self, page: &chromiumoxide::Page) -> Result<()> {
        // 滚动到底部确保可见
        let _ = page.evaluate(r#"window.scrollTo(0, document.body.scrollHeight);"#).await;
        // 方法1：JS 查找
        let js = r#"
        (function() {
            const targets = ['Next', '下一步', '继续', 'Next step'];
            const btns = document.querySelectorAll('button, div[role="button"], a');
            for (let b of btns) {
                const txt = (b.innerText || b.textContent || '').trim();
                if (targets.some(t => txt.includes(t))) {
                    const style = window.getComputedStyle(b);
                    if (style.display === 'none' || style.visibility === 'hidden') continue;
                    if (b.disabled || b.getAttribute('aria-disabled') === 'true') return 'disabled';
                    b.click();
                    return 'clicked';
                }
            }
            return 'not_found';
        })()
        "#;
        match page.evaluate(js).await {
            Ok(val) => {
                let res: String = val.into_value()?;
                if res == "clicked" {
                    info!("✅ 通过 JS 点击 Next 按钮");
                    return Ok(());
                } else if res == "disabled" {
                    warn!("⚠️ Next 按钮被禁用，检查表单错误");
                    if let Some(err) = self.check_form_errors(page).await? {
                        warn!("表单错误: {}", err);
                    }
                }
            }
            Err(e) => warn!("JS 执行错误: {}", e),
        }
        // 方法2：键盘回车（已移除，chromiumoxide 当前版本不提供直接键盘 API）
        info!("尝试使用键盘回车 – 已跳过");
        // 可选：使用 JavaScript 触发回车事件作为替代
        let _ = page.evaluate(r#"document.dispatchEvent(new KeyboardEvent('keydown', {key: 'Enter'}));"#).await;
        // 方法3：CSS 选择器备选
        for selector in Self::getNextButtonSelectors() {
            if let Ok(el) = page.find_element(selector).await {
                el.click().await?;
                info!("✅ Next 按钮已点击（selector）");
                return Ok(());
            }
        }
        anyhow::bail!("未找到 Next 按钮 / Next button not found")
    }

    // ------------------------------------------------------------
    // 检查表单错误并返回错误信息
    // ------------------------------------------------------------
    async fn check_form_errors(&self, page: &chromiumoxide::Page) -> Result<Option<String>> {
        let js = r#"
        (function() {
            const errors = document.querySelectorAll('[data-testid="app-bar-close"], div[role="alert"], span[style*="color: rgb(244, 33, 46)"], span[style*="color: rgb(220, 30, 41)"]');
            for (let e of errors) {
                const txt = e.innerText;
                if (txt && (txt.includes('taken') || txt.includes('registered') || txt.includes('已被注册') || txt.includes('占用') || txt.includes('valid'))){
                    return txt;
                }
            }
            return null;
        })()
        "#;
        if let Ok(val) = page.evaluate(js).await {
            if let Some(msg) = val.into_value::<Option<String>>()? {
                warn!("检测到表单错误: {}", msg);
                return Ok(Some(msg));
            }
        }
        Ok(None)
    }

    // ------------------------------------------------------------
    // 检测是否有人机验证
    // ------------------------------------------------------------
    async fn has_captcha(&self, page: &chromiumoxide::Page) -> bool {
        let selectors = vec![
            "iframe[src*='captcha']",
            "iframe[src*='recaptcha']",
            "iframe[src*='hcaptcha']",
            "iframe[src*='arkose']",
            "iframe[src*='funcaptcha']",
            "[id*='captcha']",
            "[class*='captcha']",
        ];
        for sel in selectors {
            if page.find_element(sel).await.is_ok() {
                return true;
            }
        }
        false
    }

    // ------------------------------------------------------------
    // 填写验证码
    // ------------------------------------------------------------
    async fn fill_verification_code(&self, page: &chromiumoxide::Page, code: &str) -> Result<()> {
        let selectors = vec![
            "input[name='verification_code']",
            "input[name='code']",
            "input[autocomplete='one-time-code']",
            "input[placeholder*='code']",
            "input[placeholder*='验证码']",
            "input[type='text'][maxlength='6']",
        ];
        for sel in selectors {
            if let Ok(el) = page.find_element(sel).await {
                el.click().await?;
                el.type_str(code).await?;
                info!("✅ 验证码已填写");
                return Ok(());
            }
        }
        anyhow::bail!("未找到验证码输入框 / Verification code input not found")
    }

    // ------------------------------------------------------------
    // 填写密码
    // ------------------------------------------------------------
    async fn fill_password(&self, page: &chromiumoxide::Page, password: &str) -> Result<()> {
        let selectors = vec![
            "input[name='password']",
            "input[type='password']",
            "input[autocomplete='new-password']",
            "input[placeholder*='Password']",
            "input[placeholder*='密码']",
        ];
        for sel in selectors {
            if let Ok(el) = page.find_element(sel).await {
                el.click().await?;
                el.type_str(password).await?;
                info!("✅ 密码已填写");
                return Ok(());
            }
        }
        anyhow::bail!("未找到密码输入框 / Password input not found")
    }

    // ------------------------------------------------------------
    // 点击 Get Started 按钮
    // ------------------------------------------------------------
    async fn click_get_started(&self, page: &chromiumoxide::Page) -> Result<()> {
        for selector in Self::getGetStartedButtonSelectors() {
            if let Ok(el) = page.find_element(selector).await {
                el.click().await?;
                info!("✅ Get Started 已点击");
                return Ok(());
            }
        }
        anyhow::bail!("未找到 Get Started 按钮 / Get Started button not found")
    }

    // ------------------------------------------------------------
    // 点击跳过按钮
    // ------------------------------------------------------------
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
            if let Ok(el) = page.find_element(selector).await {
                el.click().await?;
                return Ok(());
            }
        }
        anyhow::bail!("未找到 Skip 按钮 / Skip button not found")
    }

    // ------------------------------------------------------------
    // 截图工具
    // ------------------------------------------------------------
    async fn take_screenshot(&self, page: &chromiumoxide::Page, name: &str) -> Result<()> {
        let dir = crate::data_dir::get_screenshots_dir();
        std::fs::create_dir_all(&dir)?;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.png", name, timestamp);
        let filepath = dir.join(filename);
        info!("   保存截图到 / Saving screenshot to: {}", filepath.display());
        let screenshot = page.screenshot(chromiumoxide::page::ScreenshotParams::builder().build()).await?;
        std::fs::write(&filepath, screenshot)?;
        info!("   ✅ 截图已保存 / Screenshot saved");
        Ok(())
    }
}
