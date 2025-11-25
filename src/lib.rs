//! X 账号自动注册系统 - 库接口
//! X Account Auto Registration System - Library Interface
//!
//! 此文件提供库的公共 API，用于编译为动态链接库供其他程序调用

// 导出所有公共模块
pub mod batch;
pub mod bitbrowser;
pub mod browser_detector;
pub mod captcha;
pub mod config;
pub mod data_dir;
pub mod email;
pub mod email_provider;
pub mod gui;
pub mod i18n;
pub mod import_export;
pub mod logging;
pub mod registration;
pub mod visual;

// 重新导出常用类型
pub use config::Config;
pub use email::EmailService;
pub use registration::{XRegistration, AccountInfo, BirthDate};
pub use batch::BatchRegistrationManager;
pub use email_provider::{BatchEmailManager, EmailProvider, EmailProviderConfig};

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 初始化库（必须在使用其他功能前调用）
pub fn init() -> anyhow::Result<()> {
    // 初始化数据目录
    data_dir::init_directories()?;
    
    // 初始化日志系统
    logging::init_logging()?;
    
    Ok(())
}
