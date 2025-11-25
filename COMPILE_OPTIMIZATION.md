# 编译优化说明

## 问题分析

1. **编译慢的原因**：
   - 依赖太多（chromiumoxide、egui 等都是大型库）
   - 每次都重新编译所有依赖
   
2. **解决方案**：

### 方案 1：使用 sccache（推荐）

安装编译缓存工具，大幅加速重复编译：

```powershell
# 安装 sccache
cargo install sccache

# 配置环境变量（PowerShell）
$env:RUSTC_WRAPPER = "sccache"

# 或永久添加到系统环境变量
[Environment]::SetEnvironmentVariable("RUSTC_WRAPPER", "sccache", "User")
```

### 方案 2：减少依赖编译

修改 `Cargo.toml`，禁用不需要的 features：

```toml
[dependencies]
# 只编译需要的 features
chromiumoxide = { version = "0.6", features = ["tokio-runtime"], default-features = false }
egui = { version = "0.25", default-features = false }
```

### 方案 3：使用 mold 链接器（Linux）

```bash
# 安装 mold
sudo apt install mold

# 配置
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

### 方案 4：并行编译

在 `.cargo/config.toml` 中已经配置了增量编译，确保使用：

```toml
[build]
incremental = true
```

## 实际效果

- **首次编译**：3-5 分钟（无法避免）
- **增量编译**（修改代码后）：10-30 秒
- **使用 sccache 后**：第二次完整编译只需 30-60 秒

## 当前状态

你的项目已经配置了增量编译，但首次编译依然需要时间。
建议安装 sccache 来加速后续编译。
