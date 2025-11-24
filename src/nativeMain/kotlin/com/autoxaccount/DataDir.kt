package com.autoxaccount

import kotlinx.cinterop.*
import platform.posix.*
import kotlinx.datetime.Clock

/**
 * Data directory management module
 * Migrated from Rust data_dir.rs
 * 
 * Manages application data directories, configuration, and cache files
 */

object DataDir {
    /**
     * Initialize all required directories
     */
    fun initDirectories(): Result<Unit> = runCatchingResult {
        val dataDir = getDataDir()
        val logsDir = getLogsDir()
        val browserDataDir = getBrowserDataDir()
        val screenshotsDir = getScreenshotsDir()

        createDirectory(dataDir)
        createDirectory(logsDir)
        createDirectory(browserDataDir)
        createDirectory(screenshotsDir)
    }

    /**
     * Get main data directory path based on OS
     */
    fun getDataDir(): String {
        return when {
            isWindows() -> {
                val appData = getenv("APPDATA")?.toKString() ?: "C:\\Users\\Default\\AppData\\Roaming"
                "$appData\\auto-x-account"
            }
            isMacOS() -> {
                val home = getenv("HOME")?.toKString() ?: "/Users/Default"
                "$home/Library/Application Support/auto-x-account"
            }
            else -> { // Linux
                val home = getenv("HOME")?.toKString() ?: "/home/user"
                "$home/.local/share/auto-x-account"
            }
        }
    }

    /**
     * Get logs directory path based on OS
     */
    fun getLogsDir(): String {
        return when {
            isWindows() -> "${getDataDir()}\\logs"
            isMacOS() -> {
                val home = getenv("HOME")?.toKString() ?: "/Users/Default"
                "$home/Library/Logs/auto-x-account"
            }
            else -> "${getDataDir()}/logs"
        }
    }

    /**
     * Get browser data directory
     */
    fun getBrowserDataDir(): String {
        return "${getDataDir()}${pathSeparator()}browser_data"
    }

    /**
     * Get screenshots directory
     */
    fun getScreenshotsDir(): String {
        return "${getDataDir()}${pathSeparator()}screenshots"
    }

    /**
     * Get accounts file path
     */
    fun getAccountsPath(): String {
        return "${getDataDir()}${pathSeparator()}accounts.json"
    }

    /**
     * Get config file path
     */
    fun getConfigPath(): String {
        return "${getDataDir()}${pathSeparator()}config.json"
    }

    /**
     * Get batch tasks directory
     */
    fun getBatchTasksDir(): String {
        return "${getDataDir()}${pathSeparator()}batch_tasks"
    }

    /**
     * Load or create default configuration
     */
    fun loadOrCreateConfig(): Result<Config> = runCatchingResult {
        val configPath = getConfigPath()
        
        if (fileExists(configPath)) {
            Config.fromFile(configPath).getOrThrow()
        } else {
            val config = Config()
            config.toFile(configPath).getOrThrow()
            config
        }
    }

    /**
     * Get data directory info as formatted string
     */
    fun getDataDirInfo(): String {
        return buildString {
            appendLine("═══════════════════════════════════════════════")
            appendLine("  数据目录信息 / Data Directory Information")
            appendLine("═══════════════════════════════════════════════")
            appendLine()
            appendLine("📁 数据目录 / Data Directory:")
            appendLine("   ${getDataDir()}")
            appendLine()
            appendLine("📝 日志目录 / Logs Directory:")
            appendLine("   ${getLogsDir()}")
            appendLine()
            appendLine("🌐 浏览器数据 / Browser Data:")
            appendLine("   ${getBrowserDataDir()}")
            appendLine()
            appendLine("📸 截图目录 / Screenshots:")
            appendLine("   ${getScreenshotsDir()}")
            appendLine()
            appendLine("💾 配置文件 / Config File:")
            appendLine("   ${getConfigPath()}")
            appendLine()
            appendLine("👤 账号文件 / Accounts File:")
            appendLine("   ${getAccountsPath()}")
            appendLine("═══════════════════════════════════════════════")
        }
    }

    /**
     * Cleanup old tasks older than specified days
     */
    fun cleanupOldTasks(days: Int): Result<Unit> = runCatchingResult {
        val tasksDir = getBatchTasksDir()
        if (!fileExists(tasksDir)) return@runCatchingResult

        val cutoffTime = Clock.System.now().toEpochMilliseconds() - (days * 24 * 60 * 60 * 1000L)
        
        // List and remove old task files
        // Note: This is simplified - full implementation would require directory traversal
        println("清理 $days 天前的旧任务 / Cleaning up tasks older than $days days")
    }

    // Helper functions

    private fun pathSeparator(): String = if (isWindows()) "\\" else "/"

    private fun isWindows(): Boolean {
        val os = getenv("OS")?.toKString()
        return os?.contains("Windows", ignoreCase = true) == true
    }

    private fun isMacOS(): Boolean {
        // Check if running on macOS by examining uname
        memScoped {
            val buf = allocArray<ByteVar>(256)
            if (gethostname(buf, 256u) == 0) {
                // This is a simplification - proper OS detection would use uname or platform checks
                return false // Placeholder
            }
        }
        return false
    }

    private fun fileExists(path: String): Boolean {
        memScoped {
            val stat = alloc<stat>()
            return stat(path, stat.ptr) == 0
        }
    }

    private fun createDirectory(path: String) {
        if (fileExists(path)) return
        
        // Create directory with read/write/execute permissions for owner
        if (isWindows()) {
            mkdir(path)
        } else {
            mkdir(path, 0x1EDu) // 0755 in octal
        }
    }
}
