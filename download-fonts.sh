#!/bin/bash
# 下载 MiSans 字体 / Download MiSans Font

set -e

FONT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/fonts"
FONT_URL="https://hyperos.mi.com/font-download/MiSans.zip"

echo "正在下载 MiSans 字体... / Downloading MiSans font..."
cd "$FONT_DIR"

# 检查是否已存在字体文件
if [ -d "MiSans" ]; then
    echo "MiSans 字体已存在 / MiSans font already exists"
    read -p "是否重新下载？(y/N) / Re-download? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "跳过下载 / Skipping download"
        exit 0
    fi
    rm -rf MiSans
fi

# 下载字体
if command -v curl &> /dev/null; then
    curl -L -o MiSans.zip "$FONT_URL"
elif command -v wget &> /dev/null; then
    wget -O MiSans.zip "$FONT_URL"
else
    echo "错误: 未找到 curl 或 wget / Error: curl or wget not found"
    exit 1
fi

# 解压
if command -v unzip &> /dev/null; then
    unzip MiSans.zip
    rm MiSans.zip
else
    echo "错误: 未找到 unzip 命令 / Error: unzip command not found"
    echo "请手动解压 MiSans.zip / Please manually extract MiSans.zip"
    exit 1
fi

echo "✓ MiSans 字体下载完成 / MiSans font download completed"
