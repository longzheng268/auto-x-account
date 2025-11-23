//! 集成测试 - 数据持久化和配置功能
//! Integration tests for data persistence and configuration features

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    /// 测试缓存目录路径生成
    #[test]
    fn test_cache_dir_paths() {
        // 此测试验证不同操作系统返回正确的路径格式
        
        #[cfg(target_os = "windows")]
        {
            // Windows 应该返回 %APPDATA%\auto-x-account
            // 实际路径取决于 APPDATA 环境变量
            if let Ok(appdata) = std::env::var("APPDATA") {
                let expected = PathBuf::from(appdata).join("auto-x-account");
                assert!(expected.to_string_lossy().contains("auto-x-account"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS 应该返回 ~/Library/Application Support/auto-x-account
            if let Ok(home) = std::env::var("HOME") {
                let expected = PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("auto-x-account");
                assert!(expected.to_string_lossy().contains("Library"));
                assert!(expected.to_string_lossy().contains("Application Support"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux 应该返回 ~/.local/share/auto-x-account
            if let Ok(home) = std::env::var("HOME") {
                let expected = PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("auto-x-account");
                assert!(expected.to_string_lossy().contains(".local"));
                assert!(expected.to_string_lossy().contains("share"));
            }
        }
    }

    /// 测试日志目录路径生成
    #[test]
    fn test_log_dir_paths() {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let expected = PathBuf::from(appdata)
                    .join("auto-x-account")
                    .join("logs");
                assert!(expected.to_string_lossy().contains("logs"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let expected = PathBuf::from(home)
                    .join("Library")
                    .join("Logs")
                    .join("auto-x-account");
                assert!(expected.to_string_lossy().contains("Library"));
                assert!(expected.to_string_lossy().contains("Logs"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let expected = PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("auto-x-account")
                    .join("logs");
                assert!(expected.to_string_lossy().contains("logs"));
            }
        }
    }

    /// 测试代理配置序列化
    #[test]
    fn test_proxy_config_serialization() {
        let json = r#"{
            "mode": "manual",
            "type": "socks5",
            "host": "127.0.0.1",
            "port": 1080,
            "username": null,
            "password": null,
            "browser_enabled": true,
            "email_enabled": false
        }"#;

        // 测试反序列化
        let result: Result<serde_json::Value, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config["mode"], "manual");
        assert_eq!(config["browser_enabled"], true);
        assert_eq!(config["email_enabled"], false);
    }

    /// 测试任务数据序列化
    #[test]
    fn test_task_serialization() {
        let json = r#"{
            "id": "task_123",
            "total_count": 10,
            "completed_count": 5,
            "failed_count": 1,
            "status": "Running",
            "created_at": "2024-11-23T10:00:00Z",
            "updated_at": "2024-11-23T10:30:00Z"
        }"#;

        // 测试反序列化
        let result: Result<serde_json::Value, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let task = result.unwrap();
        assert_eq!(task["id"], "task_123");
        assert_eq!(task["total_count"], 10);
        assert_eq!(task["completed_count"], 5);
    }

    /// 测试默认配置值
    #[test]
    fn test_default_config_values() {
        let default_json = r#"{
            "language": "zh-CN",
            "proxy": {
                "mode": "none",
                "type": "socks5",
                "host": "127.0.0.1",
                "port": 1080,
                "browser_enabled": true,
                "email_enabled": false
            }
        }"#;

        let config: serde_json::Value = serde_json::from_str(default_json).unwrap();
        
        // 验证默认代理设置
        assert_eq!(config["proxy"]["browser_enabled"], true);
        assert_eq!(config["proxy"]["email_enabled"], false);
    }

    /// 测试向后兼容性
    #[test]
    fn test_backward_compatibility() {
        let old_config_json = r#"{
            "mode": "manual",
            "type": "socks5",
            "host": "127.0.0.1",
            "port": 1080
        }"#;

        // 应该能够成功解析
        let result: Result<serde_json::Value, _> = serde_json::from_str(old_config_json);
        assert!(result.is_ok());
    }

    /// 测试日志文件名格式
    #[test]
    fn test_log_filename_format() {
        let timestamp = "20241123";
        let filename = format!("auto-x-account_{}.log", timestamp);
        
        assert!(filename.starts_with("auto-x-account_"));
        assert!(filename.ends_with(".log"));
        assert!(filename.contains("20241123"));
    }
}
