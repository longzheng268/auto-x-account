#!/bin/bash
# 下载 Chromium 浏览器 / Download Chromium Browser
# 
# 这个脚本会下载适合当前操作系统的 Chromium 浏览器
# This script downloads Chromium browser for the current operating system

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHROMIUM_DIR="$SCRIPT_DIR/chromium"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Chromium 浏览器下载工具 ===${NC}"
echo -e "${GREEN}=== Chromium Browser Downloader ===${NC}"
echo ""

# 检测操作系统
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS=linux;;
        Darwin*)    OS=mac;;
        CYGWIN*|MINGW*|MSYS*)    OS=windows;;
        *)          OS=unknown;;
    esac
    echo "$OS"
}

# 检测架构
detect_arch() {
    case "$(uname -m)" in
        x86_64)     ARCH=x64;;
        aarch64|arm64)    ARCH=arm64;;
        *)          ARCH=unknown;;
    esac
    echo "$ARCH"
}

OS=$(detect_os)
ARCH=$(detect_arch)

echo -e "检测到操作系统 / Detected OS: ${YELLOW}$OS${NC}"
echo -e "检测到架构 / Detected Architecture: ${YELLOW}$ARCH${NC}"
echo ""

if [ "$OS" = "unknown" ] || [ "$ARCH" = "unknown" ]; then
    echo -e "${RED}错误: 不支持的操作系统或架构${NC}"
    echo -e "${RED}Error: Unsupported OS or architecture${NC}"
    exit 1
fi

# 检查是否已存在
if [ -d "$CHROMIUM_DIR" ]; then
    echo -e "${YELLOW}Chromium 目录已存在 / Chromium directory already exists${NC}"
    read -p "是否重新下载？/ Re-download? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "跳过下载 / Skipping download"
        exit 0
    fi
    rm -rf "$CHROMIUM_DIR"
fi

mkdir -p "$CHROMIUM_DIR"

# Chromium 下载链接
# 使用 Chromium 快照存档
# 您可以从这里找到最新的版本: https://commondatastorage.googleapis.com/chromium-browser-snapshots/index.html

CHROMIUM_BASE_URL="https://commondatastorage.googleapis.com/chromium-browser-snapshots"

# 获取最新的稳定版本号
echo -e "${GREEN}正在获取最新版本信息... / Fetching latest version...${NC}"

if [ "$OS" = "linux" ]; then
    if [ "$ARCH" = "arm64" ]; then
        PLATFORM="Linux_ARM64"
    else
        PLATFORM="Linux_x64"
    fi
    LAST_CHANGE_URL="$CHROMIUM_BASE_URL/$PLATFORM/LAST_CHANGE"
    VERSION=$(curl -s "$LAST_CHANGE_URL")
    DOWNLOAD_URL="$CHROMIUM_BASE_URL/$PLATFORM/$VERSION/chrome-linux.zip"
    EXTRACT_DIR="chrome-linux"
elif [ "$OS" = "mac" ]; then
    if [ "$ARCH" = "arm64" ]; then
        PLATFORM="Mac_Arm"
    else
        PLATFORM="Mac"
    fi
    LAST_CHANGE_URL="$CHROMIUM_BASE_URL/$PLATFORM/LAST_CHANGE"
    VERSION=$(curl -s "$LAST_CHANGE_URL")
    DOWNLOAD_URL="$CHROMIUM_BASE_URL/$PLATFORM/$VERSION/chrome-mac.zip"
    EXTRACT_DIR="chrome-mac"
elif [ "$OS" = "windows" ]; then
    if [ "$ARCH" = "arm64" ]; then
        PLATFORM="Win_ARM64"
    else
        PLATFORM="Win_x64"
    fi
    LAST_CHANGE_URL="$CHROMIUM_BASE_URL/$PLATFORM/LAST_CHANGE"
    VERSION=$(curl -s "$LAST_CHANGE_URL")
    DOWNLOAD_URL="$CHROMIUM_BASE_URL/$PLATFORM/$VERSION/chrome-win.zip"
    EXTRACT_DIR="chrome-win"
fi

echo -e "版本号 / Version: ${YELLOW}$VERSION${NC}"
echo -e "下载地址 / Download URL: ${YELLOW}$DOWNLOAD_URL${NC}"
echo ""

# 下载 Chromium
echo -e "${GREEN}正在下载 Chromium... / Downloading Chromium...${NC}"
TEMP_ZIP="/tmp/chromium_$$.zip"

if command -v curl &> /dev/null; then
    curl -# -L -o "$TEMP_ZIP" "$DOWNLOAD_URL"
elif command -v wget &> /dev/null; then
    wget --progress=bar:force -O "$TEMP_ZIP" "$DOWNLOAD_URL"
else
    echo -e "${RED}错误: 未找到 curl 或 wget / Error: curl or wget not found${NC}"
    exit 1
fi

# 解压
echo -e "${GREEN}正在解压... / Extracting...${NC}"
if command -v unzip &> /dev/null; then
    unzip -q "$TEMP_ZIP" -d "$CHROMIUM_DIR"
    # 移动文件到正确位置
    if [ -d "$CHROMIUM_DIR/$EXTRACT_DIR" ]; then
        mv "$CHROMIUM_DIR/$EXTRACT_DIR"/* "$CHROMIUM_DIR/"
        rmdir "$CHROMIUM_DIR/$EXTRACT_DIR"
    fi
else
    echo -e "${RED}错误: 未找到 unzip 命令 / Error: unzip command not found${NC}"
    echo -e "请手动解压 $TEMP_ZIP 到 $CHROMIUM_DIR 目录"
    echo -e "Please manually extract $TEMP_ZIP to $CHROMIUM_DIR directory"
    exit 1
fi

# 清理
rm -f "$TEMP_ZIP"

# 设置可执行权限
if [ "$OS" = "linux" ]; then
    chmod +x "$CHROMIUM_DIR/chrome"
elif [ "$OS" = "mac" ]; then
    # macOS 的 Chromium 下载包含 Chromium.app
    chmod +x "$CHROMIUM_DIR/Chromium.app/Contents/MacOS/Chromium" 2>/dev/null || true
fi

echo ""
echo -e "${GREEN}✓ Chromium 下载完成! / Chromium download completed!${NC}"
echo -e "安装位置 / Installation path: ${YELLOW}$CHROMIUM_DIR${NC}"
echo ""

# 显示版本信息
if [ "$OS" = "linux" ]; then
    CHROME_EXEC="$CHROMIUM_DIR/chrome"
elif [ "$OS" = "mac" ]; then
    CHROME_EXEC="$CHROMIUM_DIR/Chromium.app/Contents/MacOS/Chromium"
elif [ "$OS" = "windows" ]; then
    CHROME_EXEC="$CHROMIUM_DIR/chrome.exe"
fi

if [ -f "$CHROME_EXEC" ]; then
    echo -e "${GREEN}Chromium 可执行文件 / Chromium executable:${NC}"
    echo -e "  $CHROME_EXEC"
    
    # 尝试获取版本信息
    VERSION_INFO=$("$CHROME_EXEC" --version 2>/dev/null || echo "无法获取版本信息 / Cannot get version")
    echo -e "${GREEN}版本 / Version:${NC} $VERSION_INFO"
fi

echo ""
echo -e "${GREEN}现在可以运行程序了！/ You can now run the program!${NC}"
