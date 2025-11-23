#!/bin/bash
# GNU/Linux 一键搭建编译环境脚本
# GNU/Linux One-Click Build Environment Setup Script

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}================================================${NC}"
echo -e "${CYAN}X 账号自动注册系统 - Linux 编译环境安装${NC}"
echo -e "${CYAN}X Account Auto Registration - Linux Build Setup${NC}"
echo -e "${CYAN}================================================${NC}"
echo ""

# 检测 Linux 发行版
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    echo -e "${RED}无法检测 Linux 发行版${NC}"
    echo -e "${RED}Cannot detect Linux distribution${NC}"
    exit 1
fi

echo -e "${GREEN}检测到发行版: $DISTRO${NC}"
echo -e "${GREEN}Detected distribution: $DISTRO${NC}"
echo ""

# 根据发行版安装依赖
case "$DISTRO" in
    ubuntu|debian|linuxmint|pop)
        echo -e "${GREEN}使用 apt 安装依赖...${NC}"
        echo -e "${GREEN}Installing dependencies with apt...${NC}"
        
        sudo apt update
        sudo apt install -y curl build-essential pkg-config libssl-dev \
            libdbus-1-dev libglib2.0-dev git chromium-browser || \
            sudo apt install -y curl build-essential pkg-config libssl-dev \
            libdbus-1-dev libglib2.0-dev git chromium || \
            sudo apt install -y curl build-essential pkg-config libssl-dev \
            libdbus-1-dev libglib2.0-dev git google-chrome-stable
        ;;
    
    fedora|rhel|centos|rocky|almalinux)
        echo -e "${GREEN}使用 dnf/yum 安装依赖...${NC}"
        echo -e "${GREEN}Installing dependencies with dnf/yum...${NC}"
        
        if command -v dnf &> /dev/null; then
            sudo dnf install -y curl gcc gcc-c++ make pkg-config openssl-devel \
                dbus-devel glib2-devel git chromium
        else
            sudo yum install -y curl gcc gcc-c++ make pkg-config openssl-devel \
                dbus-devel glib2-devel git chromium
        fi
        ;;
    
    arch|manjaro)
        echo -e "${GREEN}使用 pacman 安装依赖...${NC}"
        echo -e "${GREEN}Installing dependencies with pacman...${NC}"
        
        sudo pacman -Sy --noconfirm curl base-devel pkg-config openssl \
            dbus glib2 git chromium
        ;;
    
    opensuse*|sles)
        echo -e "${GREEN}使用 zypper 安装依赖...${NC}"
        echo -e "${GREEN}Installing dependencies with zypper...${NC}"
        
        sudo zypper install -y curl gcc gcc-c++ make pkg-config libopenssl-devel \
            dbus-1-devel glib2-devel git chromium
        ;;
    
    *)
        echo -e "${YELLOW}未识别的发行版，请手动安装以下依赖:${NC}"
        echo -e "${YELLOW}Unrecognized distribution, please manually install:${NC}"
        echo "  - curl"
        echo "  - build-essential / gcc / make"
        echo "  - pkg-config"
        echo "  - openssl-dev"
        echo "  - git"
        echo "  - chromium / chrome"
        ;;
esac

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

# 确保 Rust 在 PATH 中
export PATH="$HOME/.cargo/bin:$PATH"
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
if [ -f ~/.zshrc ]; then
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
fi

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
echo ""
echo -e "${YELLOW}注意: 请重启终端或运行 'source ~/.bashrc' 以加载 Rust 环境${NC}"
echo -e "${YELLOW}Note: Please restart terminal or run 'source ~/.bashrc' to load Rust environment${NC}"
