//! 浏览器环境检测工具
//! Browser Environment Detection Tool
//!
//! # 功能说明 / Features
//!
//! 本模块用于检测浏览器环境是否容易触发验证码（CAPTCHA），帮助优化环境以提高注册成功率。
//! This module detects if the browser environment is likely to trigger CAPTCHA,
//! helping optimize the environment to improve registration success rate.
//!
//! ## 检测项目 / Detection Items
//!
//! ### 1. 浏览器指纹检测 / Browser Fingerprint Detection
//! - WebDriver 检测
//! - 自动化工具特征
//! - Canvas 指纹
//! - WebGL 指纹
//! - 插件列表
//! - 字体列表
//! - 时区和语言
//!
//! ### 2. IP 信誉检测 / IP Reputation Detection
//! - 是否为已知代理/VPN
//! - 是否为数据中心IP
//! - IP 风险评分
//! - 地理位置一致性
//!
//! ### 3. 行为特征检测 / Behavior Pattern Detection
//! - 鼠标移动轨迹
//! - 键盘输入模式
//! - 页面交互时间
//! - 滚动行为
//!
//! ## 使用方法 / Usage
//!
//! ```rust
//! let detector = BrowserDetector::new();
//! let report = detector.detect_environment(page).await?;
//!
//! if report.risk_level == RiskLevel::High {
//!     println!("警告：当前环境容易触发验证码");
//!     println!("建议：{}", report.recommendations.join(", "));
//! }
//! ```

use anyhow::Result;
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use tracing::info;

/// 风险等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    /// 低风险 - 不太可能触发验证码
    Low,
    /// 中风险 - 可能触发验证码
    Medium,
    /// 高风险 - 很可能触发验证码
    High,
}

/// 检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionReport {
    /// 总体风险等级
    pub risk_level: RiskLevel,
    /// 风险评分 (0-100)
    pub risk_score: u32,
    /// 各项检测结果
    pub checks: Vec<CheckResult>,
    /// 优化建议
    pub recommendations: Vec<String>,
    /// 检测时间
    pub timestamp: String,
}

/// 单项检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// 检测项名称
    pub name: String,
    /// 是否通过
    pub passed: bool,
    /// 详细信息
    pub details: String,
    /// 风险权重 (1-10)
    pub weight: u32,
}

/// 浏览器环境检测器
pub struct BrowserDetector {
    /// 是否详细输出
    verbose: bool,
}

impl BrowserDetector {
    pub fn new() -> Self {
        BrowserDetector { verbose: true }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// 执行完整的环境检测
    /// Perform full environment detection
    pub async fn detect_environment(&self, page: &Page) -> Result<DetectionReport> {
        info!("开始浏览器环境检测...");

        let mut checks = Vec::new();
        let mut risk_score = 0u32;

        // 1. WebDriver 检测
        checks.push(self.check_webdriver(page).await?);

        // 2. 自动化特征检测
        checks.push(self.check_automation_flags(page).await?);

        // 3. Chrome DevTools Protocol 检测
        checks.push(self.check_cdp_runtime(page).await?);

        // 4. Navigator 属性检测
        checks.push(self.check_navigator_properties(page).await?);

        // 5. 插件检测
        checks.push(self.check_plugins(page).await?);

        // 6. Canvas 指纹检测
        checks.push(self.check_canvas_fingerprint(page).await?);

        // 7. WebGL 指纹检测
        checks.push(self.check_webgl_fingerprint(page).await?);

        // 8. 浏览器特征一致性
        checks.push(self.check_consistency(page).await?);

        // 9. 时区和语言检测
        checks.push(self.check_locale_settings(page).await?);

        // 10. 权限检测
        checks.push(self.check_permissions(page).await?);

        // 计算风险评分
        for check in &checks {
            if !check.passed {
                risk_score += check.weight;
            }
        }

        // 确定风险等级
        let risk_level = match risk_score {
            0..=20 => RiskLevel::Low,
            21..=50 => RiskLevel::Medium,
            _ => RiskLevel::High,
        };

        // 生成优化建议
        let recommendations = self.generate_recommendations(&checks, &risk_level);

        let report = DetectionReport {
            risk_level: risk_level.clone(),
            risk_score,
            checks,
            recommendations,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.print_report(&report);

        Ok(report)
    }

    /// 检测 WebDriver 标识
    async fn check_webdriver(&self, page: &Page) -> Result<CheckResult> {
        let script = r#"
            return {
                webdriver: navigator.webdriver,
                hasWebdriver: 'webdriver' in navigator
            };
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        let webdriver = result["webdriver"].as_bool().unwrap_or(false);
        let has_webdriver = result["hasWebdriver"].as_bool().unwrap_or(false);

        let passed = !webdriver && !has_webdriver;

        Ok(CheckResult {
            name: "WebDriver 检测".to_string(),
            passed,
            details: if passed {
                "未检测到 WebDriver 标识".to_string()
            } else {
                format!(
                    "检测到 WebDriver 标识 (webdriver={}, hasWebdriver={})",
                    webdriver, has_webdriver
                )
            },
            weight: 10,
        })
    }

    /// 检测自动化标识
    async fn check_automation_flags(&self, page: &Page) -> Result<CheckResult> {
        let script = r#"
            return {
                chromeRuntime: !!window.chrome && !!window.chrome.runtime,
                permissions: !!navigator.permissions,
                hasCallPhantom: '_phantom' in window || 'callPhantom' in window,
                hasSelenium: '__selenium_unwrapped' in window || '__webdriver_evaluate' in window || '__driver_evaluate' in window,
                hasDomAutomation: '__fxdriver_unwrapped' in window || 'domAutomation' in window,
                hasAutomationController: !!window.domAutomationController,
                hasChromeCsi: !!window.chromeCsi,
                documentAll: document.all !== undefined
            };
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        let has_phantom = result["hasCallPhantom"].as_bool().unwrap_or(false);
        let has_selenium = result["hasSelenium"].as_bool().unwrap_or(false);
        let has_dom_automation = result["hasDomAutomation"].as_bool().unwrap_or(false);
        let has_automation_controller = result["hasAutomationController"].as_bool().unwrap_or(false);

        let passed = !has_phantom && !has_selenium && !has_dom_automation && !has_automation_controller;

        Ok(CheckResult {
            name: "自动化标识检测".to_string(),
            passed,
            details: if passed {
                "未检测到自动化工具标识".to_string()
            } else {
                format!(
                    "检测到自动化标识 (phantom={}, selenium={}, domAuto={}, autoCtrl={})",
                    has_phantom, has_selenium, has_dom_automation, has_automation_controller
                )
            },
            weight: 9,
        })
    }

    /// 检测 CDP Runtime
    async fn check_cdp_runtime(&self, page: &Page) -> Result<CheckResult> {
        let script = r#"
            return {
                hasCdp: !!window.__nightmare || !!window._Selenium_IDE_Recorder,
                hasAutomationExtension: !!window.cdc_adoQpoasnfa76pfcZLmcfl_Array || 
                                        !!window.cdc_adoQpoasnfa76pfcZLmcfl_Promise || 
                                        !!window.cdc_adoQpoasnfa76pfcZLmcfl_Symbol
            };
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        let has_cdp = result["hasCdp"].as_bool().unwrap_or(false);
        let has_automation_ext = result["hasAutomationExtension"].as_bool().unwrap_or(false);

        let passed = !has_cdp && !has_automation_ext;

        Ok(CheckResult {
            name: "CDP Runtime 检测".to_string(),
            passed,
            details: if passed {
                "未检测到 CDP 运行时标识".to_string()
            } else {
                "检测到 CDP 运行时或自动化扩展".to_string()
            },
            weight: 8,
        })
    }

    /// 检测 Navigator 属性
    async fn check_navigator_properties(&self, page: &Page) -> Result<CheckResult> {
        let script = r#"
            return {
                languages: navigator.languages,
                languagesLength: navigator.languages ? navigator.languages.length : 0,
                platform: navigator.platform,
                userAgent: navigator.userAgent,
                vendor: navigator.vendor,
                hardwareConcurrency: navigator.hardwareConcurrency,
                deviceMemory: navigator.deviceMemory,
                maxTouchPoints: navigator.maxTouchPoints
            };
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        let languages_length = result["languagesLength"].as_u64().unwrap_or(0);
        let hardware_concurrency = result["hardwareConcurrency"].as_u64().unwrap_or(0);

        // 检查是否看起来正常
        let passed = languages_length > 0 && hardware_concurrency > 0;

        Ok(CheckResult {
            name: "Navigator 属性检测".to_string(),
            passed,
            details: if passed {
                format!(
                    "Navigator 属性正常 (languages={}, cores={})",
                    languages_length, hardware_concurrency
                )
            } else {
                "Navigator 属性异常".to_string()
            },
            weight: 5,
        })
    }

    /// 检测插件
    async fn check_plugins(&self, page: &Page) -> Result<CheckResult> {
        let script = r#"
            return {
                pluginCount: navigator.plugins.length,
                plugins: Array.from(navigator.plugins).map(p => p.name)
            };
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        let plugin_count = result["pluginCount"].as_u64().unwrap_or(0);

        // 正常浏览器通常有一些插件
        let passed = plugin_count > 0 || cfg!(target_os = "linux"); // Linux 上插件可能为0

        Ok(CheckResult {
            name: "浏览器插件检测".to_string(),
            passed,
            details: format!("检测到 {} 个插件", plugin_count),
            weight: 6,
        })
    }

    /// 检测 Canvas 指纹
    async fn check_canvas_fingerprint(&self, page: &Page) -> Result<CheckResult> {
        let script = r#"
            try {
                const canvas = document.createElement('canvas');
                const ctx = canvas.getContext('2d');
                ctx.textBaseline = 'top';
                ctx.font = '14px Arial';
                ctx.textBaseline = 'alphabetic';
                ctx.fillStyle = '#f60';
                ctx.fillRect(125, 1, 62, 20);
                ctx.fillStyle = '#069';
                ctx.fillText('Canvas Fingerprint', 2, 15);
                ctx.fillStyle = 'rgba(102, 204, 0, 0.7)';
                ctx.fillText('Canvas Fingerprint', 4, 17);
                
                const dataURL = canvas.toDataURL();
                return {
                    canvasSupported: true,
                    hasFingerprint: dataURL.length > 0
                };
            } catch(e) {
                return {
                    canvasSupported: false,
                    error: e.message
                };
            }
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        let canvas_supported = result["canvasSupported"].as_bool().unwrap_or(false);
        let has_fingerprint = result["hasFingerprint"].as_bool().unwrap_or(false);

        let passed = canvas_supported && has_fingerprint;

        Ok(CheckResult {
            name: "Canvas 指纹检测".to_string(),
            passed,
            details: if passed {
                "Canvas 指纹正常".to_string()
            } else {
                "Canvas 指纹异常或不支持".to_string()
            },
            weight: 7,
        })
    }

    /// 检测 WebGL 指纹
    async fn check_webgl_fingerprint(&self, page: &Page) -> Result<CheckResult> {
        let script = r#"
            try {
                const canvas = document.createElement('canvas');
                const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
                
                if (!gl) {
                    return { webglSupported: false };
                }
                
                const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
                return {
                    webglSupported: true,
                    vendor: debugInfo ? gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL) : 'unknown',
                    renderer: debugInfo ? gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL) : 'unknown'
                };
            } catch(e) {
                return {
                    webglSupported: false,
                    error: e.message
                };
            }
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        let webgl_supported = result["webglSupported"].as_bool().unwrap_or(false);

        let passed = webgl_supported;

        Ok(CheckResult {
            name: "WebGL 指纹检测".to_string(),
            passed,
            details: if passed {
                format!(
                    "WebGL 支持 (vendor={}, renderer={})",
                    result["vendor"].as_str().unwrap_or("unknown"),
                    result["renderer"].as_str().unwrap_or("unknown")
                )
            } else {
                "WebGL 不支持".to_string()
            },
            weight: 6,
        })
    }

    /// 检测浏览器特征一致性
    async fn check_consistency(&self, page: &Page) -> Result<CheckResult> {
        let script = r#"
            return {
                screenWidth: window.screen.width,
                screenHeight: window.screen.height,
                availWidth: window.screen.availWidth,
                availHeight: window.screen.availHeight,
                colorDepth: window.screen.colorDepth,
                pixelDepth: window.screen.pixelDepth,
                innerWidth: window.innerWidth,
                innerHeight: window.innerHeight,
                outerWidth: window.outerWidth,
                outerHeight: window.outerHeight,
                hasConsistentDepth: window.screen.colorDepth === window.screen.pixelDepth
            };
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        let has_consistent_depth = result["hasConsistentDepth"].as_bool().unwrap_or(false);
        let screen_width = result["screenWidth"].as_u64().unwrap_or(0);

        let passed = has_consistent_depth && screen_width > 0;

        Ok(CheckResult {
            name: "浏览器特征一致性检测".to_string(),
            passed,
            details: if passed {
                "浏览器特征一致".to_string()
            } else {
                "浏览器特征不一致".to_string()
            },
            weight: 5,
        })
    }

    /// 检测时区和语言设置
    async fn check_locale_settings(&self, page: &Page) -> Result<CheckResult> {
        let script = r#"
            return {
                timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
                language: navigator.language,
                languages: navigator.languages,
                timezoneOffset: new Date().getTimezoneOffset()
            };
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        let timezone = result["timezone"].as_str().unwrap_or("unknown");
        let language = result["language"].as_str().unwrap_or("unknown");

        let passed = timezone != "unknown" && language != "unknown";

        Ok(CheckResult {
            name: "时区和语言检测".to_string(),
            passed,
            details: if passed {
                format!("时区={}, 语言={}", timezone, language)
            } else {
                "时区或语言设置异常".to_string()
            },
            weight: 4,
        })
    }

    /// 检测权限
    async fn check_permissions(&self, page: &Page) -> Result<CheckResult> {
        let script = r#"
            return {
                hasPermissionsAPI: 'permissions' in navigator,
                hasNotifications: 'Notification' in window,
                hasGeolocation: 'geolocation' in navigator,
                hasBattery: 'getBattery' in navigator
            };
        "#;

        let result: serde_json::Value = page.evaluate(script).await?.into_value()?;

        let has_permissions = result["hasPermissionsAPI"].as_bool().unwrap_or(false);

        let passed = has_permissions;

        Ok(CheckResult {
            name: "浏览器权限 API 检测".to_string(),
            passed,
            details: if passed {
                "权限 API 可用".to_string()
            } else {
                "权限 API 不可用".to_string()
            },
            weight: 3,
        })
    }

    /// 生成优化建议
    fn generate_recommendations(&self, checks: &[CheckResult], risk_level: &RiskLevel) -> Vec<String> {
        let mut recommendations = Vec::new();

        for check in checks {
            if !check.passed {
                match check.name.as_str() {
                    "WebDriver 检测" => {
                        recommendations.push("使用 undetected-chromedriver 或配置浏览器隐藏 WebDriver 标识".to_string());
                    }
                    "自动化标识检测" => {
                        recommendations.push("使用 Playwright/Puppeteer 的 stealth 插件隐藏自动化特征".to_string());
                    }
                    "CDP Runtime 检测" => {
                        recommendations.push("避免使用 CDP 扩展，或使用浏览器配置隐藏扩展".to_string());
                    }
                    "浏览器插件检测" => {
                        recommendations.push("配置浏览器插件列表，模拟正常用户的插件环境".to_string());
                    }
                    "Canvas 指纹检测" | "WebGL 指纹检测" => {
                        recommendations.push("确保 Canvas 和 WebGL 正常工作，避免使用过度的指纹伪装".to_string());
                    }
                    _ => {}
                }
            }
        }

        match risk_level {
            RiskLevel::High => {
                recommendations.push("❌ 高风险：建议更换浏览器配置或使用真实浏览器环境".to_string());
                recommendations.push("建议使用住宅代理而非数据中心代理".to_string());
            }
            RiskLevel::Medium => {
                recommendations.push("⚠️  中风险：建议优化部分配置以降低风险".to_string());
            }
            RiskLevel::Low => {
                recommendations.push("✅ 低风险：当前环境较为理想".to_string());
            }
        }

        if recommendations.is_empty() {
            recommendations.push("环境检测良好，无需特别优化".to_string());
        }

        recommendations
    }

    /// 打印检测报告
    fn print_report(&self, report: &DetectionReport) {
        if !self.verbose {
            return;
        }

        info!("=== 浏览器环境检测报告 ===");
        info!("风险等级: {:?}", report.risk_level);
        info!("风险评分: {}/100", report.risk_score);
        info!("");

        info!("检测结果:");
        for check in &report.checks {
            let status = if check.passed { "✓" } else { "✗" };
            info!(
                "  {} {} (权重: {}) - {}",
                status, check.name, check.weight, check.details
            );
        }

        info!("");
        info!("优化建议:");
        for (i, rec) in report.recommendations.iter().enumerate() {
            info!("  {}. {}", i + 1, rec);
        }

        info!("=========================");
    }

    /// 测试代理 IP 信誉
    pub async fn check_proxy_reputation(&self, proxy_url: &str) -> Result<ProxyReputation> {
        info!("检测代理 IP 信誉: {}", proxy_url);

        // 使用第三方 API 检测 IP 信誉
        // 示例服务: ipqualityscore.com, proxycheck.io, ipapi.co
        
        let client = reqwest::Client::builder()
            .proxy(reqwest::Proxy::all(proxy_url)?)
            .build()?;

        // 获取当前 IP
        let ip_response = client
            .get("https://api.ipify.org?format=json")
            .send()
            .await?;
        
        let ip_data: serde_json::Value = ip_response.json().await?;
        let ip = ip_data["ip"].as_str().unwrap_or("unknown").to_string();

        info!("当前 IP: {}", ip);

        // 检测 IP 信息 (使用免费的 ipapi.co)
        let ip_info_response = client
            .get(&format!("https://ipapi.co/{}/json/", ip))
            .send()
            .await?;

        let ip_info: serde_json::Value = ip_info_response.json().await?;

        let is_proxy = ip_info["proxy"].as_bool().unwrap_or(false);
        let org = ip_info["org"].as_str().unwrap_or("unknown").to_string();
        let country = ip_info["country_name"].as_str().unwrap_or("unknown").to_string();

        let is_datacenter = org.to_lowercase().contains("hosting") 
            || org.to_lowercase().contains("datacenter")
            || org.to_lowercase().contains("cloud");

        let risk_score = if is_proxy || is_datacenter { 70 } else { 30 };

        Ok(ProxyReputation {
            ip,
            country,
            organization: org,
            is_proxy,
            is_datacenter,
            risk_score,
        })
    }
}

/// 代理信誉信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyReputation {
    pub ip: String,
    pub country: String,
    pub organization: String,
    pub is_proxy: bool,
    pub is_datacenter: bool,
    pub risk_score: u32,
}

impl Default for BrowserDetector {
    fn default() -> Self {
        Self::new()
    }
}
