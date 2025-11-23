# 字体文件 / Fonts

## MiSans 字体

本项目使用 MiSans 字体作为界面字体。

### 下载方式 / Download

1. 访问 MiSans 字体下载页面：https://hyperos.mi.com/font-download/MiSans.zip
2. 下载 `MiSans.zip` 文件
3. 解压到当前 `fonts/` 目录

### 文件结构 / File Structure

解压后的文件结构应该如下：

```
fonts/
├── README.md
├── MiSans/
│   ├── MiSans-Regular.ttf
│   ├── MiSans-Bold.ttf
│   ├── MiSans-Light.ttf
│   └── ...
```

### 自动下载脚本 / Auto Download Script

**Linux/macOS:**
```bash
cd fonts
curl -L -o MiSans.zip "https://hyperos.mi.com/font-download/MiSans.zip"
unzip MiSans.zip
rm MiSans.zip
```

**Windows (PowerShell):**
```powershell
cd fonts
Invoke-WebRequest -Uri "https://hyperos.mi.com/font-download/MiSans.zip" -OutFile "MiSans.zip"
Expand-Archive -Path "MiSans.zip" -DestinationPath "."
Remove-Item "MiSans.zip"
```

### 注意事项 / Notes

- 字体文件不包含在版本控制中（已在 .gitignore 中排除）
- 首次运行前请确保已下载并解压字体文件
- Font files are not included in version control (excluded in .gitignore)
- Please ensure fonts are downloaded and extracted before first run
