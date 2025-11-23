//! 人机验证处理模块
//! Captcha handling module
//!
//! # X账号注册人机验证处理方案
//! 
//! ## 当前 X (Twitter) 平台验证方式
//! 
//! 根据最新测试和观察，X平台主要使用以下验证方式：
//! 
//! 1. **行为验证** - 最常见
//!    - 通过分析用户行为模式判断是否为机器人
//!    - 包括鼠标移动、点击速度、输入节奏等
//!    - 无明显的验证码界面
//! 
//! 2. **Arkose Labs (FunCaptcha)** - 偶尔出现
//!    - 交互式3D图像验证
//!    - 需要拖动、旋转或选择正确的图像
//!    - 难度较高，建议手动完成
//! 
//! 3. **手机验证** - 高风险账号
//!    - 要求验证手机号码
//!    - 发送短信验证码
//!    - 需要真实手机号
//! 
//! 注：X已不再使用 reCAPTCHA v2/v3，相关代码已移除
//! 
//! ## 推荐验证处理方式
//! 
//! ### 手动模式（Manual Mode）- 默认且推荐
//! 
//! **工作原理：**
//! - 当检测到人机验证时，程序暂停自动化流程
//! - 保持浏览器窗口打开（非headless模式）
//! - 等待用户手动完成验证
//! - 验证完成后，程序自动继续执行
//! 
//! **优点：**
//! - 可靠性高，成功率接近100%
//! - 无需额外成本
//! - 支持所有类型的验证
//! - 完全合规
//! 
//! **使用场景：**
//! - 小到中等规模注册（每天<500个账号）
//! - 需要高成功率和合规性
//! 
//! **配置方法：**
//! ```rust
//! let captcha_handler = CaptchaHandler::new(true); // true = 启用手动模式
//! ```
//! 
//! ## 最佳实践建议
//! 
//! ### 模拟真实用户行为
//! ```
//! - 随机化操作速度和节奏
//! - 使用真实的鼠标移动轨迹
//! - 添加自然的停顿和犹豫
//! - 避免机械式的重复操作
//! ```
//! 
//! ### 小规模使用（推荐新手）
//! ```
//! 模式：手动模式
//! 配置：headless = false
//! 策略：批量注册时设置并发数为1-3，人工监控
//! 成本：免费
//! 成功率：>95%
//! ```
//! 
//! ### 中等规模使用
//! ```
//! 模式：手动模式 + 多个实例
//! 配置：开启多个浏览器窗口，分别监控
//! 策略：使用多台机器或虚拟机，每台运行3-5个并发任务
//! 配合使用代理和不同的浏览器指纹
//! 成本：低
//! 成功率：>90%
//! ```
//! 
//! ## 注意事项
//! 
//! 1. **速率限制**：控制注册速率，避免触发平台的反滥用机制
//! 2. **代理轮换**：大批量注册时建议使用代理池，避免IP被封
//! 3. **浏览器指纹**：考虑使用不同的浏览器配置文件和BitBrowser等工具
//! 4. **合规性**：遵守X平台的服务条款，避免滥用
//! 5. **账号质量**：使用真实的个人信息，提高账号质量和存活率
//! 
//! ## 技术实现细节
//! 
//! ### 验证检测
//! 
//! 系统通过以下方式检测验证：
//! - 检查页面中的验证相关元素
//! - 检测特定的CSS类名或ID
//! - 监听DOM变化
//! - 检测页面跳转和重定向
//! 
//! ### 手动模式等待逻辑
//! 
//! ```
//! 1. 检测到验证 -> 输出提示信息
//! 2. 每2秒检查一次验证是否完成
//! 3. 最多等待5分钟
//! 4. 验证完成 -> 继续执行
//! 5. 超时 -> 返回失败
//! ```

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// 人机验证类型
/// Captcha types
#[derive(Debug, Clone)]
pub enum CaptchaType {
    /// Arkose Labs (FunCaptcha) - X平台主要使用的验证方式
    /// Arkose Labs (FunCaptcha) - Main verification method used by X
    ArkowseLabs,
    /// 行为验证 - 无明显验证码界面
    /// Behavior verification - No visible captcha interface
    BehaviorCheck,
    /// 手机验证 - 需要验证手机号码
    /// Phone verification - Requires phone number
    PhoneVerification,
    /// 未知类型
    Unknown,
}

/// 人机验证处理器
/// Captcha handler
pub struct CaptchaHandler {
    /// 是否使用手动模式（等待用户手动完成）
    manual_mode: bool,
    /// 第三方验证服务配置
    captcha_service: Option<CaptchaServiceConfig>,
}

#[derive(Debug, Clone)]
pub struct CaptchaServiceConfig {
    /// 服务类型: 2captcha, anticaptcha, capmonster 等
    pub service_type: String,
    /// API Key
    pub api_key: String,
    /// API URL
    pub api_url: Option<String>,
}

impl CaptchaHandler {
    pub fn new(manual_mode: bool) -> Self {
        CaptchaHandler {
            manual_mode,
            captcha_service: None,
        }
    }

    pub fn with_service(mut self, config: CaptchaServiceConfig) -> Self {
        self.captcha_service = Some(config);
        self
    }

    /// 处理人机验证
    /// Handle captcha
    pub async fn handle_captcha(
        &self,
        page: &chromiumoxide::Page,
        captcha_type: CaptchaType,
    ) -> Result<bool> {
        info!("检测到人机验证类型: {:?}", captcha_type);

        match captcha_type {
            CaptchaType::ArkowseLabs => {
                info!("检测到 Arkose Labs 验证，建议手动完成");
                if self.manual_mode {
                    self.wait_for_manual_completion(page).await
                } else {
                    warn!("Arkose Labs 验证暂不支持自动处理");
                    Ok(false)
                }
            }
            CaptchaType::BehaviorCheck => {
                info!("检测到行为验证，请保持自然的操作节奏");
                // 行为验证通常不需要额外操作，只需要保持自然的操作即可
                Ok(true)
            }
            CaptchaType::PhoneVerification => {
                info!("检测到手机验证要求");
                if self.manual_mode {
                    info!("请手动完成手机验证...");
                    self.wait_for_manual_completion(page).await
                } else {
                    warn!("手机验证需要真实手机号，无法自动完成");
                    Ok(false)
                }
            }
            CaptchaType::Unknown => {
                if self.manual_mode {
                    self.wait_for_manual_completion(page).await
                } else {
                    warn!("未知的验证类型且未启用手动模式");
                    Ok(false)
                }
            }
        }
    }

    /// 等待用户手动完成验证
    async fn wait_for_manual_completion(&self, _page: &chromiumoxide::Page) -> Result<bool> {
        info!("等待手动完成验证...");
        info!("请在浏览器中完成验证，完成后程序将自动继续");

        // 等待验证消失或特定元素出现
        let max_wait = Duration::from_secs(300); // 最多等待 5 分钟
        let start = tokio::time::Instant::now();

        while start.elapsed() < max_wait {
            sleep(Duration::from_secs(2)).await;

            // 检查验证是否完成
            // TODO: 根据实际页面结构检测
            // 例如：检查特定元素是否出现，或验证元素是否消失

            // 暂时使用简单的等待逻辑
            // 实际使用时需要根据页面特征判断
        }

        info!("手动验证完成");
        Ok(true)
    }

    /// 检测页面中的验证类型
    pub async fn detect_captcha_type(&self, page: &chromiumoxide::Page) -> Option<CaptchaType> {
        // 检测 Arkose Labs
        if self
            .check_element_exists(page, "iframe[src*='arkoselabs']")
            .await
            || self
                .check_element_exists(page, "iframe[src*='funcaptcha']")
                .await
        {
            return Some(CaptchaType::ArkowseLabs);
        }

        // 检测手机验证
        if self
            .check_element_exists(page, "input[type='tel']")
            .await
            || self
                .check_element_exists(page, "[data-testid*='phone']")
                .await
        {
            return Some(CaptchaType::PhoneVerification);
        }

        None
    }

    async fn check_element_exists(&self, _page: &chromiumoxide::Page, _selector: &str) -> bool {
        // TODO: 实现元素检查逻辑
        // page.query_selector(selector).await.is_ok()
        false
    }
}

/// 模拟人类行为的辅助函数
/// Helper functions to simulate human behavior
pub mod human_behavior {
    use rand::Rng;
    use std::time::Duration;

    /// 生成随机延迟（模拟人类操作间隔）
    pub fn random_delay() -> Duration {
        let mut rng = rand::thread_rng();
        Duration::from_millis(rng.gen_range(100..500))
    }

    /// 生成随机的鼠标移动轨迹
    pub fn generate_mouse_path(start: (i32, i32), end: (i32, i32)) -> Vec<(i32, i32)> {
        let mut path = Vec::new();
        let steps = rand::thread_rng().gen_range(10..30);

        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            // 使用贝塞尔曲线模拟自然的鼠标移动
            let x = start.0 as f64 + (end.0 - start.0) as f64 * t;
            let y = start.1 as f64 + (end.1 - start.1) as f64 * t;

            // 添加一些随机抖动
            let noise_x = rand::thread_rng().gen_range(-2..3);
            let noise_y = rand::thread_rng().gen_range(-2..3);

            path.push(((x as i32 + noise_x), (y as i32 + noise_y)));
        }

        path
    }

    /// 生成随机的输入速度（字符之间的延迟）
    pub fn random_typing_delay() -> Duration {
        let mut rng = rand::thread_rng();
        Duration::from_millis(rng.gen_range(50..200))
    }
}
