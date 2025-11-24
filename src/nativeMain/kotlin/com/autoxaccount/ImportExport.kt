package com.autoxaccount

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/**
 * Import/Export module
 * Migrated from Rust import_export.rs
 * 
 * Handles importing and exporting account data in various formats
 */

@Serializable
data class AccountData(
    val username: String,
    val email: String,
    val password: String? = null,
    val phone: String? = null,
    val createdAt: String? = null,
    val status: String? = null,
    val notes: String? = null
)

/**
 * Export accounts to file
 */
fun exportAccounts(accounts: List<AccountData>, filename: String, format: ExportFormat): Result<Unit> = runCatchingResult {
    logInfo("导出 ${accounts.size} 个账号到 $filename / Exporting ${accounts.size} accounts to $filename")

    when (format) {
        ExportFormat.JSON -> exportJson(accounts, filename)
        ExportFormat.CSV -> exportCsv(accounts, filename)
        ExportFormat.TXT -> exportTxt(accounts, filename)
    }

    logInfo("导出成功 / Export successful: $filename")
}

/**
 * Import accounts from file
 */
fun importAccounts(filename: String): Result<List<AccountData>> = runCatchingResult {
    logInfo("从文件导入账号 / Importing accounts from: $filename")

    val format = detectFormat(filename)
    val accounts = when (format) {
        ExportFormat.JSON -> importJson(filename)
        ExportFormat.CSV -> importCsv(filename)
        ExportFormat.TXT -> importTxt(filename)
    }

    logInfo("导入成功 / Import successful: ${accounts.size} 个账号 / accounts")
    accounts
}

/**
 * Detect file format from extension
 */
fun detectFormat(filename: String): ExportFormat {
    return when {
        filename.endsWith(".json", ignoreCase = true) -> ExportFormat.JSON
        filename.endsWith(".csv", ignoreCase = true) -> ExportFormat.CSV
        filename.endsWith(".txt", ignoreCase = true) -> ExportFormat.TXT
        else -> ExportFormat.JSON // Default to JSON
    }
}

/**
 * Export to JSON format
 */
private fun exportJson(accounts: List<AccountData>, filename: String) {
    val json = Json { prettyPrint = true }
    val content = json.encodeToString(accounts)
    writeToFile(filename, content)
}

/**
 * Export to CSV format
 */
private fun exportCsv(accounts: List<AccountData>, filename: String) {
    val csv = buildString {
        // Header
        appendLine("username,email,password,phone,created_at,status,notes")
        
        // Data rows
        accounts.forEach { account ->
            val row = listOf(
                account.username,
                account.email,
                account.password ?: "",
                account.phone ?: "",
                account.createdAt ?: "",
                account.status ?: "",
                account.notes ?: ""
            ).joinToString(",") { escapeCsvField(it) }
            
            appendLine(row)
        }
    }
    
    writeToFile(filename, csv)
}

/**
 * Export to TXT format (simple username:password:email)
 */
private fun exportTxt(accounts: List<AccountData>, filename: String) {
    val txt = buildString {
        accounts.forEach { account ->
            appendLine("${account.username}:${account.password ?: ""}:${account.email}")
        }
    }
    
    writeToFile(filename, txt)
}

/**
 * Import from JSON format
 */
private fun importJson(filename: String): List<AccountData> {
    val content = readFromFile(filename)
    val json = Json { ignoreUnknownKeys = true }
    return json.decodeFromString<List<AccountData>>(content)
}

/**
 * Import from CSV format
 */
private fun importCsv(filename: String): List<AccountData> {
    val content = readFromFile(filename)
    val lines = content.lines()
    
    if (lines.isEmpty()) {
        return emptyList()
    }

    val accounts = mutableListOf<AccountData>()
    
    // Skip header
    for (i in 1 until lines.size) {
        val line = lines[i].trim()
        if (line.isEmpty()) continue
        
        val fields = parseCsvLine(line)
        if (fields.size >= 2) {
            accounts.add(
                AccountData(
                    username = fields.getOrNull(0) ?: "",
                    email = fields.getOrNull(1) ?: "",
                    password = fields.getOrNull(2)?.takeIf { it.isNotEmpty() },
                    phone = fields.getOrNull(3)?.takeIf { it.isNotEmpty() },
                    createdAt = fields.getOrNull(4)?.takeIf { it.isNotEmpty() },
                    status = fields.getOrNull(5)?.takeIf { it.isNotEmpty() },
                    notes = fields.getOrNull(6)?.takeIf { it.isNotEmpty() }
                )
            )
        }
    }
    
    return accounts
}

/**
 * Import from TXT format
 */
private fun importTxt(filename: String): List<AccountData> {
    val content = readFromFile(filename)
    val lines = content.lines()
    
    val accounts = mutableListOf<AccountData>()
    
    for (line in lines) {
        val trimmed = line.trim()
        if (trimmed.isEmpty()) continue
        
        // Expected format: username:password:email
        val parts = trimmed.split(":")
        if (parts.size >= 2) {
            accounts.add(
                AccountData(
                    username = parts[0],
                    password = if (parts.size > 1 && parts[1].isNotEmpty()) parts[1] else null,
                    email = if (parts.size > 2) parts[2] else parts[0] // Use username as email if not provided
                )
            )
        }
    }
    
    return accounts
}

/**
 * Escape CSV field (handle commas and quotes)
 */
private fun escapeCsvField(field: String): String {
    return if (field.contains(',') || field.contains('"') || field.contains('\n')) {
        "\"${field.replace("\"", "\"\"")}\""
    } else {
        field
    }
}

/**
 * Parse CSV line (handle quoted fields)
 */
private fun parseCsvLine(line: String): List<String> {
    val fields = mutableListOf<String>()
    val currentField = StringBuilder()
    var inQuotes = false
    var i = 0
    
    while (i < line.length) {
        val char = line[i]
        
        when {
            char == '"' && !inQuotes -> {
                inQuotes = true
            }
            char == '"' && inQuotes -> {
                if (i + 1 < line.length && line[i + 1] == '"') {
                    currentField.append('"')
                    i++
                } else {
                    inQuotes = false
                }
            }
            char == ',' && !inQuotes -> {
                fields.add(currentField.toString())
                currentField.clear()
            }
            else -> {
                currentField.append(char)
            }
        }
        
        i++
    }
    
    fields.add(currentField.toString())
    return fields
}

/**
 * Write content to file
 */
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

/**
 * Read content from file
 */
private fun readFromFile(path: String): String {
    val file = platform.posix.fopen(path, "r") 
        ?: throw Exception("Failed to open file: $path")
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
