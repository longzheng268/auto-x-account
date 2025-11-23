//! GUI 模块 - 使用 egui 创建现代化的中国风界面
//! GUI module - Modern Chinese-style interface using egui

use crate::config::{Config, ProxyMode};
use eframe::egui;
use egui::{Color32, FontId, RichText, Rounding, Stroke, Vec2};
use std::sync::{Arc, Mutex};

/// Helper function to convert empty string to None
/// 辅助函数：将空字符串转换为 None
fn empty_string_to_none(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// 中国风配色方案
/// Chinese-style color scheme
pub struct ChineseColorScheme {
    /// 主色调 - 中国红
    pub primary: Color32,
    /// 次要色 - 玉石绿
    pub secondary: Color32,
    /// 背景色 - 米白色
    pub background: Color32,
    /// 卡片背景 - 浅灰白
    pub card_bg: Color32,
    /// 文字主色
    pub text_primary: Color32,
    /// 文字次色
    pub text_secondary: Color32,
    /// 成功色 - 翡翠绿
    pub success: Color32,
    /// 警告色 - 琥珀色
    pub warning: Color32,
    /// 错误色 - 朱砂红
    pub error: Color32,
    /// 金色点缀
    pub accent_gold: Color32,
}

impl Default for ChineseColorScheme {
    fn default() -> Self {
        ChineseColorScheme {
            primary: Color32::from_rgb(220, 38, 38),          // 中国红
            secondary: Color32::from_rgb(34, 139, 34),        // 玉石绿
            background: Color32::from_rgb(250, 248, 246),     // 米白色
            card_bg: Color32::from_rgb(255, 255, 255),        // 纯白
            text_primary: Color32::from_rgb(31, 41, 55),      // 深灰
            text_secondary: Color32::from_rgb(107, 114, 128), // 中灰
            success: Color32::from_rgb(16, 185, 129),         // 翡翠绿
            warning: Color32::from_rgb(245, 158, 11),         // 琥珀色
            error: Color32::from_rgb(239, 68, 68),            // 朱砂红
            accent_gold: Color32::from_rgb(251, 191, 36),     // 金色
        }
    }
}

/// 应用状态
#[derive(Default)]
pub struct AppState {
    pub email: String,
    pub status: String,
    pub progress: f32,
    pub logs: Vec<String>,
    pub accounts: Vec<AccountDisplay>,
    pub show_settings: bool,
    pub config: Config,
    /// 邮箱输入模式：true=手动输入，false=自动生成
    /// Email input mode: true=manual input, false=auto generate
    pub email_manual_mode: bool,
}

#[derive(Clone)]
pub struct AccountDisplay {
    pub email: String,
    pub username: String,
    pub status: String,
    pub created_at: String,
}

pub struct AutoXAccountApp {
    state: Arc<Mutex<AppState>>,
    colors: ChineseColorScheme,
}

impl AutoXAccountApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置字体
        Self::configure_fonts(&cc.egui_ctx);

        // 配置视觉样式
        Self::configure_style(&cc.egui_ctx);

        AutoXAccountApp {
            state: Arc::new(Mutex::new(AppState::default())),
            colors: ChineseColorScheme::default(),
        }
    }

    fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // 尝试嵌入 MiSans 字体，如果不存在则使用 fallback 字体
        // Try to embed MiSans font, fallback to bundled font if not available
        #[cfg(font_misans)]
        {
            fonts.font_data.insert(
                "MiSans".to_owned(),
                egui::FontData::from_static(include_bytes!("../fonts/MiSans/ttf/MiSans-Regular.ttf")),
            );
        }
        
        #[cfg(not(font_misans))]
        {
            // 使用内置的 fallback 字体 (DejaVu Sans)
            // Use bundled fallback font (DejaVu Sans)
            fonts.font_data.insert(
                "MiSans".to_owned(),
                egui::FontData::from_static(include_bytes!("../fonts/fallback.ttf")),
            );
        }

        // 配置字体家族优先级
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "MiSans".to_owned());

        ctx.set_fonts(fonts);
    }

    fn configure_style(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // 设置圆角
        style.visuals.window_rounding = Rounding::same(12.0);
        style.visuals.menu_rounding = Rounding::same(8.0);
        style.visuals.widgets.noninteractive.rounding = Rounding::same(8.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(8.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(8.0);
        style.visuals.widgets.active.rounding = Rounding::same(8.0);

        // 设置间距
        style.spacing.item_spacing = Vec2::new(12.0, 8.0);
        style.spacing.window_margin = egui::style::Margin::same(16.0);

        ctx.set_style(style);
    }

    fn render_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Logo 和标题
            ui.heading(
                RichText::new("🐦 X 账号自动注册系统")
                    .size(28.0)
                    .color(self.colors.primary)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 设置按钮
                if ui.button(RichText::new("⚙ 设置").size(16.0)).clicked() {
                    if let Ok(mut state) = self.state.lock() {
                        state.show_settings = !state.show_settings;
                    }
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);
    }

    fn render_main_panel(&self, ui: &mut egui::Ui) {
        let mut state = self.state.lock().unwrap();

        // 注册卡片
        egui::Frame::none()
            .fill(self.colors.card_bg)
            .stroke(Stroke::new(1.0, Color32::from_rgb(229, 231, 235)))
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::style::Margin::same(20.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("📧 注册新账号")
                        .size(20.0)
                        .color(self.colors.text_primary)
                        .strong(),
                );

                ui.add_space(12.0);

                // 邮箱提供商显示
                ui.horizontal(|ui| {
                    ui.label(RichText::new("当前邮箱提供商:").size(14.0).color(self.colors.text_secondary));
                    ui.label(
                        RichText::new(&state.config.email_provider.selected_provider)
                            .size(14.0)
                            .color(self.colors.primary)
                            .strong(),
                    );
                    ui.label(RichText::new("(可在设置中更改)").size(12.0).color(self.colors.text_secondary).italics());
                });

                ui.add_space(8.0);

                // 邮箱输入模式选择
                ui.horizontal(|ui| {
                    ui.label(RichText::new("邮箱模式:").size(16.0));
                    ui.radio_value(&mut state.email_manual_mode, true, "手动输入");
                    ui.radio_value(&mut state.email_manual_mode, false, "自动生成");
                });

                ui.add_space(8.0);

                // 根据模式显示不同的输入界面
                if state.email_manual_mode {
                    // 手动输入模式
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("邮箱地址:").size(16.0));
                        ui.add_space(8.0);

                        let email_edit = egui::TextEdit::singleline(&mut state.email)
                            .desired_width(300.0)
                            .hint_text("请输入邮箱地址")
                            .font(FontId::proportional(16.0));

                        ui.add(email_edit);
                    });
                } else {
                    // 自动生成模式
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("✨ 将使用选定的邮箱提供商自动生成临时邮箱")
                                .size(14.0)
                                .color(self.colors.success),
                        );
                    });
                    
                    if !state.email.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("生成的邮箱:").size(14.0));
                            ui.label(
                                RichText::new(&state.email)
                                    .size(14.0)
                                    .color(self.colors.text_secondary)
                                    .monospace(),
                            );
                        });
                    }
                }

                ui.add_space(16.0);

                // 进度条
                if state.progress > 0.0 {
                    ui.add(
                        egui::ProgressBar::new(state.progress)
                            .text(format!("进度: {:.0}%", state.progress * 100.0))
                            .fill(self.colors.primary)
                            .animate(true),
                    );
                    ui.add_space(8.0);
                }

                // 状态显示
                if !state.status.is_empty() {
                    ui.label(
                        RichText::new(&state.status)
                            .size(14.0)
                            .color(self.colors.text_secondary),
                    );
                    ui.add_space(8.0);
                }

                // 开始按钮
                let button = egui::Button::new(
                    RichText::new("🚀 开始注册")
                        .size(18.0)
                        .color(Color32::WHITE),
                )
                .fill(self.colors.primary)
                .min_size(Vec2::new(150.0, 45.0))
                .rounding(Rounding::same(8.0));

                if ui.add(button).clicked() {
                    // TODO: 触发注册流程
                    // 如果是自动生成模式，先生成邮箱
                    // NOTE: This requires async runtime integration - see IMPLEMENTATION_EMAIL_PROVIDER_GUI.md
                    // 这需要异步运行时集成 - 参见 IMPLEMENTATION_EMAIL_PROVIDER_GUI.md
                    if !state.email_manual_mode {
                        state.status = "正在生成临时邮箱...".to_string();
                        // 这里应该调用后端API生成邮箱
                        // This should call backend API to generate email
                        // Implementation pending: async runtime integration needed
                    }
                }
            });

        ui.add_space(16.0);

        // 日志面板
        egui::Frame::none()
            .fill(self.colors.card_bg)
            .stroke(Stroke::new(1.0, Color32::from_rgb(229, 231, 235)))
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::style::Margin::same(20.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("📝 运行日志")
                        .size(18.0)
                        .color(self.colors.text_primary)
                        .strong(),
                );

                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for log in &state.logs {
                            ui.label(
                                RichText::new(log)
                                    .size(13.0)
                                    .color(self.colors.text_secondary)
                                    .monospace(),
                            );
                        }
                    });
            });
    }

    fn render_accounts_panel(&self, ui: &mut egui::Ui) {
        let state = self.state.lock().unwrap();

        egui::Frame::none()
            .fill(self.colors.card_bg)
            .stroke(Stroke::new(1.0, Color32::from_rgb(229, 231, 235)))
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::style::Margin::same(20.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("👥 已注册账号")
                        .size(20.0)
                        .color(self.colors.text_primary)
                        .strong(),
                );

                ui.add_space(12.0);

                if state.accounts.is_empty() {
                    ui.label(
                        RichText::new("暂无已注册账号")
                            .size(14.0)
                            .color(self.colors.text_secondary)
                            .italics(),
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            for account in &state.accounts {
                                self.render_account_card(ui, account);
                                ui.add_space(8.0);
                            }
                        });
                }
            });
    }

    fn render_account_card(&self, ui: &mut egui::Ui, account: &AccountDisplay) {
        egui::Frame::none()
            .fill(Color32::from_rgb(249, 250, 251))
            .stroke(Stroke::new(1.0, Color32::from_rgb(229, 231, 235)))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::style::Margin::same(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&account.username)
                            .size(16.0)
                            .color(self.colors.text_primary)
                            .strong(),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let status_color = if account.status == "registered" {
                            self.colors.success
                        } else {
                            self.colors.warning
                        };

                        ui.label(
                            RichText::new(&account.status)
                                .size(12.0)
                                .color(status_color),
                        );
                    });
                });

                ui.label(
                    RichText::new(&account.email)
                        .size(13.0)
                        .color(self.colors.text_secondary),
                );

                ui.label(
                    RichText::new(&account.created_at)
                        .size(12.0)
                        .color(self.colors.text_secondary),
                );
            });
    }

    fn render_settings_window(&self, ctx: &egui::Context) {
        let mut state = self.state.lock().unwrap();

        if !state.show_settings {
            return;
        }

        egui::Window::new("⚙ 设置")
            .fixed_size(Vec2::new(700.0, 700.0))
            .collapsible(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // 运行模式设置
                    ui.label(
                        RichText::new("🏭 运行模式")
                            .size(18.0)
                            .color(self.colors.text_primary)
                            .strong(),
                    );
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("模式:");
                        ui.radio_value(
                            &mut state.config.mode,
                            crate::config::RunMode::Test,
                            "测试模式 (可使用临时邮箱)",
                        );
                        ui.radio_value(
                            &mut state.config.mode,
                            crate::config::RunMode::Production,
                            "生产模式 (仅自建邮箱)",
                        );
                    });
                    
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // SMTP 设置
                    ui.label(
                        RichText::new("📧 SMTP 设置")
                            .size(18.0)
                            .color(self.colors.text_primary)
                            .strong(),
                    );
                    ui.add_space(8.0);

                    ui.checkbox(&mut state.config.smtp.enable, "启用 SMTP 服务");
                    ui.horizontal(|ui| {
                        ui.label("主机:");
                        ui.text_edit_singleline(&mut state.config.smtp.host);
                    });
                    ui.horizontal(|ui| {
                        ui.label("端口:");
                        ui.add(egui::DragValue::new(&mut state.config.smtp.port));
                    });
                    ui.horizontal(|ui| {
                        ui.label("域名:");
                        ui.text_edit_singleline(&mut state.config.smtp.domain);
                    });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // 邮箱提供商设置
                    ui.label(
                        RichText::new("📮 邮箱提供商设置")
                            .size(18.0)
                            .color(self.colors.text_primary)
                            .strong(),
                    );
                    ui.add_space(8.0);

                    // 提供商选择
                    ui.horizontal(|ui| {
                        ui.label("邮箱提供商:");
                        egui::ComboBox::from_label("")
                            .selected_text(&state.config.email_provider.selected_provider)
                            .show_ui(ui, |ui| {
                                // 根据运行模式显示不同的选项
                                if state.config.is_production() {
                                    ui.selectable_value(
                                        &mut state.config.email_provider.selected_provider,
                                        "SelfHosted".to_string(),
                                        "自建邮箱 (SelfHosted)",
                                    );
                                    ui.selectable_value(
                                        &mut state.config.email_provider.selected_provider,
                                        "Custom".to_string(),
                                        "自定义 (Custom)",
                                    );
                                } else {
                                    // 测试模式显示所有选项
                                    ui.selectable_value(
                                        &mut state.config.email_provider.selected_provider,
                                        "MailTm".to_string(),
                                        "mail.tm (临时邮箱)",
                                    );
                                    ui.selectable_value(
                                        &mut state.config.email_provider.selected_provider,
                                        "GuerrillaMail".to_string(),
                                        "Guerrilla Mail",
                                    );
                                    ui.selectable_value(
                                        &mut state.config.email_provider.selected_provider,
                                        "TempMail".to_string(),
                                        "Temp-Mail.org",
                                    );
                                    ui.selectable_value(
                                        &mut state.config.email_provider.selected_provider,
                                        "TenMinuteMail".to_string(),
                                        "10minutemail",
                                    );
                                    ui.selectable_value(
                                        &mut state.config.email_provider.selected_provider,
                                        "DropMail".to_string(),
                                        "DropMail",
                                    );
                                    ui.selectable_value(
                                        &mut state.config.email_provider.selected_provider,
                                        "Mailinator".to_string(),
                                        "Mailinator",
                                    );
                                    ui.selectable_value(
                                        &mut state.config.email_provider.selected_provider,
                                        "SelfHosted".to_string(),
                                        "自建邮箱 (SelfHosted)",
                                    );
                                    ui.selectable_value(
                                        &mut state.config.email_provider.selected_provider,
                                        "Custom".to_string(),
                                        "自定义 (Custom)",
                                    );
                                }
                            });
                    });

                    ui.add_space(8.0);

                    // 根据选择的提供商显示相应的配置选项
                    match state.config.email_provider.selected_provider.as_str() {
                        "Custom" => {
                            ui.label(
                                RichText::new("自定义邮箱配置")
                                    .size(16.0)
                                    .color(self.colors.text_primary)
                                    .strong(),
                            );
                            ui.add_space(8.0);

                            ui.label(RichText::new("SMTP 配置:").size(14.0));
                            ui.horizontal(|ui| {
                                ui.label("主机:");
                                let mut host = state.config.email_provider.custom_smtp_host.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut host).changed() {
                                    state.config.email_provider.custom_smtp_host = empty_string_to_none(host);
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("端口:");
                                let mut port = state.config.email_provider.custom_smtp_port.unwrap_or(587);
                                if ui.add(egui::DragValue::new(&mut port).clamp_range(1..=65535)).changed() {
                                    state.config.email_provider.custom_smtp_port = Some(port);
                                }
                            });

                            ui.add_space(8.0);
                            ui.label(RichText::new("IMAP 配置:").size(14.0));
                            ui.horizontal(|ui| {
                                ui.label("主机:");
                                let mut host = state.config.email_provider.custom_imap_host.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut host).changed() {
                                    state.config.email_provider.custom_imap_host = empty_string_to_none(host);
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("端口:");
                                let mut port = state.config.email_provider.custom_imap_port.unwrap_or(993);
                                if ui.add(egui::DragValue::new(&mut port).clamp_range(1..=65535)).changed() {
                                    state.config.email_provider.custom_imap_port = Some(port);
                                }
                            });

                            ui.add_space(8.0);
                            ui.label(RichText::new("认证信息:").size(14.0));
                            ui.horizontal(|ui| {
                                ui.label("用户名:");
                                let mut username = state.config.email_provider.custom_username.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut username).changed() {
                                    state.config.email_provider.custom_username = empty_string_to_none(username);
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("密码:");
                                let mut password = state.config.email_provider.custom_password.clone().unwrap_or_default();
                                if ui.add(egui::TextEdit::singleline(&mut password).password(true)).changed() {
                                    state.config.email_provider.custom_password = empty_string_to_none(password);
                                }
                            });

                            ui.add_space(8.0);
                            ui.label(RichText::new("API 配置（可选）:").size(14.0));
                            ui.horizontal(|ui| {
                                ui.label("API Key:");
                                let mut api_key = state.config.email_provider.custom_api_key.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut api_key).changed() {
                                    state.config.email_provider.custom_api_key = if api_key.is_empty() { None } else { Some(api_key) };
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("API Endpoint:");
                                let mut endpoint = state.config.email_provider.custom_api_endpoint.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut endpoint).changed() {
                                    state.config.email_provider.custom_api_endpoint = empty_string_to_none(endpoint);
                                }
                            });
                        }
                        "Mailinator" => {
                            ui.label(
                                RichText::new("ℹ Mailinator 需要 API Key 才能查询邮件")
                                    .size(13.0)
                                    .color(self.colors.warning),
                            );
                            ui.horizontal(|ui| {
                                ui.label("API Key:");
                                let mut api_key = state.config.email_provider.custom_api_key.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut api_key).changed() {
                                    state.config.email_provider.custom_api_key = if api_key.is_empty() { None } else { Some(api_key) };
                                }
                            });
                        }
                        "SelfHosted" => {
                            ui.label(
                                RichText::new("ℹ 使用配置的 SMTP 域名生成随机邮箱")
                                    .size(13.0)
                                    .color(self.colors.text_secondary),
                            );
                            ui.label(
                                RichText::new(format!("当前域名: {}", state.config.smtp.domain))
                                    .size(13.0)
                                    .color(self.colors.success),
                            );
                        }
                        "MailTm" => {
                            ui.label(
                                RichText::new("✅ mail.tm 是免费的临时邮箱服务，无需额外配置")
                                    .size(13.0)
                                    .color(self.colors.success),
                            );
                        }
                        "GuerrillaMail" => {
                            ui.label(
                                RichText::new("✅ Guerrilla Mail 是老牌临时邮箱服务，无需额外配置")
                                    .size(13.0)
                                    .color(self.colors.success),
                            );
                        }
                        _ => {
                            ui.label(
                                RichText::new("✅ 此提供商无需额外配置")
                                    .size(13.0)
                                    .color(self.colors.success),
                            );
                        }
                    }

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // 人机验证设置
                    ui.label(
                        RichText::new("🤖 人机验证设置")
                            .size(18.0)
                            .color(self.colors.text_primary)
                            .strong(),
                    );
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("验证模式:");
                        ui.radio_value(
                            &mut state.config.captcha.mode,
                            crate::config::CaptchaMode::Auto,
                            "自动 (免费)",
                        );
                        ui.radio_value(
                            &mut state.config.captcha.mode,
                            crate::config::CaptchaMode::Manual,
                            "手动",
                        );
                        ui.radio_value(
                            &mut state.config.captcha.mode,
                            crate::config::CaptchaMode::ThirdParty,
                            "第三方付费",
                        );
                        ui.radio_value(
                            &mut state.config.captcha.mode,
                            crate::config::CaptchaMode::Llm,
                            "LLM (测试)",
                        );
                    });

                    ui.add_space(8.0);
                    ui.checkbox(&mut state.config.captcha.manual_fallback, "启用手动模式作为后备");

                    // 根据模式显示不同的配置
                    match state.config.captcha.mode {
                        crate::config::CaptchaMode::Auto => {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("✅ 使用自研算法自动解决 ReCAPTCHA (免费)")
                                    .size(13.0)
                                    .color(self.colors.success),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("• 音频挑战 + 免费语音识别")
                                    .size(12.0)
                                    .color(self.colors.text_secondary),
                            );
                            ui.label(
                                RichText::new("• 智能行为模拟")
                                    .size(12.0)
                                    .color(self.colors.text_secondary),
                            );
                            ui.label(
                                RichText::new("• 自动重试机制")
                                    .size(12.0)
                                    .color(self.colors.text_secondary),
                            );
                        }
                        crate::config::CaptchaMode::ThirdParty => {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("⚠️ 第三方服务需要付费 ($1-3/1000次)")
                                    .size(13.0)
                                    .color(self.colors.warning),
                            );
                            ui.add_space(8.0);

                            ui.label(RichText::new("API Keys (至少配置一个):").size(14.0));
                            
                            ui.horizontal(|ui| {
                                ui.label("2Captcha:");
                                let mut key = state.config.captcha.two_captcha_api_key.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut key).changed() {
                                    state.config.captcha.two_captcha_api_key = if key.is_empty() { None } else { Some(key) };
                                }
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Anti-Captcha:");
                                let mut key = state.config.captcha.anti_captcha_api_key.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut key).changed() {
                                    state.config.captcha.anti_captcha_api_key = if key.is_empty() { None } else { Some(key) };
                                }
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("CapMonster:");
                                let mut key = state.config.captcha.capmonster_api_key.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut key).changed() {
                                    state.config.captcha.capmonster_api_key = if key.is_empty() { None } else { Some(key) };
                                }
                            });
                        }
                        crate::config::CaptchaMode::Manual => {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("ℹ 程序会暂停并等待用户在浏览器中手动完成验证")
                                    .size(13.0)
                                    .color(self.colors.text_secondary),
                            );
                        }
                        crate::config::CaptchaMode::Llm => {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("⚠️ LLM API 是测试功能，不保证稳定性")
                                    .size(13.0)
                                    .color(self.colors.warning),
                            );
                            ui.add_space(8.0);

                            // TODO: 添加 LLM API 配置界面
                            ui.label(RichText::new("LLM API 配置待完善").size(13.0).color(self.colors.text_secondary));
                        }
                    }

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // 代理设置
                    ui.label(
                        RichText::new("🌐 代理设置")
                            .size(18.0)
                            .color(self.colors.text_primary)
                            .strong(),
                    );
                    ui.add_space(8.0);

                    // 代理模式选择
                    ui.horizontal(|ui| {
                        ui.label("代理模式:");

                        ui.radio_value(
                            &mut state.config.proxy.mode,
                            ProxyMode::None,
                            "不使用代理",
                        );
                        ui.radio_value(
                            &mut state.config.proxy.mode,
                            ProxyMode::System,
                            "系统代理",
                        );
                        ui.radio_value(
                            &mut state.config.proxy.mode,
                            ProxyMode::Manual,
                            "手动配置",
                        );
                    });

                    ui.add_space(8.0);

                    // 根据模式显示不同的设置
                    match state.config.proxy.mode {
                        ProxyMode::None => {
                            ui.label(
                                RichText::new("ℹ 不使用任何代理，直接连接")
                                    .size(14.0)
                                    .color(self.colors.text_secondary),
                            );
                        }
                        ProxyMode::System => {
                            ui.label(
                                RichText::new("ℹ 自动检测并使用系统配置的代理")
                                    .size(14.0)
                                    .color(self.colors.text_secondary),
                            );

                            // 显示检测到的系统代理
                            if let Some(proxy_url) = state.config.get_proxy_url(crate::config::ProxyTarget::Browser) {
                                ui.label(
                                    RichText::new(format!("检测到: {}", proxy_url))
                                        .size(13.0)
                                        .color(self.colors.success),
                                );
                            } else {
                                ui.label(
                                    RichText::new("⚠ 未检测到系统代理设置")
                                        .size(13.0)
                                        .color(self.colors.warning),
                                );
                            }
                        }
                        ProxyMode::Manual => {
                            ui.label(
                                RichText::new("ℹ 手动配置代理服务器")
                                    .size(14.0)
                                    .color(self.colors.text_secondary),
                            );
                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                ui.label("类型:");
                                ui.radio_value(
                                    &mut state.config.proxy.proxy_type,
                                    "http".to_string(),
                                    "HTTP",
                                );
                                ui.radio_value(
                                    &mut state.config.proxy.proxy_type,
                                    "https".to_string(),
                                    "HTTPS",
                                );
                                ui.radio_value(
                                    &mut state.config.proxy.proxy_type,
                                    "socks5".to_string(),
                                    "SOCKS5",
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("主机:");
                                ui.text_edit_singleline(&mut state.config.proxy.host);
                            });
                            ui.horizontal(|ui| {
                                ui.label("端口:");
                                ui.add(egui::DragValue::new(&mut state.config.proxy.port));
                            });

                            ui.add_space(8.0);
                            ui.label(RichText::new("认证信息（可选）").size(14.0));

                            ui.horizontal(|ui| {
                                ui.label("用户名:");
                                let mut username =
                                    state.config.proxy.username.clone().unwrap_or_default();
                                if ui.text_edit_singleline(&mut username).changed() {
                                    state.config.proxy.username = if username.is_empty() {
                                        None
                                    } else {
                                        Some(username)
                                    };
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("密码:");
                                let mut password =
                                    state.config.proxy.password.clone().unwrap_or_default();
                                if ui
                                    .add(egui::TextEdit::singleline(&mut password).password(true))
                                    .changed()
                                {
                                    state.config.proxy.password = if password.is_empty() {
                                        None
                                    } else {
                                        Some(password)
                                    };
                                }
                            });
                            
                            ui.add_space(8.0);
                            ui.checkbox(&mut state.config.proxy.browser_enabled, "浏览器使用代理");
                            ui.checkbox(&mut state.config.proxy.email_enabled, "邮箱使用代理");
                        }
                    }

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // 浏览器设置
                    ui.label(
                        RichText::new("🌐 浏览器设置")
                            .size(18.0)
                            .color(self.colors.text_primary)
                            .strong(),
                    );
                    ui.add_space(8.0);

                    ui.checkbox(&mut state.config.browser.headless, "无头模式 (后台运行)");
                    ui.horizontal(|ui| {
                        ui.label("超时时间 (ms):");
                        ui.add(egui::DragValue::new(&mut state.config.browser.timeout).speed(100));
                    });
                    ui.horizontal(|ui| {
                        ui.label("窗口宽度:");
                        ui.add(egui::DragValue::new(&mut state.config.browser.viewport.width).speed(10));
                    });
                    ui.horizontal(|ui| {
                        ui.label("窗口高度:");
                        ui.add(egui::DragValue::new(&mut state.config.browser.viewport.height).speed(10));
                    });
                    ui.horizontal(|ui| {
                        ui.label("用户数据目录:");
                        ui.text_edit_singleline(&mut state.config.browser.user_data_dir);
                    });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // 输出设置
                    ui.label(
                        RichText::new("📁 输出设置")
                            .size(18.0)
                            .color(self.colors.text_primary)
                            .strong(),
                    );
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("截图目录:");
                        ui.text_edit_singleline(&mut state.config.output.screenshots_dir);
                    });
                    ui.horizontal(|ui| {
                        ui.label("日志目录:");
                        ui.text_edit_singleline(&mut state.config.output.logs_dir);
                    });
                    ui.horizontal(|ui| {
                        ui.label("账号文件:");
                        ui.text_edit_singleline(&mut state.config.output.accounts_file);
                    });

                    ui.add_space(16.0);

                    // 保存按钮
                    if ui.button(RichText::new("💾 保存设置").size(16.0)).clicked() {
                        // 保存配置到文件
                        if let Err(e) = state.config.to_file("config.json") {
                            eprintln!("保存配置失败: {}", e);
                        }
                        state.show_settings = false;
                    }
                });
            });
    }
}

impl eframe::App for AutoXAccountApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 设置背景色
        let mut style = (*ctx.style()).clone();
        style.visuals.panel_fill = self.colors.background;
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(16.0);

            self.render_header(ui);

            ui.add_space(16.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                // 左右布局
                ui.horizontal_top(|ui| {
                    // 左侧主面板
                    ui.vertical(|ui| {
                        ui.set_min_width(600.0);
                        self.render_main_panel(ui);
                    });

                    ui.add_space(16.0);

                    // 右侧账号列表
                    ui.vertical(|ui| {
                        ui.set_min_width(350.0);
                        self.render_accounts_panel(ui);
                    });
                });
            });
        });

        // 渲染设置窗口
        self.render_settings_window(ctx);
    }
}

pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "X 账号自动注册系统",
        options,
        Box::new(|cc| Box::new(AutoXAccountApp::new(cc))),
    )
}
