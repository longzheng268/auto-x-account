# 浏览器支持说明 / Browser Support

本程序支持自动检测并使用系统已安装的 Chromium 内核浏览器。

## 🌐 支持的浏览器

### Windows
按优先级顺序检测：
1. **Microsoft Edge** (推荐) - Windows 10/11 自带
   - `C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe`
   - `C:\Program Files\Microsoft\Edge\Application\msedge.exe`

2. **Google Chrome**
   - `C:\Program Files\Google\Chrome\Application\chrome.exe`
   - `C:\Program Files (x86)\Google\Chrome\Application\chrome.exe`

3. **Chromium**
   - `C:\Program Files\Chromium\Application\chrome.exe`
   - `C:\Program Files (x86)\Chromium\Application\chrome.exe`

4. **Brave Browser**
   - `C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe`

5. **Vivaldi**
   - `C:\Program Files\Vivaldi\Application\vivaldi.exe`

### macOS
1. **Google Chrome** - `/Applications/Google Chrome.app`
2. **Microsoft Edge** - `/Applications/Microsoft Edge.app`
3. **Chromium** - `/Applications/Chromium.app`
4. **Brave** - `/Applications/Brave Browser.app`
5. **Vivaldi** - `/Applications/Vivaldi.app`

### Linux
1. **Google Chrome** - `/usr/bin/google-chrome`
2. **Chromium** - `/usr/bin/chromium` 或 `/snap/bin/chromium`
3. **Microsoft Edge** - `/usr/bin/microsoft-edge`
4. **Brave** - `/usr/bin/brave-browser`
5. **Vivaldi** - `/usr/bin/vivaldi`

## 🔍 检测优先级

程序按以下顺序查找浏览器：

1. **配置文件指定的路径**
   - 在 `config.json` 中设置 `browser.chrome_path`
   - 例如：`"chrome_path": "C:\\Custom\\Path\\chrome.exe"`

2. **打包的 Chromium**
   - 程序目录下的 `chromium` 文件夹
   - 适合便携版部署

3. **系统安装的浏览器**
   - 自动检测上述列表中的浏览器
   - Windows 优先使用 Edge（系统自带）

4. **默认查找**
   - 如果都没找到，使用 chromiumoxide 的默认检测

## ⚙️ 配置方式

### 方式 1：自动检测（推荐）

不需要任何配置，程序会自动找到系统浏览器：

```json
{
  "browser": {
    "headless": false,
    "timeout": 60000
  }
}
```

### 方式 2：指定浏览器路径

在 `config.json` 中指定：

```json
{
  "browser": {
    "chrome_path": "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
    "headless": false,
    "timeout": 60000
  }
}
```

### 方式 3：使用打包的 Chromium

1. 下载 Chromium 到程序目录：
   ```powershell
   .\download-chromium.ps1
   ```

2. 程序会自动检测并使用

## 📝 注意事项

### ✅ 推荐配置

- **Windows 用户**：使用系统自带的 Edge，无需额外安装
- **开发调试**：设置 `"headless": false` 可以看到浏览器界面
- **生产环境**：设置 `"headless": true` 提高性能

### ⚠️ 常见问题

**Q: 为什么不支持 Firefox?**
A: 本程序使用 chromiumoxide 库，仅支持 Chromium 内核浏览器。Firefox 使用不同的自动化协议。

**Q: 如何确认使用了哪个浏览器?**
A: 运行程序时会在日志中显示：
```
✅ 使用系统安装的浏览器 / Using system browser: C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe
```

**Q: 可以同时使用多个浏览器吗?**
A: 程序会按优先级使用第一个找到的浏览器。如需使用特定浏览器，请在配置中指定路径。

**Q: Edge 和 Chrome 有什么区别?**
A: 两者都基于 Chromium 内核，功能相同。Edge 是 Windows 10/11 自带的，无需额外安装。

## 🚀 快速开始

### 最简单的方式（Windows）

1. 确保系统有 Edge 浏览器（Windows 10/11 自带）
2. 直接运行程序，无需任何配置
3. 程序会自动使用 Edge

### 如果想使用 Chrome

1. 安装 Google Chrome
2. 程序会自动检测并使用（如果没有 Edge）
3. 或者在配置中指定 Chrome 路径

### 离线/便携部署

1. 运行 `.\download-chromium.ps1` 下载独立的 Chromium
2. 将整个程序文件夹复制到目标机器
3. 无需系统安装任何浏览器

## 🔧 高级配置

### 使用特定版本的浏览器

```json
{
  "browser": {
    "chrome_path": "D:\\Browsers\\Chrome-v120\\chrome.exe",
    "headless": false
  }
}
```

### 使用便携版浏览器

```json
{
  "browser": {
    "chrome_path": ".\\portable-chrome\\chrome.exe",
    "headless": true
  }
}
```

### 调试模式

```json
{
  "browser": {
    "headless": false,
    "timeout": 120000
  }
}
```

## 📊 性能对比

| 浏览器 | 启动速度 | 内存占用 | 推荐场景 |
|--------|---------|---------|---------|
| **Edge** | ⚡⚡⚡ 快 | 💾 中等 | Windows 日常使用 |
| **Chrome** | ⚡⚡ 中等 | 💾💾 较高 | 跨平台开发 |
| **Chromium** | ⚡⚡⚡ 快 | 💾 较低 | 轻量级部署 |
| **Brave** | ⚡⚡ 中等 | 💾 中等 | 隐私保护 |

## 🆘 故障排除

### 问题：找不到浏览器

**解决方案**：
1. 检查是否安装了任何 Chromium 内核浏览器
2. 在配置中手动指定浏览器路径
3. 运行 `.\download-chromium.ps1` 下载独立版本

### 问题：浏览器启动失败

**解决方案**：
1. 确认浏览器路径正确
2. 检查浏览器版本是否过旧（建议 90+）
3. 尝试使用 `headless: false` 查看错误信息

### 问题：代理设置不生效

**解决方案**：
1. 确认代理配置正确
2. 某些浏览器可能需要额外的启动参数
3. 查看日志中的代理信息确认
