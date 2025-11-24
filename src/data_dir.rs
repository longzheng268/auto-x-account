//! 数据目录管理模块
//! Data directory management module
//! 
//! 负责管理应用程序的数据目录，确保所有数据持久化到正确的位置

use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// 获取应用程序数据目录
/// Get application data directory based on OS
pub fn get_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Windows: %APPDATA%\auto-x-account
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("auto-x-account")
        } else {
            PathBuf::from(".").join("data")
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: ~/Library/Application Support/auto-x-account
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("auto-x-account")
        } else {
            PathBuf::from(".").join("data")
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Linux/Unix: ~/.local/share/auto-x-account
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("auto-x-account")
        } else if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data_home).join("auto-x-account")
        } else {
            PathBuf::from(".").join("data")
        }
    }
}

/// 获取配置文件路径
/// Get configuration file path
pub fn get_config_path() -> PathBuf {
    get_data_dir().join("config.json")
}

/// 获取账号数据文件路径
/// Get accounts data file path
pub fn get_accounts_path() -> PathBuf {
    get_data_dir().join("accounts.json")
}

/// 获取邮箱数据文件路径
/// Get emails data file path
pub fn get_emails_path() -> PathBuf {
    get_data_dir().join("emails.json")
}

/// 获取批量任务数据目录
/// Get batch tasks data directory
pub fn get_tasks_dir() -> PathBuf {
    get_data_dir().join("tasks")
}

/// 获取截图目录
/// Get screenshots directory
pub fn get_screenshots_dir() -> PathBuf {
    get_data_dir().join("screenshots")
}

/// 获取浏览器数据目录
/// Get browser data directory
pub fn get_browser_data_dir() -> PathBuf {
    get_data_dir().join("browser_data")
}

/// 初始化所有必需的目录
/// Initialize all required directories
pub fn init_directories() -> Result<()> {
    let dirs = vec![
        get_data_dir(),
        get_tasks_dir(),
        get_screenshots_dir(),
        get_browser_data_dir(),
    ];

    for dir in dirs {
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
            info!("创建目录 / Created directory: {}", dir.display());
        }
    }

    Ok(())
}

/// 从数据目录加载配置文件，如果不存在则创建默认配置
/// Load config from data directory, create default if not exists
pub fn load_or_create_config() -> Result<crate::config::Config> {
    let config_path = get_config_path();
    
    if config_path.exists() {
        info!("从数据目录加载配置 / Loading config from: {}", config_path.display());
        crate::config::Config::from_file(&config_path)
    } else {
        info!("配置文件不存在，创建默认配置 / Config not found, creating default");
        let config = crate::config::Config::default();
        
        // 保存默认配置到数据目录
        config.to_file(&config_path)?;
        info!("默认配置已保存到 / Default config saved to: {}", config_path.display());
        
        // 同时在当前目录创建示例配置（仅用于参考）
        let example_path = PathBuf::from("config.example.json");
        if !example_path.exists() {
            config.to_file(&example_path)?;
            info!("示例配置已创建 / Example config created: {}", example_path.display());
        }
        
        Ok(config)
    }
}

/// 保存配置文件到数据目录
/// Save config to data directory
pub fn save_config(config: &crate::config::Config) -> Result<()> {
    let config_path = get_config_path();
    config.to_file(&config_path)?;
    info!("配置已保存到 / Config saved to: {}", config_path.display());
    Ok(())
}

/// 获取数据目录的信息（用于显示）
/// Get data directory information for display
pub fn get_data_dir_info() -> String {
    format!(
        "数据目录 / Data Directory:\n\
        主目录 / Main: {}\n\
        配置文件 / Config: {}\n\
        账号数据 / Accounts: {}\n\
        邮箱数据 / Emails: {}\n\
        任务数据 / Tasks: {}\n\
        截图目录 / Screenshots: {}\n\
        浏览器数据 / Browser: {}\n\
        日志目录 / Logs: {}",
        get_data_dir().display(),
        get_config_path().display(),
        get_accounts_path().display(),
        get_emails_path().display(),
        get_tasks_dir().display(),
        get_screenshots_dir().display(),
        get_browser_data_dir().display(),
        crate::logging::get_log_dir().display()
    )
}

/// 清理旧的任务数据（保留最近N天）
/// Clean up old task data (keep last N days)
pub fn cleanup_old_tasks(days_to_keep: u32) -> Result<()> {
    let tasks_dir = get_tasks_dir();
    
    if !tasks_dir.exists() {
        return Ok(());
    }

    let cutoff_time = chrono::Local::now() - chrono::Duration::days(days_to_keep as i64);

    for entry in std::fs::read_dir(tasks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    match chrono::DateTime::<chrono::Local>::from(modified).partial_cmp(&cutoff_time) {
                        Some(std::cmp::Ordering::Less) => {
                            info!("删除旧任务文件 / Removing old task file: {}", path.display());
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_data_dir() {
        let dir = get_data_dir();
        assert!(dir.to_string_lossy().contains("auto-x-account"));
    }

    #[test]
    fn test_get_config_path() {
        let path = get_config_path();
        assert!(path.to_string_lossy().ends_with("config.json"));
    }
}
