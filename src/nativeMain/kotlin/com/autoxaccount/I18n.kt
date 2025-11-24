package com.autoxaccount

/**
 * Internationalization (i18n) module
 * Migrated from Rust i18n.rs
 * 
 * Provides multi-language support for the application
 */

class I18n(private val language: String = "zh-CN") {
    
    private val translations = when (language) {
        "en-US", "en" -> englishTranslations
        else -> chineseTranslations // Default to Chinese
    }

    /**
     * Translate a key to the current language
     */
    fun t(key: String): String = translations[key] ?: key

    companion object {
        private val chineseTranslations = mapOf(
            "app_name" to "X 账号自动注册系统",
            "starting" to "正在启动...",
            "smtp_starting" to "正在启动 SMTP 服务...",
            "smtp_started" to "SMTP 服务已启动",
            "proxy_disabled" to "代理已禁用 / Proxy disabled",
            "proxy_system" to "系统代理",
            "proxy_manual" to "手动代理",
            "registration_start" to "开始注册账号",
            "registration_success" to "✅ 账号注册成功！",
            "registration_failed" to "❌ 账号注册失败",
            "batch_registration_start" to "开始批量注册",
            "batch_registration_complete" to "批量注册完成",
            "email_creating" to "正在创建邮箱...",
            "email_created" to "邮箱创建成功",
            "email_failed" to "邮箱创建失败",
            "browser_launching" to "正在启动浏览器...",
            "browser_launched" to "浏览器已启动",
            "browser_closed" to "浏览器已关闭",
            "waiting_for_verification" to "等待邮箱验证...",
            "verification_received" to "已收到验证码",
            "verification_failed" to "验证码接收失败",
            "captcha_detected" to "检测到人机验证",
            "captcha_waiting_manual" to "等待手动完成验证...",
            "captcha_completed" to "验证完成",
            "saving_account" to "正在保存账号信息...",
            "account_saved" to "账号信息已保存",
            "export_success" to "导出成功",
            "export_failed" to "导出失败",
            "import_success" to "导入成功",
            "import_failed" to "导入失败",
            "config_loaded" to "配置已加载",
            "config_saved" to "配置已保存",
            "error" to "错误",
            "warning" to "警告",
            "info" to "信息",
            "success" to "成功"
        )

        private val englishTranslations = mapOf(
            "app_name" to "X Account Auto Registration System",
            "starting" to "Starting...",
            "smtp_starting" to "Starting SMTP service...",
            "smtp_started" to "SMTP service started",
            "proxy_disabled" to "Proxy disabled",
            "proxy_system" to "System proxy",
            "proxy_manual" to "Manual proxy",
            "registration_start" to "Starting account registration",
            "registration_success" to "✅ Account registered successfully!",
            "registration_failed" to "❌ Account registration failed",
            "batch_registration_start" to "Starting batch registration",
            "batch_registration_complete" to "Batch registration complete",
            "email_creating" to "Creating email...",
            "email_created" to "Email created successfully",
            "email_failed" to "Email creation failed",
            "browser_launching" to "Launching browser...",
            "browser_launched" to "Browser launched",
            "browser_closed" to "Browser closed",
            "waiting_for_verification" to "Waiting for email verification...",
            "verification_received" to "Verification code received",
            "verification_failed" to "Failed to receive verification code",
            "captcha_detected" to "Captcha detected",
            "captcha_waiting_manual" to "Waiting for manual completion...",
            "captcha_completed" to "Verification completed",
            "saving_account" to "Saving account information...",
            "account_saved" to "Account information saved",
            "export_success" to "Export successful",
            "export_failed" to "Export failed",
            "import_success" to "Import successful",
            "import_failed" to "Import failed",
            "config_loaded" to "Configuration loaded",
            "config_saved" to "Configuration saved",
            "error" to "Error",
            "warning" to "Warning",
            "info" to "Info",
            "success" to "Success"
        )
    }
}
