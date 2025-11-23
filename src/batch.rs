//! 批量注册模块
//! Batch registration module

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use crate::config::Config;
use crate::email::EmailHandler;
use crate::email_provider::{BatchEmailManager, TempEmail};
use crate::registration::{AccountInfo, XRegistration};

/// 批量注册任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 已暂停
    Paused,
    /// 失败
    Failed,
}

/// 批量注册任务
#[derive(Clone)]
pub struct BatchTask {
    pub id: String,
    pub total_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub status: BatchStatus,
    pub created_at: String,
    pub updated_at: String,
}

/// 批量注册管理器
pub struct BatchRegistrationManager {
    config: Config,
    email_handler: EmailHandler,
    email_manager: Arc<Mutex<BatchEmailManager>>,
    tasks: Arc<Mutex<Vec<BatchTask>>>,
    accounts: Arc<Mutex<Vec<AccountInfo>>>,
}

impl BatchRegistrationManager {
    pub fn new(
        config: Config,
        email_handler: EmailHandler,
        email_manager: BatchEmailManager,
    ) -> Self {
        BatchRegistrationManager {
            config,
            email_handler,
            email_manager: Arc::new(Mutex::new(email_manager)),
            tasks: Arc::new(Mutex::new(Vec::new())),
            accounts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 开始批量注册任务
    /// Start batch registration task
    pub async fn start_batch_registration(
        &self,
        count: usize,
        concurrent: usize,
        use_existing_emails: bool,
    ) -> Result<String> {
        let task_id = format!("task_{}", chrono::Utc::now().timestamp_millis());

        // 创建任务
        let task = BatchTask {
            id: task_id.clone(),
            total_count: count,
            completed_count: 0,
            failed_count: 0,
            status: BatchStatus::Running,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        {
            let mut tasks = self.tasks.lock().await;
            tasks.push(task.clone());
        }

        info!("开始批量注册任务: {} 个账号，并发数: {}", count, concurrent);

        // 启动批量注册
        let manager = self.clone_for_task();
        tokio::spawn(async move {
            if let Err(e) = manager
                .execute_batch_registration(task_id.clone(), count, concurrent, use_existing_emails)
                .await
            {
                error!("批量注册任务 {} 失败: {}", task_id, e);
            }
        });

        Ok(task_id)
    }

    /// 执行批量注册
    async fn execute_batch_registration(
        &self,
        task_id: String,
        count: usize,
        concurrent: usize,
        use_existing_emails: bool,
    ) -> Result<()> {
        let mut handles = Vec::new();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent));

        for i in 0..count {
            let sem = semaphore.clone();
            let manager = self.clone_for_task();
            let task_id = task_id.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                info!("开始注册第 {}/{} 个账号", i + 1, count);

                match manager.register_single_account(use_existing_emails).await {
                    Ok(account) => {
                        info!("成功注册账号 {}/{}: {}", i + 1, count, account.username);
                        manager.update_task_progress(&task_id, true).await;
                        manager.save_account(account).await;
                    }
                    Err(e) => {
                        error!("注册账号 {}/{} 失败: {}", i + 1, count, e);
                        manager.update_task_progress(&task_id, false).await;
                    }
                }

                // 添加延迟避免请求过快
                sleep(Duration::from_secs(5)).await;
            });

            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let _ = handle.await;
        }

        // 更新任务状态为完成
        self.complete_task(&task_id).await;

        info!("批量注册任务 {} 完成", task_id);
        Ok(())
    }

    /// 注册单个账号
    async fn register_single_account(&self, use_existing_email: bool) -> Result<AccountInfo> {
        // 获取或创建邮箱
        let email = if use_existing_email {
            let mut email_mgr = self.email_manager.lock().await;
            email_mgr.get_unused_email().context("没有可用的邮箱")?
        } else {
            // 创建新邮箱
            let mut email_mgr = self.email_manager.lock().await;
            let emails = email_mgr.create_batch(1, false).await?;
            emails.into_iter().next().context("创建邮箱失败")?
        };

        info!("使用邮箱: {}", email.address);

        // 执行注册
        let registration = XRegistration::new(self.config.clone(), self.email_handler.clone());
        let account = registration.register_account(email.address).await?;

        Ok(account)
    }

    /// 更新任务进度
    async fn update_task_progress(&self, task_id: &str, success: bool) {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            if success {
                task.completed_count += 1;
            } else {
                task.failed_count += 1;
            }
            task.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// 完成任务
    async fn complete_task(&self, task_id: &str) {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = BatchStatus::Completed;
            task.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// 保存账号信息
    async fn save_account(&self, account: AccountInfo) {
        let mut accounts = self.accounts.lock().await;
        accounts.push(account);
    }

    /// 获取所有任务
    pub async fn get_tasks(&self) -> Vec<BatchTask> {
        let tasks = self.tasks.lock().await;
        tasks.clone()
    }

    /// 获取所有账号
    pub async fn get_accounts(&self) -> Vec<AccountInfo> {
        let accounts = self.accounts.lock().await;
        accounts.clone()
    }

    /// 获取任务统计
    pub async fn get_task_stats(&self, task_id: &str) -> Option<BatchTaskStats> {
        let tasks = self.tasks.lock().await;
        tasks.iter().find(|t| t.id == task_id).map(|task| {
            let progress = if task.total_count > 0 {
                (task.completed_count as f32 / task.total_count as f32) * 100.0
            } else {
                0.0
            };

            BatchTaskStats {
                task_id: task.id.clone(),
                total: task.total_count,
                completed: task.completed_count,
                failed: task.failed_count,
                pending: task.total_count - task.completed_count - task.failed_count,
                progress,
                status: task.status.clone(),
            }
        })
    }

    /// 暂停任务（待实现）
    pub async fn pause_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = BatchStatus::Paused;
            task.updated_at = chrono::Utc::now().to_rfc3339();
            info!("任务 {} 已暂停", task_id);
            Ok(())
        } else {
            anyhow::bail!("任务不存在: {}", task_id)
        }
    }

    /// 恢复任务（待实现）
    pub async fn resume_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = BatchStatus::Running;
            task.updated_at = chrono::Utc::now().to_rfc3339();
            info!("任务 {} 已恢复", task_id);
            Ok(())
        } else {
            anyhow::bail!("任务不存在: {}", task_id)
        }
    }

    /// 导出账号到文件
    pub async fn export_accounts(&self, filename: &str, format: ExportFormat) -> Result<()> {
        let accounts = self.accounts.lock().await;

        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(filename)?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&*accounts)?;
                file.write_all(json.as_bytes())?;
            }
            ExportFormat::Csv => {
                writeln!(file, "邮箱,用户名,密码,状态,创建时间")?;
                for account in accounts.iter() {
                    writeln!(
                        file,
                        "{},{},{},{},{}",
                        account.email,
                        account.username,
                        account.password,
                        account.status,
                        account.created_at
                    )?;
                }
            }
            ExportFormat::Txt => {
                for account in accounts.iter() {
                    writeln!(
                        file,
                        "邮箱: {} | 用户名: {} | 密码: {} | 状态: {}",
                        account.email, account.username, account.password, account.status
                    )?;
                }
            }
        }

        info!("已导出 {} 个账号到文件: {}", accounts.len(), filename);
        Ok(())
    }

    /// 清空已完成的任务
    pub async fn clear_completed_tasks(&self) {
        let mut tasks = self.tasks.lock().await;
        tasks.retain(|t| !matches!(t.status, BatchStatus::Completed));
        info!("已清空完成的任务");
    }

    /// 克隆用于任务
    fn clone_for_task(&self) -> Self {
        BatchRegistrationManager {
            config: self.config.clone(),
            email_handler: self.email_handler.clone(),
            email_manager: Arc::clone(&self.email_manager),
            tasks: Arc::clone(&self.tasks),
            accounts: Arc::clone(&self.accounts),
        }
    }
}

/// 批量任务统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTaskStats {
    pub task_id: String,
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub pending: usize,
    pub progress: f32,
    pub status: BatchStatus,
}

/// 导出格式
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Txt,
}

/// 批量注册配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRegistrationConfig {
    /// 要注册的账号数量
    pub count: usize,
    /// 并发数
    pub concurrent: usize,
    /// 失败重试次数
    pub retry_times: usize,
    /// 是否使用现有邮箱
    pub use_existing_emails: bool,
    /// 每次注册后的延迟（秒）
    pub delay_seconds: u64,
}

impl Default for BatchRegistrationConfig {
    fn default() -> Self {
        BatchRegistrationConfig {
            count: 10,
            concurrent: 3,
            retry_times: 3,
            use_existing_emails: false,
            delay_seconds: 5,
        }
    }
}
