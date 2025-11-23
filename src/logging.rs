//! 日志系统模块
//! Logging system module

use anyhow::Result;
use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// 获取日志目录路径
/// Get log directory path based on OS
pub fn get_log_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Windows: %APPDATA%\auto-x-account\logs
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("auto-x-account").join("logs")
        } else {
            PathBuf::from("./logs")
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: ~/Library/Logs/auto-x-account
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join("Library")
                .join("Logs")
                .join("auto-x-account")
        } else {
            PathBuf::from("./logs")
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Linux/Unix: ~/.local/share/auto-x-account/logs or /var/log/auto-x-account
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("auto-x-account")
                .join("logs")
        } else if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data_home)
                .join("auto-x-account")
                .join("logs")
        } else {
            PathBuf::from("./logs")
        }
    }
}

/// 初始化日志系统
/// Initialize logging system with file and console output
pub fn init_logging() -> Result<()> {
    let log_dir = get_log_dir();
    
    // 确保日志目录存在
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)?;
    }

    // 生成日志文件名（包含日期）
    let log_file = log_dir.join(format!(
        "auto-x-account_{}.log",
        chrono::Local::now().format("%Y%m%d")
    ));

    // 创建文件日志
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    let file_layer = fmt::layer()
        .with_writer(std::sync::Arc::new(file))
        .with_ansi(false)
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(true);

    // 控制台日志
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(false)
        .with_thread_ids(false);

    // 组合日志层
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with(file_layer)
        .with(console_layer)
        .init();

    tracing::info!("日志系统已初始化 / Logging system initialized");
    tracing::info!("日志文件路径 / Log file path: {}", log_file.display());

    Ok(())
}

/// 清理旧日志文件（保留最近N天）
/// Clean up old log files (keep last N days)
pub fn cleanup_old_logs(days_to_keep: u32) -> Result<()> {
    let log_dir = get_log_dir();
    
    if !log_dir.exists() {
        return Ok(());
    }

    let cutoff_time = chrono::Local::now() - chrono::Duration::days(days_to_keep as i64);

    for entry in std::fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "log" {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            // Safely convert SystemTime to DateTime
                            match chrono::DateTime::<chrono::Local>::from(modified).partial_cmp(&cutoff_time) {
                                Some(std::cmp::Ordering::Less) => {
                                    tracing::info!("删除旧日志文件 / Removing old log file: {}", path.display());
                                    let _ = std::fs::remove_file(&path);
                                }
                                _ => {
                                    // File is newer or comparison failed, keep it
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// 获取日志文件列表
/// Get list of log files
pub fn list_log_files() -> Result<Vec<PathBuf>> {
    let log_dir = get_log_dir();
    let mut log_files = Vec::new();

    if !log_dir.exists() {
        return Ok(log_files);
    }

    for entry in std::fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "log" {
                    log_files.push(path);
                }
            }
        }
    }

    log_files.sort();
    Ok(log_files)
}
