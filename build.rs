use std::path::Path;

fn main() {
    // 检查 MiSans 字体文件是否存在
    // Check if MiSans font file exists
    let misans_path = Path::new("fonts/MiSans/ttf/MiSans-Regular.ttf");
    
    if misans_path.exists() {
        println!("cargo:rustc-cfg=font_misans");
        println!("cargo:warning=Using MiSans font");
    } else {
        println!("cargo:warning=MiSans font not found, using fallback font (DejaVu Sans)");
    }
    
    // 确保在字体文件改变时重新构建
    // Rebuild if font files change
    println!("cargo:rerun-if-changed=fonts/MiSans/ttf/MiSans-Regular.ttf");
    println!("cargo:rerun-if-changed=fonts/fallback.ttf");
}
