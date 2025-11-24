package com.autoxaccount

import kotlinx.coroutines.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/**
 * Email provider module
 * Migrated from Rust email_provider.rs
 * 
 * Provides abstraction for different email service providers
 */

@Serializable
enum class EmailProvider {
    MAILTM,           // mail.tm temporary email service
    GUERRILLAMAIL,    // Guerrilla Mail temporary email service
    SELFHOSTED,       // Self-hosted email server
    CUSTOM            // Custom IMAP/SMTP configuration
}

/**
 * Email provider configuration
 */
@Serializable
data class EmailProviderConfig(
    val provider: EmailProvider,
    val smtpHost: String? = null,
    val smtpPort: UShort? = null,
    val imapHost: String? = null,
    val imapPort: UShort? = null,
    val username: String? = null,
    val password: String? = null,
    val domain: String,
    val apiKey: String? = null,
    val apiEndpoint: String? = null
)

/**
 * Email account information
 */
@Serializable
data class EmailAccount(
    val address: String,
    val password: String,
    val provider: EmailProvider,
    val createdAt: String,
    val verified: Boolean = false
)

/**
 * Batch email manager
 * Creates and manages multiple email accounts
 */
class BatchEmailManager(private val config: EmailProviderConfig) {
    private val accounts = mutableListOf<EmailAccount>()

    /**
     * Create batch of email accounts
     */
    suspend fun createBatch(count: Int, verify: Boolean = false): Result<Int> = runCatchingResult {
        logInfo("批量创建 $count 个邮箱 / Creating $count email accounts")

        val created = mutableListOf<EmailAccount>()

        for (i in 1..count) {
            try {
                val account = createSingleEmail().getOrThrow()
                created.add(account)
                accounts.add(account)
                
                logInfo("[$i/$count] 邮箱创建成功 / Email created: ${account.address}")
                
                if (verify) {
                    verifyEmail(account)
                }
                
                // Small delay to avoid rate limiting
                delay(500)
            } catch (e: Exception) {
                logError("邮箱创建失败 / Failed to create email: ${e.message}")
            }
        }

        logInfo("成功创建 ${created.size}/$count 个邮箱 / Successfully created ${created.size}/$count emails")
        created.size
    }

    /**
     * Create a single email account
     */
    private suspend fun createSingleEmail(): Result<EmailAccount> = runCatchingResult {
        when (config.provider) {
            EmailProvider.MAILTM -> createMailTmEmail()
            EmailProvider.GUERRILLAMAIL -> createGuerrillaMailEmail()
            EmailProvider.SELFHOSTED -> createSelfHostedEmail()
            EmailProvider.CUSTOM -> createCustomEmail()
        }
    }

    /**
     * Create mail.tm email
     */
    private fun createMailTmEmail(): EmailAccount {
        // Simplified - in production would use HTTP API
        val username = generateRandomUsername()
        val domain = config.domain
        val password = generateRandomPassword()
        
        return EmailAccount(
            address = "$username@$domain",
            password = password,
            provider = EmailProvider.MAILTM,
            createdAt = kotlinx.datetime.Clock.System.now().toString(),
            verified = false
        )
    }

    /**
     * Create Guerrilla Mail email
     */
    private fun createGuerrillaMailEmail(): EmailAccount {
        // Simplified - in production would use HTTP API
        val username = generateRandomUsername()
        val domain = "guerrillamail.com"
        val password = generateRandomPassword()
        
        return EmailAccount(
            address = "$username@$domain",
            password = password,
            provider = EmailProvider.GUERRILLAMAIL,
            createdAt = kotlinx.datetime.Clock.System.now().toString(),
            verified = false
        )
    }

    /**
     * Create self-hosted email
     */
    private fun createSelfHostedEmail(): EmailAccount {
        val username = generateRandomUsername()
        val domain = config.domain
        val password = generateRandomPassword()
        
        return EmailAccount(
            address = "$username@$domain",
            password = password,
            provider = EmailProvider.SELFHOSTED,
            createdAt = kotlinx.datetime.Clock.System.now().toString(),
            verified = false
        )
    }

    /**
     * Create custom email
     */
    private fun createCustomEmail(): EmailAccount {
        val username = config.username ?: generateRandomUsername()
        val domain = config.domain
        val password = config.password ?: generateRandomPassword()
        
        return EmailAccount(
            address = "$username@$domain",
            password = password,
            provider = EmailProvider.CUSTOM,
            createdAt = kotlinx.datetime.Clock.System.now().toString(),
            verified = false
        )
    }

    /**
     * Verify email account
     */
    private suspend fun verifyEmail(account: EmailAccount) {
        logInfo("验证邮箱 / Verifying email: ${account.address}")
        // Simplified - in production would actually verify email accessibility
        delay(1000)
    }

    /**
     * Get all created email accounts
     */
    fun getAllEmails(): List<EmailAccount> = accounts.toList()

    /**
     * Export emails to file
     */
    fun export(filename: String, format: ExportFormat): Result<Unit> = runCatchingResult {
        val json = Json { prettyPrint = true }
        
        when (format) {
            ExportFormat.JSON -> {
                val content = json.encodeToString(accounts)
                writeToFile(filename, content)
            }
            ExportFormat.CSV -> {
                val csv = buildString {
                    appendLine("address,password,provider,created_at,verified")
                    accounts.forEach { account ->
                        appendLine("${account.address},${account.password},${account.provider},${account.createdAt},${account.verified}")
                    }
                }
                writeToFile(filename, csv)
            }
            ExportFormat.TXT -> {
                val txt = buildString {
                    accounts.forEach { account ->
                        appendLine("${account.address}:${account.password}")
                    }
                }
                writeToFile(filename, txt)
            }
        }
        
        logInfo("邮箱导出成功 / Emails exported to: $filename")
    }

    private fun writeToFile(path: String, content: String) {
        val file = platform.posix.fopen(path, "w") 
            ?: throw Exception("Failed to open file for writing: $path")
        try {
            val bytes = content.encodeToByteArray()
            platform.posix.fwrite(bytes.refTo(0), 1, bytes.size.toULong(), file)
        } finally {
            platform.posix.fclose(file)
        }
    }

    // Helper functions

    private fun generateRandomUsername(): String {
        val chars = "abcdefghijklmnopqrstuvwxyz0123456789"
        val length = (8..12).random()
        return (1..length).map { chars.random() }.joinToString("")
    }

    private fun generateRandomPassword(): String {
        val chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%"
        return (1..16).map { chars.random() }.joinToString("")
    }
}

/**
 * Export format for email accounts
 */
enum class ExportFormat {
    JSON, CSV, TXT
}

/**
 * Extension to convert string to EmailProvider
 */
fun String.toEmailProvider(): EmailProvider? = when (this.uppercase()) {
    "MAILTM" -> EmailProvider.MAILTM
    "GUERRILLAMAIL" -> EmailProvider.GUERRILLAMAIL
    "SELFHOSTED" -> EmailProvider.SELFHOSTED
    "CUSTOM" -> EmailProvider.CUSTOM
    else -> null
}
