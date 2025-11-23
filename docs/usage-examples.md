# 数据持久化和代理配置使用示例

## 示例 1: 查看缓存目录和日志目录

在不同操作系统上，数据会保存在不同的位置：

### Windows
```powershell
# 查看缓存目录
dir "$env:APPDATA\auto-x-account"

# 查看日志目录
dir "$env:APPDATA\auto-x-account\logs"

# 查看今天的日志
Get-Content "$env:APPDATA\auto-x-account\logs\auto-x-account_$(Get-Date -Format 'yyyyMMdd').log"
```

### macOS
```bash
# 查看缓存目录
ls -la ~/Library/Application\ Support/auto-x-account/

# 查看日志目录
ls -la ~/Library/Logs/auto-x-account/

# 实时查看日志
tail -f ~/Library/Logs/auto-x-account/auto-x-account_$(date +%Y%m%d).log
```

### Linux
```bash
# 查看缓存目录
ls -la ~/.local/share/auto-x-account/

# 查看日志目录
ls -la ~/.local/share/auto-x-account/logs/

# 实时查看日志
tail -f ~/.local/share/auto-x-account/logs/auto-x-account_$(date +%Y%m%d).log
```

## 示例 2: 配置代理分离

### 场景 A: 仅浏览器使用代理

适用于：Twitter 需要代理访问，但邮件服务器在本地

**config.json:**
```json
{
  "proxy": {
    "mode": "manual",
    "type": "socks5",
    "host": "127.0.0.1",
    "port": 1080,
    "browser_enabled": true,
    "email_enabled": false
  },
  "smtp": {
    "host": "localhost",
    "port": 25,
    "domain": "example.com",
    "enable": true
  }
}
```

### 场景 B: 两者都使用代理

适用于：完全隔离的网络环境

**config.json:**
```json
{
  "proxy": {
    "mode": "manual",
    "type": "socks5",
    "host": "proxy.example.com",
    "port": 1080,
    "username": "user",
    "password": "pass",
    "browser_enabled": true,
    "email_enabled": true
  }
}
```

### 场景 C: 使用系统代理

自动检测系统代理设置：

**config.json:**
```json
{
  "proxy": {
    "mode": "system",
    "browser_enabled": true,
    "email_enabled": false
  }
}
```

## 示例 3: 批量注册并自动保存

```bash
# 启动批量注册，数据会自动保存到缓存
./auto-x-account batch --count 10 --concurrent 3

# 查看保存的任务数据
cat ~/.local/share/auto-x-account/tasks.json

# 查看保存的账号数据
cat ~/.local/share/auto-x-account/accounts.json
```

## 示例 4: 程序重启后恢复数据

```bash
# 第一次运行
./auto-x-account batch --count 100 --concurrent 5

# 如果程序中断，重新启动会自动加载之前的数据
# 查看日志确认数据已加载
# [INFO] 已从缓存加载 5 个任务 / Loaded 5 tasks from cache
# [INFO] 已从缓存加载 23 个账号 / Loaded 23 accounts from cache

./auto-x-account batch --count 50 --concurrent 3
```

## 示例 5: 手动验证码模式

在非 headless 模式下，程序会等待你手动完成验证：

**config.json:**
```json
{
  "browser": {
    "headless": false,
    "timeout": 30000,
    "viewport": {
      "width": 1280,
      "height": 720
    },
    "user_data_dir": "browser_data"
  }
}
```

运行时会看到：
```
[INFO] 检测到人机验证类型: HCaptcha
[INFO] 请手动完成验证码...
[INFO] 请在浏览器中完成验证，完成后程序将自动继续
```

## 示例 6: 导出数据备份

```bash
# 导出为 JSON（包含完整信息）
./auto-x-account export --output backup_20241123.json --format json

# 导出为 CSV（表格格式）
./auto-x-account export --output accounts.csv --format csv

# 导出为纯文本（易读格式）
./auto-x-account export --output accounts.txt --format txt
```

## 示例 7: 日志分析

```bash
# 查看错误日志
grep "ERROR" ~/.local/share/auto-x-account/logs/*.log

# 查看警告信息
grep "WARN" ~/.local/share/auto-x-account/logs/*.log

# 统计成功注册的账号数量
grep "成功注册账号" ~/.local/share/auto-x-account/logs/*.log | wc -l

# 查看代理相关日志
grep "代理\|proxy\|Proxy" ~/.local/share/auto-x-account/logs/*.log
```

## 示例 8: 清理和维护

```bash
# 手动清理旧日志（保留最近7天）
find ~/.local/share/auto-x-account/logs -name "*.log" -mtime +7 -delete

# 备份缓存数据
cp -r ~/.local/share/auto-x-account ~/backup/auto-x-account_backup_$(date +%Y%m%d)

# 清空所有缓存重新开始
rm -rf ~/.local/share/auto-x-account/{tasks.json,accounts.json}
```

## 示例 9: 调试模式

启用详细日志输出：

```bash
# Linux/macOS
export RUST_LOG=debug
./auto-x-account batch --count 5 --concurrent 1

# Windows PowerShell
$env:RUST_LOG="debug"
.\auto-x-account.exe batch --count 5 --concurrent 1
```

## 示例 10: 多实例运行

使用不同的用户数据目录运行多个实例：

**Instance 1 - config1.json:**
```json
{
  "browser": {
    "user_data_dir": "browser_data_1"
  }
}
```

**Instance 2 - config2.json:**
```json
{
  "browser": {
    "user_data_dir": "browser_data_2"
  }
}
```

运行：
```bash
# 终端 1
./auto-x-account --config config1.json batch --count 10

# 终端 2
./auto-x-account --config config2.json batch --count 10
```

## 故障排查

### 问题 1: 找不到缓存目录

```bash
# 检查环境变量
echo $HOME          # Linux/macOS
echo $APPDATA       # Windows

# 手动创建
mkdir -p ~/.local/share/auto-x-account
```

### 问题 2: 日志文件权限问题

```bash
# 检查权限
ls -la ~/.local/share/auto-x-account/logs/

# 修复权限
chmod 755 ~/.local/share/auto-x-account/logs/
```

### 问题 3: 代理不生效

检查配置和日志：
```bash
# 查看配置
cat config.json | grep -A 10 "proxy"

# 查看启动日志中的代理信息
grep "代理\|proxy" ~/.local/share/auto-x-account/logs/auto-x-account_$(date +%Y%m%d).log | head
```

## 性能监控

```bash
# 监控磁盘使用
du -sh ~/.local/share/auto-x-account

# 监控日志文件大小
ls -lh ~/.local/share/auto-x-account/logs/

# 监控进程资源使用
ps aux | grep auto-x-account
```

## 高级技巧

### 自动化脚本

创建一个自动化脚本 `batch_register.sh`:

```bash
#!/bin/bash
# 自动批量注册脚本

# 设置日志级别
export RUST_LOG=info

# 记录开始时间
START_TIME=$(date +%s)

# 运行注册
./auto-x-account batch --count 100 --concurrent 5

# 计算耗时
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

echo "总耗时: $ELAPSED 秒"

# 备份结果
BACKUP_DIR=~/backups/auto-x-account_$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR
cp ~/.local/share/auto-x-account/*.json $BACKUP_DIR/

echo "数据已备份到: $BACKUP_DIR"
```

### 定时任务

使用 cron 定期运行（Linux/macOS）:

```bash
# 编辑 crontab
crontab -e

# 每天凌晨2点运行
0 2 * * * /path/to/auto-x-account batch --count 50 --concurrent 3

# 每天清理30天前的日志
0 3 * * * find ~/.local/share/auto-x-account/logs -name "*.log" -mtime +30 -delete
```

Windows 任务计划:

```powershell
# 创建任务计划
$action = New-ScheduledTaskAction -Execute "C:\path\to\auto-x-account.exe" -Argument "batch --count 50"
$trigger = New-ScheduledTaskTrigger -Daily -At 2am
Register-ScheduledTask -Action $action -Trigger $trigger -TaskName "AutoXAccount" -Description "自动注册任务"
```
