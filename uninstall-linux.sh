#!/bin/bash
# GNU/Linux 卸载和清理脚本
# GNU/Linux Uninstall and Cleanup Script

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${RED}================================================${NC}"
echo -e "${RED}X 账号自动注册系统 - Linux 环境清理${NC}"
echo -e "${RED}X Account Auto Registration - Linux Cleanup${NC}"
echo -e "${RED}================================================${NC}"
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
        for rc_file in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
            if [ -f "$rc_file" ]; then
                sed -i.bak '/cargo\/bin/d' "$rc_file"
                echo -e "  ${GREEN}✓ 已清理 $(basename $rc_file) 中的 Rust 环境变量${NC}"
            fi
        done
        
        echo -e "  ${GREEN}✓ Rust 已卸载${NC}"
    else
        echo -e "  ${NC}Rust 未安装，跳过${NC}"
    fi
else
    echo -e "  ${NC}保留 Rust${NC}"
fi

echo ""

# 3. 询问是否通过包管理器卸载相关工具
echo -e "${CYAN}[3/6] 包管理器清理...${NC}"
echo -e "${CYAN}[3/6] Package manager cleanup...${NC}"

read -p "是否卸载通过包管理器安装的开发工具？(Y/N) / Uninstall dev tools via package manager? (Y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "  ${YELLOW}警告: 这些工具可能被其他项目使用${NC}"
    echo -e "  ${YELLOW}Warning: These tools might be used by other projects${NC}"
    read -p "确认继续？(Y/N) / Confirm? (Y/N): " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        case "$DISTRO" in
            ubuntu|debian|linuxmint|pop)
                echo -e "  ${YELLOW}使用 apt 卸载...${NC}"
                sudo apt remove -y build-essential pkg-config libssl-dev libdbus-1-dev libglib2-dev chromium-browser chromium google-chrome-stable 2>/dev/null || true
                sudo apt autoremove -y
                ;;
            
            fedora|rhel|centos|rocky|almalinux)
                echo -e "  ${YELLOW}使用 dnf/yum 卸载...${NC}"
                if command -v dnf &> /dev/null; then
                    sudo dnf remove -y gcc gcc-c++ make pkg-config openssl-devel dbus-devel glib2-devel chromium 2>/dev/null || true
                    sudo dnf autoremove -y
                else
                    sudo yum remove -y gcc gcc-c++ make pkg-config openssl-devel dbus-devel glib2-devel chromium 2>/dev/null || true
                    sudo yum autoremove -y
                fi
                ;;
            
            arch|manjaro)
                echo -e "  ${YELLOW}使用 pacman 卸载...${NC}"
                sudo pacman -Rs --noconfirm base-devel pkg-config openssl dbus glib2 chromium 2>/dev/null || true
                ;;
            
            opensuse*|sles)
                echo -e "  ${YELLOW}使用 zypper 卸载...${NC}"
                sudo zypper remove -y gcc gcc-c++ make pkg-config libopenssl-devel dbus-1-devel glib2-devel chromium 2>/dev/null || true
                ;;
            
            *)
                echo -e "  ${YELLOW}未识别的发行版，跳过包管理器清理${NC}"
                ;;
        esac
        
        echo -e "  ${GREEN}✓ 已清理开发工具${NC}"
    fi
else
    echo -e "  ${NC}保留开发工具${NC}"
fi

echo ""

# 4. 清理应用程序数据
echo -e "${CYAN}[4/6] 清理应用程序数据...${NC}"
echo -e "${CYAN}[4/6] Cleaning application data...${NC}"

read -p "是否清理应用程序数据？(Y/N) / Clean application data? (Y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    app_data_dirs=(
        "$HOME/.local/share/auto-x-account"
        "$HOME/.cache/auto-x-account"
        "$HOME/.config/auto-x-account"
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
        "$HOME/.cache/chromiumoxide"
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
echo -e "${NC}1. 请重启终端或运行 'source ~/.bashrc' 以使环境变量更改生效${NC}"
echo -e "${NC}   Please restart terminal or run 'source ~/.bashrc' for env changes to take effect${NC}"
echo ""
echo -e "${NC}2. 如果卸载了 Rust 或开发工具，它们可能被其他项目使用${NC}"
echo -e "${NC}   If you uninstalled Rust or dev tools, they might be used by other projects${NC}"
echo ""
echo -e "${NC}3. 系统包管理器的包可能被其他项目使用，请谨慎卸载${NC}"
echo -e "${NC}   System packages might be used by other projects${NC}"
echo ""

read -p "按 Enter 键退出 / Press Enter to exit"
