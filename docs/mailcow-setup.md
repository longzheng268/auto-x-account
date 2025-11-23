# MailCow Dockerized 部署指南

本文档提供 MailCow Dockerized 邮件服务器的完整部署方案，用于配合 Auto X Account 项目的自定义邮箱模式。

## 📋 目录

- [为什么选择 MailCow](#为什么选择-mailcow)
- [系统要求](#系统要求)
- [安装步骤](#安装步骤)
- [配置 MailCow](#配置-mailcow)
- [批量创建邮箱](#批量创建邮箱)
- [与 Auto X Account 集成](#与-auto-x-account-集成)
- [维护和备份](#维护和备份)
- [故障排查](#故障排查)

## 🎯 为什么选择 MailCow

✅ **Docker 一键部署** - 最简单的自建邮件服务器方案  
✅ **完整的 Web 管理界面** - 可以方便地批量创建邮箱  
✅ **支持 SMTP/IMAP** - 与项目的 Custom 模式完美兼容  
✅ **稳定性好** - 生产环境可用  
✅ **开源且活跃维护** - 社区支持强大  
✅ **内置反垃圾邮件** - Rspamd、ClamAV 等安全组件  
✅ **支持多域名** - 可以管理多个邮件域名  

## 💻 系统要求

### 硬件要求
- **CPU**: 2 核心或以上（推荐 4 核心）
- **内存**: 最低 6 GB（推荐 8 GB 或以上）
- **存储**: 最低 20 GB（推荐 50 GB SSD）
- **网络**: 公网 IP，开放端口 25、80、110、143、443、465、587、993、995

### 软件要求
- **操作系统**: 
  - Ubuntu 20.04 LTS / 22.04 LTS（推荐）
  - Debian 11 / 12
  - CentOS 8+ / Rocky Linux 8+
- **Docker**: 20.10 或以上
- **Docker Compose**: 2.0 或以上
- **域名**: 一个可用的域名，并配置好 DNS 记录

### DNS 配置要求
在开始安装前，请确保以下 DNS 记录已正确配置：

```
# A 记录
mail.yourdomain.com    IN A    YOUR_SERVER_IP
autodiscover.yourdomain.com    IN A    YOUR_SERVER_IP

# MX 记录
yourdomain.com    IN MX 10    mail.yourdomain.com

# SPF 记录
yourdomain.com    IN TXT    "v=spf1 mx ~all"

# DMARC 记录
_dmarc.yourdomain.com    IN TXT    "v=DMARC1; p=quarantine; rua=mailto:postmaster@yourdomain.com"

# PTR 记录（反向 DNS，需要在主机商处配置）
YOUR_SERVER_IP    IN PTR    mail.yourdomain.com
```

## 🚀 安装步骤

### 1. 准备服务器

```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装必要工具
sudo apt install -y curl git

# 设置主机名（重要！）
sudo hostnamectl set-hostname mail.yourdomain.com
echo "127.0.0.1 mail.yourdomain.com" | sudo tee -a /etc/hosts
```

### 2. 安装 Docker 和 Docker Compose

```bash
# 安装 Docker
curl -fsSL https://get.docker.com | sh

# 启动并启用 Docker
sudo systemctl enable docker
sudo systemctl start docker

# 将当前用户添加到 docker 组
sudo usermod -aG docker $USER

# 重新登录或执行以下命令应用组变更
newgrp docker

# 验证安装
docker --version
docker compose version
```

### 3. 下载 MailCow

```bash
# 创建安装目录
cd /opt
sudo git clone https://github.com/mailcow/mailcow-dockerized
cd mailcow-dockerized

# 生成配置文件
sudo ./generate_config.sh
```

在执行 `generate_config.sh` 时，会提示输入：
- **邮件服务器主机名**: 输入 `mail.yourdomain.com`
- **时区**: 输入 `Asia/Shanghai`（中国大陆）

### 4. 配置 MailCow

编辑 `mailcow.conf` 文件：

```bash
sudo nano mailcow.conf
```

重要配置项：

```bash
# 邮件服务器主机名
MAILCOW_HOSTNAME=mail.yourdomain.com

# 时区
TZ=Asia/Shanghai

# HTTP 绑定地址（如果使用反向代理，设置为 127.0.0.1）
HTTP_BIND=0.0.0.0

# HTTPS 绑定地址
HTTPS_BIND=0.0.0.0

# HTTPS 端口（如果 443 被占用，可以改为其他端口）
HTTPS_PORT=443

# HTTP 端口（用于 Let's Encrypt 验证）
HTTP_PORT=80

# 数据库 root 密码（请修改为强密码）
DBROOT=YOUR_STRONG_PASSWORD_HERE

# 数据库密码（请修改为强密码）
DBPASS=YOUR_STRONG_PASSWORD_HERE

# 启用 Let's Encrypt SSL 证书
SKIP_LETS_ENCRYPT=n

# Let's Encrypt 邮箱
ACME_CONTACT=admin@yourdomain.com

# 额外的 SAN 域名（可选）
ADDITIONAL_SAN=autodiscover.yourdomain.com,autoconfig.yourdomain.com

# 启用 IPv6（如果服务器支持）
ENABLE_IPV6=y

# 邮箱配额（MB，0 表示无限制）
MAILBOX_QUOTA=10240

# API 密钥（用于 API 访问，请修改）
API_KEY=YOUR_RANDOM_API_KEY_HERE

# 允许通过 API 创建邮箱
API_ALLOW_FROM=127.0.0.1,YOUR_LOCAL_IP
```

### 5. 启动 MailCow

```bash
# 拉取镜像（首次运行会较慢）
sudo docker compose pull

# 启动所有服务
sudo docker compose up -d

# 查看日志
sudo docker compose logs -f
```

等待所有容器启动完成（大约 5-10 分钟），然后访问：
- **管理界面**: https://mail.yourdomain.com
- **默认管理员账号**: `admin`
- **默认密码**: `moohoo`

**⚠️ 重要：首次登录后立即修改管理员密码！**

### 6. 防火墙配置

```bash
# Ubuntu/Debian (UFW)
sudo ufw allow 25/tcp    # SMTP
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 110/tcp   # POP3
sudo ufw allow 143/tcp   # IMAP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 465/tcp   # SMTPS
sudo ufw allow 587/tcp   # Submission
sudo ufw allow 993/tcp   # IMAPS
sudo ufw allow 995/tcp   # POP3S
sudo ufw reload

# CentOS/RHEL (Firewalld)
sudo firewall-cmd --permanent --add-service=smtp
sudo firewall-cmd --permanent --add-service=http
sudo firewall-cmd --permanent --add-service=https
sudo firewall-cmd --permanent --add-port=110/tcp
sudo firewall-cmd --permanent --add-port=143/tcp
sudo firewall-cmd --permanent --add-port=465/tcp
sudo firewall-cmd --permanent --add-port=587/tcp
sudo firewall-cmd --permanent --add-port=993/tcp
sudo firewall-cmd --permanent --add-port=995/tcp
sudo firewall-cmd --reload
```

## ⚙️ 配置 MailCow

### 1. 首次登录和安全设置

1. 访问 https://mail.yourdomain.com
2. 使用默认账号登录（admin / moohoo）
3. 立即修改管理员密码：
   - 点击右上角用户图标
   - 选择 "Edit administrator details"
   - 修改密码并启用两步验证（推荐）

### 2. 添加域名

1. 进入 **Configuration → Mail setup → Domains**
2. 点击 **Add domain**
3. 输入域名：`yourdomain.com`
4. 设置配额：`10240` MB（或根据需要调整）
5. 点击 **Add domain and restart SOGo**

### 3. 配置 DKIM

DKIM 可以提高邮件送达率：

1. 进入 **Configuration → Configuration & Details → Configuration**
2. 找到刚添加的域名
3. 点击 **DKIM** 选项卡
4. 点击 **Generate DKIM key**
5. 复制 DKIM 公钥并添加到 DNS 记录：

```
dkim._domainkey.yourdomain.com    IN TXT    "v=DKIM1; k=rsa; p=YOUR_PUBLIC_KEY_HERE"
```

6. 验证 DKIM：
```bash
# 安装 dig 工具
sudo apt install dnsutils -y

# 查询 DKIM 记录
dig TXT dkim._domainkey.yourdomain.com +short
```

### 4. 创建邮箱

#### 单个创建

1. 进入 **Configuration → Mail setup → Mailboxes**
2. 点击 **Add mailbox**
3. 填写信息：
   - **Username**: 邮箱用户名（例如：user1）
   - **Domain**: 选择域名
   - **Full name**: 显示名称
   - **Quota**: 邮箱配额（MB）
   - **Password**: 设置密码
4. 点击 **Add**

## 📧 批量创建邮箱

### 方法 1：使用 MailCow API（推荐）

创建一个 Python 脚本来批量创建邮箱：

```python
#!/usr/bin/env python3
"""
MailCow 批量邮箱创建脚本
"""
import requests
import json
import random
import string

# MailCow 配置
MAILCOW_URL = "https://mail.yourdomain.com"
API_KEY = "YOUR_API_KEY_HERE"  # 从 mailcow.conf 中获取
DOMAIN = "yourdomain.com"

# 生成随机密码
def generate_password(length=16):
    chars = string.ascii_letters + string.digits + "!@#$%^&*"
    return ''.join(random.choice(chars) for _ in range(length))

# 创建邮箱
def create_mailbox(username, password, quota=10240):
    url = f"{MAILCOW_URL}/api/v1/add/mailbox"
    headers = {
        "X-API-Key": API_KEY,
        "Content-Type": "application/json"
    }
    data = {
        "local_part": username,
        "domain": DOMAIN,
        "name": username,
        "quota": quota,
        "password": password,
        "password2": password,
        "active": "1"
    }
    
    response = requests.post(url, headers=headers, json=data, verify=True)
    return response.json()

# 批量创建
def batch_create(count=10):
    results = []
    for i in range(1, count + 1):
        username = f"user{i}"
        password = generate_password()
        
        print(f"创建邮箱 {i}/{count}: {username}@{DOMAIN}")
        result = create_mailbox(username, password)
        
        if result and result[0].get("type") == "success":
            print(f"✓ 成功: {username}@{DOMAIN}")
            results.append({
                "email": f"{username}@{DOMAIN}",
                "password": password
            })
        else:
            print(f"✗ 失败: {result}")
        
    # 保存到文件
    with open("mailboxes.json", "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2, ensure_ascii=False)
    
    print(f"\n完成！已创建 {len(results)} 个邮箱")
    print("邮箱信息已保存到 mailboxes.json")

if __name__ == "__main__":
    # 创建 50 个邮箱
    batch_create(50)
```

运行脚本：

```bash
# 安装依赖
pip3 install requests

# 运行脚本
python3 mailcow_batch_create.py
```

### 方法 2：使用 Web 界面手动创建

适合少量邮箱的创建，步骤见上文"创建邮箱"部分。

### 方法 3：使用 CSV 导入（推荐大量创建）

1. 准备 CSV 文件 `mailboxes.csv`：

```csv
username,domain,name,quota,password
user1,yourdomain.com,User 1,10240,SecurePass1!
user2,yourdomain.com,User 2,10240,SecurePass2!
user3,yourdomain.com,User 3,10240,SecurePass3!
```

2. 创建导入脚本 `import_mailboxes.sh`：

```bash
#!/bin/bash

API_KEY="YOUR_API_KEY_HERE"
MAILCOW_URL="https://mail.yourdomain.com"

while IFS=, read -r username domain name quota password; do
    # 跳过标题行
    if [ "$username" == "username" ]; then
        continue
    fi
    
    echo "创建邮箱: $username@$domain"
    
    curl -X POST "${MAILCOW_URL}/api/v1/add/mailbox" \
        -H "X-API-Key: ${API_KEY}" \
        -H "Content-Type: application/json" \
        -d "{
            \"local_part\": \"$username\",
            \"domain\": \"$domain\",
            \"name\": \"$name\",
            \"quota\": $quota,
            \"password\": \"$password\",
            \"password2\": \"$password\",
            \"active\": \"1\"
        }"
    
    echo ""
done < mailboxes.csv
```

3. 执行导入：

```bash
chmod +x import_mailboxes.sh
./import_mailboxes.sh
```

## 🔌 与 Auto X Account 集成

### 配置方式

在 Auto X Account 的 `config.json` 中配置：

```json
{
  "email_provider": "Custom",
  "email_config": {
    "smtp_server": "mail.yourdomain.com",
    "smtp_port": 587,
    "smtp_use_tls": true,
    "imap_server": "mail.yourdomain.com",
    "imap_port": 993,
    "imap_use_ssl": true,
    "email_address": "user1@yourdomain.com",
    "password": "YourMailboxPassword"
  }
}
```

### 连接参数说明

| 协议 | 服务器地址 | 端口 | 加密方式 | 说明 |
|------|-----------|------|---------|------|
| SMTP | mail.yourdomain.com | 25 | STARTTLS | 标准 SMTP |
| SMTP | mail.yourdomain.com | 587 | STARTTLS | 推荐使用 |
| SMTP | mail.yourdomain.com | 465 | SSL/TLS | 加密 SMTP |
| IMAP | mail.yourdomain.com | 143 | STARTTLS | 标准 IMAP |
| IMAP | mail.yourdomain.com | 993 | SSL/TLS | 推荐使用 |
| POP3 | mail.yourdomain.com | 110 | STARTTLS | 标准 POP3 |
| POP3 | mail.yourdomain.com | 995 | SSL/TLS | 加密 POP3 |

### 测试连接

使用以下命令测试邮件服务器连接：

```bash
# 测试 SMTP
telnet mail.yourdomain.com 587

# 测试 IMAP
openssl s_client -connect mail.yourdomain.com:993

# 发送测试邮件
echo "This is a test email" | mail -s "Test" -S smtp=mail.yourdomain.com:587 \
    -S smtp-use-starttls -S smtp-auth=login \
    -S smtp-auth-user=user1@yourdomain.com \
    -S smtp-auth-password=YourPassword \
    test@example.com
```

## 🛠️ 维护和备份

### 日常维护命令

```bash
# 查看所有容器状态
cd /opt/mailcow-dockerized
docker compose ps

# 查看日志
docker compose logs -f [service_name]

# 重启所有服务
docker compose restart

# 重启单个服务
docker compose restart postfix-mailcow

# 停止所有服务
docker compose down

# 启动所有服务
docker compose up -d

# 更新 MailCow
git pull
docker compose pull
docker compose up -d
```

### 备份策略

#### 自动备份脚本

创建 `/opt/mailcow-backup.sh`：

```bash
#!/bin/bash

# MailCow 备份脚本
BACKUP_DIR="/backup/mailcow"
MAILCOW_DIR="/opt/mailcow-dockerized"
DATE=$(date +%Y%m%d_%H%M%S)

# 创建备份目录
mkdir -p $BACKUP_DIR

# 进入 MailCow 目录
cd $MAILCOW_DIR

# 使用 MailCow 内置备份工具
./helper-scripts/backup_and_restore.sh backup all --delete-days 30

# 备份到外部目录
cp -r /opt/mailcow-dockerized/data/assets/backups/* $BACKUP_DIR/

# 压缩备份
tar -czf $BACKUP_DIR/mailcow-backup-$DATE.tar.gz -C $BACKUP_DIR .

# 删除 30 天前的备份
find $BACKUP_DIR -name "mailcow-backup-*.tar.gz" -mtime +30 -delete

echo "备份完成: mailcow-backup-$DATE.tar.gz"
```

设置定时任务：

```bash
# 编辑 crontab
sudo crontab -e

# 添加每天凌晨 2 点执行备份
0 2 * * * /opt/mailcow-backup.sh >> /var/log/mailcow-backup.log 2>&1
```

#### 手动备份

```bash
cd /opt/mailcow-dockerized
./helper-scripts/backup_and_restore.sh backup all
```

#### 恢复备份

```bash
cd /opt/mailcow-dockerized
./helper-scripts/backup_and_restore.sh restore /path/to/backup.tar.gz
```

### 监控

设置邮件服务器监控：

```bash
# 安装监控工具
sudo apt install -y prometheus-node-exporter

# MailCow 内置 Prometheus 监控
# 访问 https://mail.yourdomain.com/metrics
```

## 🔧 故障排查

### 常见问题

#### 1. 无法发送邮件

**症状**: 邮件发送失败，显示 "Connection refused"

**解决方法**:
```bash
# 检查 Postfix 状态
docker compose logs postfix-mailcow

# 检查端口是否开放
sudo netstat -tulpn | grep -E '25|587|465'

# 检查防火墙
sudo ufw status
sudo iptables -L -n | grep -E '25|587|465'

# 重启 Postfix
docker compose restart postfix-mailcow
```

#### 2. 无法接收邮件

**症状**: 外部邮件无法送达

**解决方法**:
```bash
# 检查 MX 记录
dig MX yourdomain.com +short

# 检查 SPF 记录
dig TXT yourdomain.com +short

# 检查端口 25 是否被 ISP 封锁
telnet smtp.gmail.com 25

# 查看 Postfix 日志
docker compose logs postfix-mailcow | grep -i "reject\|error"
```

#### 3. SSL 证书问题

**症状**: 浏览器显示不安全连接

**解决方法**:
```bash
# 手动更新证书
cd /opt/mailcow-dockerized
docker compose restart acme-mailcow

# 查看证书日志
docker compose logs acme-mailcow

# 手动申请证书
docker compose exec acme-mailcow /usr/local/bin/acme.sh --issue \
    -d mail.yourdomain.com --standalone
```

#### 4. 数据库连接失败

**症状**: Web 界面无法访问

**解决方法**:
```bash
# 检查 MySQL 容器
docker compose logs mysql-mailcow

# 重启 MySQL
docker compose restart mysql-mailcow

# 进入 MySQL 容器
docker compose exec mysql-mailcow mysql -u root -p
```

#### 5. 容器无法启动

**症状**: docker compose up 失败

**解决方法**:
```bash
# 查看具体错误
docker compose logs

# 检查磁盘空间
df -h

# 检查端口占用
sudo netstat -tulpn | grep -E '80|443|25|587'

# 清理并重启
docker compose down
docker system prune -a
docker compose up -d
```

### 性能优化

#### 调整 Postfix 性能

编辑 `data/conf/postfix/main.cf`：

```
# 增加并发连接数
smtpd_client_connection_count_limit = 50
smtpd_client_connection_rate_limit = 100

# 增加队列处理速度
default_process_limit = 200
qmgr_message_active_limit = 30000
```

#### 调整 MySQL 性能

编辑 `data/conf/mysql/extra.cnf`：

```ini
[mysqld]
innodb_buffer_pool_size = 2G
max_connections = 500
query_cache_size = 128M
```

重启服务使配置生效：

```bash
docker compose restart postfix-mailcow mysql-mailcow
```

### 日志位置

- **Postfix**: `data/logs/postfix/`
- **Dovecot**: `data/logs/dovecot/`
- **Rspamd**: `data/logs/rspamd/`
- **SOGo**: `data/logs/sogo/`
- **MySQL**: `data/logs/mysql/`

## 📚 参考资料

- [MailCow 官方文档](https://docs.mailcow.email/)
- [MailCow GitHub](https://github.com/mailcow/mailcow-dockerized)
- [Let's Encrypt](https://letsencrypt.org/)
- [SPF 配置指南](https://www.spfwizard.net/)
- [DKIM 配置指南](https://www.dkim.org/)
- [邮件服务器测试工具](https://mxtoolbox.com/)

## 🆘 获取帮助

如果遇到问题：

1. **查看日志**: `docker compose logs -f`
2. **MailCow 论坛**: https://community.mailcow.email/
3. **GitHub Issues**: https://github.com/mailcow/mailcow-dockerized/issues
4. **Auto X Account Issues**: https://github.com/longzheng268/auto-x-account/issues

---

**注意**: 本文档仅供参考，邮件服务器可以独立部署，不必与 Auto X Account 放在同一服务器。建议使用专用服务器部署邮件服务以获得最佳性能和稳定性。
