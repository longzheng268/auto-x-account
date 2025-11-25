//! 导入导出模块
//! Import/Export module
//! 
//! 支持多种格式的账号和邮箱数据导入导出：
//! - JSON
//! - CSV
//! - XLS/XLSX (Excel)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use tracing::{info, warn};

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Excel,
}

impl ExportFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(ExportFormat::Json),
            "csv" => Some(ExportFormat::Csv),
            "xls" | "xlsx" => Some(ExportFormat::Excel),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::Excel => "xlsx",
        }
    }
}

/// 账号数据（用于导入导出）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub birth_date: Option<String>,
    pub phone: Option<String>,
    pub created_at: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

/// 导出账号到文件
pub fn export_accounts<P: AsRef<Path>>(
    accounts: &[AccountData],
    path: P,
    format: Option<ExportFormat>,
) -> Result<()> {
    let path = path.as_ref();
    
    // 自动检测格式
    let format = format.or_else(|| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(ExportFormat::from_extension)
    }).unwrap_or(ExportFormat::Json);

    info!("导出 {} 个账号到 {} (格式: {:?})", accounts.len(), path.display(), format);

    match format {
        ExportFormat::Json => export_accounts_json(accounts, path),
        ExportFormat::Csv => export_accounts_csv(accounts, path),
        ExportFormat::Excel => export_accounts_excel(accounts, path),
    }
}

/// 从文件导入账号
pub fn import_accounts<P: AsRef<Path>>(path: P) -> Result<Vec<AccountData>> {
    let path = path.as_ref();
    
    if !path.exists() {
        anyhow::bail!("文件不存在 / File not found: {}", path.display());
    }

    // 自动检测格式
    let format = path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(ExportFormat::from_extension)
        .unwrap_or(ExportFormat::Json);

    info!("从 {} 导入账号 (格式: {:?})", path.display(), format);

    match format {
        ExportFormat::Json => import_accounts_json(path),
        ExportFormat::Csv => import_accounts_csv(path),
        ExportFormat::Excel => import_accounts_excel(path),
    }
}

// ==================== JSON 格式 ====================

fn export_accounts_json<P: AsRef<Path>>(accounts: &[AccountData], path: P) -> Result<()> {
    let file = File::create(path.as_ref())?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, accounts)?;
    Ok(())
}

fn import_accounts_json<P: AsRef<Path>>(path: P) -> Result<Vec<AccountData>> {
    let file = File::open(path.as_ref())?;
    let accounts: Vec<AccountData> = serde_json::from_reader(file)?;
    Ok(accounts)
}

// ==================== CSV 格式 ====================

fn export_accounts_csv<P: AsRef<Path>>(accounts: &[AccountData], path: P) -> Result<()> {
    let file = File::create(path.as_ref())?;
    let mut writer = csv::Writer::from_writer(file);

    // 写入表头
    writer.write_record(&[
        "username",
        "email",
        "password",
        "birth_date",
        "phone",
        "created_at",
        "status",
        "notes",
    ])?;

    // 写入数据
    for account in accounts {
        writer.write_record(&[
            &account.username,
            &account.email,
            account.password.as_deref().unwrap_or(""),
            account.birth_date.as_deref().unwrap_or(""),
            account.phone.as_deref().unwrap_or(""),
            account.created_at.as_deref().unwrap_or(""),
            account.status.as_deref().unwrap_or(""),
            account.notes.as_deref().unwrap_or(""),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

fn import_accounts_csv<P: AsRef<Path>>(path: P) -> Result<Vec<AccountData>> {
    let file = File::open(path.as_ref())?;
    let mut reader = csv::Reader::from_reader(file);
    let headers = reader.headers()?.clone();

    let find_index = |names: &[&str]| -> Option<usize> {
        headers.iter().position(|h| {
            let h_norm = h.trim().to_lowercase();
            names
                .iter()
                .any(|n| h_norm == n.trim().to_lowercase())
        })
    };

    let username_idx = find_index(&["username", "user", "账号", "帳號", "帐号"]);
    let email_idx = find_index(&["email", "邮箱", "mail"]);
    let password_idx = find_index(&["password", "pass", "密码"]);
    let birth_idx = find_index(&["birthdate", "birth_date", "birthday", "出生日期"]);
    let phone_idx = find_index(&["phone", "mobile", "手机号", "電話"]);
    let created_idx = find_index(&["created_at", "created", "创建时间"]);
    let status_idx = find_index(&["status", "状态"]);
    let notes_idx = find_index(&["notes", "备注"]);

    let mut accounts = Vec::new();
    for result in reader.records() {
        let record = result?;

        let get_value = |idx: Option<usize>| -> Option<String> {
            idx.and_then(|i| record.get(i))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        };

        let email = match get_value(email_idx) {
            Some(e) => e,
            None => {
                warn!("跳过无邮箱的 CSV 行: {:?}", record);
                continue;
            }
        };

        accounts.push(AccountData {
            username: get_value(username_idx).unwrap_or_default(),
            email,
            password: get_value(password_idx),
            birth_date: get_value(birth_idx),
            phone: get_value(phone_idx),
            created_at: get_value(created_idx),
            status: get_value(status_idx),
            notes: get_value(notes_idx),
        });
    }

    Ok(accounts)
}

// ==================== Excel 格式 ====================

fn export_accounts_excel<P: AsRef<Path>>(accounts: &[AccountData], path: P) -> Result<()> {
    use simple_excel_writer::*;

    let mut workbook = Workbook::create(path.as_ref().to_str().unwrap());
    let mut sheet = workbook.create_sheet("Accounts");

    // 设置列宽
    sheet.add_column(Column { width: 20.0 });
    sheet.add_column(Column { width: 30.0 });
    sheet.add_column(Column { width: 20.0 });
    sheet.add_column(Column { width: 20.0 });
    sheet.add_column(Column { width: 20.0 });
    sheet.add_column(Column { width: 15.0 });
    sheet.add_column(Column { width: 30.0 });

    // 写入表头
    workbook.write_sheet(&mut sheet, |sheet_writer| {
        let sw = sheet_writer;
        sw.append_row(row![
            "Username",
            "Email",
            "Password",
            "BirthDate",
            "Phone",
            "Created At",
            "Status",
            "Notes"
        ])?;

        // 写入数据
        for account in accounts {
            sw.append_row(row![
                account.username.clone(),
                account.email.clone(),
                account.password.as_deref().unwrap_or(""),
                account.birth_date.as_deref().unwrap_or(""),
                account.phone.as_deref().unwrap_or(""),
                account.created_at.as_deref().unwrap_or(""),
                account.status.as_deref().unwrap_or(""),
                account.notes.as_deref().unwrap_or("")
            ])?;
        }

        Ok(())
    })?;

    workbook.close()?;
    Ok(())
}

fn import_accounts_excel<P: AsRef<Path>>(path: P) -> Result<Vec<AccountData>> {
    use calamine::{open_workbook, Reader, Xlsx};

    let mut workbook: Xlsx<_> = open_workbook(path.as_ref()).context("无法打开Excel文件")?;

    let sheet_name = workbook
        .sheet_names()
        .get(0)
        .context("Excel文件中没有工作表")?
        .clone();

    let range = workbook
        .worksheet_range(&sheet_name)
        .context("无法读取工作表")?;

    if range.is_empty() {
        warn!("工作表为空，返回空账号列表");
        return Ok(Vec::new());
    }

    let mut rows = range.rows();
    let header_row = rows.next().context("Excel 缺少表头")?;
    let mut header_map = HashMap::new();
    for (idx, cell) in header_row.iter().enumerate() {
        let key = cell.to_string().trim().to_lowercase();
        header_map.insert(key, idx);
    }

    let find_idx = |names: &[&str]| -> Option<usize> {
        for name in names {
            let key = name.trim().to_lowercase();
            if let Some(idx) = header_map.get(&key) {
                return Some(*idx);
            }
        }
        None
    };

    let username_idx = find_idx(&["username", "user", "账号", "帳號", "帐号"]);
    let email_idx = find_idx(&["email", "邮箱", "mail"]);
    let password_idx = find_idx(&["password", "pass", "密码"]);
    let birth_idx = find_idx(&["birthdate", "birth_date", "birthday", "出生日期"]);
    let phone_idx = find_idx(&["phone", "mobile", "手机号", "電話"]);
    let created_idx = find_idx(&["created_at", "created", "创建时间"]);
    let status_idx = find_idx(&["status", "状态"]);
    let notes_idx = find_idx(&["notes", "备注"]);

    let mut accounts = Vec::new();
    for row in rows {
        let get_value = |idx: Option<usize>| -> Option<String> {
            idx.and_then(|i| row.get(i))
                .map(|cell| cell.to_string().trim().to_string())
                .filter(|s| !s.is_empty())
        };

        let email = match get_value(email_idx) {
            Some(e) => e,
            None => {
                warn!("跳过无邮箱的 Excel 行: {:?}", row);
                continue;
            }
        };

        accounts.push(AccountData {
            username: get_value(username_idx).unwrap_or_default(),
            email,
            password: get_value(password_idx),
            birth_date: get_value(birth_idx),
            phone: get_value(phone_idx),
            created_at: get_value(created_idx),
            status: get_value(status_idx),
            notes: get_value(notes_idx),
        });
    }

    Ok(accounts)
}

// ==================== 邮箱数据 ====================

/// 邮箱数据（用于导入导出）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailData {
    pub address: String,
    pub password: Option<String>,
    pub provider: Option<String>,
    pub created_at: Option<String>,
    pub status: Option<String>,
}

/// 导出邮箱到文件
pub fn export_emails<P: AsRef<Path>>(
    emails: &[EmailData],
    path: P,
    format: Option<ExportFormat>,
) -> Result<()> {
    let path = path.as_ref();
    
    let format = format.or_else(|| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(ExportFormat::from_extension)
    }).unwrap_or(ExportFormat::Json);

    info!("导出 {} 个邮箱到 {} (格式: {:?})", emails.len(), path.display(), format);

    match format {
        ExportFormat::Json => {
            let file = File::create(path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, emails)?;
            Ok(())
        }
        ExportFormat::Csv => {
            let file = File::create(path)?;
            let mut writer = csv::Writer::from_writer(file);
            writer.write_record(&["address", "password", "provider", "created_at", "status"])?;
            for email in emails {
                writer.write_record(&[
                    &email.address,
                    email.password.as_deref().unwrap_or(""),
                    email.provider.as_deref().unwrap_or(""),
                    email.created_at.as_deref().unwrap_or(""),
                    email.status.as_deref().unwrap_or(""),
                ])?;
            }
            writer.flush()?;
            Ok(())
        }
        ExportFormat::Excel => {
            use simple_excel_writer::*;
            let mut workbook = Workbook::create(path.to_str().unwrap());
            let mut sheet = workbook.create_sheet("Emails");
            
            workbook.write_sheet(&mut sheet, |sheet_writer| {
                let sw = sheet_writer;
                sw.append_row(row!["Address", "Password", "Provider", "Created At", "Status"])?;
                for email in emails {
                    sw.append_row(row![
                        email.address.clone(),
                        email.password.as_deref().unwrap_or(""),
                        email.provider.as_deref().unwrap_or(""),
                        email.created_at.as_deref().unwrap_or(""),
                        email.status.as_deref().unwrap_or("")
                    ])?;
                }
                Ok(())
            })?;
            
            workbook.close()?;
            Ok(())
        }
    }
}

/// 从文件导入邮箱
pub fn import_emails<P: AsRef<Path>>(path: P) -> Result<Vec<EmailData>> {
    let path = path.as_ref();
    
    if !path.exists() {
        anyhow::bail!("文件不存在 / File not found: {}", path.display());
    }

    let format = path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(ExportFormat::from_extension)
        .unwrap_or(ExportFormat::Json);

    info!("从 {} 导入邮箱 (格式: {:?})", path.display(), format);

    match format {
        ExportFormat::Json => {
            let file = File::open(path)?;
            let emails: Vec<EmailData> = serde_json::from_reader(file)?;
            Ok(emails)
        }
        ExportFormat::Csv => {
            let file = File::open(path)?;
            let mut reader = csv::Reader::from_reader(file);
            let mut emails = Vec::new();
            
            for result in reader.records() {
                let record = result?;
                if record.len() < 1 {
                    continue;
                }
                
                emails.push(EmailData {
                    address: record.get(0).unwrap_or("").to_string(),
                    password: record.get(1).and_then(|s| {
                        if s.is_empty() { None } else { Some(s.to_string()) }
                    }),
                    provider: record.get(2).and_then(|s| {
                        if s.is_empty() { None } else { Some(s.to_string()) }
                    }),
                    created_at: record.get(3).and_then(|s| {
                        if s.is_empty() { None } else { Some(s.to_string()) }
                    }),
                    status: record.get(4).and_then(|s| {
                        if s.is_empty() { None } else { Some(s.to_string()) }
                    }),
                });
            }
            
            Ok(emails)
        }
        ExportFormat::Excel => {
            use calamine::{Reader, open_workbook, Xlsx};
            
            let mut workbook: Xlsx<_> = open_workbook(path)?;
            let sheet_name = workbook.sheet_names().get(0).context("No worksheet")?.clone();
            let range = workbook.worksheet_range(&sheet_name)?;
            
            // 如果工作表为空，返回空列表
            if range.is_empty() {
                warn!("Worksheet is empty, returning empty list");
                return Ok(Vec::new());
            }
            
            let mut emails = Vec::new();
            let mut rows = range.rows();
            rows.next(); // Skip header
            
            for row in rows {
                if row.is_empty() {
                    continue;
                }
                
                let get_cell_string = |idx: usize| -> Option<String> {
                    row.get(idx).and_then(|cell| {
                        let s = cell.to_string();
                        if s.is_empty() { None } else { Some(s) }
                    })
                };
                
                emails.push(EmailData {
                    address: get_cell_string(0).unwrap_or_default(),
                    password: get_cell_string(1),
                    provider: get_cell_string(2),
                    created_at: get_cell_string(3),
                    status: get_cell_string(4),
                });
            }
            
            Ok(emails)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_from_extension() {
        assert_eq!(ExportFormat::from_extension("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_extension("csv"), Some(ExportFormat::Csv));
        assert_eq!(ExportFormat::from_extension("xlsx"), Some(ExportFormat::Excel));
        assert_eq!(ExportFormat::from_extension("xls"), Some(ExportFormat::Excel));
        assert_eq!(ExportFormat::from_extension("txt"), None);
    }
}
