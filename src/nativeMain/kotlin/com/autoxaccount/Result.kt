package com.autoxaccount

/**
 * Kotlin Result type to replace Rust's Result<T, E>
 * Using Kotlin's built-in Result with extensions
 */

/**
 * Extension to convert Result to nullable value
 */
fun <T> Result<T>.toNullable(): T? = getOrNull()

/**
 * Extension to check if result is Ok
 */
fun <T> Result<T>.isOk(): Boolean = isSuccess

/**
 * Extension to check if result is Err
 */
fun <T> Result<T>.isErr(): Boolean = isFailure

/**
 * Extension to map error
 */
inline fun <T> Result<T>.mapError(transform: (Throwable) -> Throwable): Result<T> =
    when {
        isSuccess -> this
        else -> Result.failure(transform(exceptionOrNull()!!))
    }

/**
 * Extension for unwrap or throw (like Rust's unwrap)
 */
fun <T> Result<T>.unwrap(): T = getOrThrow()

/**
 * Extension for unwrap or default value
 */
fun <T> Result<T>.unwrapOr(default: T): T = getOrDefault(default)

/**
 * Extension for unwrap or compute default
 */
inline fun <T> Result<T>.unwrapOrElse(onFailure: (Throwable) -> T): T = getOrElse(onFailure)

/**
 * Create a Result from a nullable value
 */
fun <T : Any> T?.toResult(error: () -> Throwable = { NullPointerException("Value is null") }): Result<T> =
    this?.let { Result.success(it) } ?: Result.failure(error())

/**
 * Catch exceptions and return Result
 */
inline fun <T> runCatchingResult(block: () -> T): Result<T> = runCatching(block)
