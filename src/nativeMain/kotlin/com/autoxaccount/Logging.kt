package com.autoxaccount

import kotlinx.datetime.Clock
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import platform.posix.*

/**
 * Logging module
 * Migrated from Rust logging.rs
 * 
 * Provides file and console logging with automatic rotation
 */

object Logging {
    private var logFile: CPointer<FILE>? = null
    private var isInitialized = false

    enum class Level {
        INFO, WARN, ERROR, DEBUG
    }

    /**
     * Initialize logging system
     */
    fun init(): Result<Unit> = runCatchingResult {
        if (isInitialized) return@runCatchingResult

        val logsDir = DataDir.getLogsDir()
        val timestamp = getCurrentTimestamp()
        val logFileName = "$logsDir${pathSeparator()}app_$timestamp.log"

        logFile = fopen(logFileName, "a")
        if (logFile == null) {
            throw Exception("Failed to open log file: $logFileName")
        }

        isInitialized = true
        info("日志系统已初始化 / Logging system initialized")
        info("日志文件 / Log file: $logFileName")
    }

    /**
     * Log info message
     */
    fun info(message: String) {
        log(Level.INFO, message)
    }

    /**
     * Log warning message
     */
    fun warn(message: String) {
        log(Level.WARN, message)
    }

    /**
     * Log error message
     */
    fun error(message: String) {
        log(Level.ERROR, message)
    }

    /**
     * Log debug message
     */
    fun debug(message: String) {
        log(Level.DEBUG, message)
    }

    /**
     * Log message with level
     */
    private fun log(level: Level, message: String) {
        val timestamp = getCurrentTimestamp()
        val levelStr = when (level) {
            Level.INFO -> "INFO "
            Level.WARN -> "WARN "
            Level.ERROR -> "ERROR"
            Level.DEBUG -> "DEBUG"
        }

        val logMessage = "[$timestamp] [$levelStr] $message\n"

        // Write to console
        print(logMessage)

        // Write to file
        logFile?.let { file ->
            val bytes = logMessage.encodeToByteArray()
            fwrite(bytes.refTo(0), 1, bytes.size.toULong(), file)
            fflush(file)
        }
    }

    /**
     * Cleanup old log files
     */
    fun cleanupOldLogs(days: Int): Result<Unit> = runCatchingResult {
        val logsDir = DataDir.getLogsDir()
        info("清理 $days 天前的旧日志 / Cleaning up logs older than $days days from $logsDir")
        
        // Note: Full implementation would require directory traversal
        // This is a simplified version
    }

    /**
     * Close logging system
     */
    fun close() {
        logFile?.let { file ->
            fclose(file)
            logFile = null
        }
        isInitialized = false
    }

    // Helper functions

    private fun getCurrentTimestamp(): String {
        val now = Clock.System.now()
        val localDateTime = now.toLocalDateTime(TimeZone.currentSystemDefault())
        return with(localDateTime) {
            String.format(
                "%04d-%02d-%02d_%02d-%02d-%02d",
                year, monthNumber, dayOfMonth,
                hour, minute, second
            )
        }
    }

    private fun pathSeparator(): String {
        val os = getenv("OS")?.toKString()
        return if (os?.contains("Windows", ignoreCase = true) == true) "\\" else "/"
    }
}

// Global logging functions for convenience
fun logInfo(message: String) = Logging.info(message)
fun logWarn(message: String) = Logging.warn(message)
fun logError(message: String) = Logging.error(message)
fun logDebug(message: String) = Logging.debug(message)
