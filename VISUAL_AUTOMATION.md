# 视觉自动化方案 - Visual Automation Strategy

## 问题背景

X/Twitter 使用 React 框架，生成了大量动态 CSS 类名（如 `css-175oi2r r-sdzlij r-1phboty` 等），这些类名在不同时间可能会变化，不适合作为稳定的 Playwright 选择器。

## 解决方案

### 方案一：优先使用稳定选择器（推荐）

在尝试视觉识别之前，应该优先使用更稳定的选择器：

#### 1. 基于文本内容的选择器
```rust
// 查找包含特定文本的元素
page.find_element("text='创建账号'").await
page.find_element("text='Sign up'").await
page.find_element("button:has-text('Next')").await
```

#### 2. 基于 ARIA 属性
```rust
// 使用 aria-label 属性
page.find_element("[aria-label='Sign up']").await
page.find_element("[aria-label='Next']").await
```

#### 3. 基于 role 属性
```rust
// 查找特定角色的元素
page.find_element("button[role='button']").await
page.find_element("input[role='textbox']").await
```

#### 4. XPath 表达式
```rust
// 使用 XPath 查找元素
page.find_element("//button[contains(text(), '创建账号')]").await
page.find_element("//span[text()='Sign up']").await
```

### 方案二：图像识别方案（备选）

如果稳定选择器不可用，可以使用图像识别：

#### Rust 图像识别库

1. **imageproc** - 图像处理算法
2. **image** - 图像加载和保存
3. **opencv-binding** - OpenCV Rust 绑定

#### 实现步骤

1. **截图页面**
```rust
use chromiumoxide::page::ScreenshotParams;

let screenshot = page.screenshot(ScreenshotParams::builder().build()).await?;
std::fs::write("screenshot.png", screenshot)?;
```

2. **模板匹配**（需要添加依赖）
```rust
use image::io::Reader as ImageReader;
use imageproc::template_matching::match_template;

// 加载页面截图
let page_img = ImageReader::open("screenshot.png")?.decode()?;

// 加载按钮模板（预先保存的按钮截图）
let template_img = ImageReader::open("button_template.png")?.decode()?;

// 模板匹配
let result = match_template(&page_img, &template_img, MatchTemplateMethod::Ccoeff);
```

3. **点击匹配位置**
```rust
// 找到最佳匹配位置后，计算点击坐标
let (max_x, max_y) = find_max_position(&result);
let click_x = max_x + template_width / 2;
let click_y = max_y + template_height / 2;

// 使用 CDP 协议发送点击事件
page.evaluate(format!(
    "document.elementFromPoint({}, {}).click()",
    click_x, click_y
)).await?;
```

### 方案三：混合方案（最佳实践）

结合选择器和图像识别的优点：

```rust
async fn click_signup_button(page: &Page) -> Result<()> {
    // 1. 尝试稳定选择器（优先级从高到低）
    let selectors = vec![
        "text='创建账号'",
        "text='Sign up'",
        "[aria-label*='Sign up']",
        "[data-testid='signup-button']",
        "button:has-text('Sign up')",
        "//button[contains(text(), 'Sign up')]",
    ];
    
    for selector in selectors {
        if let Ok(element) = page.find_element(selector).await {
            element.click().await?;
            info!("使用选择器找到按钮: {}", selector);
            return Ok(());
        }
    }
    
    // 2. 选择器失败，尝试图像识别
    warn!("所有选择器均失败，尝试图像识别...");
    
    // 截图
    let screenshot = page.screenshot(ScreenshotParams::builder().build()).await?;
    
    // 图像识别（调用图像识别模块）
    if let Some((x, y)) = find_button_by_image(&screenshot)? {
        // 通过坐标点击
        page.evaluate(format!("document.elementFromPoint({}, {}).click()", x, y)).await?;
        info!("通过图像识别找到按钮位置: ({}, {})", x, y);
        return Ok(());
    }
    
    anyhow::bail!("无法找到创建账号按钮");
}
```

## 推荐实现

### 1. 先增强现有选择器

在 `src/registration.rs` 中，为关键按钮添加更多备选选择器：

```rust
/// 查找并点击"创建账号"按钮
async fn click_signup_button(&self, page: &chromiumoxide::Page) -> Result<()> {
    let selectors = vec![
        // 文本匹配（中英文）
        "text='创建账号'",
        "text='Sign up'",
        "text='Create account'",
        
        // ARIA 标签
        "[aria-label*='Sign up']",
        "[aria-label*='创建']",
        
        // data-testid (如果存在)
        "[data-testid*='signup']",
        "[data-testid*='signupButton']",
        
        // XPath 文本匹配
        "//button[contains(text(), '创建账号')]",
        "//button[contains(text(), 'Sign up')]",
        "//span[contains(text(), '创建账号')]",
        "//div[@role='button'][contains(., 'Sign up')]",
    ];
    
    for selector in selectors {
        if let Ok(element) = page.find_element(selector).await {
            info!("找到创建账号按钮: {}", selector);
            element.click().await?;
            return Ok(());
        }
    }
    
    anyhow::bail!("未找到创建账号按钮");
}
```

### 2. 如果需要图像识别

添加以下依赖到 `Cargo.toml`：

```toml
[dependencies]
image = "0.25"
imageproc = "0.25"
```

创建新模块 `src/visual.rs`：

```rust
//! 视觉识别模块
//! Visual recognition module

use anyhow::Result;
use image::{DynamicImage, GenericImageView};
use imageproc::template_matching::{match_template, MatchTemplateMethod};

/// 在截图中查找按钮模板
pub fn find_button_by_template(screenshot: &[u8], template_path: &str) -> Result<Option<(u32, u32)>> {
    // 加载截图
    let page_img = image::load_from_memory(screenshot)?;
    
    // 加载模板
    let template_img = image::open(template_path)?;
    
    // 转换为灰度图（提高匹配效率）
    let page_gray = page_img.to_luma8();
    let template_gray = template_img.to_luma8();
    
    // 模板匹配
    let result = match_template(
        &page_gray,
        &template_gray,
        MatchTemplateMethod::CcoeffNormed,
    );
    
    // 找到最佳匹配位置
    let (max_val, max_loc) = find_max_value(&result);
    
    // 如果匹配度足够高（阈值 0.8）
    if max_val > 0.8 {
        let (template_width, template_height) = template_img.dimensions();
        let click_x = max_loc.0 + template_width / 2;
        let click_y = max_loc.1 + template_height / 2;
        Ok(Some((click_x, click_y)))
    } else {
        Ok(None)
    }
}

fn find_max_value(img: &image::GrayImage) -> (f32, (u32, u32)) {
    let mut max_val = f32::MIN;
    let mut max_loc = (0, 0);
    
    for (x, y, pixel) in img.enumerate_pixels() {
        let val = pixel[0] as f32;
        if val > max_val {
            max_val = val;
            max_loc = (x, y);
        }
    }
    
    (max_val / 255.0, max_loc)
}
```

## 总结

1. **优先使用稳定选择器**：基于文本、ARIA 属性、XPath 等
2. **备选图像识别**：当选择器不稳定时使用
3. **混合方案**：先尝试选择器，失败后使用图像识别
4. **定期更新**：监控选择器有效性，及时调整

## 下一步

建议先尝试增强选择器方案，如果确实无法定位元素，再考虑添加图像识别功能。
