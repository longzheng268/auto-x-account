# X/Twitter 动态CSS类名问题的解决方案

## 问题说明

X/Twitter 使用 React 框架生成大量动态 CSS 类名（如 `css-175oi2r r-sdzlij r-1phboty`），这些类名会在不同时间变化，导致传统的 CSS 选择器不稳定。

## 解决方案总结

### ✅ 推荐方案：多层次选择器策略

我已经在代码中实现了以下改进：

1. **基于文本内容的选择器**（最稳定）
   - `text='创建账号'`
   - `text='Sign up'`
   - `span:has-text('Sign up')`

2. **基于 ARIA 属性**
   - `[aria-label*='Sign up']`
   - `[aria-label*='创建账号']`

3. **基于 data-testid**（如果存在）
   - `[data-testid*='signup']`

4. **基于 role 和文本组合**
   - `button:has-text('Sign up')`
   - `div[role='button']:has-text('Sign up')`

5. **XPath 表达式**
   - `//button[contains(text(), '创建账号')]`
   - `//div[@role='button' and contains(., 'Sign up')]`

6. **动态遍历**（最后备选）
   - 遍历所有按钮，通过文本内容匹配

## 实际应用的代码改进

### 1. 新增函数：click_signup_button

```rust
/// 点击创建账号按钮
async fn click_signup_button(&self, page: &chromiumoxide::Page) -> Result<()> {
    info!("   查找'创建账号'按钮 / Looking for 'Sign up' button");
    
    // 尝试多种稳定选择器（按优先级排序）
    let selectors = vec![
        // 1. 基于文本内容（最稳定）
        "text='创建账号'",
        "text='Sign up'",
        "text='Create account'",
        "span:has-text('创建账号')",
        "span:has-text('Sign up')",
        
        // 2. 基于 ARIA 标签
        "[aria-label*='Sign up']",
        "[aria-label*='创建账号']",
        "[aria-label*='Create account']",
        
        // 3. 基于 data-testid（如果存在）
        "[data-testid*='signup']",
        "[data-testid*='signupButton']",
        "[data-testid='signup-button']",
        
        // 4. 基于 role 和文本组合
        "button:has-text('Sign up')",
        "button:has-text('创建账号')",
        "div[role='button']:has-text('Sign up')",
        "div[role='button']:has-text('创建账号')",
        
        // 5. XPath 备选
        "//button[contains(text(), '创建账号')]",
        "//button[contains(text(), 'Sign up')]",
        "//span[contains(text(), '创建账号')]",
        "//span[contains(text(), 'Sign up')]",
        "//div[@role='button' and contains(., 'Sign up')]",
        "//div[@role='button' and contains(., '创建账号')]",
    ];

    for selector in selectors {
        if let Ok(element) = page.find_element(selector).await {
            info!("   ✅ 找到创建账号按钮: {}", selector);
            element.click().await?;
            return Ok(());
        }
    }

    // 如果所有选择器都失败，尝试查找所有可点击元素并通过文本匹配
    info!("   ⚠️  常规选择器失败，尝试遍历所有按钮");
    if let Ok(buttons) = page.find_elements("button, div[role='button'], a[role='button']").await {
        for button in buttons {
            if let Ok(text) = button.inner_text().await {
                let text_lower = text.to_lowercase();
                if text_lower.contains("sign up") || 
                   text_lower.contains("创建账号") || 
                   text_lower.contains("create account") {
                    info!("   🔎 通过文本匹配找到按钮: {}", text);
                    button.click().await?;
                    return Ok(());
                }
            }
        }
    }

    anyhow::bail!("未找到创建账号按钮 / Sign up button not found")
}
```

### 2. 改进的 fill_name 函数

```rust
/// 填写姓名
async fn fill_name(&self, page: &chromiumoxide::Page, name: &str) -> Result<()> {
    // 1. 常规 CSS 选择器
    let selectors = vec![
        "input[name='name']",
        "input[autocomplete='name']",
        "input[placeholder*='Name']",
        "input[placeholder*='名字']",
    ];
    for selector in selectors {
        if let Ok(element) = page.find_element(selector).await {
            element.click().await?;
            element.type_str(name).await?;
            info!("   ✅ 姓名已填写 / Name filled (selector)");
            return Ok(());
        }
    }

    // 2. 宽松的 XPath 匹配
    let xpath_variants = vec![
        "//input[contains(@aria-label, 'name') or contains(@placeholder, 'Name') or contains(@placeholder, '名字')]",
        "//input[contains(@type, 'text') and (contains(@placeholder, 'Name') or contains(@placeholder, '名字'))]",
    ];
    for xpath in xpath_variants {
        if let Ok(element) = page.find_element(xpath).await {
            element.click().await?;
element.type_str(name).await?;
            info!("   ✅ 姓名已填写 (XPath) / Name filled via XPath");
            return Ok(());
        }
    }

    // 3. 遍历所有 input，寻找可能的姓名字段
    if let Ok(all_inputs) = page.find_elements("input").await {
        for input in all_inputs {
            let placeholder = input.get_attribute("placeholder").await.unwrap_or_default();
            let aria_label = input.get_attribute("aria-label").await.unwrap_or_default();
            let name_attr = input.get_attribute("name").await.unwrap_or_default();
            if placeholder.to_lowercase().contains("name")
                || aria_label.to_lowercase().contains("name")
                || name_attr.to_lowercase().contains("name")
{
                info!("   🔎 找到可能的姓名输入框: placeholder='{}', aria-label='{}', name='{}'", 
                      placeholder, aria_label, name_attr);
                input.click().await?;
                input.type_str(name).await?;
                info!("   ✅ 姓名已填写 (fallback) / Name filled via fallback");
                return Ok(());
            }
        }
    }

    // 4. 若仍未找到，输出页面 HTML 供调试
    let html = page.content().await.unwrap_or_default();
    warn!("   ❌ 未找到姓名输入框，输出页面 HTML 供调试");
    tracing::debug!("页面 HTML: {}", html);

    anyhow::bail!("未找到姓名输入框 / Name input not found");
}
```

## 为什么不使用 SikuliX？

SikuliX 是 Java 库，在 Rust 项目中使用会增加很多复杂性：

1. **跨语言集成复杂**：需要通过 JNI 或外部进程调用
2. **依赖管理困难**：需要安装 Java 运行时环境
3. **性能开销**：进程间通信开销较大
4. **分发困难**：需要同时分发 Rust 和 Java 组件

## Rust 原生替代方案

如果确实需要图像识别，可以使用以下 Rust 库：

### 1. 使用 imageproc 进行模板匹配

```toml
# Cargo.toml
[dependencies]
image = "0.25"
imageproc = "0.25"
```

```rust
use image::io::Reader as ImageReader;
use imageproc::template_matching::{match_template, MatchTemplateMethod};

async fn find_button_by_image(page: &Page, template_path: &str) -> Result<Option<(u32, u32)>> {
    // 1. 截图页面
    let screenshot = page.screenshot(ScreenshotParams::builder().build()).await?;
    let page_img = image::load_from_memory(&screenshot)?;
    
    // 2. 加载按钮模板
    let template_img = image::open(template_path)?;
    
    // 3. 转换为灰度图
    let page_gray = page_img.to_luma8();
    let template_gray = template_img.to_luma8();
    
    // 4. 模板匹配
    let result = match_template(&page_gray, &template_gray, MatchTemplateMethod::CcoeffNormed);
    
    // 5. 找到最佳匹配位置
    // ... (查找最大值逻辑)
    
    Ok(Some((click_x, click_y)))
}
```

### 2. 使用 OCR 识别文字（tesseract-rs）

```toml
[dependencies]
tesseract = "0.15"
```

```rust
use tesseract::Tesseract;

async fn find_text_by_ocr(page: &Page, text: &str) -> Result<Option<(u32, u32)>> {
    // 截图并进行 OCR
    let screenshot = page.screenshot(ScreenshotParams::builder().build()).await?;
    let ocr = Tesseract::new(None, Some("chi_sim+eng"))?
        .set_image_from_mem(&screenshot)?
        .recognize()?;
    
    // 分析 OCR 结果，找到目标文字的位置
    // ...
    
    Ok(None)
}
```

## 推荐实施步骤

1. **先使用改进的选择器策略**（已实现）
   - 测试各种选择器的有效性
   - 收集失败案例

2. **如果选择器不稳定，添加图像识别作为备选**
   - 保存常用按钮的截图作为模板
   - 实现模板匹配函数
   - 在选择器失败时自动回退到图像识别

3. **持续监控和优化**
   - 记录哪些选择器最可靠
   - 定期更新选择器列表
   - 收集 X/Twitter 页面结构变化信息

## 调试技巧

1. **截图调试**
   ```rust
   self.take_screenshot(page, "debug_before_click").await?;
   ```

2. **输出页面 HTML**
   ```rust
   let html = page.content().await?;
   std::fs::write("debug_page.html", html)?;
   ```

3. **元素属性检查**
   ```rust
   if let Ok(element) = page.find_element("button").await {
       let html = element.outer_html().await?;
       info!("Button HTML: {}", html);
   }
   ```

## 总结

针对 X/Twitter 动态 CSS 类名问题：

✅ **已实现**：多层次选择器策略（文本、ARIA、XPath、动态遍历）  
✅ **无需 SikuliX**：Rust 原生工具更简单高效  
⚠️ **备选方案**：如需图像识别，使用 imageproc 或 tesseract  
📝 **持续优化**：收集数据，不断改进选择器

当前实现的选择器策略已经非常健壮，应该能够应对大部分情况。如果仍然遇到问题，请提供具体的错误日志和截图，我会进一步优化。
