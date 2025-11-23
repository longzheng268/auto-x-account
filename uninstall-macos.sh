#!/bin/bash
# macOS 卸载和清理脚本
# macOS Uninstall and Cleanup Script

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${RED}================================================${NC}"
echo -e "${RED}X 账号自动注册系统 - macOS 环境清理${NC}"
echo -e "${RED}X Account Auto Registration - macOS Cleanup${NC}"
echo -e "${RED}================================================${NC}"
echo ""

echo -e "${YELLOW}此脚本将清理本项目安装的所有开发环境${NC}"
echo -e "${YELLOW}This script will clean up all development environments installed by this project${NC}"
echo ""
read -p "确认继续？(Y/N) / Continue? (Y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}已取消清理${NC}"
    echo -e "${YELLOW}Cleanup cancelled${NC}"
    exit 0
fi

echo ""
echo -e "${GREEN}开始清理...${NC}"
echo -e "${GREEN}Starting cleanup...${NC}"
echo ""

# 1. 清理本项目的构建产物和数据
echo -e "${CYAN}[1/6] 清理项目构建产物和数据...${NC}"
echo -e "${CYAN}[1/6] Cleaning project build artifacts and data...${NC}"

items_to_remove=(
    "target"
    "browser_data"
    "screenshots"
    "logs"
    "accounts.json"
    "config.json"
    "Cargo.lock"
)

for item in "${items_to_remove[@]}"; do
    if [ -e "$item" ]; then
        rm -rf "$item"
        echo -e "  ${GREEN}✓ 已删除: $item${NC}"
    fi
done

echo ""

# 2. 询问是否卸载 Rust
echo -e "${CYAN}[2/6] Rust 卸载...${NC}"
echo -e "${CYAN}[2/6] Rust uninstall...${NC}"
read -p "是否卸载 Rust？(Y/N) / Uninstall Rust? (Y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    if command -v rustup &> /dev/null; then
        echo -e "  ${YELLOW}正在卸载 Rust...${NC}"
        rustup self uninstall -y
        
        # 清理 Rust 相关目录
        [ -d "$HOME/.cargo" ] && rm -rf "$HOME/.cargo" && echo -e "  ${GREEN}✓ 已删除: ~/.cargo${NC}"
        [ -d "$HOME/.rustup" ] && rm -rf "$HOME/.rustup" && echo -e "  ${GREEN}✓ 已删除: ~/.rustup${NC}"
        
        # 清理环境变量配置
        if [ -f "$HOME/.zprofile" ]; then
            sed -i.bak '/cargo\/env/d' "$HOME/.zprofile"
            echo -e "  ${GREEN}✓ 已清理 .zprofile 中的 Rust 环境变量${NC}"
        fi
        
        if [ -f "$HOME/.bash_profile" ]; then
            sed -i.bak '/cargo\/env/d' "$HOME/.bash_profile"
            echo -e "  ${GREEN}✓ 已清理 .bash_profile 中的 Rust 环境变量${NC}"
        fi
        
        echo -e "  ${GREEN}✓ Rust 已卸载${NC}"
    else
        echo -e "  ${NC}Rust 未安装，跳过${NC}"
    fi
else
    echo -e "  ${NC}保留 Rust${NC}"
fi

echo ""

# 3. 询问是否通过 Homebrew 卸载相关工具
echo -e "${CYAN}[3/6] Homebrew 包管理器清理...${NC}"
echo -e "${CYAN}[3/6] Homebrew package manager cleanup...${NC}"

if command -v brew &> /dev/null; then
    read -p "是否卸载通过 Homebrew 安装的工具？(Y/N) / Uninstall Homebrew packages? (Y/N): " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # 定义包及其描述
        declare -A packages=(
            ["git"]="Git 版本控制 / Git version control"
            ["pkg-config"]="包配置工具 / Package config tool"
            ["google-chrome"]="Google Chrome 浏览器 / Google Chrome browser"
        )
        
        for pkg in "${!packages[@]}"; do
            if brew list "$pkg" &> /dev/null || brew list --cask "$pkg" &> /dev/null; then
                echo ""
                echo -e "  ${YELLOW}发现包: $pkg - ${packages[$pkg]}${NC}"
                echo -e "  ${YELLOW}Found package: $pkg - ${packages[$pkg]}${NC}"
                read -p "  是否卸载此包？(Y/N) / Uninstall this package? (Y/N): " -n 1 -r
                echo ""
                
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    echo -e "  ${YELLOW}正在卸载: $pkg${NC}"
                    brew uninstall "$pkg" --ignore-dependencies 2>/dev/null || brew uninstall --cask "$pkg" --ignore-dependencies 2>/dev/null || true
                    echo -e "  ${GREEN}✓ 已卸载: $pkg${NC}"
                else
                    echo -e "  ${NC}跳过: $pkg${NC}"
                fi
            fi
        done
        
        # 询问是否卸载 Homebrew 本身
        read -p "是否卸载 Homebrew 本身？(Y/N) / Uninstall Homebrew itself? (Y/N): " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo -e "  ${YELLOW}正在卸载 Homebrew...${NC}"
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/uninstall.sh)"
            
            # 清理 Homebrew 目录
            if [[ $(uname -m) == 'arm64' ]]; then
                [ -d "/opt/homebrew" ] && sudo rm -rf "/opt/homebrew" && echo -e "  ${GREEN}✓ 已删除 Homebrew 目录${NC}"
            else
                [ -d "/usr/local/Homebrew" ] && sudo rm -rf "/usr/local/Homebrew" && echo -e "  ${GREEN}✓ 已删除 Homebrew 目录${NC}"
            fi
            
            # 清理环境变量
            if [ -f "$HOME/.zprofile" ]; then
                sed -i.bak '/brew shellenv/d' "$HOME/.zprofile"
                echo -e "  ${GREEN}✓ 已清理 .zprofile 中的 Homebrew 环境变量${NC}"
            fi
            
            echo -e "  ${GREEN}✓ Homebrew 已卸载${NC}"
        fi
    fi
else
    echo -e "  ${NC}Homebrew 未安装，跳过${NC}"
fi

echo ""

# 4. 清理应用程序数据
echo -e "${CYAN}[4/6] 清理应用程序数据...${NC}"
echo -e "${CYAN}[4/6] Cleaning application data...${NC}"

read -p "是否清理应用程序数据？(Y/N) / Clean application data? (Y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    app_data_dirs=(
        "$HOME/Library/Application Support/auto-x-account"
        "$HOME/Library/Caches/auto-x-account"
        "$HOME/Library/Logs/auto-x-account"
    )
    
    for dir in "${app_data_dirs[@]}"; do
        if [ -d "$dir" ]; then
            rm -rf "$dir"
            echo -e "  ${GREEN}✓ 已删除: $dir${NC}"
        fi
    done
else
    echo -e "  ${NC}保留应用程序数据${NC}"
fi

echo ""

# 5. 清理临时文件
echo -e "${CYAN}[5/6] 清理临时文件...${NC}"
echo -e "${CYAN}[5/6] Cleaning temporary files...${NC}"

read -p "是否清理临时文件？(Y/N) / Clean temporary files? (Y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    temp_dirs=(
        "/tmp/auto-x-account*"
        "$HOME/.auto-x-account"
    )
    
    for pattern in "${temp_dirs[@]}"; do
        for dir in $pattern; do
            if [ -e "$dir" ]; then
                rm -rf "$dir"
                echo -e "  ${GREEN}✓ 已删除: $dir${NC}"
            fi
        done
    done
else
    echo -e "  ${NC}保留临时文件${NC}"
fi

echo ""

# 6. 清理缓存
echo -e "${CYAN}[6/6] 清理缓存...${NC}"
echo -e "${CYAN}[6/6] Cleaning cache...${NC}"

read -p "是否清理缓存？(Y/N) / Clean cache? (Y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    cache_dirs=(
        "$HOME/Library/Caches/chromiumoxide"
    )
    
    for dir in "${cache_dirs[@]}"; do
        if [ -d "$dir" ]; then
            rm -rf "$dir"
            echo -e "  ${GREEN}✓ 已删除缓存: $dir${NC}"
        fi
    done
else
    echo -e "  ${NC}保留缓存${NC}"
fi

echo ""
echo -e "${GREEN}================================================${NC}"
echo -e "${GREEN}清理完成!${NC}"
echo -e "${GREEN}Cleanup completed!${NC}"
echo -e "${GREEN}================================================${NC}"
echo ""
echo -e "${YELLOW}注意事项 / Notes:${NC}"
echo -e "${NC}1. 请重启终端以使环境变量更改生效${NC}"
echo -e "${NC}   Please restart terminal for environment variable changes to take effect${NC}"
echo ""
echo -e "${NC}2. 如果卸载了 Rust 或 Git，它们可能被其他项目使用${NC}"
echo -e "${NC}   If you uninstalled Rust or Git, they might be used by other projects${NC}"
echo ""
echo -e "${NC}3. Homebrew 和其包可能被其他项目使用，请谨慎卸载${NC}"
echo -e "${NC}   Homebrew and its packages might be used by other projects${NC}"
echo ""

read -p "按 Enter 键退出 / Press Enter to exit"
