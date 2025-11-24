//! BitBrowser 集成模块
//! BitBrowser Integration Module
//! 
//! BitBrowser（比特浏览器）是一款专业的指纹浏览器，用于管理多个浏览器配置文件
//! 每个配置文件都有独立的浏览器指纹，适合批量账号注册和管理
//! 
//! BitBrowser is a professional fingerprint browser for managing multiple browser profiles.
//! Each profile has an independent browser fingerprint, suitable for batch account registration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// BitBrowser API 客户端
pub struct BitBrowserClient {
    api_url: String,
    api_port: u16,
    client: reqwest::Client,
}

/// 浏览器配置文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserProfile {
    /// 配置文件 ID
    pub id: String,
    /// 配置文件名称
    pub name: String,
    /// 是否正在使用
    pub in_use: bool,
    /// 创建时间
    pub created_at: Option<String>,
}

/// 创建配置文件请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateProfileRequest {
    /// 配置文件名称
    pub name: String,
    /// 浏览器类型 (chrome, firefox)
    pub browser_type: String,
    /// 操作系统 (windows, macos, linux)
    pub os: String,
    /// 代理配置
    pub proxy: Option<ProxyConfig>,
}

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 代理类型 (http, socks5)
    pub proxy_type: String,
    /// 代理主机
    pub host: String,
    /// 代理端口
    pub port: u16,
    /// 用户名（可选）
    pub username: Option<String>,
    /// 密码（可选）
    pub password: Option<String>,
}

impl BitBrowserClient {
    pub fn new(api_url: String, api_port: u16) -> Self {
        BitBrowserClient {
            api_url,
            api_port,
            client: reqwest::Client::new(),
        }
    }

    /// 获取 API 基础 URL
    fn base_url(&self) -> String {
        format!("{}:{}", self.api_url, self.api_port)
    }

    /// 检查 BitBrowser 是否运行
    pub async fn is_running(&self) -> bool {
        let url = format!("{}/api/v1/browser/status", self.base_url());
        self.client.get(&url).send().await.is_ok()
    }

    /// 获取所有配置文件
    pub async fn list_profiles(&self) -> Result<Vec<BrowserProfile>> {
        let url = format!("{}/api/v1/browser/list", self.base_url());
        
        info!("获取 BitBrowser 配置文件列表 / Fetching BitBrowser profiles");
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("无法连接到 BitBrowser API")?;

        if !response.status().is_success() {
            anyhow::bail!("BitBrowser API 返回错误: {}", response.status());
        }

        let profiles: Vec<BrowserProfile> = response.json().await?;
        info!("找到 {} 个配置文件", profiles.len());

        Ok(profiles)
    }

    /// 创建新的浏览器配置文件
    pub async fn create_profile(&self, request: CreateProfileRequest) -> Result<BrowserProfile> {
        let url = format!("{}/api/v1/browser/create", self.base_url());
        
        info!("创建新的 BitBrowser 配置文件: {}", request.name);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("无法连接到 BitBrowser API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("创建配置文件失败: {}", error_text);
        }

        let profile: BrowserProfile = response.json().await?;
        info!("✅ 配置文件创建成功: {} (ID: {})", profile.name, profile.id);

        Ok(profile)
    }

    /// 打开浏览器配置文件
    pub async fn open_profile(&self, profile_id: &str) -> Result<()> {
        let url = format!("{}/api/v1/browser/open", self.base_url());
        
        info!("打开 BitBrowser 配置文件: {}", profile_id);
        
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "id": profile_id
            }))
            .send()
            .await
            .context("无法连接到 BitBrowser API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("打开配置文件失败: {}", error_text);
        }

        info!("✅ 配置文件已打开");
        Ok(())
    }

    /// 关闭浏览器配置文件
    pub async fn close_profile(&self, profile_id: &str) -> Result<()> {
        let url = format!("{}/api/v1/browser/close", self.base_url());
        
        info!("关闭 BitBrowser 配置文件: {}", profile_id);
        
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "id": profile_id
            }))
            .send()
            .await
            .context("无法连接到 BitBrowser API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("关闭配置文件失败: {}", error_text);
        }

        info!("✅ 配置文件已关闭");
        Ok(())
    }

    /// 删除浏览器配置文件
    pub async fn delete_profile(&self, profile_id: &str) -> Result<()> {
        let url = format!("{}/api/v1/browser/delete", self.base_url());
        
        info!("删除 BitBrowser 配置文件: {}", profile_id);
        
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "id": profile_id
            }))
            .send()
            .await
            .context("无法连接到 BitBrowser API")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("删除配置文件失败: {}", error_text);
        }

        info!("✅ 配置文件已删除");
        Ok(())
    }

    /// 获取可用的配置文件（未被使用的）
    pub async fn get_available_profile(&self) -> Result<Option<BrowserProfile>> {
        let profiles = self.list_profiles().await?;
        
        // 查找第一个未被使用的配置文件
        for profile in profiles {
            if !profile.in_use {
                return Ok(Some(profile));
            }
        }
        
        Ok(None)
    }

    /// 为账号分配或创建配置文件
    pub async fn allocate_profile_for_account(
        &self,
        account_email: &str,
        auto_create: bool,
    ) -> Result<BrowserProfile> {
        // 先尝试获取可用的配置文件
        if let Some(profile) = self.get_available_profile().await? {
            info!("为账号 {} 分配现有配置文件: {}", account_email, profile.id);
            return Ok(profile);
        }

        // 如果没有可用的，且允许自动创建
        if auto_create {
            info!("没有可用配置文件，为账号 {} 创建新配置文件", account_email);
            
            let profile_name = format!("profile_{}", account_email.replace('@', "_"));
            
            let request = CreateProfileRequest {
                name: profile_name,
                browser_type: "chrome".to_string(),
                os: Self::detect_os(),
                proxy: None, // 代理配置可以从主配置中获取
            };

            return self.create_profile(request).await;
        }

        anyhow::bail!("没有可用的 BitBrowser 配置文件，且不允许自动创建")
    }

    /// 检测当前操作系统
    fn detect_os() -> String {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else {
            "linux".to_string()
        }
    }
}

/// BitBrowser 配置文件管理器
pub struct BitBrowserProfileManager {
    client: BitBrowserClient,
    /// 配置文件池
    profiles: Vec<BrowserProfile>,
}

impl BitBrowserProfileManager {
    pub fn new(client: BitBrowserClient) -> Self {
        BitBrowserProfileManager {
            client,
            profiles: Vec::new(),
        }
    }

    /// 初始化配置文件池
    pub async fn initialize(&mut self) -> Result<()> {
        info!("初始化 BitBrowser 配置文件池 / Initializing BitBrowser profile pool");
        
        // 检查 BitBrowser 是否运行
        if !self.client.is_running().await {
            warn!("⚠️  BitBrowser 未运行，请先启动 BitBrowser");
            anyhow::bail!("BitBrowser 未运行 / BitBrowser is not running");
        }

        // 加载现有配置文件
        self.profiles = self.client.list_profiles().await?;
        
        info!("✅ 配置文件池初始化完成，共 {} 个配置文件", self.profiles.len());
        
        Ok(())
    }

    /// 获取下一个可用的配置文件
    pub async fn get_next_profile(&mut self, auto_create: bool) -> Result<BrowserProfile> {
        // 优先使用未使用的配置文件
        for profile in &self.profiles {
            if !profile.in_use {
                return Ok(profile.clone());
            }
        }

        // 如果允许自动创建
        if auto_create {
            let profile_name = format!("auto_profile_{}", chrono::Utc::now().timestamp());
            let request = CreateProfileRequest {
                name: profile_name,
                browser_type: "chrome".to_string(),
                os: BitBrowserClient::detect_os(),
                proxy: None,
            };

            let new_profile = self.client.create_profile(request).await?;
            self.profiles.push(new_profile.clone());
            
            return Ok(new_profile);
        }

        anyhow::bail!("没有可用的 BitBrowser 配置文件")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_os() {
        let os = BitBrowserClient::detect_os();
        assert!(!os.is_empty());
    }
}
