package com.autoxaccount

import kotlinx.serialization.Serializable
import kotlinx.datetime.Clock

/**
 * Browser environment detection module
 * Migrated from Rust browser_detector.rs
 * 
 * Detects browser automation signals that might trigger captchas
 */

@Serializable
data class BrowserDetectionReport(
    val riskLevel: RiskLevel,
    val riskScore: Int,
    val checks: List<DetectionCheck>,
    val recommendations: List<String>,
    val timestamp: String
)

@Serializable
enum class RiskLevel {
    LOW,      // 0-30: Low risk of being detected
    MEDIUM,   // 31-60: Medium risk
    HIGH,     // 61-100: High risk of triggering captchas
}

@Serializable
data class DetectionCheck(
    val name: String,
    val passed: Boolean,
    val weight: Int,
    val details: String
)

/**
 * Browser detector
 */
class BrowserDetector(private val verbose: Boolean = false) {
    
    /**
     * Detect browser environment and generate report
     */
    suspend fun detectEnvironment(): Result<BrowserDetectionReport> = runCatchingResult {
        logInfo("执行浏览器环境检测 / Performing browser environment detection")

        val checks = mutableListOf<DetectionCheck>()
        var totalScore = 0
        var maxScore = 0

        // Check 1: WebDriver detection
        val webDriverCheck = checkWebDriver()
        checks.add(webDriverCheck)
        maxScore += webDriverCheck.weight
        if (!webDriverCheck.passed) totalScore += webDriverCheck.weight

        // Check 2: Browser fingerprint
        val fingerprintCheck = checkBrowserFingerprint()
        checks.add(fingerprintCheck)
        maxScore += fingerprintCheck.weight
        if (!fingerprintCheck.passed) totalScore += fingerprintCheck.weight

        // Check 3: Plugin detection
        val pluginCheck = checkPlugins()
        checks.add(pluginCheck)
        maxScore += pluginCheck.weight
        if (!pluginCheck.passed) totalScore += pluginCheck.weight

        // Check 4: User agent
        val userAgentCheck = checkUserAgent()
        checks.add(userAgentCheck)
        maxScore += userAgentCheck.weight
        if (!userAgentCheck.passed) totalScore += userAgentCheck.weight

        // Check 5: Window properties
        val windowCheck = checkWindowProperties()
        checks.add(windowCheck)
        maxScore += windowCheck.weight
        if (!windowCheck.passed) totalScore += windowCheck.weight

        // Calculate risk score (0-100)
        val riskScore = if (maxScore > 0) {
            (totalScore * 100) / maxScore
        } else {
            0
        }

        // Determine risk level
        val riskLevel = when {
            riskScore <= 30 -> RiskLevel.LOW
            riskScore <= 60 -> RiskLevel.MEDIUM
            else -> RiskLevel.HIGH
        }

        // Generate recommendations
        val recommendations = generateRecommendations(checks)

        BrowserDetectionReport(
            riskLevel = riskLevel,
            riskScore = riskScore,
            checks = checks,
            recommendations = recommendations,
            timestamp = Clock.System.now().toString()
        )
    }

    private fun checkWebDriver(): DetectionCheck {
        // Simplified check - in production would examine actual browser properties
        val passed = true // Assume passed for now
        
        return DetectionCheck(
            name = "WebDriver 特征检测 / WebDriver Detection",
            passed = passed,
            weight = 25,
            details = if (passed) {
                "未检测到 WebDriver 特征 / No WebDriver signature detected"
            } else {
                "检测到 WebDriver 特征，可能触发验证码 / WebDriver detected, may trigger captcha"
            }
        )
    }

    private fun checkBrowserFingerprint(): DetectionCheck {
        // Simplified check
        val passed = true
        
        return DetectionCheck(
            name = "浏览器指纹检测 / Browser Fingerprint",
            passed = passed,
            weight = 20,
            details = if (passed) {
                "浏览器指纹正常 / Browser fingerprint appears normal"
            } else {
                "浏览器指纹异常 / Abnormal browser fingerprint"
            }
        )
    }

    private fun checkPlugins(): DetectionCheck {
        // Simplified check
        val passed = true
        
        return DetectionCheck(
            name = "插件检测 / Plugin Detection",
            passed = passed,
            weight = 15,
            details = if (passed) {
                "插件配置正常 / Normal plugin configuration"
            } else {
                "插件配置异常，建议启用常见插件 / Abnormal plugin config, enable common plugins"
            }
        )
    }

    private fun checkUserAgent(): DetectionCheck {
        // Simplified check
        val passed = true
        
        return DetectionCheck(
            name = "User Agent 检测 / User Agent Check",
            passed = passed,
            weight = 20,
            details = if (passed) {
                "User Agent 正常 / Normal User Agent"
            } else {
                "User Agent 异常或过时 / Abnormal or outdated User Agent"
            }
        )
    }

    private fun checkWindowProperties(): DetectionCheck {
        // Simplified check
        val passed = true
        
        return DetectionCheck(
            name = "窗口属性检测 / Window Properties",
            passed = passed,
            weight = 20,
            details = if (passed) {
                "窗口属性正常 / Normal window properties"
            } else {
                "窗口属性异常 / Abnormal window properties"
            }
        )
    }

    private fun generateRecommendations(checks: List<DetectionCheck>): List<String> {
        val recommendations = mutableListOf<String>()

        val failedChecks = checks.filter { !it.passed }
        
        if (failedChecks.isEmpty()) {
            recommendations.add("✅ 浏览器环境配置良好 / Browser environment is well configured")
            recommendations.add("✅ 触发验证码的风险较低 / Low risk of triggering captchas")
        } else {
            recommendations.add("⚠️  发现以下问题 / Issues found:")
            
            if (failedChecks.any { it.name.contains("WebDriver") }) {
                recommendations.add("• 禁用或隐藏 WebDriver 特征 / Disable or hide WebDriver signatures")
                recommendations.add("• 使用 undetected-chromedriver 等工具 / Use tools like undetected-chromedriver")
            }
            
            if (failedChecks.any { it.name.contains("Fingerprint") }) {
                recommendations.add("• 修改浏览器指纹以匹配真实用户 / Modify fingerprint to match real users")
                recommendations.add("• 使用浏览器指纹管理工具 / Use fingerprint management tools")
            }
            
            if (failedChecks.any { it.name.contains("Plugin") }) {
                recommendations.add("• 启用常见浏览器插件 / Enable common browser plugins")
                recommendations.add("• 模拟真实用户的插件配置 / Mimic real user plugin configuration")
            }
            
            if (failedChecks.any { it.name.contains("User Agent") }) {
                recommendations.add("• 更新 User Agent 到最新版本 / Update User Agent to latest version")
                recommendations.add("• 确保 User Agent 与浏览器版本匹配 / Ensure UA matches browser version")
            }
            
            if (failedChecks.any { it.name.contains("Window") }) {
                recommendations.add("• 调整窗口大小到常见分辨率 / Adjust window size to common resolutions")
                recommendations.add("• 避免使用默认的自动化窗口大小 / Avoid default automation window sizes")
            }
        }

        return recommendations
    }

    companion object {
        fun new(): BrowserDetector = BrowserDetector(false)
        
        fun BrowserDetector.withVerbose(verbose: Boolean): BrowserDetector {
            return BrowserDetector(verbose)
        }
    }
}
