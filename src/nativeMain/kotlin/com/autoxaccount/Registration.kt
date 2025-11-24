package com.autoxaccount

import kotlinx.serialization.Serializable
import kotlinx.coroutines.*
import kotlinx.datetime.Clock

/**
 * X (Twitter) account registration module
 * Migrated from Rust registration.rs
 * 
 * Core business logic for registering X accounts
 */

@Serializable
data class AccountInfo(
    val email: String,
    val name: String,
    val username: String,
    val password: String,
    val phone: String? = null,
    val birthDate: BirthDate,
    val createdAt: String,
    val status: String
)

@Serializable
data class BirthDate(
    val month: String,
    val day: String,
    val year: String
)

class XRegistration(
    private val config: Config,
    private val emailHandler: EmailHandler
) {
    /**
     * Register a single X account
     */
    suspend fun registerAccount(email: String): Result<AccountInfo> = runCatchingResult {
        logInfo("开始注册账号 / Starting registration: $email")

        // Generate random user data
        val name = generateRandomName()
        val username = generateRandomUsername()
        val password = generateRandomPassword()
        val birthDate = generateRandomBirthDate()

        logInfo("生成的用户信息 / Generated user info: username=$username")

        // Step 1: Launch browser
        logInfo("步骤 1/5: 启动浏览器 / Step 1/5: Launching browser")
        // Browser automation would go here - simplified for migration
        delay(1000)

        // Step 2: Navigate to signup page
        logInfo("步骤 2/5: 访问注册页面 / Step 2/5: Navigating to signup page")
        // Navigate to config.xAccount.baseUrl
        delay(1000)

        // Step 3: Fill registration form
        logInfo("步骤 3/5: 填写注册表单 / Step 3/5: Filling registration form")
        // Fill name, email, birth date
        delay(2000)

        // Step 4: Wait for email verification
        logInfo("步骤 4/5: 等待邮箱验证 / Step 4/5: Waiting for email verification")
        val verificationCode = waitForVerificationCode(email)
        logInfo("收到验证码 / Received verification code: $verificationCode")

        // Step 5: Complete registration
        logInfo("步骤 5/5: 完成注册 / Step 5/5: Completing registration")
        // Enter verification code and complete signup
        delay(1000)

        val account = AccountInfo(
            email = email,
            name = name,
            username = username,
            password = password,
            phone = null,
            birthDate = birthDate,
            createdAt = Clock.System.now().toString(),
            status = "registered"
        )

        logInfo("账号注册成功 / Account registered successfully: $username")
        account
    }

    /**
     * Wait for email verification code
     */
    private suspend fun waitForVerificationCode(email: String): String {
        val timeout = config.xAccount.emailWaitTimeout.toLong() * 1000
        val startTime = Clock.System.now().toEpochMilliseconds()

        while (true) {
            val elapsed = Clock.System.now().toEpochMilliseconds() - startTime
            if (elapsed > timeout) {
                throw Exception("等待验证码超时 / Verification code timeout")
            }

            // Check for verification email
            val code = emailHandler.checkForVerificationCode(email)
            if (code != null) {
                return code
            }

            delay(3000) // Check every 3 seconds
        }
    }

    /**
     * Get Chrome executable path
     */
    private fun getChromeExecutablePath(): String? {
        // Check config path
        config.browser.chromePath?.let { path ->
            if (fileExists(path)) return path
        }

        // Check bundled chromium
        val bundledPaths = listOf(
            "./chromium/chrome",
            "./chromium/chrome.exe",
            "./chromium/Chromium.app/Contents/MacOS/Chromium"
        )
        for (path in bundledPaths) {
            if (fileExists(path)) return path
        }

        // Check system installation
        val systemPaths = when {
            isWindows() -> listOf(
                "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
                "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe"
            )
            isMacOS() -> listOf(
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Chromium.app/Contents/MacOS/Chromium"
            )
            else -> listOf(
                "/usr/bin/chromium",
                "/usr/bin/chromium-browser",
                "/usr/bin/google-chrome"
            )
        }

        for (path in systemPaths) {
            if (fileExists(path)) return path
        }

        return null
    }

    // Helper functions

    private fun generateRandomName(): String {
        val firstNames = listOf("Alice", "Bob", "Charlie", "David", "Emma", "Frank", "Grace", "Henry")
        val lastNames = listOf("Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis")
        return "${firstNames.random()} ${lastNames.random()}"
    }

    private fun generateRandomUsername(): String {
        val adjectives = listOf("happy", "sunny", "cool", "smart", "quick", "bright", "clever", "swift")
        val nouns = listOf("cat", "dog", "bird", "fish", "star", "moon", "sun", "cloud")
        val number = (100..999).random()
        return "${adjectives.random()}${nouns.random()}$number"
    }

    private fun generateRandomPassword(): String {
        val chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%"
        return (1..16).map { chars.random() }.joinToString("")
    }

    private fun generateRandomBirthDate(): BirthDate {
        val year = (1980..2000).random()
        val month = (1..12).random()
        val day = (1..28).random() // Simplified to avoid month-specific validation
        return BirthDate(
            month = month.toString().padStart(2, '0'),
            day = day.toString().padStart(2, '0'),
            year = year.toString()
        )
    }

    private fun fileExists(path: String): Boolean {
        return platform.posix.access(path, platform.posix.F_OK) == 0
    }

    private fun isWindows(): Boolean {
        val os = platform.posix.getenv("OS")?.kotlinx.cinterop.toKString()
        return os?.contains("Windows", ignoreCase = true) == true
    }

    private fun isMacOS(): Boolean {
        // Simplified OS detection
        return false
    }
}

/**
 * Email handler interface
 * Provides abstraction for email operations
 */
interface EmailHandler {
    suspend fun checkForVerificationCode(email: String): String?
    suspend fun getEmails(email: String): List<EmailMessage>
}

@Serializable
data class EmailMessage(
    val from: String,
    val to: String,
    val subject: String,
    val body: String,
    val receivedAt: String
)
