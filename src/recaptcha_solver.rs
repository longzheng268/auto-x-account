//! ReCAPTCHA 自动解决模块
//! ReCAPTCHA Automatic Solver Module
//!
//! # 说明 / Description
//!
//! 本模块提供多种方式自动解决 ReCAPTCHA 验证码：
//! This module provides multiple ways to automatically solve ReCAPTCHA:
//!
//! 1. **第三方验证服务** / Third-party CAPTCHA Services
//!    - 2Captcha (https://2captcha.com)
//!    - Anti-Captcha (https://anti-captcha.com)
//!    - CapMonster (https://capmonster.cloud)
//!
//! 2. **音频识别** / Audio Recognition
//!    - 使用语音识别 API 解决音频挑战
//!    - Use speech recognition API to solve audio challenges
//!
//! 3. **图像识别** / Image Recognition (实验性)
//!    - 使用 AI 模型识别图片
//!    - Use AI models to recognize images
//!
//! ## 使用方法 / Usage
//!
//! ```rust
//! let solver = RecaptchaSolver::new()
//!     .with_2captcha_api_key("YOUR_API_KEY".to_string());
//!
//! let token = solver.solve_recaptcha(page, site_key).await?;
//! solver.inject_recaptcha_token(page, token).await?;
//! ```

use anyhow::{Context, Result};
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// ReCAPTCHA 解决器
pub struct RecaptchaSolver {
    /// 2Captcha API Key
    two_captcha_api_key: Option<String>,
    /// Anti-Captcha API Key
    anti_captcha_api_key: Option<String>,
    /// CapMonster API Key
    capmonster_api_key: Option<String>,
    /// 是否启用音频识别后备方案
    enable_audio_fallback: bool,
}

/// 验证码类型
#[derive(Debug, Clone)]
pub enum CaptchaType {
    /// ReCAPTCHA v2 复选框
    ReCaptchaV2Checkbox,
    /// ReCAPTCHA v2 图片验证
    ReCaptchaV2Image,
    /// ReCAPTCHA v3
    ReCaptchaV3,
    /// hCaptcha
    HCaptcha,
}

/// 验证码解决结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaSolution {
    pub token: String,
    pub solved_at: String,
    pub method: String,
}

impl RecaptchaSolver {
    pub fn new() -> Self {
        RecaptchaSolver {
            two_captcha_api_key: None,
            anti_captcha_api_key: None,
            capmonster_api_key: None,
            enable_audio_fallback: true,
        }
    }

    pub fn with_2captcha_api_key(mut self, api_key: String) -> Self {
        self.two_captcha_api_key = Some(api_key);
        self
    }

    pub fn with_anti_captcha_api_key(mut self, api_key: String) -> Self {
        self.anti_captcha_api_key = Some(api_key);
        self
    }

    pub fn with_capmonster_api_key(mut self, api_key: String) -> Self {
        self.capmonster_api_key = Some(api_key);
        self
    }

    pub fn with_audio_fallback(mut self, enable: bool) -> Self {
        self.enable_audio_fallback = enable;
        self
    }

    /// 自动解决 ReCAPTCHA
    /// Automatically solve ReCAPTCHA
    pub async fn solve_recaptcha(
        &self,
        page: &Page,
        site_key: &str,
    ) -> Result<CaptchaSolution> {
        info!("开始解决 ReCAPTCHA, site_key: {}", site_key);

        // 优先使用第三方服务
        if let Some(api_key) = &self.two_captcha_api_key {
            info!("使用 2Captcha 服务解决验证码");
            match self.solve_with_2captcha(page, site_key, api_key).await {
                Ok(solution) => return Ok(solution),
                Err(e) => warn!("2Captcha 解决失败: {}", e),
            }
        }

        if let Some(api_key) = &self.anti_captcha_api_key {
            info!("使用 Anti-Captcha 服务解决验证码");
            match self.solve_with_anticaptcha(page, site_key, api_key).await {
                Ok(solution) => return Ok(solution),
                Err(e) => warn!("Anti-Captcha 解决失败: {}", e),
            }
        }

        if let Some(api_key) = &self.capmonster_api_key {
            info!("使用 CapMonster 服务解决验证码");
            match self.solve_with_capmonster(page, site_key, api_key).await {
                Ok(solution) => return Ok(solution),
                Err(e) => warn!("CapMonster 解决失败: {}", e),
            }
        }

        // 如果没有配置第三方服务，或者都失败了，尝试音频识别
        if self.enable_audio_fallback {
            info!("尝试使用音频识别解决验证码");
            match self.solve_with_audio_recognition(page).await {
                Ok(solution) => return Ok(solution),
                Err(e) => warn!("音频识别失败: {}", e),
            }
        }

        anyhow::bail!("所有验证码解决方法都失败了")
    }

    /// 使用 2Captcha 服务解决
    async fn solve_with_2captcha(
        &self,
        page: &Page,
        site_key: &str,
        api_key: &str,
    ) -> Result<CaptchaSolution> {
        let client = reqwest::Client::new();

        // 获取当前页面 URL
        let page_url = self.get_current_url(page).await?;

        // 1. 提交验证码任务
        info!("提交验证码任务到 2Captcha...");
        let submit_response = client
            .post("https://2captcha.com/in.php")
            .form(&[
                ("key", api_key),
                ("method", "userrecaptcha"),
                ("googlekey", site_key),
                ("pageurl", &page_url),
                ("json", "1"),
            ])
            .send()
            .await?;

        let submit_data: serde_json::Value = submit_response.json().await?;

        if submit_data["status"].as_i64() != Some(1) {
            anyhow::bail!(
                "提交验证码任务失败: {}",
                submit_data["request"].as_str().unwrap_or("unknown error")
            );
        }

        let captcha_id = submit_data["request"]
            .as_str()
            .context("无法获取验证码 ID")?;

        info!("验证码任务已提交，ID: {}", captcha_id);

        // 2. 轮询获取结果 (通常需要 10-30 秒)
        let mut attempts = 0;
        let max_attempts = 60; // 最多等待 2 分钟

        loop {
            attempts += 1;
            sleep(Duration::from_secs(2)).await;

            let result_response = client
                .get("https://2captcha.com/res.php")
                .query(&[
                    ("key", api_key),
                    ("action", "get"),
                    ("id", captcha_id),
                    ("json", "1"),
                ])
                .send()
                .await?;

            let result_data: serde_json::Value = result_response.json().await?;

            if result_data["status"].as_i64() == Some(1) {
                let token = result_data["request"]
                    .as_str()
                    .context("无法获取验证码 token")?
                    .to_string();

                info!("验证码解决成功！");

                return Ok(CaptchaSolution {
                    token,
                    solved_at: chrono::Utc::now().to_rfc3339(),
                    method: "2Captcha".to_string(),
                });
            }

            if attempts >= max_attempts {
                anyhow::bail!("等待验证码结果超时");
            }

            let status = result_data["request"]
                .as_str()
                .unwrap_or("CAPTCHA_NOT_READY");

            if status != "CAPTCHA_NOT_READY" {
                anyhow::bail!("验证码解决失败: {}", status);
            }
        }
    }

    /// 使用 Anti-Captcha 服务解决
    async fn solve_with_anticaptcha(
        &self,
        page: &Page,
        site_key: &str,
        api_key: &str,
    ) -> Result<CaptchaSolution> {
        let client = reqwest::Client::new();
        let page_url = self.get_current_url(page).await?;

        // 1. 创建任务
        info!("创建 Anti-Captcha 任务...");
        let create_task_request = serde_json::json!({
            "clientKey": api_key,
            "task": {
                "type": "NoCaptchaTaskProxyless",
                "websiteURL": page_url,
                "websiteKey": site_key
            }
        });

        let create_response = client
            .post("https://api.anti-captcha.com/createTask")
            .json(&create_task_request)
            .send()
            .await?;

        let create_data: serde_json::Value = create_response.json().await?;

        if create_data["errorId"].as_i64() != Some(0) {
            anyhow::bail!(
                "创建任务失败: {}",
                create_data["errorDescription"].as_str().unwrap_or("unknown")
            );
        }

        let task_id = create_data["taskId"].as_i64().context("无法获取任务 ID")?;

        info!("任务创建成功，ID: {}", task_id);

        // 2. 轮询获取结果
        let mut attempts = 0;
        let max_attempts = 60;

        loop {
            attempts += 1;
            sleep(Duration::from_secs(2)).await;

            let result_request = serde_json::json!({
                "clientKey": api_key,
                "taskId": task_id
            });

            let result_response = client
                .post("https://api.anti-captcha.com/getTaskResult")
                .json(&result_request)
                .send()
                .await?;

            let result_data: serde_json::Value = result_response.json().await?;

            if result_data["errorId"].as_i64() != Some(0) {
                anyhow::bail!(
                    "获取结果失败: {}",
                    result_data["errorDescription"].as_str().unwrap_or("unknown")
                );
            }

            let status = result_data["status"].as_str().unwrap_or("processing");

            if status == "ready" {
                let token = result_data["solution"]["gRecaptchaResponse"]
                    .as_str()
                    .context("无法获取验证码 token")?
                    .to_string();

                info!("验证码解决成功！");

                return Ok(CaptchaSolution {
                    token,
                    solved_at: chrono::Utc::now().to_rfc3339(),
                    method: "Anti-Captcha".to_string(),
                });
            }

            if attempts >= max_attempts {
                anyhow::bail!("等待验证码结果超时");
            }
        }
    }

    /// 使用 CapMonster 服务解决
    async fn solve_with_capmonster(
        &self,
        page: &Page,
        site_key: &str,
        api_key: &str,
    ) -> Result<CaptchaSolution> {
        let client = reqwest::Client::new();
        let page_url = self.get_current_url(page).await?;

        // CapMonster API 与 Anti-Captcha 兼容
        info!("创建 CapMonster 任务...");
        let create_task_request = serde_json::json!({
            "clientKey": api_key,
            "task": {
                "type": "NoCaptchaTaskProxyless",
                "websiteURL": page_url,
                "websiteKey": site_key
            }
        });

        let create_response = client
            .post("https://api.capmonster.cloud/createTask")
            .json(&create_task_request)
            .send()
            .await?;

        let create_data: serde_json::Value = create_response.json().await?;

        if create_data["errorId"].as_i64() != Some(0) {
            anyhow::bail!(
                "创建任务失败: {}",
                create_data["errorDescription"].as_str().unwrap_or("unknown")
            );
        }

        let task_id = create_data["taskId"].as_i64().context("无法获取任务 ID")?;

        info!("任务创建成功，ID: {}", task_id);

        // 轮询获取结果
        let mut attempts = 0;
        let max_attempts = 60;

        loop {
            attempts += 1;
            sleep(Duration::from_secs(2)).await;

            let result_request = serde_json::json!({
                "clientKey": api_key,
                "taskId": task_id
            });

            let result_response = client
                .post("https://api.capmonster.cloud/getTaskResult")
                .json(&result_request)
                .send()
                .await?;

            let result_data: serde_json::Value = result_response.json().await?;

            if result_data["errorId"].as_i64() != Some(0) {
                anyhow::bail!(
                    "获取结果失败: {}",
                    result_data["errorDescription"].as_str().unwrap_or("unknown")
                );
            }

            let status = result_data["status"].as_str().unwrap_or("processing");

            if status == "ready" {
                let token = result_data["solution"]["gRecaptchaResponse"]
                    .as_str()
                    .context("无法获取验证码 token")?
                    .to_string();

                info!("验证码解决成功！");

                return Ok(CaptchaSolution {
                    token,
                    solved_at: chrono::Utc::now().to_rfc3339(),
                    method: "CapMonster".to_string(),
                });
            }

            if attempts >= max_attempts {
                anyhow::bail!("等待验证码结果超时");
            }
        }
    }

    /// 使用音频识别解决 (实验性)
    /// Use audio recognition to solve (experimental)
    async fn solve_with_audio_recognition(&self, page: &Page) -> Result<CaptchaSolution> {
        info!("尝试音频识别解决 ReCAPTCHA...");

        // 1. 切换到音频挑战
        self.switch_to_audio_challenge(page).await?;

        // 2. 获取音频文件 URL
        let audio_url = self.get_audio_challenge_url(page).await?;
        info!("获取到音频挑战 URL: {}", audio_url);

        // 3. 下载音频文件
        let audio_data = self.download_audio(audio_url).await?;

        // 4. 使用语音识别服务识别音频
        // 可以使用 Google Speech-to-Text, Wit.ai, 或其他服务
        let text = self.recognize_audio(&audio_data).await?;
        info!("音频识别结果: {}", text);

        // 5. 输入识别结果
        self.submit_audio_answer(page, &text).await?;

        // 6. 获取 token
        let token = self.extract_recaptcha_token(page).await?;

        Ok(CaptchaSolution {
            token,
            solved_at: chrono::Utc::now().to_rfc3339(),
            method: "AudioRecognition".to_string(),
        })
    }

    /// 切换到音频挑战
    async fn switch_to_audio_challenge(&self, page: &Page) -> Result<()> {
        // 查找并点击音频按钮
        let script = r#"
            const audioButton = document.querySelector('#recaptcha-audio-button') ||
                               document.querySelector('.rc-button-audio');
            if (audioButton) {
                audioButton.click();
                return true;
            }
            return false;
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        if !result.as_bool().unwrap_or(false) {
            anyhow::bail!("无法找到音频按钮");
        }

        sleep(Duration::from_secs(2)).await;
        Ok(())
    }

    /// 获取音频挑战 URL
    async fn get_audio_challenge_url(&self, page: &Page) -> Result<String> {
        let script = r#"
            const audioSource = document.querySelector('.rc-audiochallenge-tdownload-link') ||
                              document.querySelector('audio source');
            return audioSource ? audioSource.src : null;
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        result
            .as_str()
            .map(|s| s.to_string())
            .context("无法获取音频 URL")
    }

    /// 下载音频文件
    async fn download_audio(&self, url: String) -> Result<Vec<u8>> {
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// 识别音频内容
    async fn recognize_audio(&self, _audio_data: &[u8]) -> Result<String> {
        // TODO: 集成实际的语音识别服务
        // 可以使用：
        // 1. Google Speech-to-Text API
        // 2. Wit.ai (Facebook)
        // 3. Azure Speech Services
        // 4. 本地 Vosk/Whisper 模型

        // 示例实现（需要实际的 API key）
        warn!("音频识别功能未完全实现，需要配置语音识别服务");
        anyhow::bail!("音频识别服务未配置")
    }

    /// 提交音频答案
    async fn submit_audio_answer(&self, page: &Page, answer: &str) -> Result<()> {
        let script = format!(
            r#"
            const input = document.querySelector('#audio-response');
            if (input) {{
                input.value = '{}';
                const button = document.querySelector('#recaptcha-verify-button');
                if (button) {{
                    button.click();
                    return true;
                }}
            }}
            return false;
        "#,
            answer
        );

        let result: serde_json::Value = page.evaluate(script.as_str()).await?.into_value()?;

        if !result.as_bool().unwrap_or(false) {
            anyhow::bail!("无法提交音频答案");
        }

        sleep(Duration::from_secs(2)).await;
        Ok(())
    }

    /// 注入 ReCAPTCHA token 到页面
    /// Inject ReCAPTCHA token into page
    pub async fn inject_recaptcha_token(&self, page: &Page, token: &str) -> Result<()> {
        info!("注入 ReCAPTCHA token 到页面...");

        let script = format!(
            r#"
            document.getElementById('g-recaptcha-response').innerHTML = '{}';
            if (typeof ___grecaptcha_cfg !== 'undefined') {{
                for (var key in ___grecaptcha_cfg.clients) {{
                    ___grecaptcha_cfg.clients[key].callback('{}');
                }}
            }}
        "#,
            token, token
        );

        page.evaluate(script.as_str()).await?;

        info!("Token 注入成功");
        Ok(())
    }

    /// 提取页面中的 ReCAPTCHA token
    async fn extract_recaptcha_token(&self, page: &Page) -> Result<String> {
        let script = r#"
            return document.getElementById('g-recaptcha-response')?.value || null;
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        result
            .as_str()
            .map(|s| s.to_string())
            .context("无法提取 ReCAPTCHA token")
    }

    /// 获取当前页面 URL
    async fn get_current_url(&self, page: &Page) -> Result<String> {
        let script = "return window.location.href;";
        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;
        result
            .as_str()
            .map(|s| s.to_string())
            .context("无法获取页面 URL")
    }

    /// 检测页面中的 ReCAPTCHA site key
    /// Detect ReCAPTCHA site key from page
    pub async fn detect_site_key(&self, page: &Page) -> Result<String> {
        let script = r#"
            // 方法 1: 从 iframe 中获取
            const iframes = document.querySelectorAll('iframe[src*="recaptcha"]');
            for (const iframe of iframes) {
                const src = iframe.src;
                const match = src.match(/k=([^&]+)/);
                if (match) return match[1];
            }
            
            // 方法 2: 从 div 中获取
            const div = document.querySelector('.g-recaptcha');
            if (div) {
                const sitekey = div.getAttribute('data-sitekey');
                if (sitekey) return sitekey;
            }
            
            // 方法 3: 从 window 对象中获取
            if (typeof grecaptcha !== 'undefined' && grecaptcha.enterprise) {
                // Enterprise version
                return grecaptcha.enterprise.getResponse();
            }
            
            return null;
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        result
            .as_str()
            .map(|s| s.to_string())
            .context("无法检测到 ReCAPTCHA site key")
    }
}

impl Default for RecaptchaSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置助手 - 从环境变量或配置文件加载 API keys
pub struct CaptchaSolverConfig {
    pub two_captcha_api_key: Option<String>,
    pub anti_captcha_api_key: Option<String>,
    pub capmonster_api_key: Option<String>,
}

impl CaptchaSolverConfig {
    pub fn from_env() -> Self {
        CaptchaSolverConfig {
            two_captcha_api_key: std::env::var("TWO_CAPTCHA_API_KEY").ok(),
            anti_captcha_api_key: std::env::var("ANTI_CAPTCHA_API_KEY").ok(),
            capmonster_api_key: std::env::var("CAPMONSTER_API_KEY").ok(),
        }
    }

    pub fn to_solver(self) -> RecaptchaSolver {
        let mut solver = RecaptchaSolver::new();

        if let Some(key) = self.two_captcha_api_key {
            solver = solver.with_2captcha_api_key(key);
        }
        if let Some(key) = self.anti_captcha_api_key {
            solver = solver.with_anti_captcha_api_key(key);
        }
        if let Some(key) = self.capmonster_api_key {
            solver = solver.with_capmonster_api_key(key);
        }

        solver
    }
}
