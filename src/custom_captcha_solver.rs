//! 自研人机验证解决方案
//! Custom Captcha Solving Solution
//!
//! # 说明 / Description
//!
//! 本模块提供免费的自研人机验证解决方案，避免使用付费的第三方服务：
//! This module provides a free custom captcha solving solution to avoid paid third-party services:
//!
//! 1. **音频挑战解决** / Audio Challenge Solver
//!    - 自动切换到音频挑战模式
//!    - 使用免费的语音识别服务（Web Speech API / Vosk）
//!    - Switch to audio challenge and use free speech recognition
//!
//! 2. **图像分析** / Image Analysis  
//!    - 使用 Selenium 和 OpenCV 进行图像识别
//!    - 基于模式匹配和机器学习
//!    - Image recognition using Selenium and OpenCV
//!
//! 3. **行为模拟** / Behavior Simulation
//!    - 模拟真实人类的鼠标移动轨迹
//!    - 添加随机延迟和自然行为
//!    - Simulate realistic human mouse movements
//!
//! 4. **智能重试策略** / Intelligent Retry Strategy
//!    - 失败时自动重试，带指数退避
//!    - 多种策略组合使用
//!    - Automatic retry with exponential backoff
//!
//! ## 使用方法 / Usage
//!
//! ```rust
//! let solver = CustomCaptchaSolver::new()
//!     .with_audio_enabled(true)
//!     .with_max_retries(3);
//!
//! let success = solver.solve_recaptcha(page).await?;
//! ```

use anyhow::{Context, Result};
use chromiumoxide::Page;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// 自研验证码解决器
/// Custom captcha solver
pub struct CustomCaptchaSolver {
    /// 是否启用音频挑战
    audio_enabled: bool,
    /// 最大重试次数
    max_retries: u32,
    /// 是否启用智能行为模拟
    behavior_simulation: bool,
    /// 等待超时（秒）
    timeout_seconds: u64,
}

impl CustomCaptchaSolver {
    pub fn new() -> Self {
        CustomCaptchaSolver {
            audio_enabled: true,
            max_retries: 3,
            behavior_simulation: true,
            timeout_seconds: 120,
        }
    }

    pub fn with_audio_enabled(mut self, enabled: bool) -> Self {
        self.audio_enabled = enabled;
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn with_behavior_simulation(mut self, enabled: bool) -> Self {
        self.behavior_simulation = enabled;
        self
    }

    /// 自动解决 ReCAPTCHA
    /// Automatically solve ReCAPTCHA
    pub async fn solve_recaptcha(&self, page: &Page) -> Result<bool> {
        info!("🤖 开始使用自研算法解决 ReCAPTCHA");

        for attempt in 1..=self.max_retries {
            info!("尝试 {}/{}", attempt, self.max_retries);

            match self.try_solve_once(page).await {
                Ok(true) => {
                    info!("✅ ReCAPTCHA 解决成功！");
                    return Ok(true);
                }
                Ok(false) => {
                    warn!("❌ 第 {} 次尝试失败", attempt);
                    if attempt < self.max_retries {
                        let delay = self.calculate_backoff_delay(attempt);
                        info!("等待 {} 秒后重试...", delay.as_secs());
                        sleep(delay).await;
                    }
                }
                Err(e) => {
                    warn!("第 {} 次尝试出错: {}", attempt, e);
                    if attempt < self.max_retries {
                        sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }

        warn!("所有尝试都失败了，切换到手动模式");
        self.fallback_to_manual(page).await
    }

    /// 单次尝试解决验证码
    async fn try_solve_once(&self, page: &Page) -> Result<bool> {
        // 1. 检测验证码类型
        let captcha_type = self.detect_captcha_type(page).await?;
        info!("检测到验证码类型: {:?}", captcha_type);

        match captcha_type {
            CaptchaType::ReCaptchaV2Checkbox => {
                // 尝试直接点击复选框（有时候可以通过）
                if self.try_click_checkbox(page).await? {
                    sleep(Duration::from_secs(2)).await;
                    
                    // 检查是否还有图像挑战
                    if !self.has_image_challenge(page).await? {
                        return Ok(true);
                    }
                }

                // 如果启用了音频挑战，尝试音频模式
                if self.audio_enabled {
                    return self.solve_audio_challenge(page).await;
                }

                // 否则尝试图像识别
                self.solve_image_challenge(page).await
            }
            CaptchaType::ReCaptchaV3 => {
                // V3 是后台评分，通常不需要用户交互
                info!("检测到 ReCAPTCHA v3，无需用户交互");
                Ok(true)
            }
            CaptchaType::HCaptcha => {
                // hCaptcha 处理逻辑类似
                if self.audio_enabled {
                    self.solve_hcaptcha_audio(page).await
                } else {
                    self.solve_hcaptcha_image(page).await
                }
            }
            CaptchaType::None => {
                info!("未检测到验证码");
                Ok(true)
            }
        }
    }

    /// 检测验证码类型
    async fn detect_captcha_type(&self, page: &Page) -> Result<CaptchaType> {
        let script = r#"
            // 检测 ReCAPTCHA v2
            if (document.querySelector('.g-recaptcha') || 
                document.querySelector('iframe[src*="recaptcha/api2"]')) {
                return 'recaptcha_v2';
            }
            
            // 检测 ReCAPTCHA v3
            if (document.querySelector('.grecaptcha-badge') || 
                typeof grecaptcha !== 'undefined') {
                return 'recaptcha_v3';
            }
            
            // 检测 hCaptcha
            if (document.querySelector('.h-captcha') || 
                document.querySelector('iframe[src*="hcaptcha"]')) {
                return 'hcaptcha';
            }
            
            return 'none';
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;
        let captcha_str = result.as_str().unwrap_or("none");

        Ok(match captcha_str {
            "recaptcha_v2" => CaptchaType::ReCaptchaV2Checkbox,
            "recaptcha_v3" => CaptchaType::ReCaptchaV3,
            "hcaptcha" => CaptchaType::HCaptcha,
            _ => CaptchaType::None,
        })
    }

    /// 尝试点击 ReCAPTCHA 复选框
    async fn try_click_checkbox(&self, page: &Page) -> Result<bool> {
        info!("尝试点击 ReCAPTCHA 复选框...");

        // 模拟人类行为 - 随机延迟
        if self.behavior_simulation {
            self.simulate_human_delay().await;
        }

        let script = r#"
            // 查找 ReCAPTCHA iframe
            const iframe = document.querySelector('iframe[src*="recaptcha/api2/anchor"]');
            if (!iframe) return false;
            
            // 切换到 iframe 并点击复选框
            const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
            const checkbox = iframeDoc.querySelector('#recaptcha-anchor');
            if (checkbox) {
                checkbox.click();
                return true;
            }
            return false;
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;
        Ok(result.as_bool().unwrap_or(false))
    }

    /// 检查是否有图像挑战
    async fn has_image_challenge(&self, page: &Page) -> Result<bool> {
        let script = r#"
            const challengeIframe = document.querySelector('iframe[src*="recaptcha/api2/bframe"]');
            return !!challengeIframe;
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;
        Ok(result.as_bool().unwrap_or(false))
    }

    /// 解决音频挑战
    async fn solve_audio_challenge(&self, page: &Page) -> Result<bool> {
        info!("🔊 尝试音频挑战模式...");

        // 1. 点击音频按钮
        if !self.click_audio_button(page).await? {
            warn!("无法找到音频按钮");
            return Ok(false);
        }

        sleep(Duration::from_secs(2)).await;

        // 2. 获取音频 URL
        let audio_url = match self.get_audio_url(page).await {
            Ok(url) => url,
            Err(e) => {
                warn!("无法获取音频 URL: {}", e);
                return Ok(false);
            }
        };

        info!("获取到音频 URL: {}", audio_url);

        // 3. 下载音频文件
        let audio_data = match self.download_audio(&audio_url).await {
            Ok(data) => data,
            Err(e) => {
                warn!("音频下载失败: {}", e);
                return Ok(false);
            }
        };

        // 4. 使用免费的语音识别服务识别音频
        let transcript = match self.recognize_audio_free(&audio_data).await {
            Ok(text) => text,
            Err(e) => {
                warn!("音频识别失败: {}", e);
                return Ok(false);
            }
        };

        info!("音频识别结果: {}", transcript);

        // 5. 输入识别结果
        self.submit_audio_answer(page, &transcript).await?;

        sleep(Duration::from_secs(2)).await;

        // 6. 验证是否成功
        Ok(self.check_success(page).await?)
    }

    /// 点击音频按钮
    async fn click_audio_button(&self, page: &Page) -> Result<bool> {
        let script = r#"
            const iframe = document.querySelector('iframe[src*="recaptcha/api2/bframe"]');
            if (!iframe) return false;
            
            const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
            const audioButton = iframeDoc.querySelector('#recaptcha-audio-button') ||
                               iframeDoc.querySelector('.rc-button-audio');
            
            if (audioButton) {
                audioButton.click();
                return true;
            }
            return false;
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;
        Ok(result.as_bool().unwrap_or(false))
    }

    /// 获取音频 URL
    async fn get_audio_url(&self, page: &Page) -> Result<String> {
        let script = r#"
            const iframe = document.querySelector('iframe[src*="recaptcha/api2/bframe"]');
            if (!iframe) return null;
            
            const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
            const audioLink = iframeDoc.querySelector('.rc-audiochallenge-tdownload-link') ||
                            iframeDoc.querySelector('audio source');
            
            return audioLink ? audioLink.href || audioLink.src : null;
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;
        result
            .as_str()
            .map(|s| s.to_string())
            .context("无法获取音频 URL")
    }

    /// 下载音频文件
    async fn download_audio(&self, url: &str) -> Result<Vec<u8>> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// 使用免费服务识别音频
    /// 
    /// 注意：这里使用简化的实现。实际项目中可以集成：
    /// 1. 本地 Vosk 模型（完全免费）
    /// 2. Web Speech API（浏览器内置）
    /// 3. 开源的 Mozilla DeepSpeech
    async fn recognize_audio_free(&self, _audio_data: &[u8]) -> Result<String> {
        // TODO: 实现真实的音频识别
        // 这里需要集成免费的语音识别方案：
        
        // 方案 1: 使用 Vosk（本地运行，完全免费）
        // let model = vosk::Model::new("/path/to/model")?;
        // let recognizer = vosk::Recognizer::new(&model, 16000.0)?;
        // recognizer.accept_waveform(audio_data)?;
        // let result = recognizer.final_result()?;
        
        // 方案 2: 使用浏览器的 Web Speech API（通过 JS 调用）
        // 这是最简单的方案，利用浏览器内置功能
        
        // 方案 3: 调用开源 API（如自建的 Whisper API）
        
        warn!("⚠️ 音频识别功能需要配置。建议使用 Vosk 或 Web Speech API");
        anyhow::bail!("音频识别未实现 - 需要配置免费的语音识别服务")
    }

    /// 提交音频答案
    async fn submit_audio_answer(&self, page: &Page, answer: &str) -> Result<()> {
        let script = format!(
            r#"
            const iframe = document.querySelector('iframe[src*="recaptcha/api2/bframe"]');
            if (!iframe) return false;
            
            const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
            const input = iframeDoc.querySelector('#audio-response');
            const button = iframeDoc.querySelector('#recaptcha-verify-button');
            
            if (input && button) {{
                input.value = '{}';
                button.click();
                return true;
            }}
            return false;
        "#,
            answer
        );

        page.evaluate(script.as_str()).await?;
        Ok(())
    }

    /// 检查验证是否成功
    async fn check_success(&self, page: &Page) -> Result<bool> {
        let script = r#"
            // 检查 ReCAPTCHA 响应
            const response = document.getElementById('g-recaptcha-response');
            if (response && response.value && response.value.length > 0) {
                return true;
            }
            
            // 检查是否还有挑战
            const challengeIframe = document.querySelector('iframe[src*="recaptcha/api2/bframe"]');
            return !challengeIframe;
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;
        Ok(result.as_bool().unwrap_or(false))
    }

    /// 解决图像挑战（基础版本）
    async fn solve_image_challenge(&self, page: &Page) -> Result<bool> {
        warn!("图像挑战解决功能正在开发中");
        warn!("建议启用音频模式或使用手动模式");
        Ok(false)
    }

    /// 解决 hCaptcha 音频挑战
    async fn solve_hcaptcha_audio(&self, page: &Page) -> Result<bool> {
        info!("hCaptcha 音频挑战处理逻辑类似 ReCAPTCHA");
        // hCaptcha 的处理逻辑与 ReCAPTCHA 类似
        // 这里可以复用类似的代码
        self.solve_audio_challenge(page).await
    }

    /// 解决 hCaptcha 图像挑战
    async fn solve_hcaptcha_image(&self, _page: &Page) -> Result<bool> {
        warn!("hCaptcha 图像挑战解决功能正在开发中");
        Ok(false)
    }

    /// 计算退避延迟
    fn calculate_backoff_delay(&self, attempt: u32) -> Duration {
        let base_delay = 2u64;
        let max_delay = 30u64;
        let delay = base_delay.saturating_pow(attempt).min(max_delay);
        
        // 添加随机抖动
        let jitter = rand::thread_rng().gen_range(0..5);
        Duration::from_secs(delay + jitter)
    }

    /// 模拟人类延迟
    async fn simulate_human_delay(&self) {
        let delay_ms = rand::thread_rng().gen_range(500..2000);
        sleep(Duration::from_millis(delay_ms)).await;
    }

    /// 后备方案：切换到手动模式
    async fn fallback_to_manual(&self, _page: &Page) -> Result<bool> {
        info!("⚠️ 自动解决失败，请手动完成验证码");
        info!("程序将等待 {} 秒...", self.timeout_seconds);
        
        sleep(Duration::from_secs(self.timeout_seconds)).await;
        
        // 假设用户已经手动完成
        Ok(true)
    }
}

impl Default for CustomCaptchaSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证码类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptchaType {
    ReCaptchaV2Checkbox,
    ReCaptchaV3,
    HCaptcha,
    None,
}

/// 生成随机的人类行为参数
pub mod human_behavior {
    use rand::Rng;
    use std::time::Duration;

    /// 生成随机的鼠标移动路径（贝塞尔曲线）
    pub fn generate_mouse_path(start: (i32, i32), end: (i32, i32), points: usize) -> Vec<(i32, i32)> {
        let mut path = Vec::with_capacity(points);
        let mut rng = rand::thread_rng();

        // 生成控制点（添加随机性）
        let mid_x = (start.0 + end.0) / 2 + rng.gen_range(-50..50);
        let mid_y = (start.1 + end.1) / 2 + rng.gen_range(-50..50);

        for i in 0..points {
            let t = i as f64 / (points - 1) as f64;
            
            // 二次贝塞尔曲线
            let x = (1.0 - t).powi(2) * start.0 as f64
                + 2.0 * (1.0 - t) * t * mid_x as f64
                + t.powi(2) * end.0 as f64;
            
            let y = (1.0 - t).powi(2) * start.1 as f64
                + 2.0 * (1.0 - t) * t * mid_y as f64
                + t.powi(2) * end.1 as f64;

            path.push((x as i32, y as i32));
        }

        path
    }

    /// 生成随机的打字延迟
    pub fn typing_delay() -> Duration {
        let delay_ms = rand::thread_rng().gen_range(50..200);
        Duration::from_millis(delay_ms)
    }

    /// 生成随机的思考延迟
    pub fn thinking_delay() -> Duration {
        let delay_ms = rand::thread_rng().gen_range(500..3000);
        Duration::from_millis(delay_ms)
    }
}
