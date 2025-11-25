//! 视觉识别模块 - Visual Recognition Module
//! 使用图像模板匹配来定位页面元素

use anyhow::Result;
use image::{DynamicImage, GenericImageView};
use tracing::{info, warn};

/// 在截图中查找按钮模板
/// 
/// # 参数
/// - `screenshot`: 完整页面截图的字节数据
/// - `template_path`: 按钮模板图片的路径
/// - `threshold`: 匹配阈值（0.0-1.0），默认 0.75
/// 
/// # 返回
/// - `Some((x, y))`: 找到按钮，返回点击坐标（按钮中心点）
/// - `None`: 未找到匹配
pub fn find_button_by_template(
    screenshot: &[u8],
    template_path: &str,
    threshold: Option<f32>,
) -> Result<Option<(u32, u32)>> {
    let match_threshold = threshold.unwrap_or(0.7); // 降低阈值到 0.7，更宽松
    
    info!("🔍 开始图像识别 / Starting image recognition");
    info!("   模板路径 / Template path: {}", template_path);
    info!("   匹配阈值 / Match threshold: {}", match_threshold);
    
    // 1. 加载截图
    let page_img = image::load_from_memory(screenshot)
        .map_err(|e| anyhow::anyhow!("加载截图失败 / Failed to load screenshot: {}", e))?;
    info!("   ✅ 截图已加载: {}x{} / Screenshot loaded: {}x{}", 
          page_img.width(), page_img.height(), page_img.width(), page_img.height());
    
    // 2. 加载模板
    let template_img = image::open(template_path)
        .map_err(|e| anyhow::anyhow!("加载模板失败 / Failed to load template: {}", e))?;
    info!("   ✅ 模板已加载: {}x{} / Template loaded: {}x{}", 
          template_img.width(), template_img.height(), template_img.width(), template_img.height());
    
    // 3. 简单的模板匹配（滑动窗口）
    info!("   🔎 执行模板匹配 / Performing template matching...");
    let (best_match, best_loc) = simple_template_match(&page_img, &template_img);
    
    info!("   匹配值 / Match value: {:.4} (阈值 / threshold: {})", best_match, match_threshold);
    
    // 4. 检查匹配度是否足够高
    if best_match >= match_threshold {
        let (template_width, template_height) = template_img.dimensions();
        // 计算按钮中心点坐标
        let click_x = best_loc.0 + template_width / 2;
        let click_y = best_loc.1 + template_height / 2;
        
        info!("   ✅ 找到匹配! / Match found!");
        info!("   位置 / Location: ({}, {})", best_loc.0, best_loc.1);
        info!("   点击坐标 / Click coordinates: ({}, {})", click_x, click_y);
        
        // 保存带标记的调试图片
        save_debug_image_with_marker(&page_img, best_loc, (template_width, template_height), screenshot)?;
        
        Ok(Some((click_x, click_y)))
    } else {
        warn!("   ❌ 未找到匹配 / No match found (value: {:.4} < threshold: {})", 
              best_match, match_threshold);
        Ok(None)
    }
}

/// 保存带标记的调试图片
fn save_debug_image_with_marker(
    page_img: &DynamicImage,
    match_loc: (u32, u32),
    template_size: (u32, u32),
    screenshot_bytes: &[u8],
) -> Result<()> {
    use image::{Rgba, RgbaImage};
    
    // 加载原始截图
    let mut marked_img = page_img.to_rgba8();
    
    // 在匹配位置画一个红色矩形框
    let (x, y) = match_loc;
    let (w, h) = template_size;
    let red = Rgba([255u8, 0u8, 0u8, 255u8]);
    
    // 画矩形边框（4像素宽）
    for i in 0..4 {
        // 上边
        for dx in 0..w {
            if let Some(pixel) = marked_img.get_pixel_mut_checked(x + dx, y + i) {
                *pixel = red;
            }
        }
        // 下边
        for dx in 0..w {
            if let Some(pixel) = marked_img.get_pixel_mut_checked(x + dx, y + h - i - 1) {
                *pixel = red;
            }
        }
        // 左边
        for dy in 0..h {
            if let Some(pixel) = marked_img.get_pixel_mut_checked(x + i, y + dy) {
                *pixel = red;
            }
        }
        // 右边
        for dy in 0..h {
            if let Some(pixel) = marked_img.get_pixel_mut_checked(x + w - i - 1, y + dy) {
                *pixel = red;
            }
        }
    }
    
    // 保存到 AppData
    // 这里需要访问 data_dir，但这个函数在 visual 模块中，所以先不实现
    // 调用者会自己保存调试图片
    
    Ok(())
}

/// 简单的模板匹配算法（归一化互相关）
fn simple_template_match(haystack: &DynamicImage, needle: &DynamicImage) -> (f32, (u32, u32)) {
    let haystack_rgb = haystack.to_rgb8();
    let needle_rgb = needle.to_rgb8();
    
    let (h_width, h_height) = haystack_rgb.dimensions();
    let (n_width, n_height) = needle_rgb.dimensions();
    
    let mut best_score = 0.0f32;
    let mut best_loc = (0u32, 0u32);
    
    // 滑动窗口遍历
    for y in 0..=(h_height.saturating_sub(n_height)) {
        for x in 0..=(h_width.saturating_sub(n_width)) {
            let score = calculate_similarity(&haystack_rgb, &needle_rgb, x, y);
            if score > best_score {
                best_score = score;
                best_loc = (x, y);
            }
        }
    }
    
    (best_score, best_loc)
}

/// 计算两个图像区域的相似度
fn calculate_similarity(
    haystack: &image::RgbImage,
    needle: &image::RgbImage,
    offset_x: u32,
    offset_y: u32,
) -> f32 {
    let (n_width, n_height) = needle.dimensions();
    let mut sum = 0.0f32;
    let mut count = 0u32;
    
    for y in 0..n_height {
        for x in 0..n_width {
            let h_pixel = haystack.get_pixel(offset_x + x, offset_y + y);
            let n_pixel = needle.get_pixel(x, y);
            
            // 计算RGB差异
            let diff_r = (h_pixel[0] as i32 - n_pixel[0] as i32).abs() as f32;
            let diff_g = (h_pixel[1] as i32 - n_pixel[1] as i32).abs() as f32;
            let diff_b = (h_pixel[2] as i32 - n_pixel[2] as i32).abs() as f32;
            
            let pixel_diff = (diff_r + diff_g + diff_b) / 3.0;
            // 转换为相似度（0-1，1表示完全相同）
            sum += 1.0 - (pixel_diff / 255.0);
            count += 1;
        }
    }
    
    sum / count as f32
}

/// 通过坐标点击元素（使用 JavaScript）
pub async fn click_at_coordinates(page: &chromiumoxide::Page, x: u32, y: u32) -> Result<()> {
    info!("🖱️  点击坐标 / Clicking at: ({}, {})", x, y);
    
    // 使用 JavaScript 在指定坐标点击
    let script = format!(
        r#"(function() {{
            const element = document.elementFromPoint({}, {});
            if (element) {{ element.click(); }}
        }})()"#,
        x, y
    );
    
    page.evaluate(script.as_str()).await
        .map_err(|e| anyhow::anyhow!("点击失败 / Click failed: {}", e))?;
    
    info!("   ✅ 点击成功 / Click succeeded");
    Ok(())
}
