#!/bin/bash
# macOS 一键搭建编译环境脚本
# macOS One-Click Build Environment Setup Script

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}================================================${NC}"
echo -e "${CYAN}X 账号自动注册系统 - macOS 编译环境安装${NC}"
echo -e "${CYAN}X Account Auto Registration - macOS Build Setup${NC}"
echo -e "${CYAN}================================================${NC}"
echo ""

# 检查是否安装了 Homebrew
if ! command -v brew &> /dev/null; then
    echo -e "${GREEN}正在安装 Homebrew...${NC}"
    echo -e "${GREEN}Installing Homebrew...${NC}"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # 根据架构添加 Homebrew 到 PATH
    if [[ $(uname -m) == 'arm64' ]]; then
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
        eval "$(/opt/homebrew/bin/brew shellenv)"
    else
        echo 'eval "$(/usr/local/bin/brew shellenv)"' >> ~/.zprofile
        eval "$(/usr/local/bin/brew shellenv)"
    fi
    
    echo -e "${GREEN}Homebrew 安装成功!${NC}"
    echo -e "${GREEN}Homebrew installed successfully!${NC}"
else
    echo -e "${GREEN}Homebrew 已安装${NC}"
    echo -e "${GREEN}Homebrew is already installed${NC}"
    
    # 更新 Homebrew
    echo -e "${YELLOW}更新 Homebrew...${NC}"
    echo -e "${YELLOW}Updating Homebrew...${NC}"
    brew update
fi

echo ""

# 安装 Rust
echo -e "${GREEN}正在安装 Rust...${NC}"
echo -e "${GREEN}Installing Rust...${NC}"

if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}Rust 安装成功!${NC}"
    echo -e "${GREEN}Rust installed successfully!${NC}"
else
    echo -e "${GREEN}Rust 已安装${NC}"
    echo -e "${GREEN}Rust is already installed${NC}"
    
    # 更新 Rust
    echo -e "${YELLOW}更新 Rust...${NC}"
    echo -e "${YELLOW}Updating Rust...${NC}"
    rustup update
fi

echo ""

# 安装 Git
echo -e "${GREEN}正在检查 Git...${NC}"
echo -e "${GREEN}Checking Git...${NC}"

if ! command -v git &> /dev/null; then
    brew install git
    echo -e "${GREEN}Git 安装成功!${NC}"
    echo -e "${GREEN}Git installed successfully!${NC}"
else
    echo -e "${GREEN}Git 已安装${NC}"
    echo -e "${GREEN}Git is already installed${NC}"
fi

echo ""

# 安装 Chrome/Chromium (用于浏览器自动化)
echo -e "${GREEN}正在检查 Chrome...${NC}"
echo -e "${GREEN}Checking Chrome...${NC}"

if ! brew list --cask google-chrome &> /dev/null; then
    echo -e "${YELLOW}安装 Google Chrome...${NC}"
    echo -e "${YELLOW}Installing Google Chrome...${NC}"
    brew install --cask google-chrome || echo -e "${YELLOW}Chrome 可能已通过其他方式安装${NC}"
else
    echo -e "${GREEN}Google Chrome 已安装${NC}"
    echo -e "${GREEN}Google Chrome is already installed${NC}"
fi

echo ""

# 安装 pkg-config (某些依赖需要)
echo -e "${GREEN}安装必要的开发工具...${NC}"
echo -e "${GREEN}Installing necessary development tools...${NC}"
brew install pkg-config || true

echo ""

# 显示版本信息
echo -e "${CYAN}================================================${NC}"
echo -e "${GREEN}安装完成! 当前版本信息:${NC}"
echo -e "${GREEN}Installation complete! Current versions:${NC}"
echo -e "${CYAN}================================================${NC}"
echo ""

echo -e "${YELLOW}Rust:${NC}"
rustc --version
cargo --version

echo ""
echo -e "${YELLOW}Git:${NC}"
git --version

echo ""
echo -e "${CYAN}================================================${NC}"
echo -e "${GREEN}下一步 / Next Steps:${NC}"
echo -e "${CYAN}================================================${NC}"
echo ""
echo -e "${NC}1. 克隆项目 / Clone the project:${NC}"
echo -e "   ${YELLOW}git clone https://github.com/longzheng268/auto-x-account.git${NC}"
echo ""
echo -e "${NC}2. 进入项目目录 / Enter project directory:${NC}"
echo -e "   ${YELLOW}cd auto-x-account${NC}"
echo ""
echo -e "${NC}3. 编译项目 / Build the project:${NC}"
echo -e "   ${YELLOW}cargo build --release${NC}"
echo ""
echo -e "${NC}4. 运行程序 / Run the program:${NC}"
echo -e "   ${YELLOW}./target/release/auto-x-account --email your@email.com${NC}"
echo ""
echo -e "${CYAN}================================================${NC}"

# 询问是否立即编译项目
if [ -f "Cargo.toml" ]; then
    echo ""
    read -p "检测到 Cargo.toml，是否立即编译项目？(Y/N) / Cargo.toml detected, compile now? (Y/N): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        echo -e "${GREEN}正在编译项目...${NC}"
        echo -e "${GREEN}Compiling project...${NC}"
        cargo build --release
        
        echo ""
        echo -e "${GREEN}编译成功! 可执行文件位于: ./target/release/auto-x-account${NC}"
        echo -e "${GREEN}Build successful! Executable located at: ./target/release/auto-x-account${NC}"
    fi
fi

echo ""
echo -e "${GREEN}脚本执行完成!${NC}"
echo -e "${GREEN}Script execution completed!${NC}"
