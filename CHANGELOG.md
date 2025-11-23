# 变更日志 / Changelog

所有重要的变更都会记录在这个文件中。

## [Unreleased]

### 新增 / Added
- ✨ GUI 界面，采用中国风设计，使用小米 MiSans 字体
- ✨ 批量注册 X 账号功能
- ✨ 批量创建临时邮箱功能
- ✨ 三种代理模式：不使用代理、系统代理、手动配置
- ✨ 支持多种邮箱服务提供商：mail.tm、Guerrilla Mail、自建 SMTP
- ✨ 人机验证处理模块（支持手动和第三方服务）
- ✨ 多语言支持（中文、英文）
- ✨ 完整的 CI/CD 工作流
- ✨ 跨平台支持：Windows、macOS（Intel & Apple Silicon）、Linux

### 功能特性 / Features
- 🚀 使用 Rust 开发，性能优秀
- 🎨 现代化中国风 GUI 界面
- 🌐 支持代理访问（适合中国大陆用户）
- 📧 灵活的邮箱服务配置
- 📊 实时进度显示和日志记录
- 💾 支持导出账号信息（JSON、CSV、TXT 格式）
- 🔒 安全的账号信息存储

### 技术栈 / Tech Stack
- 语言：Rust 2021
- GUI 框架：egui
- 浏览器自动化：chromiumoxide
- 异步运行时：tokio
- HTTP 客户端：reqwest

## [0.1.0] - 2024-XX-XX

### 初始版本 / Initial Release
- 🎉 项目启动
- 📝 基础架构搭建
- 🔧 核心功能实现
