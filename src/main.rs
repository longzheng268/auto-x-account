//! X 账号自动注册系统 - 主程序
//! X Account Auto Registration System - Main Program
//!
//! 支持 SMTP 邮箱验证、代理访问、多语言界面、批量注册
//! Supports SMTP email verification, proxy access, multi-language UI, batch registration

mod batch;
mod browser_detector;
mod captcha;
mod config;
mod custom_captcha_solver;
mod data_dir;
mod email;
mod email_provider;
mod gui;
mod i18n;
mod import_export;
mod logging;
mod registration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info, warn};
use tracing_subscriber;

use batch::BatchRegistrationManager;
use config::Config;
use email::EmailService;
use email_provider::{BatchEmailManager, EmailProvider, EmailProviderConfig};
use i18n::I18n;
use registration::XRegistration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 配置文件路径 / Config file path
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    /// 语言 / Language (zh-CN, en-US)
    #[arg(short, long)]
    language: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 启动 GUI 界面 / Launch GUI interface
    Gui,

    /// 注册单个账号 / Register single account
    Register {
        /// 注册邮箱地址 / Email address for registration
        #[arg(short, long)]
        email: String,

        /// 代理地址 / Proxy URL (e.g., socks5://127.0.0.1:1080)
        #[arg(short, long)]
        proxy: Option<String>,
    },

    /// 批量注册账号 / Batch register accounts
    Batch {
        /// 要注册的账号数量 / Number of accounts to register
        #[arg(short, long, default_value = "10")]
        count: usize,

        /// 并发数 / Concurrent tasks
        #[arg(short = 'j', long, default_value = "3")]
        concurrent: usize,

        /// 是否使用现有邮箱 / Use existing emails
        #[arg(long)]
        use_existing_emails: bool,
    },

    /// 批量创建邮箱 / Batch create emails
    CreateEmails {
        /// 要创建的邮箱数量 / Number of emails to create
        #[arg(short, long, default_value = "10")]
        count: usize,

        /// 导出文件路径 / Export file path
        #[arg(short, long)]
        output: Option<String>,

        /// 是否验证邮箱 / Verify emails
        #[arg(long)]
        verify: bool,
    },

    /// 导出账号 / Export accounts
    Export {
        /// 导出文件路径 / Export file path
        #[arg(short, long, default_value = "accounts_export.xlsx")]
        output: String,

        /// 格式 / Format (json, csv, xlsx)
        #[arg(short, long, default_value = "xlsx")]
        format: String,
    },

    /// 导入账号 / Import accounts
    Import {
        /// 导入文件路径 / Import file path
        #[arg(short, long)]
        input: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化数据目录
    // Initialize data directories
    if let Err(e) = data_dir::init_directories() {
        eprintln!("初始化数据目录失败 / Failed to initialize data directories: {}", e);
    }

    // 初始化日志系统（文件 + 控制台）
    // Initialize logging system (file + console)
    if let Err(e) = logging::init_logging() {
        eprintln!("日志系统初始化失败 / Failed to initialize logging: {}", e);
        // 降级到简单的控制台日志
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::INFO.into()),
            )
            .init();
    }

    // 打印数据目录信息
    info!("\n{}", data_dir::get_data_dir_info());

    // 清理超过30天的旧日志和任务
    // Clean up logs and tasks older than 30 days
    let _ = logging::cleanup_old_logs(30);
    let _ = data_dir::cleanup_old_tasks(30);

    let args = Args::parse();

    // 如果没有指定子命令，默认启动 GUI
    if args.command.is_none() {
        info!("启动 GUI 界面...");
        return gui::run_gui().map_err(|e| anyhow::anyhow!("GUI error: {}", e));
    }

    // 加载配置（从数据目录）
    let mut config = data_dir::load_or_create_config()?;

    // 命令行参数覆盖配置
    if let Some(lang) = args.language {
        config.language = lang;
    }

    // 初始化多语言
    let i18n = I18n::new(&config.language);

    info!("=== {} ===", i18n.t("app_name"));

    match args.command.unwrap() {
        Commands::Gui => {
            gui::run_gui().map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;
        }

        Commands::Register { email, proxy } => {
            if let Some(proxy_url) = proxy {
                config.proxy.mode = crate::config::ProxyMode::Manual;
                if let Some((proxy_type, rest)) = proxy_url.split_once("://") {
                    config.proxy.proxy_type = proxy_type.to_string();
                    if let Some((host, port)) = rest.split_once(':') {
                        config.proxy.host = host.to_string();
                        if let Ok(p) = port.parse() {
                            config.proxy.port = p;
                        }
                    }
                }
            }

            run_single_registration(config, email, &i18n).await?;
        }

        Commands::Batch {
            count,
            concurrent,
            use_existing_emails,
        } => {
            run_batch_registration(config, count, concurrent, use_existing_emails, &i18n).await?;
        }

        Commands::CreateEmails {
            count,
            output,
            verify,
        } => {
            run_create_emails(config, count, output, verify, &i18n).await?;
        }

        Commands::Export { output, format } => {
            run_export_accounts(output, format, &i18n).await?;
        }

        Commands::Import { input } => {
            run_import_accounts(input, &i18n).await?;
        }
    }

    Ok(())
}

async fn run_single_registration(config: Config, email: String, i18n: &I18n) -> Result<()> {
    info!("{}", i18n.t("starting"));

    // 启动 SMTP 服务
    let email_service = EmailService::new(config.smtp.clone());
    if config.smtp.enable {
        info!("{}", i18n.t("smtp_starting"));
        email_service.start().await?;
        info!(
            "{}: {}:{}",
            i18n.t("smtp_started"),
            config.smtp.host,
            config.smtp.port
        );
    }

    // 显示代理状态
    match config.proxy.mode {
        config::ProxyMode::None => {
            info!("{}", i18n.t("proxy_disabled"));
        }
        config::ProxyMode::System => {
            if let Some(proxy_url) = config.get_proxy_url(config::ProxyTarget::Browser) {
                info!("{}: {}", i18n.t("proxy_system"), proxy_url);
            } else {
                warn!("{}", "系统代理模式已启用但未检测到代理设置");
            }
        }
        config::ProxyMode::Manual => {
            info!(
                "浏览器 {}: {}://{}:{}",
                i18n.t("proxy_manual"),
                config.proxy.proxy_type,
                config.proxy.host,
                config.proxy.port
            );
            info!(
                "邮箱代理: {}",
                if config.proxy.email_enabled {
                    "已启用 / Enabled"
                } else {
                    "已禁用 / Disabled"
                }
            );
        }
    }

    // 创建输出目录
    std::fs::create_dir_all(&config.output.screenshots_dir)?;
    std::fs::create_dir_all(&config.output.logs_dir)?;

    // 初始化注册器
    let registration = XRegistration::new(config.clone(), email_service.get_handler());

    // 执行注册
    info!("{}: {}", i18n.t("registration_start"), email);

    match registration.register_account(email).await {
        Ok(account) => {
            info!("{}", i18n.t("registration_success"));
            info!("用户名 / Username: {}", account.username);
            info!("邮箱 / Email: {}", account.email);

            // 保存账号信息
            save_account_info(&config, &account)?;
        }
        Err(e) => {
            error!("{}: {}", i18n.t("registration_failed"), e);
            return Err(e);
        }
    }

    Ok(())
}

async fn run_batch_registration(
    config: Config,
    count: usize,
    concurrent: usize,
    use_existing_emails: bool,
    _i18n: &I18n,
) -> Result<()> {
    info!("开始批量注册 {} 个账号，并发数: {}", count, concurrent);

    // 启动 SMTP 服务
    let email_service = EmailService::new(config.smtp.clone());
    if config.smtp.enable {
        email_service.start().await?;
    }

    // 创建邮箱管理器
    let email_config = EmailProviderConfig {
        provider: EmailProvider::MailTm,
        smtp_host: Some(config.smtp.host.clone()),
        smtp_port: Some(config.smtp.port),
        imap_host: None,
        imap_port: None,
        username: None,
        password: None,
        domain: config.smtp.domain.clone(),
        api_key: None,
        api_endpoint: None,
    };

    let mut email_manager = BatchEmailManager::new(email_config);

    // 如果不使用现有邮箱，先批量创建邮箱
    if !use_existing_emails {
        info!("批量创建 {} 个邮箱...", count);
        email_manager.create_batch(count, true).await?;
        info!("邮箱创建完成");
    }

    // 创建批量注册管理器
    let batch_manager =
        BatchRegistrationManager::new(config.clone(), email_service.get_handler(), email_manager);

    // 加载缓存数据
    if let Err(e) = batch_manager.load_from_cache().await {
        warn!("加载缓存数据失败 / Failed to load cache: {}", e);
    }

    // 开始批量注册
    let task_id = batch_manager
        .start_batch_registration(count, concurrent, use_existing_emails)
        .await?;

    info!("批量注册任务已启动: {}", task_id);

    // 等待任务完成
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        if let Some(stats) = batch_manager.get_task_stats(&task_id).await {
            info!(
                "进度: {:.1}% ({}/{}) - 成功: {}, 失败: {}",
                stats.progress,
                stats.completed + stats.failed,
                stats.total,
                stats.completed,
                stats.failed
            );

            if matches!(stats.status, batch::BatchStatus::Completed) {
                info!("批量注册任务完成!");
                break;
            }
        }
    }

    // 导出账号
    info!("导出账号信息...");
    batch_manager
        .export_accounts("batch_accounts.json", batch::ExportFormat::Json)
        .await?;

    Ok(())
}

async fn run_create_emails(
    config: Config,
    count: usize,
    output: Option<String>,
    verify: bool,
    _i18n: &I18n,
) -> Result<()> {
    info!("开始批量创建 {} 个邮箱", count);

    let email_config = EmailProviderConfig {
        provider: EmailProvider::MailTm,
        smtp_host: Some(config.smtp.host.clone()),
        smtp_port: Some(config.smtp.port),
        imap_host: None,
        imap_port: None,
        username: None,
        password: None,
        domain: config.smtp.domain.clone(),
        api_key: None,
        api_endpoint: None,
    };

    let mut email_manager = BatchEmailManager::new(email_config);
    let created = email_manager.create_batch(count, verify).await?;

    info!("成功创建 {} 个邮箱", created);

    // 导出到文件
    if let Some(output_file) = output {
        email_manager.export(&output_file, email_provider::ExportFormat::Json)?;
        info!("已导出到文件: {}", output_file);
    } else {
        // 打印到控制台
        let emails = email_manager.get_all_emails();
        println!("\n创建的邮箱列表:");
        for email in emails {
            println!("  - {}", email.address);
        }
    }

    Ok(())
}

async fn run_export_accounts(output: String, format: String, _i18n: &I18n) -> Result<()> {
    info!("导出账号到文件: {}", output);

    // 加载账号数据
    let accounts_file = data_dir::get_accounts_path();
    
    if !accounts_file.exists() {
        warn!("账号文件不存在: {}", accounts_file.display());
        info!("没有账号数据可导出");
        return Ok(());
    }

    // 读取账号数据
    let content = std::fs::read_to_string(&accounts_file)?;
    let accounts: Vec<registration::AccountInfo> = serde_json::from_str(&content)?;

    if accounts.is_empty() {
        info!("没有账号数据可导出");
        return Ok(());
    }

    // 转换为导出格式
    let export_data: Vec<import_export::AccountData> = accounts
        .iter()
        .map(|acc| import_export::AccountData {
            username: acc.username.clone(),
            email: acc.email.clone(),
            password: acc.password.clone(),
            phone: acc.phone.clone(),
            created_at: Some(acc.created_at.clone()),
            status: Some("active".to_string()),
            notes: None,
        })
        .collect();

    // 自动检测格式
    let export_format = import_export::ExportFormat::from_extension(&format);

    // 导出
    import_export::export_accounts(&export_data, &output, export_format)?;

    info!("成功导出 {} 个账号到 {}", export_data.len(), output);

    Ok(())
}

async fn run_import_accounts(input: String, _i18n: &I18n) -> Result<()> {
    info!("从文件导入账号: {}", input);

    // 导入账号数据
    let imported = import_export::import_accounts(&input)?;

    if imported.is_empty() {
        warn!("导入的文件中没有账号数据");
        return Ok(());
    }

    info!("成功导入 {} 个账号", imported.len());

    // 显示导入的账号
    println!("\n导入的账号列表:");
    for (idx, account) in imported.iter().enumerate() {
        println!(
            "  {}. {} ({}) - 状态: {}",
            idx + 1,
            account.username,
            account.email,
            account.status.as_deref().unwrap_or("未知")
        );
    }

    // 将导入的账号保存到本地（追加模式）
    let accounts_file = data_dir::get_accounts_path();
    let mut existing_accounts = if accounts_file.exists() {
        let content = std::fs::read_to_string(&accounts_file)?;
        serde_json::from_str::<Vec<registration::AccountInfo>>(&content)
            .unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };

    // 转换并追加
    for account in imported {
        existing_accounts.push(registration::AccountInfo {
            username: account.username,
            email: account.email,
            password: account.password,
            phone: account.phone,
            created_at: account.created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        });
    }

    // 保存
    let content = serde_json::to_string_pretty(&existing_accounts)?;
    std::fs::write(&accounts_file, content)?;

    info!("账号已保存到本地数据库");

    Ok(())
}

fn save_account_info(config: &Config, account: &registration::AccountInfo) -> Result<()> {
    use std::fs;

    // 使用数据目录中的账号文件
    let accounts_file = data_dir::get_accounts_path();

    let mut accounts = if accounts_file.exists() {
        let content = fs::read_to_string(&accounts_file)?;
        serde_json::from_str::<Vec<registration::AccountInfo>>(&content)
            .unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };

    accounts.push(account.clone());

    let content = serde_json::to_string_pretty(&accounts)?;
    fs::write(&accounts_file, content)?;

    info!(
        "账号信息已保存到 / Account info saved to: {}",
        accounts_file.display()
    );

    Ok(())
}
