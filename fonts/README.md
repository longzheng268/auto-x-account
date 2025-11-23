# 字体文件 / Fonts

## 字体说明 / Font Description

本项目 GUI 界面支持使用 MiSans 字体，以提供更好的中文显示效果。如果 MiSans 字体不可用，程序会自动使用内置的 fallback 字体（DejaVu Sans）。

This project's GUI supports MiSans font for better Chinese text display. If MiSans is unavailable, the program automatically uses the bundled fallback font (DejaVu Sans).

## 内置字体 / Bundled Fonts

- **fallback.ttf**: DejaVu Sans 字体，已内置在项目中，确保程序在任何环境都能正常显示文本
- **fallback.ttf**: DejaVu Sans font, bundled in the project to ensure text displays correctly in any environment

## MiSans 字体（可选）/ MiSans Font (Optional)

MiSans 是小米公司设计的开源字体，对中文支持友好。下载后会在编译时嵌入到二进制文件中。

MiSans is an open-source font designed by Xiaomi with excellent Chinese language support. When downloaded, it will be embedded into the binary during compilation.

### 下载方式 / Download

**自动下载（推荐）/ Automatic Download (Recommended):**

**Linux/macOS:**
```bash
cd /path/to/auto-x-account
./download-fonts.sh
```

**Windows (PowerShell):**
```powershell
cd \path\to\auto-x-account
.\download-fonts.ps1
```

**手动下载 / Manual Download:**

1. 访问 MiSans 字体下载页面：https://hyperos.mi.com/font-download/MiSans.zip
2. 下载 `MiSans.zip` 文件
3. 解压到当前 `fonts/` 目录

### 文件结构 / File Structure

解压后的文件结构应该如下：

```
fonts/
├── README.md
├── fallback.ttf          # 内置 fallback 字体 (已提供)
└── MiSans/               # MiSans 字体 (可选下载)
    └── ttf/
        └── MiSans-Regular.ttf
```

### 编译时行为 / Compile-time Behavior

- 如果存在 `fonts/MiSans/ttf/MiSans-Regular.ttf`，编译时会使用 MiSans 字体
- 如果不存在 MiSans 字体，编译时会自动使用 `fallback.ttf`
- 无论哪种情况，字体都会被嵌入到最终的二进制文件中
- 预编译的发布版本已包含嵌入的字体，无需额外下载

- If `fonts/MiSans/ttf/MiSans-Regular.ttf` exists, MiSans will be used during compilation
- If MiSans doesn't exist, `fallback.ttf` will be used automatically
- In either case, the font is embedded into the final binary
- Pre-compiled release versions already include embedded fonts, no additional download needed

### 注意事项 / Notes

- 字体文件不包含在版本控制中（MiSans 字体除外已在 .gitignore 排除）
- `fallback.ttf` 已包含在版本控制中，确保所有用户都能正常使用
- 首次编译前建议下载 MiSans 字体以获得最佳视觉效果
- Font files are not included in version control (MiSans is excluded in .gitignore)
- `fallback.ttf` is included in version control to ensure all users can run the program
- It's recommended to download MiSans font before first compilation for best visual experience
