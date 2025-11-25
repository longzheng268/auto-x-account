# 截图策略和调试建议

## 📸 截图类型说明

### 1. 完整页面截图（当前实现）✅

**用途**：
- 调试页面结构
- 查看元素位置
- 了解页面状态

**实现**：
```rust
let screenshot = page
    .screenshot(chromiumoxide::page::ScreenshotParams::builder().build())
    .await?;
```

**优点**：
- 看到完整上下文
- 帮助定位问题
- 记录完整流程

**位置**：
```
C:\Users\Xiaomi\AppData\Roaming\auto-x-account\screenshots\
```

---

### 2. 元素截图（如需精确定位）

**用途**：
- 截取特定按钮/输入框
- 创建图像识别模板
- 精确定位元素

**实现**：
```rust
if let Ok(element) = page.find_element("button").await {
    let element_screenshot = element.screenshot().await?;
    std::fs::write("button.png", element_screenshot)?;
}
```

---

### 3. 视口截图 vs 全页面截图

#### 视口截图（当前）
```rust
// 只截取可见区域
.screenshot(ScreenshotParams::builder().build())
```

#### 全页面截图（包括滚动内容）
```rust
// 截取整个页面（包括需要滚动才能看到的部分）
.screenshot(
    ScreenshotParams::builder()
        .full_page(true)
        .build()
)
```

---

## 🔍 当前问题分析

根据日志输出：
```
2025-11-25T03:51:57.609483Z  INFO ✏️  步骤2: 填写姓名 / Step 2: Fill in name
2025-11-25T03:51:57.610022Z  INFO    姓名 / Name: Luigi Bins
2025-11-25T03:51:57.772578Z  INFO 🧹 清理资源 / Cleaning up resources...
2025-11-25T03:51:57.791685Z ERROR ❌ 账号注册失败 / Account registration failed: 未找到姓名输入框
```

**问题**：在没有先点击"创建账号"按钮的情况下，直接尝试填写姓名。

**原因**：文件已恢复到原始版本，缺少 `click_signup_button()` 函数。

---

## 🛠️ 调试步骤

### 步骤 1：查看截图

打开以下文件查看页面实际情况：
```
C:\Users\Xiaomi\AppData\Roaming\auto-x-account\screenshots\01_signup_page_20251125_035155.png
C:\Users\Xiaomi\AppData\Roaming\auto-x-account\screenshots\02_email_mode_20251125_035157.png
```

### 步骤 2：检查页面 HTML

添加 HTML 导出功能（临时调试）：

```rust
// 在 fill_name 函数失败前添加
async fn debug_page_inputs(&self, page: &chromiumoxide::Page) -> Result<()> {
    // 1. 保存完整 HTML
    let html = page.content().await?;
    let debug_dir = crate::data_dir::get_screenshots_dir();
    std::fs::write(debug_dir.join("debug_page.html"), &html)?;
    info!("页面 HTML 已保存到: {}", debug_dir.join("debug_page.html").display());
    
    // 2. 列出所有输入框
    if let Ok(inputs) = page.find_elements("input").await {
        info!("找到 {} 个输入框:", inputs.len());
        for (i, input) in inputs.iter().enumerate() {
            let input_type = input.get_attribute("type").await.unwrap_or_default();
            let name_attr = input.get_attribute("name").await.unwrap_or_default();
            let placeholder = input.get_attribute("placeholder").await.unwrap_or_default();
            let aria_label = input.get_attribute("aria-label").await.unwrap_or_default();
            
            info!("  输入框 #{}: type='{}', name='{}', placeholder='{}', aria-label='{}'",
                  i + 1, input_type, name_attr, placeholder, aria_label);
        }
    }
    
    // 3. 列出所有按钮
    if let Ok(buttons) = page.find_elements("button, div[role='button'], a[role='button']").await {
        info!("找到 {} 个按钮:", buttons.len());
        for (i, button) in buttons.iter().enumerate() {
            if let Ok(text) = button.inner_text().await {
                info!("  按钮 #{}: '{}'", i + 1, text);
            }
        }
    }
    
    Ok(())
}
```

### 步骤 3：使用浏览器开发者工具

在非 headless 模式下运行：
```toml
# config.json
{
  "browser": {
    "headless": false  # 显示浏览器窗口
  }
}
```

然后手动检查页面元素。

---

## 💡 推荐方案

### 方案 A：先查看现有截图（推荐）

1. 打开 `01_signup_page_20251125_035155.png`
2. 查看页面上是否有：
   - "创建账号" 按钮
   - 姓名输入框
   - 邮箱输入框
3. 根据实际情况调整选择器

### 方案 B：添加调试输出

在 `src/registration.rs` 的 `fill_name` 函数前添加调试代码：

```rust
// 在尝试填写姓名前，先输出页面信息
info!("========== 调试信息 ==========");
let html = page.content().await?;
std::fs::write("debug_page.html", &html)?;

if let Ok(inputs) = page.find_elements("input").await {
    info!("页面上的所有输入框:");
    for (i, input) in inputs.iter().enumerate() {
        let attrs = format!(
            "type={}, name={}, placeholder={}, aria-label={}",
            input.get_attribute("type").await.unwrap_or_default(),
            input.get_attribute("name").await.unwrap_or_default(),
            input.get_attribute("placeholder").await.unwrap_or_default(),
            input.get_attribute("aria-label").await.unwrap_or_default()
        );
        info!("  Input #{}: {}", i + 1, attrs);
    }
}
```

### 方案 C：使用浏览器检查工具

```bash
# 修改 config.json，设置 headless: false
# 然后运行，浏览器会显示出来，你可以手动检查
.\target\debug\auto-x-account.exe register --email test@example.com
```

---

## 📊 图像识别需要的截图类型

如果后续需要实现图像识别：

### 1. 模板图像（小图）
- **内容**：只包含按钮本身
- **尺寸**：刚好覆盖按钮（如 200x50 像素）
- **格式**：PNG 带透明度更好
- **示例**：
  ```
  templates/
  ├── button_signup.png       (只包含"创建账号"按钮)
  ├── button_next.png         (只包含"下一步"按钮)
  └── input_name.png          (姓名输入框的特征)
  ```

### 2. 待搜索图像（大图）
- **内容**：完整页面或区域
- **尺寸**：整个浏览器视口
- **格式**：PNG
- **示例**：当前的截图就是这种

### 3. 匹配流程
```
1. 加载完整页面截图（大图）
2. 加载按钮模板（小图）
3. 在大图中搜索小图的位置
4. 找到后获取坐标
5. 在该坐标点击
```

---

## ✅ 结论

**当前截图策略（完整页面）是正确的**，非常适合：
- ✅ 调试问题
- ✅ 记录流程
- ✅ 图像识别（作为待搜索图像）

**下一步建议**：
1. 先查看已保存的截图了解页面结构
2. 根据实际页面调整选择器
3. 如果选择器无法解决，再考虑图像识别

**对于图像识别**：
- 需要额外准备模板图像（小图）
- 当前截图已经可以作为待搜索图像
