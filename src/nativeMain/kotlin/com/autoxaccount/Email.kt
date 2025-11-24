package com.autoxaccount

import kotlinx.coroutines.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/**
 * Email service module
 * Migrated from Rust email.rs
 * 
 * Provides SMTP email service and verification code handling
 */

class EmailService(private val config: SmtpConfig) {
    private var isRunning = false
    private var job: Job? = null

    /**
     * Start SMTP service
     */
    suspend fun start(): Result<Unit> = runCatchingResult {
        if (isRunning) {
            logWarn("SMTP 服务已在运行 / SMTP service is already running")
            return@runCatchingResult
        }

        logInfo("启动 SMTP 服务 / Starting SMTP service at ${config.host}:${config.port}")
        
        // Launch SMTP server in background
        job = CoroutineScope(Dispatchers.Default).launch {
            try {
                runSmtpServer()
            } catch (e: Exception) {
                logError("SMTP 服务错误 / SMTP service error: ${e.message}")
            }
        }

        isRunning = true
        logInfo("SMTP 服务已启动 / SMTP service started")
    }

    /**
     * Stop SMTP service
     */
    fun stop() {
        if (!isRunning) return

        job?.cancel()
        job = null
        isRunning = false
        logInfo("SMTP 服务已停止 / SMTP service stopped")
    }

    /**
     * Get email handler for checking verification codes
     */
    fun getHandler(): EmailHandler {
        return EmailHandlerImpl(this)
    }

    /**
     * Run SMTP server loop
     */
    private suspend fun runSmtpServer() {
        logInfo("SMTP 服务器监听 / SMTP server listening on ${config.host}:${config.port}")
        
        // Simplified SMTP server - in production would implement full SMTP protocol
        while (isRunning) {
            delay(1000)
            // Handle incoming SMTP connections
        }
    }

    /**
     * Internal storage for received emails
     */
    private val receivedEmails = mutableMapOf<String, MutableList<EmailMessage>>()

    /**
     * Store received email
     */
    internal fun storeEmail(email: EmailMessage) {
        val list = receivedEmails.getOrPut(email.to) { mutableListOf() }
        list.add(email)
        logInfo("收到邮件 / Received email: ${email.from} -> ${email.to}")
    }

    /**
     * Get emails for an address
     */
    internal fun getEmails(address: String): List<EmailMessage> {
        return receivedEmails[address] ?: emptyList()
    }

    /**
     * Extract verification code from email body
     */
    internal fun extractVerificationCode(body: String): String? {
        // Common patterns for verification codes
        val patterns = listOf(
            Regex("code:\\s*(\\d{6})"),
            Regex("verification code:\\s*(\\d{6})"),
            Regex("验证码[：:](\\d{6})"),
            Regex("(\\d{6})"),  // Just 6 digits
        )

        for (pattern in patterns) {
            val match = pattern.find(body)
            if (match != null && match.groupValues.size > 1) {
                return match.groupValues[1]
            }
        }

        return null
    }
}

/**
 * Email handler implementation
 */
private class EmailHandlerImpl(private val service: EmailService) : EmailHandler {
    override suspend fun checkForVerificationCode(email: String): String? {
        val emails = service.getEmails(email)
        
        // Check for verification code in recent emails
        for (message in emails.reversed()) {
            // Look for X/Twitter verification emails
            if (message.subject.contains("verification", ignoreCase = true) ||
                message.subject.contains("verify", ignoreCase = true) ||
                message.subject.contains("code", ignoreCase = true) ||
                message.subject.contains("验证", ignoreCase = true)) {
                
                val code = service.extractVerificationCode(message.body)
                if (code != null) {
                    logInfo("找到验证码 / Found verification code: $code")
                    return code
                }
            }
        }

        return null
    }

    override suspend fun getEmails(email: String): List<EmailMessage> {
        return service.getEmails(email)
    }
}
