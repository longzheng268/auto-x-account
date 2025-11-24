package com.autoxaccount

import kotlinx.coroutines.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.datetime.Clock

/**
 * Batch registration module
 * Migrated from Rust batch.rs
 * 
 * Manages batch registration of multiple X accounts with concurrency control
 */

@Serializable
data class BatchTask(
    val id: String,
    val totalAccounts: Int,
    val completedAccounts: Int = 0,
    val failedAccounts: Int = 0,
    val status: BatchStatus,
    val createdAt: String,
    val updatedAt: String,
    val accounts: List<AccountInfo> = emptyList()
)

@Serializable
enum class BatchStatus {
    PENDING,    // Task created but not started
    RUNNING,    // Task is currently running
    PAUSED,     // Task is paused
    COMPLETED,  // Task completed successfully
    FAILED,     // Task failed
    CANCELLED   // Task was cancelled
}

data class BatchStats(
    val total: Int,
    val completed: Int,
    val failed: Int,
    val pending: Int,
    val progress: Double,
    val status: BatchStatus
)

/**
 * Batch registration manager
 */
class BatchRegistrationManager(
    private val config: Config,
    private val emailHandler: EmailHandler,
    private val emailManager: BatchEmailManager
) {
    private val tasks = mutableMapOf<String, BatchTask>()
    private val runningJobs = mutableMapOf<String, Job>()
    private val json = Json { prettyPrint = true }

    /**
     * Start batch registration task
     */
    suspend fun startBatchRegistration(
        count: Int,
        concurrent: Int,
        useExistingEmails: Boolean
    ): Result<String> = runCatchingResult {
        logInfo("启动批量注册任务 / Starting batch registration task")
        logInfo("总数 / Total: $count, 并发数 / Concurrent: $concurrent")

        // Generate task ID
        val taskId = generateTaskId()

        // Create task
        val task = BatchTask(
            id = taskId,
            totalAccounts = count,
            status = BatchStatus.PENDING,
            createdAt = Clock.System.now().toString(),
            updatedAt = Clock.System.now().toString()
        )

        tasks[taskId] = task

        // Start registration in background
        val job = CoroutineScope(Dispatchers.Default).launch {
            try {
                executeBatchRegistration(taskId, count, concurrent, useExistingEmails)
            } catch (e: Exception) {
                logError("批量注册失败 / Batch registration failed: ${e.message}")
                updateTaskStatus(taskId, BatchStatus.FAILED)
            }
        }

        runningJobs[taskId] = job

        logInfo("批量注册任务已创建 / Batch registration task created: $taskId")
        taskId
    }

    /**
     * Execute batch registration
     */
    private suspend fun executeBatchRegistration(
        taskId: String,
        count: Int,
        concurrent: Int,
        useExistingEmails: Boolean
    ) {
        updateTaskStatus(taskId, BatchStatus.RUNNING)

        val accounts = mutableListOf<AccountInfo>()
        val failedCount = mutableListOf<Int>()

        // Get or create email accounts
        val emails = if (useExistingEmails) {
            emailManager.getAllEmails().map { it.address }
        } else {
            // Create new emails
            emailManager.createBatch(count, false)
            emailManager.getAllEmails().map { it.address }
        }

        if (emails.size < count) {
            logWarn("邮箱数量不足 / Insufficient emails: ${emails.size} < $count")
        }

        // Process accounts with concurrency control
        val semaphore = kotlinx.coroutines.sync.Semaphore(concurrent)
        val jobs = mutableListOf<Deferred<Result<AccountInfo>>>()

        for (i in 0 until minOf(count, emails.size)) {
            val email = emails[i]
            
            val deferred = CoroutineScope(Dispatchers.Default).async {
                semaphore.withPermit {
                    registerAccountWithRetry(email)
                }
            }
            
            jobs.add(deferred)
        }

        // Wait for all jobs to complete
        for ((index, job) in jobs.withIndex()) {
            try {
                val result = job.await()
                
                result.onSuccess { account ->
                    accounts.add(account)
                    updateTaskProgress(taskId, accounts.size, failedCount.size)
                    logInfo("[${index + 1}/$count] 注册成功 / Registered: ${account.username}")
                }.onFailure { error ->
                    failedCount.add(index)
                    updateTaskProgress(taskId, accounts.size, failedCount.size)
                    logError("[${index + 1}/$count] 注册失败 / Failed: ${error.message}")
                }
            } catch (e: Exception) {
                failedCount.add(index)
                updateTaskProgress(taskId, accounts.size, failedCount.size)
                logError("[${index + 1}/$count] 注册异常 / Exception: ${e.message}")
            }
        }

        // Update final task state
        val finalTask = tasks[taskId]?.copy(
            accounts = accounts,
            completedAccounts = accounts.size,
            failedAccounts = failedCount.size,
            status = BatchStatus.COMPLETED,
            updatedAt = Clock.System.now().toString()
        )

        if (finalTask != null) {
            tasks[taskId] = finalTask
            saveTaskToCache(finalTask)
        }

        logInfo("批量注册完成 / Batch registration completed")
        logInfo("成功 / Success: ${accounts.size}, 失败 / Failed: ${failedCount.size}")
    }

    /**
     * Register account with retry logic
     */
    private suspend fun registerAccountWithRetry(email: String): Result<AccountInfo> {
        val maxRetries = config.xAccount.retryTimes.toInt()
        var lastError: Throwable? = null

        repeat(maxRetries) { attempt ->
            try {
                val registration = XRegistration(config, emailHandler)
                return registration.registerAccount(email)
            } catch (e: Exception) {
                lastError = e
                if (attempt < maxRetries - 1) {
                    logWarn("注册失败，重试 / Registration failed, retrying: ${attempt + 1}/$maxRetries")
                    delay(3000) // Wait before retry
                }
            }
        }

        return Result.failure(lastError ?: Exception("Registration failed"))
    }

    /**
     * Get task statistics
     */
    suspend fun getTaskStats(taskId: String): BatchStats? {
        val task = tasks[taskId] ?: return null

        val pending = task.totalAccounts - task.completedAccounts - task.failedAccounts
        val progress = if (task.totalAccounts > 0) {
            (task.completedAccounts + task.failedAccounts).toDouble() / task.totalAccounts * 100.0
        } else {
            0.0
        }

        return BatchStats(
            total = task.totalAccounts,
            completed = task.completedAccounts,
            failed = task.failedAccounts,
            pending = pending,
            progress = progress,
            status = task.status
        )
    }

    /**
     * Export accounts to file
     */
    suspend fun exportAccounts(filename: String, format: ExportFormat): Result<Unit> = runCatchingResult {
        val allAccounts = tasks.values.flatMap { it.accounts }
        
        if (allAccounts.isEmpty()) {
            logWarn("没有账号可导出 / No accounts to export")
            return@runCatchingResult
        }

        when (format) {
            ExportFormat.JSON -> {
                val content = json.encodeToString(allAccounts)
                writeToFile(filename, content)
            }
            ExportFormat.CSV -> {
                val csv = buildString {
                    appendLine("email,name,username,password,phone,birth_date,created_at,status")
                    allAccounts.forEach { acc ->
                        appendLine("${acc.email},${acc.name},${acc.username},${acc.password},${acc.phone ?: ""},${acc.birthDate.year}-${acc.birthDate.month}-${acc.birthDate.day},${acc.createdAt},${acc.status}")
                    }
                }
                writeToFile(filename, csv)
            }
            ExportFormat.TXT -> {
                val txt = buildString {
                    allAccounts.forEach { acc ->
                        appendLine("${acc.username}:${acc.password}:${acc.email}")
                    }
                }
                writeToFile(filename, txt)
            }
        }

        logInfo("账号导出成功 / Accounts exported: $filename")
    }

    /**
     * Load tasks from cache
     */
    suspend fun loadFromCache(): Result<Unit> = runCatchingResult {
        val cacheDir = DataDir.getBatchTasksDir()
        logInfo("从缓存加载任务 / Loading tasks from cache: $cacheDir")
        
        // Simplified - would list and load all task files
        logInfo("任务加载完成 / Tasks loaded")
    }

    // Helper methods

    private fun generateTaskId(): String {
        val timestamp = Clock.System.now().toEpochMilliseconds()
        val random = (1000..9999).random()
        return "batch_${timestamp}_$random"
    }

    private fun updateTaskStatus(taskId: String, status: BatchStatus) {
        tasks[taskId]?.let { task ->
            tasks[taskId] = task.copy(
                status = status,
                updatedAt = Clock.System.now().toString()
            )
        }
    }

    private fun updateTaskProgress(taskId: String, completed: Int, failed: Int) {
        tasks[taskId]?.let { task ->
            tasks[taskId] = task.copy(
                completedAccounts = completed,
                failedAccounts = failed,
                updatedAt = Clock.System.now().toString()
            )
        }
    }

    private fun saveTaskToCache(task: BatchTask) {
        try {
            val cacheDir = DataDir.getBatchTasksDir()
            val filename = "$cacheDir/${task.id}.json"
            val content = json.encodeToString(task)
            writeToFile(filename, content)
        } catch (e: Exception) {
            logError("保存任务缓存失败 / Failed to save task cache: ${e.message}")
        }
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
}
