# X 注册功能修复需求文档
# X Registration Fixes Required

## 更新日期 / Update Date
2025-11-23

## 问题总结 / Issue Summary

### 1. 验证方式错误 ❌
**发现**: X (Twitter) 使用的人机验证**不是 ReCAPTCHA**

**当前代码假设**:
- `src/recaptcha_solver.rs` - 假设使用 ReCAPTCHA
- `src/captcha.rs` 文档中提到 hCaptcha、reCAPTCHA、Arkose Labs

**实际情况**:
- 用户测试时遇到: "We were unable to confirm you're human. Please try again."
- 错误代码: `g;176391287258186801:-1763913078926:nUBBMrtY5dimutoR6wvfAmiK:0`
- 需要实际测试确认当前 X 使用的验证方式

**可能的验证方式**:
1. **Arkose Labs (FunCaptcha)** - X 近期主要使用
2. **hCaptcha** - 可能在某些情况下使用
3. **自定义验证** - X 可能使用自己的验证系统

### 2. 默认注册方式错误 ❌
**发现**: X 注册页面默认使用**手机号**，不是邮箱

**当前代码问题**:
- 直接访问注册页面后，没有切换到邮箱模式
- 缺少"改用电子邮箱"按钮点击逻辑

**需要的步骤**:
1. 访问注册页面
2. **点击"改用电子邮箱"链接/按钮**（新增）
3. 填写邮箱和其他信息

### 3. 表单填写逻辑缺失 ❌
**发现**: `src/registration.rs:276-277` 只有 TODO 注释

```rust
info!("填写注册信息...");
// TODO: 实现具体的表单填写逻辑
```

**缺少的实现**:
- 元素定位（CSS选择器或XPath）
- 输入框填写逻辑
- 按钮点击逻辑
- 多步骤流程导航

## 详细修复方案 / Detailed Fix Plan

### 修复 1: 确定并支持正确的验证方式

#### 步骤 1: 实际测试确认验证类型

**测试方法**:
```bash
# 运行带调试的测试
./auto-x-account register \
  --email test@example.com \
  --config config.json

# 在浏览器中观察：
# 1. 验证码出现时的 DOM 结构
# 2. 网络请求（DevTools -> Network）
# 3. iframe 或 script 标签来源
```

**识别特征**:

**Arkose Labs (FunCaptcha)**:
```html
<!-- Arkose Labs 特征 -->
<iframe src="https://client-api.arkoselabs.com/...">
<div id="arkose-container">
<script src="https://client-api.arkoselabs.com/...">
```

**hCaptcha**:
```html
<!-- hCaptcha 特征 -->
<iframe src="https://hcaptcha.com/captcha/...">
<div class="h-captcha">
<script src="https://hcaptcha.com/1/api.js">
```

**reCAPTCHA**:
```html
<!-- reCAPTCHA 特征（已知不是这个）-->
<iframe src="https://www.google.com/recaptcha/...">
<div class="g-recaptcha">
```

#### 步骤 2: 创建对应的验证处理器

**如果是 Arkose Labs**:
```rust
// 新建文件: src/arkose_solver.rs
pub struct ArkoseSolver {
    // Arkose Labs API 配置
    pub_key: String,
    api_url: String,
}

impl ArkoseSolver {
    /// 检测 Arkose Labs 验证码
    pub async fn detect_arkose(page: &Page) -> Result<bool> {
        let exists = page.evaluate(r#"
            document.querySelector('iframe[src*="arkoselabs.com"]') !== null ||
            document.querySelector('#arkose-container') !== null
        "#).await?;
        Ok(exists.into_value()?)
    }

    /// 获取 Arkose 公钥
    pub async fn get_public_key(page: &Page) -> Result<String> {
        // 从页面中提取公钥
        let script = page.evaluate(r#"
            let iframe = document.querySelector('iframe[src*="arkoselabs.com"]');
            if (iframe) {
                let src = iframe.src;
                let match = src.match(/pk=([^&]+)/);
                return match ? match[1] : null;
            }
            return null;
        "#).await?;
        
        script.into_value()
    }

    /// 手动模式：等待用户完成 Arkose 验证
    pub async fn wait_for_manual_solve(page: &Page, timeout: Duration) -> Result<()> {
        info!("⚠️  检测到 Arkose Labs 验证码");
        info!("📌 请手动完成验证，程序将在验证完成后继续...");
        
        // 等待验证完成的标志
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            // 检查验证码是否完成
            let solved = page.evaluate(r#"
                document.querySelector('#arkose-container') === null ||
                document.querySelector('.arkose-solved') !== null
            "#).await?;
            
            if solved.into_value::<bool>()? {
                info!("✅ Arkose 验证完成！");
                return Ok(());
            }
            
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        
        anyhow::bail!("Arkose 验证超时")
    }
}
```

**如果是 hCaptcha**:
```rust
// 新建文件: src/hcaptcha_solver.rs
pub struct HCaptchaSolver {
    // 类似 Arkose 的实现
}
```

### 修复 2: 添加邮箱模式切换逻辑

**实现位置**: `src/registration.rs` 的 `perform_registration()` 方法

**修改前**:
```rust
async fn perform_registration(...) -> Result<AccountInfo> {
    // 访问注册页面
    info!("访问注册页面: {}", self.config.x_account.base_url);
    page.goto(&self.config.x_account.base_url).await?;
    sleep(Duration::from_secs(3)).await;

    info!("填写注册信息...");
    // TODO: 实现具体的表单填写逻辑
    ...
}
```

**修改后**:
```rust
async fn perform_registration(...) -> Result<AccountInfo> {
    // 1. 访问注册页面
    info!("访问注册页面: {}", self.config.x_account.base_url);
    page.goto(&self.config.x_account.base_url).await?;
    sleep(Duration::from_secs(3)).await;
    self.take_screenshot(page, "01_initial_page").await?;

    // 2. 切换到邮箱模式（新增）
    info!("切换到邮箱注册模式...");
    self.switch_to_email_mode(page).await?;
    sleep(Duration::from_secs(2)).await;
    self.take_screenshot(page, "02_email_mode").await?;

    // 3. 填写注册信息
    info!("填写注册信息...");
    self.fill_registration_form(page, account_info).await?;
    
    ...
}

/// 切换到邮箱注册模式（新增方法）
async fn switch_to_email_mode(&self, page: &Page) -> Result<()> {
    // 方法 1: 尝试常见的选择器
    let selectors = vec![
        "span:has-text('改用电子邮箱')",                    // 中文
        "span:has-text('Use email instead')",             // 英文
        "a[href*='email']",                               // 包含email的链接
        "button:has-text('Email')",                       // Email按钮
        "[data-testid='signupWithEmailLink']",            // 测试ID（需实际确认）
    ];

    for selector in selectors {
        info!("尝试选择器: {}", selector);
        
        // 检查元素是否存在
        let exists = page.evaluate(&format!(r#"
            document.querySelector('{}') !== null
        "#, selector)).await?;
        
        if exists.into_value::<bool>()? {
            info!("✓ 找到邮箱切换按钮: {}", selector);
            
            // 点击按钮
            page.evaluate(&format!(r#"
                document.querySelector('{}').click()
            "#, selector)).await?;
            
            info!("✓ 已切换到邮箱模式");
            return Ok(());
        }
    }

    // 方法 2: 如果上面都不行，尝试通过文本查找
    let clicked = page.evaluate(r#"
        function clickEmailLink() {
            // 查找所有 span 和 a 标签
            let elements = [...document.querySelectorAll('span, a, button')];
            
            for (let el of elements) {
                let text = el.textContent.trim();
                // 匹配中英文
                if (text.includes('邮箱') || text.includes('email') || 
                    text.includes('Email') || text.includes('邮件')) {
                    el.click();
                    return true;
                }
            }
            return false;
        }
        clickEmailLink();
    "#).await?;

    if clicked.into_value::<bool>()? {
        info!("✓ 通过文本查找成功切换到邮箱模式");
        return Ok(());
    }

    // 如果都失败了，提供截图和错误信息
    self.take_screenshot(page, "ERROR_cannot_find_email_switch").await?;
    anyhow::bail!("❌ 无法找到邮箱切换按钮，请检查页面结构是否变化")
}
```

### 修复 3: 实现完整的表单填写逻辑

**新增方法**: `fill_registration_form()`

```rust
/// 填写注册表单（新增方法）
async fn fill_registration_form(
    &self,
    page: &Page,
    account_info: &AccountInfo,
) -> Result<()> {
    info!("开始填写注册表单...");

    // 步骤 1: 填写姓名
    info!("填写姓名: {}", account_info.name);
    self.fill_input(page, "name", &account_info.name).await?;
    sleep(Duration::from_millis(500)).await;

    // 步骤 2: 填写邮箱
    info!("填写邮箱: {}", account_info.email);
    self.fill_input(page, "email", &account_info.email).await?;
    sleep(Duration::from_millis(500)).await;

    // 步骤 3: 选择生日
    info!("选择生日: {}/{}/{}", 
        account_info.birth_date.month,
        account_info.birth_date.day,
        account_info.birth_date.year
    );
    self.select_birth_date(page, &account_info.birth_date).await?;
    sleep(Duration::from_millis(500)).await;

    // 截图确认
    self.take_screenshot(page, "03_form_filled").await?;

    // 步骤 4: 点击"下一步"按钮
    info!("点击下一步...");
    self.click_next_button(page).await?;
    sleep(Duration::from_secs(2)).await;
    
    self.take_screenshot(page, "04_after_next").await?;

    // 步骤 5: 可能需要确认
    if self.is_confirmation_page(page).await? {
        info!("进入确认页面，点击确认...");
        self.click_confirm_button(page).await?;
        sleep(Duration::from_secs(2)).await;
        self.take_screenshot(page, "05_after_confirm").await?;
    }

    // 步骤 6: 处理验证码
    info!("检查是否有验证码...");
    if self.has_captcha(page).await? {
        info!("⚠️  检测到验证码，等待手动完成...");
        self.handle_captcha(page).await?;
        self.take_screenshot(page, "06_captcha_solved").await?;
    }

    // 步骤 7: 继续到验证码页面
    info!("继续到验证码页面...");
    self.click_next_button(page).await?;
    sleep(Duration::from_secs(3)).await;
    self.take_screenshot(page, "07_verification_code_page").await?;

    Ok(())
}

/// 填写输入框的辅助方法
async fn fill_input(&self, page: &Page, field_type: &str, value: &str) -> Result<()> {
    // 根据字段类型确定选择器
    let selectors = match field_type {
        "name" => vec![
            "input[name='name']",
            "input[autocomplete='name']",
            "input[placeholder*='名']",
            "input[placeholder*='Name']",
        ],
        "email" => vec![
            "input[name='email']",
            "input[type='email']",
            "input[autocomplete='email']",
            "input[placeholder*='邮箱']",
            "input[placeholder*='Email']",
        ],
        _ => vec![],
    };

    for selector in selectors {
        let exists = page.evaluate(&format!(r#"
            document.querySelector('{}') !== null
        "#, selector)).await?;

        if exists.into_value::<bool>()? {
            // 清空输入框
            page.evaluate(&format!(r#"
                let input = document.querySelector('{}');
                input.value = '';
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
            "#, selector)).await?;

            // 逐字输入（模拟人工输入）
            for ch in value.chars() {
                page.evaluate(&format!(r#"
                    let input = document.querySelector('{}');
                    input.value += '{}';
                    input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                "#, selector, ch)).await?;
                
                // 随机延迟 50-150ms
                let delay = 50 + (rand::random::<u64>() % 100);
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }

            info!("✓ 成功填写 {}: {}", field_type, value);
            return Ok(());
        }
    }

    anyhow::bail!("无法找到 {} 输入框", field_type)
}

/// 选择生日
async fn select_birth_date(&self, page: &Page, birth_date: &BirthDate) -> Result<()> {
    // 月份
    self.select_dropdown(page, "month", &birth_date.month).await?;
    sleep(Duration::from_millis(300)).await;

    // 日期
    self.select_dropdown(page, "day", &birth_date.day).await?;
    sleep(Duration::from_millis(300)).await;

    // 年份
    self.select_dropdown(page, "year", &birth_date.year).await?;

    Ok(())
}

/// 选择下拉框
async fn select_dropdown(&self, page: &Page, field: &str, value: &str) -> Result<()> {
    // 实现下拉框选择逻辑
    // 根据实际页面结构调整
    let script = format!(r#"
        let select = document.querySelector('select[name="{}"]') ||
                     document.querySelector('select[id*="{}"]');
        if (select) {{
            select.value = '{}';
            select.dispatchEvent(new Event('change', {{ bubbles: true }}));
            return true;
        }}
        return false;
    "#, field, field, value);

    let success = page.evaluate(&script).await?;
    if success.into_value::<bool>()? {
        info!("✓ 成功选择 {}: {}", field, value);
        Ok(())
    } else {
        anyhow::bail!("无法选择 {}", field)
    }
}

/// 点击下一步按钮
async fn click_next_button(&self, page: &Page) -> Result<()> {
    let selectors = vec![
        "button:has-text('下一步')",
        "button:has-text('Next')",
        "div[role='button']:has-text('Next')",
        "[data-testid='nextButton']",
    ];

    for selector in selectors {
        let exists = page.evaluate(&format!(r#"
            document.querySelector('{}') !== null
        "#, selector)).await?;

        if exists.into_value::<bool>()? {
            page.evaluate(&format!(r#"
                document.querySelector('{}').click()
            "#, selector)).await?;
            
            info!("✓ 点击下一步按钮");
            return Ok(());
        }
    }

    anyhow::bail!("无法找到下一步按钮")
}
```

## 实现优先级 / Implementation Priority

### 🔴 高优先级（必须实现）

1. **确定验证类型** - 阻塞性问题
   - [ ] 实际测试 X 注册流程
   - [ ] 确认使用的验证方式（Arkose Labs / hCaptcha / 其他）
   - [ ] 截图保存验证码 DOM 结构
   - [ ] 记录网络请求

2. **添加邮箱切换逻辑** - 阻塞性问题
   - [ ] 实现 `switch_to_email_mode()` 方法
   - [ ] 测试不同语言环境
   - [ ] 添加错误处理

3. **实现基本表单填写** - 阻塞性问题
   - [ ] 实现 `fill_input()` 方法
   - [ ] 实现 `select_birth_date()` 方法
   - [ ] 实现 `click_next_button()` 方法

### 🟡 中优先级（重要）

4. **验证码处理**
   - [ ] 根据确定的类型实现对应solver
   - [ ] 手动模式完善
   - [ ] 添加超时处理

5. **验证码输入**
   - [ ] 定位验证码输入框
   - [ ] 实现输入逻辑
   - [ ] 提交验证码

### 🟢 低优先级（优化）

6. **错误处理和重试**
7. **详细日志**
8. **性能优化**

## 测试计划 / Testing Plan

### 测试环境准备

```bash
# 1. 准备测试邮箱
./auto-x-account create-emails --count 5 --output test_emails.json

# 2. 配置测试环境
cp config.example.json config.test.json
# 编辑 config.test.json:
# - mode = "test"
# - browser.headless = false  # 重要：观察流程
# - browser.timeout = 60000
```

### 测试步骤

#### 测试 1: 验证类型确认
```bash
# 运行并观察
./auto-x-account register --email test@example.com

# 在浏览器中：
# 1. 打开 DevTools (F12)
# 2. 切换到 Network 标签
# 3. 观察验证码加载时的请求
# 4. 查看 Elements 标签中的 iframe 来源
# 5. 截图保存
```

#### 测试 2: 邮箱切换
```bash
# 添加调试日志后运行
./auto-x-account register --email test@example.com

# 检查：
# - 是否找到"改用电子邮箱"按钮
# - 是否成功切换
# - 切换后页面是否正确
```

#### 测试 3: 表单填写
```bash
# 完整流程测试
./auto-x-account register --email test@example.com

# 观察：
# - 每个字段是否正确填写
# - 生日选择是否正确
# - 按钮点击是否生效
```

## 参考资料 / References

### Arkose Labs (FunCaptcha) 识别特征
- URL: `https://client-api.arkoselabs.com/`
- 元素: `<div id="arkose">`, `<iframe src="...arkoselabs...">` 
- 特点: 3D 旋转图片验证

### hCaptcha 识别特征
- URL: `https://hcaptcha.com/`
- 元素: `<div class="h-captcha">`, `<iframe title="hCaptcha">` 
- 特点: 图片选择验证

### X 注册流程文档
- 官方文档: https://help.twitter.com/
- 注册 URL: https://twitter.com/i/flow/signup

## 下一步行动 / Next Actions

1. **立即执行**: 测试确认验证类型
   ```bash
   # 使用现有代码测试一次
   ./auto-x-account register \
     --email testuser@iwatermail.com \
     --config config.test.json
   
   # 观察并记录：
   # - 错误信息
   # - 验证码类型
   # - 页面截图
   ```

2. **根据测试结果**: 实现对应的验证处理器

3. **实现邮箱切换**: 添加 `switch_to_email_mode()` 方法

4. **逐步实现表单填写**: 从最简单的开始

5. **迭代测试**: 每完成一个功能就测试

## 总结 / Summary

当前代码存在三个关键问题：
1. ❌ 验证方式假设错误（不是 ReCAPTCHA）
2. ❌ 缺少邮箱模式切换
3. ❌ 表单填写逻辑为空

需要按优先级逐个修复，建议从测试确认验证类型开始，然后实现邮箱切换和表单填写。

---

**文档状态**: 待验证和实现
**最后更新**: 2025-11-23
**负责人**: 需要实际测试后确认实现方案
