package com.autoxaccount

import kotlinx.coroutines.*
import kotlinx.serialization.json.Json
import kotlinx.serialization.encodeToString

/**
 * Main entry point for the auto-x-account application
 * Migrated from Rust main.rs
 * 
 * X Account Auto Registration System - Kotlin/Native Edition
 */

suspend fun main(args: Array<String>) {
    // Initialize data directories
    DataDir.initDirectories().onFailure { error ->
        println("初始化数据目录失败 / Failed to initialize data directories: ${error.message}")
    }

    // Initialize logging system
    Logging.init().onFailure { error ->
        println("日志系统初始化失败 / Failed to initialize logging: ${error.message}")
    }

    // Print data directory info
    logInfo("\n${DataDir.getDataDirInfo()}")

    // Cleanup old logs and tasks
    Logging.cleanupOldLogs(30)
    DataDir.cleanupOldTasks(30)

    // Parse command line arguments
    val command = parseArgs(args)

    // Load or create configuration
    val config = DataDir.loadOrCreateConfig().getOrElse {
        logError("配置加载失败 / Failed to load config: ${it.message}")
        Config() // Use default config
    }

    // Initialize i18n
    val i18n = I18n(config.language)
    
    logInfo("=== ${i18n.t("app_name")} ===")

    // Execute command
    when (command) {
        is Command.Gui -> runGui(config, i18n)
        is Command.Register -> runSingleRegistration(config, command.email, command.proxy, i18n)
        is Command.Batch -> runBatchRegistration(config, command.count, command.concurrent, command.useExistingEmails, i18n)
        is Command.CreateEmails -> runCreateEmails(config, command.count, command.output, command.verify, i18n)
        is Command.Export -> runExportAccounts(command.output, command.format, i18n)
        is Command.Import -> runImportAccounts(command.input, i18n)
        is Command.DetectBrowser -> runBrowserDetection(config, command.verbose, i18n)
    }

    // Close logging system
    Logging.close()
}

/**
 * Command line arguments
 */
sealed class Command {
    object Gui : Command()
    
    data class Register(
        val email: String,
        val proxy: String? = null
    ) : Command()
    
    data class Batch(
        val count: Int,
        val concurrent: Int,
        val useExistingEmails: Boolean
    ) : Command()
    
    data class CreateEmails(
        val count: Int,
        val output: String?,
        val verify: Boolean
    ) : Command()
    
    data class Export(
        val output: String,
        val format: String
    ) : Command()
    
    data class Import(
        val input: String
    ) : Command()
    
    data class DetectBrowser(
        val verbose: Boolean
    ) : Command()
}

/**
 * Parse command line arguments
 */
fun parseArgs(args: Array<String>): Command {
    if (args.isEmpty()) {
        return Command.Gui
    }

    return when (args[0]) {
        "gui" -> Command.Gui
        
        "register" -> {
            val email = args.getOrNull(1) ?: throw Exception("Email is required")
            val proxy = if (args.size > 2 && args[2] == "--proxy") args.getOrNull(3) else null
            Command.Register(email, proxy)
        }
        
        "batch" -> {
            val count = args.getOrNull(1)?.toIntOrNull() ?: 10
            val concurrent = args.getOrNull(2)?.toIntOrNull() ?: 3
            val useExistingEmails = args.contains("--use-existing-emails")
            Command.Batch(count, concurrent, useExistingEmails)
        }
        
        "create-emails" -> {
            val count = args.getOrNull(1)?.toIntOrNull() ?: 10
            val output = if (args.contains("--output")) {
                args.getOrNull(args.indexOf("--output") + 1)
            } else null
            val verify = args.contains("--verify")
            Command.CreateEmails(count, output, verify)
        }
        
        "export" -> {
            val output = args.getOrNull(1) ?: "accounts_export.xlsx"
            val format = args.getOrNull(2) ?: "xlsx"
            Command.Export(output, format)
        }
        
        "import" -> {
            val input = args.getOrNull(1) ?: throw Exception("Input file is required")
            Command.Import(input)
        }
        
        "detect-browser" -> {
            val verbose = args.contains("--verbose")
            Command.DetectBrowser(verbose)
        }
        
        else -> {
            println("未知命令 / Unknown command: ${args[0]}")
            println("可用命令 / Available commands: gui, register, batch, create-emails, export, import, detect-browser")
            Command.Gui
        }
    }
}

/**
 * Run GUI mode
 */
suspend fun runGui(config: Config, i18n: I18n) {
    logInfo("${i18n.t("starting")} GUI 模式...")
    println("注意 / Note: GUI 功能正在开发中 / GUI functionality is under development")
    println("请使用命令行模式 / Please use CLI mode")
}

/**
 * Run single registration
 */
suspend fun runSingleRegistration(config: Config, email: String, proxy: String?, i18n: I18n) {
    logInfo("${i18n.t("starting")} 单账号注册...")

    // Create dummy email handler for now
    val emailHandler = object : EmailHandler {
        override suspend fun checkForVerificationCode(email: String): String? {
            // Simulate verification code
            delay(5000)
            return "123456"
        }

        override suspend fun getEmails(email: String): List<EmailMessage> {
            return emptyList()
        }
    }

    val registration = XRegistration(config, emailHandler)
    
    val result = registration.registerAccount(email)
    
    result.onSuccess { account ->
        logInfo("${i18n.t("registration_success")}")
        logInfo("用户名 / Username: ${account.username}")
        logInfo("邮箱 / Email: ${account.email}")
        
        // Save account info
        saveAccountInfo(account)
    }.onFailure { error ->
        logError("${i18n.t("registration_failed")}: ${error.message}")
    }
}

/**
 * Run batch registration
 */
suspend fun runBatchRegistration(
    config: Config,
    count: Int,
    concurrent: Int,
    useExistingEmails: Boolean,
    i18n: I18n
) {
    logInfo("${i18n.t("batch_registration_start")}: $count 个账号，并发数: $concurrent")
    
    println("注意 / Note: 批量注册功能正在开发中 / Batch registration is under development")
}

/**
 * Run create emails
 */
suspend fun runCreateEmails(
    config: Config,
    count: Int,
    output: String?,
    verify: Boolean,
    i18n: I18n
) {
    logInfo("${i18n.t("email_creating")}: $count 个邮箱")
    
    println("注意 / Note: 邮箱创建功能正在开发中 / Email creation is under development")
}

/**
 * Run export accounts
 */
suspend fun runExportAccounts(output: String, format: String, i18n: I18n) {
    logInfo("导出账号到文件: $output")
    
    val accountsPath = DataDir.getAccountsPath()
    
    println("从 $accountsPath 导出到 $output ($format 格式)")
    println("注意 / Note: 导出功能正在开发中 / Export functionality is under development")
}

/**
 * Run import accounts
 */
suspend fun runImportAccounts(input: String, i18n: I18n) {
    logInfo("从文件导入账号: $input")
    
    println("注意 / Note: 导入功能正在开发中 / Import functionality is under development")
}

/**
 * Run browser detection
 */
suspend fun runBrowserDetection(config: Config, verbose: Boolean, i18n: I18n) {
    logInfo("🔍 开始浏览器环境检测 / Starting browser environment detection")
    
    println("═══════════════════════════════════════════════")
    println("  浏览器环境检测报告 / Browser Environment Report")
    println("═══════════════════════════════════════════════")
    println()
    println("注意 / Note: 浏览器检测功能正在开发中 / Browser detection is under development")
}

/**
 * Save account information
 */
fun saveAccountInfo(account: AccountInfo) {
    val accountsPath = DataDir.getAccountsPath()
    
    // Read existing accounts
    val existingAccounts = if (fileExists(accountsPath)) {
        try {
            val content = readFileContent(accountsPath)
            Json.decodeFromString<List<AccountInfo>>(content).toMutableList()
        } catch (e: Exception) {
            mutableListOf()
        }
    } else {
        mutableListOf()
    }
    
    // Add new account
    existingAccounts.add(account)
    
    // Save to file
    val json = Json { prettyPrint = true }
    val content = json.encodeToString(existingAccounts)
    writeFileContent(accountsPath, content)
    
    logInfo("账号信息已保存到 / Account info saved to: $accountsPath")
}

// Helper functions

private fun fileExists(path: String): Boolean {
    return platform.posix.access(path, platform.posix.F_OK) == 0
}

private fun readFileContent(path: String): String {
    val file = platform.posix.fopen(path, "r") ?: throw Exception("Failed to open file: $path")
    try {
        val buffer = StringBuilder()
        val chunk = ByteArray(4096)
        while (true) {
            val bytesRead = platform.posix.fread(chunk.refTo(0), 1, chunk.size.toULong(), file).toInt()
            if (bytesRead <= 0) break
            buffer.append(chunk.decodeToString(0, bytesRead))
        }
        return buffer.toString()
    } finally {
        platform.posix.fclose(file)
    }
}

private fun writeFileContent(path: String, content: String) {
    val file = platform.posix.fopen(path, "w") ?: throw Exception("Failed to open file for writing: $path")
    try {
        val bytes = content.encodeToByteArray()
        platform.posix.fwrite(bytes.refTo(0), 1, bytes.size.toULong(), file)
    } finally {
        platform.posix.fclose(file)
    }
}
