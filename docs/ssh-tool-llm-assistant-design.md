# EasySSH — LLM AI 助手模块设计文档

> 模块代号：**AI Assistant**
> 文档版本：v1.0
> 创建时间：2026-05-08

---

## 一、模块概述

### 1.1 功能定位

AI 助手模块将大语言模型（LLM）深度集成到 SSH 终端使用流程中，为用户提供**无缝的命令行智能辅助**。用户在操作服务器时，可以随时通过自然语言提问来获取命令建议、解释错误信息、学习系统管理知识，极大降低命令行使用门槛。

### 1.2 核心价值

| 场景 | 传统方式 | 有 AI 助手后 |
|------|---------|-------------|
| 不知道命令怎么写 | 切到浏览器搜索 → 切回终端 | 终端内直接提问，获得可执行命令 |
| 命令报错看不懂 | 复制错误 → Google → 找答案 | AI 自动分析错误，给出解决方案 |
| 复杂脚本编写 | 参考文档慢慢写 | 描述需求 → AI 生成脚本 |
| 学习新命令 | 看 man page（枯燥） | AI 用通俗语言解释 + 给实例 |

---

## 二、功能需求说明

### 2.1 功能模块总览

```
AI Assistant
├── 智能问答（Chat）
│   ├── 命令生成
│   ├── 命令解释
│   ├── 错误诊断
│   └── 脚本生成
├── 终端内联辅助（Inline）
│   ├── 命令补全建议
│   ├── 错误自动解析
│   └── 危险命令警告
├── 上下文感知
│   ├── 当前目录/系统信息
│   ├── 最近命令历史
│   └── 终端输出分析
├── LLM 配置
│   ├── 多模型支持
│   ├── API Key 管理
│   └── 自定义 System Prompt
└── 隐私保护
    ├── 敏感信息过滤
    ├── 本地/云端模式选择
    └── 数据发送确认
```

### 2.2 智能问答（Chat Mode）

#### 2.2.1 命令生成

用户用自然语言描述需求，AI 返回对应的命令：

```
用户: 怎么查看当前目录下大于 100MB 的文件？
AI:   find . -type f -size +100M -exec ls -lh {} \;
      
      解释：
      • find . — 从当前目录开始搜索
      • -type f — 只搜索文件
      • -size +100M — 大于 100MB
      • -exec ls -lh {} \; — 对找到的文件执行 ls 显示详情
      
      [📋 复制] [▶ 执行] [🔄 换一种写法]
```

#### 2.2.2 命令解释

用户粘贴一条命令，AI 逐段解释：

```
用户: 解释一下 awk '{sum+=$5} END {print sum}' /var/log/access.log
AI:   这条命令的作用是：统计 access.log 第5列数字的总和
      
      逐段解析：
      ┌─ awk           → 文本处理工具
      ├─ '{sum+=$5}'   → 将每行第5列的值累加到 sum 变量
      ├─ 'END {print sum}' → 读完所有行后打印总和
      └─ /var/log/access.log → 目标文件
      
      常见用途：统计流量总量、请求总数等
      
      [👍] [👎] [追问]
```

#### 2.2.3 错误诊断

AI 自动或手动分析命令执行错误：

```
终端输出:
$ systemctl restart nginx
Job for nginx.service failed because the control process exited with error code.
See "systemctl status nginx.service" and "journalctl -xe" for details.

AI 自动分析:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 错误诊断：Nginx 启动失败

可能原因：
1. 配置文件语法错误
2. 端口被其他进程占用
3. 权限问题

建议排查步骤：
① 检查配置语法: nginx -t
② 查看详细错误: journalctl -u nginx --no-pager -n 50
③ 检查端口占用: ss -tlnp | grep :80

[▶ 执行 nginx -t] [▶ 查看日志] [继续追问]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### 2.2.4 脚本生成

根据需求描述生成完整脚本：

```
用户: 帮我写一个备份 MySQL 数据库的脚本，保留最近7天的备份

AI:   #!/bin/bash
      # MySQL 自动备份脚本
      # 保留最近 7 天的备份
      
      DB_USER="root"
      DB_PASS="your_password"
      DB_NAME="your_database"
      BACKUP_DIR="/backup/mysql"
      DAYS_TO_KEEP=7
      
      # 创建备份目录
      mkdir -p $BACKUP_DIR
      
      # 执行备份
      DATE=$(date +%Y%m%d_%H%M%S)
      FILENAME="${DB_NAME}_${DATE}.sql.gz"
      
      mysqldump -u$DB_USER -p$DB_PASS $DB_NAME | gzip > "$BACKUP_DIR/$FILENAME"
      
      if [ $? -eq 0 ]; then
          echo "✅ 备份成功: $FILENAME"
      else
          echo "❌ 备份失败"
          exit 1
      fi
      
      # 清理旧备份
      find $BACKUP_DIR -name "*.sql.gz" -mtime +$DAYS_TO_KEEP -delete
      echo "🧹 已清理 ${DAYS_TO_KEEP} 天前的备份"
      
      [📋 复制] [💾 保存为文件] [✏️ 修改参数] [▶ 在服务器上创建]
```

### 2.3 终端内联辅助（Inline Mode）

#### 2.3.1 命令补全建议

```
用户输入: docker 
                  ┌──────────────────────────────────────┐
                  │ 💡 AI 建议:                           │
                  │   docker ps -a          # 查看所有容器 │
                  │   docker images         # 查看镜像列表 │
                  │   docker compose up -d  # 启动服务     │
                  │   docker logs -f xxx    # 查看容器日志 │
                  └──────────────────────────────────────┘
按 Tab 选择 / Esc 关闭
```

#### 2.3.2 错误自动解析

当命令执行返回错误码时，自动在终端底部显示简要诊断：

```
$ pip install tensorflow
ERROR: Could not find a version that satisfies the requirement tensorflow

┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
💡 AI: Python 版本可能不兼容。检查: python --version
       TensorFlow 2.x 需要 Python 3.9-3.12
       尝试: pip install tensorflow==2.15.0
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
```

#### 2.3.3 危险命令警告

用户输入危险命令时实时提醒：

```
$ rm -rf /
  ⚠️ 危险！此命令将删除根目录下所有文件！
  确定执行？按 Enter 继续 / Ctrl+C 取消

$ chmod 777 /etc/passwd
  ⚠️ 高风险：对 passwd 文件设置 777 权限可能导致安全漏洞
  建议使用: chmod 644 /etc/passwd

$ > /dev/sda
  🚨 极度危险！这将清除整个磁盘数据！
  已自动拦截，按 Ctrl+Shift+Enter 强制执行
```

### 2.4 上下文感知

#### 2.4.1 自动收集的上下文信息

| 上下文 | 收集方式 | 用途 |
|--------|---------|------|
| 操作系统类型 | `uname -a` / `cat /etc/os-release` | 生成平台相关命令 |
| 当前用户 | `whoami` | 判断权限相关建议 |
| 当前目录 | `pwd` | 文件路径相关问答 |
| Shell 类型 | `echo $SHELL` | 生成对应 Shell 语法 |
| 最近 N 条命令 | 终端历史记录 | 理解用户当前任务上下文 |
| 最近终端输出 | 终端缓冲区 | 分析错误和结果 |
| 已安装包管理器 | 检测 apt/yum/brew 等 | 给出正确安装命令 |

#### 2.4.2 上下文注入策略

```
System Prompt（固定）
  + 服务器环境信息（连接时获取一次）
  + 最近 5 条命令历史（动态）
  + 最近终端输出（仅在分析错误时包含，最多 100 行）
  + 用户当前问题
```

### 2.5 LLM 配置

#### 2.5.1 支持的模型

| 提供商 | 模型 | 特点 |
|--------|------|------|
| OpenAI | GPT-4o / GPT-4o-mini | 综合能力强 |
| Anthropic | Claude 3.5 Sonnet | 代码能力优秀 |
| 本地模型 | Ollama (Llama/Qwen/DeepSeek) | 隐私保护，无需联网 |
| 自定义 | 兼容 OpenAI API 的任意端点 | 企业自建模型 |
| DeepSeek | DeepSeek-V3 / Chat | 性价比高 |
| 阿里云 | 通义千问 | 中文能力好 |

#### 2.5.2 模型配置项

```json
{
  "ai": {
    "enabled": true,
    "provider": "openai",
    "model": "gpt-4o-mini",
    "api_key": "sk-xxx (加密存储)",
    "api_base_url": "https://api.openai.com/v1",
    "temperature": 0.3,
    "max_tokens": 2048,
    "system_prompt_override": "",
    "inline_assist": true,
    "auto_error_diagnosis": true,
    "dangerous_command_warning": true,
    "context_history_lines": 5,
    "language": "zh-CN"
  }
}
```

#### 2.5.3 自定义 System Prompt

用户可以添加自定义指令来个性化 AI 行为：

```
默认 System Prompt:
"你是一个 Linux/Unix 命令行专家助手，嵌入在 SSH 终端工具中。
- 回答要简洁、实用，优先给出可直接执行的命令
- 对危险命令要明确警告
- 如果不确定用户的系统环境，先确认
- 用中文回答（除非用户使用英文提问）"

用户追加:
"我主要管理的是 CentOS 7 和 Ubuntu 22.04 服务器。
常用 Docker 和 Kubernetes。
不需要解释太基础的概念。"
```

### 2.6 隐私保护

#### 2.6.1 敏感信息过滤

发送给 LLM 前自动过滤/脱敏：

| 敏感信息类型 | 处理方式 |
|-------------|---------|
| IP 地址 | 替换为 `[SERVER_IP]` |
| 密码明文 | 替换为 `[PASSWORD]` |
| API Key / Token | 替换为 `[REDACTED]` |
| 私钥内容 | 完全移除 |
| 数据库连接串 | 脱敏处理 |
| 环境变量中的密钥 | 替换为 `[SECRET]` |

#### 2.6.2 数据发送控制

```
隐私等级设置：

🔒 严格模式（推荐）
   - 仅发送用户主动输入的问题
   - 不自动发送终端输出
   - 每次发送前显示确认框

🔓 标准模式
   - 发送问题 + 脱敏后的上下文
   - 自动错误诊断需要发送错误信息
   - 首次使用时确认，后续自动

🌐 开放模式
   - 自动发送相关上下文
   - 无需每次确认
   - 仅过滤明显的密钥/密码
```

#### 2.6.3 本地模型支持

对安全性要求极高的用户，支持完全本地运行的 LLM：

```
本地模型方案：
├── Ollama 集成
│   ├── 自动检测本地 Ollama 服务
│   ├── 模型列表选择
│   └── 一键下载推荐模型
├── 推荐本地模型：
│   ├── Qwen2.5-Coder-7B（代码专精，中文好）
│   ├── DeepSeek-Coder-V2-Lite（轻量高效）
│   └── CodeLlama-13B（代码能力强）
└── 优势：
    ├── 数据完全不出本机
    ├── 无需 API Key
    ├── 无网络也能用
    └── 无调用费用
```

### 2.7 对话管理

- 每个终端会话可以有独立的 AI 对话上下文
- 对话历史可保存和导出
- 支持清除当前对话重新开始
- 对话支持收藏（标记有价值的回答）
- 支持对话搜索（全文检索历史问答）

### 2.8 费用控制

- 显示当前月度 Token 使用量
- 可设置每日/每月 Token 上限
- 到达上限时提醒或自动切换到更便宜的模型
- 显示每次对话的预估费用

---

## 三、交互设计说明

### 3.1 AI 面板布局

AI 助手通过**侧边面板**或**底部面板**展示，与终端并行使用：

```
布局方案 A：右侧面板
┌────────────────────────────────────────────────────────────────────┐
│  [Server-A] [Server-B] [+]                                         │
├────────────────────────────────────────┬───────────────────────────┤
│                                        │  🤖 AI 助手               │
│                                        ├───────────────────────────┤
│          SSH 终端                       │  ┌─────────────────────┐ │
│                                        │  │ 你: 怎么查看磁盘空间 │ │
│  $ df -h                               │  │                     │ │
│  Filesystem  Size  Used Avail Use%      │  │ AI: 使用 df -h 命令│ │
│  /dev/sda1   50G   32G   18G  64%     │  │                     │ │
│  $ _                                   │  │ df -h              │ │
│                                        │  │ [复制] [执行]       │ │
│                                        │  └─────────────────────┘ │
│                                        │                           │
│                                        │  ┌─────────────────────┐ │
│                                        │  │ 💬 输入问题...       │ │
│                                        │  │              [发送]  │ │
│                                        │  └─────────────────────┘ │
├────────────────────────────────────────┴───────────────────────────┤
│  Status: Connected | Ping: 23ms | UTF-8                            │
└────────────────────────────────────────────────────────────────────┘

布局方案 B：底部面板（类似 VSCode 终端 + Copilot Chat）
┌────────────────────────────────────────────────────────────────────┐
│  [Server-A] [Server-B] [+]                                         │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│          SSH 终端（上半部分）                                        │
│                                                                     │
│  $ systemctl status nginx                                          │
│  ● nginx.service - A high performance web server                   │
│     Active: active (running)                                       │
│  $ _                                                               │
│                                                                     │
├────────────────────────────────────────────────────────────────────┤
│  🤖 AI 助手                                          [−] [□] [×]   │
│  ──────────────────────────────────────────────────────────────── │
│  你: nginx 怎么配置反向代理到 3000 端口？                          │
│                                                                     │
│  AI: 编辑 /etc/nginx/conf.d/proxy.conf:                           │
│                                                                     │
│  server {                                                          │
│      listen 80;                                                    │
│      server_name example.com;                                      │
│      location / {                                                  │
│          proxy_pass http://localhost:3000;                         │
│          proxy_set_header Host $host;                              │
│      }                                                            │
│  }                                                                │
│                                                                     │
│  然后重载: nginx -s reload                                         │
│  [📋 复制配置] [▶ 创建文件] [▶ 重载nginx]                         │
│  ──────────────────────────────────────────────────────────────── │
│  💬 [继续提问...]                                        [发送 ⏎]  │
└────────────────────────────────────────────────────────────────────┘
```

### 3.2 触发方式

| 触发方式 | 操作 | 说明 |
|---------|------|------|
| 快捷键 | `Ctrl+L` / `⌘+L` | 打开/关闭 AI 面板 |
| 快捷键 | `Ctrl+I` / `⌘+I` | 终端内联快速提问（小弹窗） |
| 命令前缀 | 终端输入 `?` + 问题 | 如 `? how to find large files` |
| 右键菜单 | 选中文本 → "Ask AI" | 解释选中的命令或错误 |
| 自动触发 | 命令执行失败 | 底部自动显示诊断建议 |
| 状态栏 | 点击 AI 图标 | 打开 AI 面板 |

### 3.3 内联快速提问（Mini Mode）

按 `Ctrl+I` 触发的轻量弹窗，不打开完整面板：

```
┌───────────────────────────────────────────────────┐
│  🤖 问 AI:                                        │
│  ┌─────────────────────────────────────────────┐ │
│  │ 查找最近 24 小时修改过的 log 文件             │ │
│  └─────────────────────────────────────────────┘ │
│                                                   │
│  💡 find /var/log -name "*.log" -mtime -1        │
│                                                   │
│  [📋 复制]  [▶ 粘贴到终端]  [▶ 直接执行]         │
│  [展开详细解释 ↓]                                 │
└───────────────────────────────────────────────────┘
```

### 3.4 命令执行确认

AI 建议的命令执行前需要用户确认：

```
AI 建议执行: rm -rf /tmp/old_backups/*

┌───────────────────────────────────────────┐
│  ⚡ 确认执行命令？                          │
│                                           │
│  rm -rf /tmp/old_backups/*                │
│                                           │
│  ⚠️ 此命令将永久删除文件                    │
│                                           │
│  [取消]        [复制到剪贴板]    [执行 ▶]  │
└───────────────────────────────────────────┘
```

**安全规则：**
- 所有 AI 建议的命令默认需要用户确认才执行
- 危险命令（rm -rf, dd, mkfs 等）需要二次确认
- 用户可设置"信任模式"对非危险命令自动执行（需明确开启）

### 3.5 代码块交互

AI 返回的代码块提供丰富的操作按钮：

```
AI 回答中的代码块:
┌─ bash ──────────────────────────────────────────┐
│  #!/bin/bash                                     │
│  for file in /var/log/*.log; do                  │
│      if [ $(stat -f%z "$file") -gt 104857600 ]; │
│          echo "Large: $file"                     │
│      fi                                         │
│  done                                           │
├──────────────────────────────────────────────────┤
│  [📋 复制] [▶ 执行] [💾 保存为文件] [✏️ 编辑]   │
└──────────────────────────────────────────────────┘
```

### 3.6 多轮对话流程

```
用户: Docker 容器无法启动，怎么排查？
AI:   可以按以下步骤排查：
      1. 查看容器日志: docker logs <container_id>
      2. 检查容器状态: docker inspect <container_id>
      3. 查看资源限制: docker stats
      
      你的容器 ID 是什么？或者你可以执行:
      docker ps -a --filter "status=exited"
      [▶ 执行此命令]

用户: [点击执行]
终端: CONTAINER ID  IMAGE       STATUS
      abc123        nginx:1.25  Exited (1) 5 min ago

AI:   检测到容器 abc123 (nginx:1.25) 退出码为 1
      让我查看它的日志:
      docker logs abc123
      [▶ 执行查看日志]

用户: [点击执行]
终端: nginx: [emerg] bind() to 0.0.0.0:80 failed (98: Address in use)

AI:   ✅ 找到问题！端口 80 被占用了。
      
      解决方案：
      ① 查看谁占用了 80 端口:
         ss -tlnp | grep :80
      ② 或者修改容器端口映射:
         docker run -p 8080:80 nginx:1.25
      
      [▶ 检查端口] [▶ 改用8080启动]
```

### 3.7 设置界面

```
┌─────────────────────────────────────────────────────────────┐
│  设置 > AI 助手                                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  模型配置                                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  AI 提供商:   [OpenAI        ▾]                    │     │
│  │  模型:        [gpt-4o-mini   ▾]                    │     │
│  │  API Key:     [sk-••••••••••••••••]  [测试连接]    │     │
│  │  API 地址:    [https://api.openai.com/v1]          │     │
│  │               □ 使用自定义代理地址                   │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  ──── 或使用本地模型 ────                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  ☑ 使用本地 Ollama                                 │     │
│  │  Ollama 地址: [http://localhost:11434]              │     │
│  │  模型:        [qwen2.5-coder:7b  ▾]  [刷新列表]   │     │
│  │  状态:        ● 已连接                              │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  功能开关                                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  AI 助手总开关         [开启 ●───]                  │     │
│  │  内联命令补全           [开启 ●───]                  │     │
│  │  自动错误诊断           [开启 ●───]                  │     │
│  │  危险命令警告           [开启 ●───]                  │     │
│  │  命令执行确认           [开启 ●───]                  │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  隐私设置                                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  隐私模式:    (●) 严格  ( ) 标准  ( ) 开放         │     │
│  │  IP 脱敏:             [开启 ●───]                   │     │
│  │  密码过滤:            [开启 ●───]                   │     │
│  │  发送前预览:          [开启 ●───]                   │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  高级设置                                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Temperature:        [0.3         ]                 │     │
│  │  最大回复长度:        [2048 tokens ]                 │     │
│  │  上下文命令条数:      [5           ]                 │     │
│  │  回答语言:           [跟随系统 ▾]                    │     │
│  │  自定义 Prompt:      [编辑...]                      │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  使用统计                                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  本月 Token: 45,230 / 100,000                      │     │
│  │  ████████░░░░░░░ 45%                               │     │
│  │  预估费用: $0.23                                    │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 四、技术架构设计

### 4.1 架构总览

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (Vue 3)                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    AI Panel Component                     │    │
│  │  ├── ChatView.vue (对话主界面)                            │    │
│  │  ├── InlineAssist.vue (内联弹窗)                          │    │
│  │  ├── CodeBlock.vue (代码块+操作按钮)                      │    │
│  │  ├── ErrorDiagnosis.vue (错误诊断卡片)                    │    │
│  │  └── AISettings.vue (设置界面)                            │    │
│  └──────────────────────────┬──────────────────────────────┘    │
│                              │ Tauri IPC                          │
├──────────────────────────────┼──────────────────────────────────┤
│                     Rust Backend                                   │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                   AI Service Layer                         │    │
│  │  ┌──────────────────────────────────────────────────┐    │    │
│  │  │              LLM Gateway                          │    │    │
│  │  │  ├── OpenAI Adapter                              │    │    │
│  │  │  ├── Anthropic Adapter                           │    │    │
│  │  │  ├── Ollama Adapter (本地)                       │    │    │
│  │  │  └── Custom OpenAI-Compatible Adapter            │    │    │
│  │  └──────────────────────────────────────────────────┘    │    │
│  │  ┌──────────────────────────────────────────────────┐    │    │
│  │  │           Context Manager                         │    │    │
│  │  │  ├── 系统信息收集                                  │    │    │
│  │  │  ├── 命令历史管理                                  │    │    │
│  │  │  ├── 终端输出缓存                                  │    │    │
│  │  │  └── Prompt 模板引擎                               │    │    │
│  │  └──────────────────────────────────────────────────┘    │    │
│  │  ┌──────────────────────────────────────────────────┐    │    │
│  │  │           Privacy Filter                          │    │    │
│  │  │  ├── 正则匹配敏感信息                              │    │    │
│  │  │  ├── 脱敏替换                                     │    │    │
│  │  │  └── 发送前审计日志                                │    │    │
│  │  └──────────────────────────────────────────────────┘    │    │
│  │  ┌──────────────────────────────────────────────────┐    │    │
│  │  │           Danger Detector                         │    │    │
│  │  │  ├── 命令风险评估                                  │    │    │
│  │  │  ├── 规则引擎（内置规则）                          │    │    │
│  │  │  └── AI 二次确认（可选）                           │    │    │
│  │  └──────────────────────────────────────────────────┘    │    │
│  └──────────────────────────────────────────────────────────┘    │
│                              │                                    │
│                              ▼                                    │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  LLM API (外部)           │  Ollama (本地)               │    │
│  │  • api.openai.com         │  • localhost:11434           │    │
│  │  • api.anthropic.com      │  • qwen2.5-coder            │    │
│  │  • api.deepseek.com       │  • codellama                │    │
│  └──────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 LLM Gateway 设计

```rust
// LLM 统一接口（Trait）
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 发送聊天请求
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    
    /// 流式聊天（SSE）
    async fn chat_stream(&self, request: ChatRequest) -> Result<impl Stream<Item = ChatChunk>>;
    
    /// 测试连接
    async fn test_connection(&self) -> Result<bool>;
    
    /// 获取可用模型列表
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
}

// 请求/响应结构
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

pub struct Message {
    pub role: Role,          // System / User / Assistant
    pub content: String,
}

pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
}

pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// 具体实现
pub struct OpenAIProvider { api_key: String, base_url: String, model: String }
pub struct AnthropicProvider { api_key: String, model: String }
pub struct OllamaProvider { base_url: String, model: String }
pub struct CustomProvider { api_key: Option<String>, base_url: String, model: String }

// Gateway 路由
pub struct LlmGateway {
    provider: Box<dyn LlmProvider>,
    config: AiConfig,
    usage_tracker: UsageTracker,
}

impl LlmGateway {
    pub fn new(config: AiConfig) -> Self {
        let provider: Box<dyn LlmProvider> = match config.provider.as_str() {
            "openai" => Box::new(OpenAIProvider::new(&config)),
            "anthropic" => Box::new(AnthropicProvider::new(&config)),
            "ollama" => Box::new(OllamaProvider::new(&config)),
            "custom" => Box::new(CustomProvider::new(&config)),
            _ => panic!("Unknown provider"),
        };
        Self { provider, config, usage_tracker: UsageTracker::new() }
    }
}
```

### 4.3 Context Manager 设计

```rust
pub struct ContextManager {
    system_info: Option<ServerInfo>,
    command_history: VecDeque<CommandRecord>,
    terminal_buffer: String,
    max_history: usize,
    max_buffer_lines: usize,
}

pub struct ServerInfo {
    pub os: String,              // "Ubuntu 22.04 LTS"
    pub kernel: String,          // "5.15.0-91-generic"
    pub shell: String,           // "/bin/bash"
    pub user: String,            // "deploy"
    pub hostname: String,        // "web-server-01"
    pub arch: String,            // "x86_64"
    pub package_managers: Vec<String>, // ["apt", "snap"]
}

pub struct CommandRecord {
    pub command: String,
    pub exit_code: Option<i32>,
    pub output_snippet: Option<String>,  // 最后 N 行输出
    pub timestamp: Instant,
}

impl ContextManager {
    /// 构建发送给 LLM 的完整 Prompt
    pub fn build_prompt(&self, user_question: &str, mode: PromptMode) -> Vec<Message> {
        let mut messages = vec![];
        
        // 1. System Prompt
        messages.push(Message {
            role: Role::System,
            content: self.build_system_prompt(),
        });
        
        // 2. 服务器环境上下文
        if let Some(info) = &self.system_info {
            messages.push(Message {
                role: Role::System,
                content: format!("当前服务器环境: {} {} | 用户: {} | Shell: {}",
                    info.os, info.arch, info.user, info.shell),
            });
        }
        
        // 3. 最近命令上下文（仅在需要时）
        if mode == PromptMode::ErrorDiagnosis || mode == PromptMode::ContextAware {
            let history_context = self.format_command_history();
            messages.push(Message {
                role: Role::System,
                content: format!("最近执行的命令:\n{}", history_context),
            });
        }
        
        // 4. 用户问题
        messages.push(Message {
            role: Role::User,
            content: user_question.to_string(),
        });
        
        messages
    }
}
```

### 4.4 隐私过滤器

```rust
pub struct PrivacyFilter {
    rules: Vec<FilterRule>,
    mode: PrivacyMode,
}

pub enum PrivacyMode {
    Strict,    // 严格：最大化脱敏
    Standard,  // 标准：智能脱敏
    Open,      // 开放：仅过滤明显密钥
}

pub struct FilterRule {
    name: String,
    pattern: Regex,
    replacement: String,
}

impl PrivacyFilter {
    pub fn new(mode: PrivacyMode) -> Self {
        let rules = vec![
            // IP 地址
            FilterRule {
                name: "IPv4".into(),
                pattern: Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap(),
                replacement: "[IP_ADDR]".into(),
            },
            // API Keys (通用模式)
            FilterRule {
                name: "API Key".into(),
                pattern: Regex::new(r"(?i)(api[_-]?key|token|secret)\s*[:=]\s*\S+").unwrap(),
                replacement: "[REDACTED_KEY]".into(),
            },
            // 密码字段
            FilterRule {
                name: "Password".into(),
                pattern: Regex::new(r"(?i)(password|passwd|pwd)\s*[:=]\s*\S+").unwrap(),
                replacement: "[REDACTED_PASSWORD]".into(),
            },
            // SSH 私钥
            FilterRule {
                name: "Private Key".into(),
                pattern: Regex::new(r"-----BEGIN .* PRIVATE KEY-----[\s\S]*?-----END .* PRIVATE KEY-----").unwrap(),
                replacement: "[PRIVATE_KEY_REMOVED]".into(),
            },
            // AWS Key
            FilterRule {
                name: "AWS Key".into(),
                pattern: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
                replacement: "[AWS_KEY]".into(),
            },
        ];
        Self { rules, mode }
    }
    
    pub fn filter(&self, text: &str) -> (String, Vec<String>) {
        let mut filtered = text.to_string();
        let mut applied_rules = vec![];
        
        for rule in &self.rules {
            if rule.pattern.is_match(&filtered) {
                filtered = rule.pattern.replace_all(&filtered, &rule.replacement).to_string();
                applied_rules.push(rule.name.clone());
            }
        }
        
        (filtered, applied_rules)
    }
}
```

### 4.5 危险命令检测器

```rust
pub struct DangerDetector {
    rules: Vec<DangerRule>,
}

pub struct DangerRule {
    pattern: Regex,
    level: DangerLevel,
    message: String,
    suggestion: Option<String>,
}

pub enum DangerLevel {
    Warning,    // 黄色提醒
    Danger,     // 红色警告
    Critical,   // 拦截
}

impl DangerDetector {
    pub fn new() -> Self {
        let rules = vec![
            DangerRule {
                pattern: Regex::new(r"rm\s+(-[rRf]+\s+)*/([\s]|$)").unwrap(),
                level: DangerLevel::Critical,
                message: "此命令将删除根目录下所有文件！".into(),
                suggestion: None,
            },
            DangerRule {
                pattern: Regex::new(r"rm\s+-[rRf]*\s").unwrap(),
                level: DangerLevel::Warning,
                message: "递归删除文件，请确认目标路径正确".into(),
                suggestion: Some("建议先用 ls 确认文件列表".into()),
            },
            DangerRule {
                pattern: Regex::new(r"chmod\s+777").unwrap(),
                level: DangerLevel::Warning,
                message: "777 权限过于开放，可能存在安全隐患".into(),
                suggestion: Some("建议使用更精确的权限如 755 或 644".into()),
            },
            DangerRule {
                pattern: Regex::new(r"dd\s+.*of=/dev/").unwrap(),
                level: DangerLevel::Critical,
                message: "直接写入设备文件，可能导致数据丢失".into(),
                suggestion: None,
            },
            DangerRule {
                pattern: Regex::new(r"mkfs\.\w+\s+/dev/").unwrap(),
                level: DangerLevel::Critical,
                message: "格式化磁盘将永久删除所有数据！".into(),
                suggestion: None,
            },
            DangerRule {
                pattern: Regex::new(r">\s*/dev/sd[a-z]").unwrap(),
                level: DangerLevel::Critical,
                message: "重定向到磁盘设备将清除所有数据！".into(),
                suggestion: None,
            },
            DangerRule {
                pattern: Regex::new(r":()\{.*\};\s*:").unwrap(),
                level: DangerLevel::Critical,
                message: "检测到 Fork Bomb！将耗尽系统资源".into(),
                suggestion: None,
            },
        ];
        Self { rules }
    }
    
    pub fn check(&self, command: &str) -> Option<DangerAssessment> {
        for rule in &self.rules {
            if rule.pattern.is_match(command) {
                return Some(DangerAssessment {
                    level: rule.level.clone(),
                    message: rule.message.clone(),
                    suggestion: rule.suggestion.clone(),
                });
            }
        }
        None
    }
}
```

### 4.6 前端 AI 组件架构

```typescript
// composables/useAI.ts
export function useAI() {
  const config = ref<AIConfig>(loadAIConfig())
  const messages = ref<ChatMessage[]>([])
  const isLoading = ref(false)
  const streamContent = ref('')
  
  // 发送消息（支持流式）
  async function sendMessage(content: string, mode: 'chat' | 'inline' | 'diagnosis') {
    isLoading.value = true
    messages.value.push({ role: 'user', content })
    
    // 通过 Tauri IPC 调用 Rust 后端
    const stream = await invoke('ai_chat_stream', {
      message: content,
      mode,
      sessionId: currentSession.value?.id,
    })
    
    // 处理流式响应
    await listen('ai-stream-chunk', (event) => {
      streamContent.value += event.payload.content
    })
    
    await listen('ai-stream-end', (event) => {
      messages.value.push({ role: 'assistant', content: streamContent.value })
      streamContent.value = ''
      isLoading.value = false
    })
  }
  
  // 执行 AI 建议的命令
  async function executeCommand(command: string, sessionId: string) {
    // 危险检测
    const danger = await invoke('check_danger', { command })
    if (danger) {
      const confirmed = await showDangerConfirm(danger)
      if (!confirmed) return
    }
    // 发送到终端
    await invoke('terminal_write', { sessionId, data: command + '\n' })
  }
  
  // 自动错误诊断
  async function diagnoseError(errorOutput: string, command: string) {
    return await sendMessage(
      `命令 "${command}" 执行后输出了以下错误，请诊断原因并给出解决方案:\n\n${errorOutput}`,
      'diagnosis'
    )
  }
  
  return { config, messages, isLoading, streamContent, sendMessage, executeCommand, diagnoseError }
}
```

### 4.7 IPC 命令定义

```rust
// Tauri Commands
#[tauri::command]
async fn ai_chat(message: String, mode: String, session_id: Option<String>) -> Result<String, String>;

#[tauri::command]
async fn ai_chat_stream(message: String, mode: String, session_id: Option<String>, window: Window) -> Result<(), String>;

#[tauri::command]
async fn ai_test_connection(config: AiConfig) -> Result<bool, String>;

#[tauri::command]
async fn ai_list_models(provider: String, config: AiConfig) -> Result<Vec<ModelInfo>, String>;

#[tauri::command]
async fn check_danger(command: String) -> Result<Option<DangerAssessment>, String>;

#[tauri::command]
async fn ai_get_usage() -> Result<UsageStats, String>;

#[tauri::command]
async fn ai_update_config(config: AiConfig) -> Result<(), String>;

#[tauri::command]
async fn ai_collect_server_info(session_id: String) -> Result<ServerInfo, String>;
```

---

## 五、Prompt Engineering

### 5.1 System Prompt 模板

```
你是 EasySSH 内置的命令行智能助手。你的任务是帮助用户高效地管理 Linux/Unix 服务器。

## 角色定义
- 你是一个资深的 Linux 系统管理员和 DevOps 工程师
- 精通各种命令行工具、Shell 脚本、系统管理
- 回答简洁实用，优先给出可直接执行的命令

## 回答规范
1. 命令建议用代码块包裹
2. 复杂命令要逐段解释
3. 危险操作必须明确警告
4. 如果有多种方案，简要列出并推荐最佳
5. 不确定时说明并建议用户先确认环境

## 环境信息
{server_info}

## 约束
- 不生成恶意或危险脚本
- 不泄露系统安全信息
- 用 {language} 回答
```

### 5.2 不同模式的 Prompt 变体

```
# 命令生成模式
"用户需要一条命令来完成以下操作。直接给出命令，然后简要解释。"

# 错误诊断模式
"用户执行命令后遇到错误，请分析可能原因（按可能性排序），并给出解决步骤。"

# 脚本生成模式
"用户需要一个 Shell 脚本。请生成完整可运行的脚本，包含注释和错误处理。"

# 解释模式
"用户想理解以下命令的含义。请逐段解释每个参数和选项的作用。"
```

---

## 六、性能与体验优化

### 6.1 流式响应
- 使用 Server-Sent Events (SSE) 流式接收 LLM 回复
- 打字机效果实时显示，无需等待完整响应
- 支持用户中途取消生成

### 6.2 请求优化
- 非关键请求（补全建议）设置短超时（5s）
- 错误诊断异步进行，不阻塞终端操作
- 相同问题缓存响应（短期内存缓存）
- 防抖：快速输入时延迟触发补全

### 6.3 降级策略
- API 超时 → 显示"AI 暂时不可用"，不影响终端使用
- 配额用完 → 提示升级或切换模型
- 网络断开 → 如果有本地模型则自动切换，否则禁用 AI 功能
- 模型返回空/异常 → 重试一次，仍失败则提示用户
