package com.autoxaccount

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.encodeToString
import platform.posix.*

/**
 * Configuration management module
 * Migrated from Rust config.rs
 */

@Serializable
data class Config(
    val language: String = "zh-CN",
    val smtp: SmtpConfig = SmtpConfig(),
    val proxy: ProxyConfig = ProxyConfig(),
    val browser: BrowserConfig = BrowserConfig(),
    val xAccount: XAccountConfig = XAccountConfig(),
    val output: OutputConfig = OutputConfig(),
    val ui: UiConfig = UiConfig(),
    val mode: RunMode = RunMode.TEST,
    val emailProvider: EmailProviderSettings = EmailProviderSettings(),
    val captcha: CaptchaConfig = CaptchaConfig()
) {
    companion object {
        private val json = Json { 
            prettyPrint = true
            ignoreUnknownKeys = true
            encodeDefaults = true
        }

        /**
         * Load configuration from file
         */
        fun fromFile(path: String): Result<Config> = runCatchingResult {
            val content = readFileToString(path)
            json.decodeFromString<Config>(content)
        }

        /**
         * Read file content as string
         */
        private fun readFileToString(path: String): String {
            val file = fopen(path, "r") ?: throw Exception("Failed to open file: $path")
            try {
                val buffer = StringBuilder()
                val chunk = ByteArray(4096)
                while (true) {
                    val bytesRead = fread(chunk.refTo(0), 1, chunk.size.toULong(), file).toInt()
                    if (bytesRead <= 0) break
                    buffer.append(chunk.decodeToString(0, bytesRead))
                }
                return buffer.toString()
            } finally {
                fclose(file)
            }
        }
    }

    /**
     * Save configuration to file
     */
    fun toFile(path: String): Result<Unit> = runCatchingResult {
        val content = json.encodeToString(this)
        writeStringToFile(path, content)
    }

    private fun writeStringToFile(path: String, content: String) {
        val file = fopen(path, "w") ?: throw Exception("Failed to open file for writing: $path")
        try {
            val bytes = content.encodeToByteArray()
            fwrite(bytes.refTo(0), 1, bytes.size.toULong(), file)
        } finally {
            fclose(file)
        }
    }

    /**
     * Get proxy URL based on proxy mode and target
     */
    fun getProxyUrl(target: ProxyTarget): String? {
        // Check if proxy is enabled for target
        val enabled = when (target) {
            ProxyTarget.BROWSER -> proxy.browserEnabled
            ProxyTarget.EMAIL -> proxy.emailEnabled
        }
        
        if (!enabled) return null

        return when (proxy.mode) {
            ProxyMode.NONE -> null
            ProxyMode.SYSTEM -> detectSystemProxy()
            ProxyMode.MANUAL -> {
                val auth = if (!proxy.username.isNullOrEmpty() && !proxy.password.isNullOrEmpty()) {
                    "${proxy.username}:${proxy.password}@"
                } else {
                    ""
                }
                "${proxy.proxyType}://$auth${proxy.host}:${proxy.port}"
            }
        }
    }

    /**
     * Detect system proxy settings
     */
    private fun detectSystemProxy(): String? {
        // Check environment variables in priority order
        val envVars = listOf(
            "HTTPS_PROXY", "https_proxy",
            "HTTP_PROXY", "http_proxy",
            "ALL_PROXY", "all_proxy"
        )
        
        for (envVar in envVars) {
            val value = getenv(envVar)?.toKString()
            if (!value.isNullOrEmpty()) {
                return value
            }
        }
        
        return null
    }

    /**
     * Get proxy description for display
     */
    fun getProxyDescription(target: ProxyTarget): String {
        val targetStr = when (target) {
            ProxyTarget.BROWSER -> "浏览器 / Browser"
            ProxyTarget.EMAIL -> "邮箱 / Email"
        }

        val enabled = when (target) {
            ProxyTarget.BROWSER -> proxy.browserEnabled
            ProxyTarget.EMAIL -> proxy.emailEnabled
        }

        if (!enabled) {
            return "$targetStr: 未启用代理 / Proxy disabled"
        }

        return when (proxy.mode) {
            ProxyMode.NONE -> "$targetStr: 不使用代理 / No Proxy"
            ProxyMode.SYSTEM -> {
                val proxyUrl = getProxyUrl(target)
                if (proxyUrl != null) {
                    "$targetStr: 系统代理 / System Proxy: $proxyUrl"
                } else {
                    "$targetStr: 系统代理（未检测到）/ System Proxy (Not Detected)"
                }
            }
            ProxyMode.MANUAL -> {
                "$targetStr: 手动代理 / Manual Proxy: ${proxy.proxyType}://${proxy.host}:${proxy.port}"
            }
        }
    }

    /**
     * Check if running in production mode
     */
    fun isProduction(): Boolean = 
        mode == RunMode.PRODUCTION || emailProvider.forceSelfHosted
}

@Serializable
data class SmtpConfig(
    val host: String = "0.0.0.0",
    val port: UShort = 8025u,
    val domain: String = "example.com",
    val enable: Boolean = true
)

@Serializable
enum class ProxyMode {
    NONE, SYSTEM, MANUAL
}

@Serializable
enum class ProxyTarget {
    BROWSER, EMAIL
}

@Serializable
data class ProxyConfig(
    val mode: ProxyMode = ProxyMode.NONE,
    val proxyType: String = "socks5",
    val host: String = "127.0.0.1",
    val port: UShort = 1080u,
    val username: String? = null,
    val password: String? = null,
    val browserEnabled: Boolean = true,
    val emailEnabled: Boolean = false
)

@Serializable
enum class BrowserType {
    NATIVE, BITBROWSER
}

@Serializable
data class BrowserConfig(
    val browserType: BrowserType = BrowserType.NATIVE,
    val headless: Boolean = false,
    val timeout: ULong = 30000u,
    val viewport: ViewportConfig = ViewportConfig(),
    val userDataDir: String = "browser_data",
    val chromePath: String? = null,
    val bitbrowser: BitBrowserConfig? = null
)

@Serializable
data class ViewportConfig(
    val width: UInt = 1280u,
    val height: UInt = 720u
)

@Serializable
data class BitBrowserConfig(
    val apiUrl: String = "http://127.0.0.1",
    val apiPort: UShort = 54345u,
    val profileIds: List<String> = emptyList(),
    val autoCreateProfile: Boolean = false,
    val separateProfilePerAccount: Boolean = true
)

@Serializable
data class XAccountConfig(
    val baseUrl: String = "https://twitter.com/i/flow/signup",
    val emailWaitTimeout: ULong = 120u,
    val retryTimes: UInt = 3u
)

@Serializable
data class OutputConfig(
    val screenshotsDir: String = "screenshots",
    val logsDir: String = "logs",
    val accountsFile: String = "accounts.json"
)

@Serializable
data class UiConfig(
    val font: String = "MiSans",
    val fontPath: String = "fonts/MiSans-Regular.ttf"
)

@Serializable
enum class RunMode {
    PRODUCTION, TEST
}

@Serializable
data class EmailProviderSettings(
    val forceSelfHosted: Boolean = false,
    val allowedTempProviders: List<String> = listOf("MailTm", "GuerrillaMail"),
    val productionDomains: List<String> = listOf("example.com"),
    val selectedProvider: String = "MailTm",
    val customSmtpHost: String? = null,
    val customSmtpPort: UShort? = null,
    val customImapHost: String? = null,
    val customImapPort: UShort? = null,
    val customUsername: String? = null,
    val customPassword: String? = null,
    val customApiKey: String? = null,
    val customApiEndpoint: String? = null,
    val plusMode: EmailPlusMode = EmailPlusMode()
)

@Serializable
data class EmailPlusMode(
    val enabled: Boolean = false,
    val baseEmail: String? = null,
    val suffixMode: PlusSuffixMode = PlusSuffixMode.AUTO,
    val manualSuffix: String? = null
) {
    /**
     * Generate email address with + suffix
     */
    fun generatePlusEmail(index: Int? = null): Result<String> = runCatchingResult {
        if (!enabled) {
            throw Exception("Plus 模式未启用 / Plus mode is not enabled")
        }

        val base = baseEmail ?: throw Exception("基础邮箱地址未设置 / Base email is not set")
        
        val parts = base.split("@")
        if (parts.size != 2) {
            throw Exception("无效的邮箱地址格式 / Invalid email format: $base")
        }

        val username = parts[0]
        val domain = parts[1]

        val suffix = when (suffixMode) {
            PlusSuffixMode.MANUAL -> {
                manualSuffix ?: throw Exception("手动模式下必须指定后缀 / Manual suffix is required in manual mode")
            }
            PlusSuffixMode.AUTO -> generateHumanLikeSuffix(index)
        }

        "$username+$suffix@$domain"
    }

    /**
     * Generate human-like suffix
     */
    private fun generateHumanLikeSuffix(index: Int?): String {
        // Simplified fake name generation - in production use a proper fake data library
        val firstNames = listOf("john", "jane", "alice", "bob", "charlie", "david", "emma", "frank")
        val name = firstNames.random()
        val num = index ?: (1..999).random()
        return "$name$num"
    }
}

@Serializable
enum class PlusSuffixMode {
    MANUAL, AUTO
}

@Serializable
data class CaptchaConfig(
    val mode: CaptchaMode = CaptchaMode.AUTO,
    val manualFallback: Boolean = true,
    val twoCaptchaApiKey: String? = null,
    val antiCaptchaApiKey: String? = null,
    val capmonsterApiKey: String? = null,
    val llmApi: LlmApiConfig? = null
)

@Serializable
enum class CaptchaMode {
    AUTO, MANUAL, THIRDPARTY, LLM
}

@Serializable
data class LlmApiConfig(
    val provider: String,
    val apiKey: String,
    val endpoint: String? = null,
    val model: String
)
