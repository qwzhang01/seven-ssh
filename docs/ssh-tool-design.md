# SSH Desktop Tool — 产品设计文档

> 项目代号：**EasySSH**
> 文档版本：v1.0
> 创建时间：2026-05-08

---

## 一、产品概述

### 1.1 产品定位

EasySSH 是一款跨平台（macOS / Windows）的 SSH 终端管理工具，集成 SFTP 文件传输功能，面向开发者和运维人员，提供高效、安全、易用的远程服务器管理体验。

### 1.2 目标用户

| 用户角色 | 核心需求 |
|---------|---------|
| 后端开发者 | 日常登录开发/测试服务器，查看日志、部署代码 |
| 运维工程师 | 批量管理大量服务器，文件传输，自动化运维 |
| 全栈开发者 | SSH + SFTP 一体化，减少工具切换 |
| 学生/个人开发者 | 免费好用的 SSH 工具替代 Xshell/SecureCRT |

### 1.3 竞品对标

| 竞品 | 平台 | 优势 | 不足 |
|------|------|------|------|
| Xshell | Windows | 功能全面，稳定 | 仅 Windows，收费 |
| Termius | 全平台 | UI 现代，云同步 | 高级功能收费 |
| MobaXterm | Windows | 集成度高 | 仅 Windows，界面复杂 |
| Royal TSX | macOS | macOS 原生体验 | 仅 Mac，收费 |
| Tabby | 全平台 | 开源，可扩展 | 资源占用较大 |

### 1.4 差异化优势（规划）

- ✅ 真正跨平台：macOS + Windows 一致体验
- ✅ SSH + SFTP 深度整合，无缝切换
- ✅ 现代化 UI，暗色主题优先
- ✅ 本地加密存储，安全可控
- ✅ 轻量级，启动快，资源占用低
- ✅ 免费 + 开源（或 Freemium 模式）

---

## 二、功能需求说明

### 2.1 功能模块总览

```
EasySSH
├── 连接管理模块
│   ├── 新建连接
│   ├── 连接分组
│   ├── 快速连接
│   ├── 导入/导出连接
│   └── 连接状态管理
├── SSH 终端模块
│   ├── 多标签终端
│   ├── 分屏操作
│   ├── 终端主题
│   ├── 快捷命令
│   └── 会话日志
├── SFTP 文件管理模块
│   ├── 双栏文件浏览器
│   ├── 文件上传/下载
│   ├── 拖拽传输
│   ├── 传输队列管理
│   └── 文件编辑
├── 安全模块
│   ├── 密钥管理
│   ├── 数据加密存储
│   └── 主密码保护
└── 系统设置模块
    ├── 全局配置
    ├── 外观设置
    ├── 快捷键管理
    └── 数据备份/恢复
```

---

### 2.2 连接管理模块

#### 2.2.1 新建/编辑连接

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| 连接名称 | text | ✅ | 用户自定义名称，用于标识 |
| 主机地址 | text | ✅ | IP 或域名 |
| 端口 | number | ✅ | 默认 22 |
| 认证方式 | select | ✅ | 密码 / 密钥 / 密钥+密码短语 / Agent |
| 用户名 | text | ✅ | 登录用户名 |
| 密码 | password | 条件 | 认证方式为"密码"时必填 |
| 私钥文件 | file | 条件 | 认证方式为"密钥"时必填 |
| 密钥密码短语 | password | 可选 | 加密的私钥需要 |
| 所属分组 | select | 可选 | 选择已有分组或新建 |
| 标签/颜色 | tag | 可选 | 自定义标签便于识别 |
| 备注 | textarea | 可选 | 自定义备注信息 |
| 启动命令 | text | 可选 | 连接成功后自动执行的命令 |
| 跳板机 | ref | 可选 | 配置代理跳转（ProxyJump） |
| 字符编码 | select | 可选 | UTF-8（默认）/ GBK / ISO-8859-1 |
| 保活间隔 | number | 可选 | Keep-Alive 间隔秒数，默认 60 |

#### 2.2.2 连接分组

- 支持多级树形分组（目录结构）
- 支持拖拽排序和移动
- 分组支持展开/折叠
- 分组支持批量操作（全部连接/断开）
- 分组支持颜色标记和图标

#### 2.2.3 快速连接

- 地址栏直接输入 `user@host:port` 快速连接
- 支持解析 `~/.ssh/config` 文件中的配置
- 最近连接历史记录（最近 20 条）
- 搜索框模糊匹配连接名称/地址/标签

#### 2.2.4 导入/导出

- 支持导入格式：
  - SSH Config（`~/.ssh/config`）
  - Xshell 会话文件（.xsh）
  - MobaXterm 配置
  - PuTTY Registry 导出
  - JSON / CSV 格式
- 支持导出格式：
  - 加密 JSON（默认，含密码信息）
  - 纯文本 JSON（不含敏感信息）
  - SSH Config 格式

---

### 2.3 SSH 终端模块

#### 2.3.1 多标签终端

- Tab 标签栏管理多个终端会话
- 标签显示连接名称 + 状态指示器（连接中/已连接/已断开）
- 支持标签拖拽排序、拖出独立窗口
- 双击标签重命名
- 右键菜单：复制会话、关闭、关闭其他、关闭右侧

#### 2.3.2 分屏操作

- 支持水平分屏 / 垂直分屏
- 最大支持 4 分屏（2×2 网格）
- 分屏之间支持拖拽调整大小
- 支持"同步输入"模式（一个输入同步到所有分屏）

#### 2.3.3 终端能力

- 完整的 VT100/xterm-256color 终端仿真
- 支持 Unicode / Emoji 显示
- 支持自定义字体、字号、行间距
- 支持终端背景色/透明度设置
- 内置多种颜色主题（Dracula/One Dark/Solarized/Monokai 等）
- 支持自定义主题（导入 iTerm2 配色方案）
- 滚动缓冲区大小可配置（默认 10000 行）
- 支持搜索终端输出内容（Ctrl+F）
- 支持选中即复制
- 右键粘贴（可配置）

#### 2.3.4 快捷命令（Snippets）

- 全局命令片段库
- 支持分类管理
- 命令支持变量占位符（如 `${server_ip}`）
- 快捷键触发命令面板（类似 VSCode Command Palette）
- 支持发送命令到指定/所有终端

#### 2.3.5 会话日志

- 自动记录终端会话（可选开启）
- 日志格式：纯文本 / 带时间戳
- 日志存储路径可配置
- 支持按会话查看历史日志

---

### 2.4 SFTP 文件管理模块

#### 2.4.1 双栏文件浏览器

```
┌──────────────────────────────────────────────────────┐
│  本地文件 (Local)          │  远程文件 (Remote)        │
├──────────────────────────────────────────────────────┤
│  📁 Documents             │  📁 /home/user           │
│  📁 Desktop               │  📁 /var/www             │
│  📁 Downloads             │  📁 /etc/nginx           │
│  📄 deploy.sh             │  📄 app.conf             │
│  📄 readme.md             │  📄 index.html           │
├──────────────────────────────────────────────────────┤
│  路径: /Users/me/projects │  路径: /home/user/app    │
└──────────────────────────────────────────────────────┘
```

- 左侧：本地文件系统
- 右侧：远程服务器文件系统
- 支持地址栏直接输入路径跳转
- 支持书签/收藏常用路径
- 支持文件搜索/过滤
- 显示文件详情：大小、权限、修改时间、所有者

#### 2.4.2 文件操作

| 操作 | 说明 |
|------|------|
| 上传 | 本地 → 远程，支持文件和文件夹 |
| 下载 | 远程 → 本地，支持文件和文件夹 |
| 拖拽传输 | 左右栏拖拽实现上传/下载 |
| 新建文件/文件夹 | 在远程创建 |
| 删除 | 支持批量删除，二次确认 |
| 重命名 | 远程文件重命名 |
| 修改权限 | chmod 可视化设置（rwx 复选框） |
| 在线编辑 | 内置编辑器打开远程文本文件 |

#### 2.4.3 传输队列

- 传输任务队列管理面板
- 显示：文件名、大小、进度、速度、状态
- 支持暂停/恢复/取消单个任务
- 支持传输失败自动重试（可配置次数）
- 支持断点续传（大文件）
- 传输完成通知
- 传输历史记录

#### 2.4.4 内置文件编辑器

- 基于 Monaco Editor / CodeMirror
- 语法高亮（常见配置文件格式）
- 自动检测文件编码
- 保存后自动上传回服务器
- 支持对比差异（Diff 视图）

---

### 2.5 安全模块

#### 2.5.1 认证方式支持

| 认证方式 | 说明 |
|---------|------|
| 密码认证 | 传统用户名+密码 |
| 公钥认证 | RSA / ED25519 / ECDSA 私钥文件 |
| 密钥+密码短语 | 加密私钥，需输入 Passphrase |
| SSH Agent | 系统 SSH Agent 代理（macOS Keychain / Pageant） |
| 键盘交互 | Keyboard-Interactive 认证（2FA 等场景） |
| 跳板机 | ProxyJump / ProxyCommand 多跳连接 |

#### 2.5.2 密钥管理

- 内置密钥对生成器（RSA 2048/4096, ED25519）
- 密钥导入/管理界面
- 一键部署公钥到服务器（ssh-copy-id 功能）
- 密钥文件加密存储

#### 2.5.3 本地数据安全

- 所有敏感数据（密码、密钥密码短语）AES-256-GCM 加密存储
- 支持设置主密码（Master Password）
- 应用锁定：一定时间无操作后自动锁定
- 敏感数据不写入日志
- 清除剪贴板中的敏感内容（可配置）

---

### 2.6 系统设置模块

#### 2.6.1 全局配置

- 默认编码设置
- 默认终端字体/字号
- 代理设置（HTTP/SOCKS5 全局代理）
- 启动行为（启动时自动连接上次会话）
- 自动更新设置

#### 2.6.2 外观设置

- 主题：暗色（默认）/ 亮色 / 跟随系统
- 语言：中文 / English
- 侧边栏位置和宽度
- 窗口透明度

#### 2.6.3 快捷键

- 全局快捷键完全可自定义
- 内置多种预设方案（默认 / Vim 风格 / Emacs 风格）
- 支持导入/导出快捷键配置

#### 2.6.4 数据备份与恢复

- 一键导出所有配置和连接数据（加密压缩包）
- 从备份文件恢复
- 可选：WebDAV / 本地目录 同步（未来可扩展云同步）

---

## 三、交互设计说明

### 3.1 整体布局

```
┌─────────────────────────────────────────────────────────────────┐
│  Menu Bar (macOS) / Title Bar (Windows)                          │
├──────┬──────────────────────────────────────────────────────────┤
│      │  Tab Bar: [Server-A] [Server-B] [Server-C] [+]           │
│      ├──────────────────────────────────────────────────────────┤
│  S   │                                                           │
│  I   │                                                           │
│  D   │              Terminal / SFTP 主工作区                      │
│  E   │                                                           │
│  B   │                                                           │
│  A   │                                                           │
│  R   │                                                           │
│      ├──────────────────────────────────────────────────────────┤
│      │  Status Bar: 连接状态 | 延迟 | 编码 | 通知                 │
└──────┴──────────────────────────────────────────────────────────┘
```

### 3.2 侧边栏（Sidebar）

```
┌──────────┐
│ 🔍 搜索   │
├──────────┤
│ ⭐ 收藏    │
│  Server-1│
│  Server-2│
├──────────┤
│ 📂 生产环境│
│  ├ Web-01│
│  ├ Web-02│
│  └ DB-01 │
│ 📂 测试环境│
│  ├ Dev-01│
│  └ Dev-02│
│ 📂 个人    │
│  └ VPS   │
├──────────┤
│ 🕐 最近    │
│  Server-A│
│  Server-B│
└──────────┘
```

- 顶部：搜索框（全局模糊搜索）
- 收藏：置顶的常用连接
- 分组：树形结构展示
- 最近：最近连接过的服务器

### 3.3 核心交互流程

#### 3.3.1 新建连接流程

```
用户操作                    系统响应
───────                    ─────
点击 [+新建连接]      →     弹出新建连接表单
                           （智能默认值：端口22，UTF-8）
填写基本信息          →     实时校验（IP格式、端口范围）
选择认证方式          →     动态显示对应的认证字段
点击 [测试连接]       →     Loading → 成功/失败提示
                           失败时显示错误原因和建议
点击 [保存]           →     保存到本地，出现在侧边栏
双击连接项            →     新开标签页，建立SSH连接
```

#### 3.3.2 终端操作流程

```
双击连接              →     新标签页 → 显示连接动画 → 终端就绪
右键连接 → SFTP      →     在当前标签页下方打开 SFTP 面板
终端内 Ctrl+Shift+F  →     弹出搜索浮窗
终端断开              →     显示重连提示 → 点击自动重连
```

#### 3.3.3 SFTP 操作流程

```
连接状态下点击 SFTP    →    下方滑出双栏文件管理器
拖拽文件到远程栏       →    添加到传输队列 → 自动开始上传
双击远程文本文件       →    新标签页打开编辑器
Ctrl+S 保存编辑       →    自动上传到服务器 → 状态栏提示
```

### 3.4 关键交互细节

#### 3.4.1 连接状态指示

| 状态 | 视觉表现 |
|------|---------|
| 未连接 | 灰色圆点 |
| 连接中 | 蓝色脉冲动画 |
| 已连接 | 绿色实心圆点 |
| 连接失败 | 红色圆点 + 感叹号 |
| 连接断开 | 黄色圆点 + 重连按钮 |

#### 3.4.2 快捷键设计

| 功能 | macOS | Windows |
|------|-------|---------|
| 新建连接 | ⌘+N | Ctrl+N |
| 新建标签 | ⌘+T | Ctrl+T |
| 关闭标签 | ⌘+W | Ctrl+W |
| 切换标签 | ⌘+数字 | Ctrl+数字 |
| 打开 SFTP | ⌘+Shift+S | Ctrl+Shift+S |
| 全局搜索 | ⌘+K | Ctrl+K |
| 命令面板 | ⌘+Shift+P | Ctrl+Shift+P |
| 水平分屏 | ⌘+Shift+H | Ctrl+Shift+H |
| 垂直分屏 | ⌘+Shift+V | Ctrl+Shift+V |
| 终端搜索 | ⌘+F | Ctrl+F |
| 复制 | ⌘+C | Ctrl+C（无选中时发送 SIGINT）|
| 粘贴 | ⌘+V | Ctrl+Shift+V |

#### 3.4.3 右键菜单

**终端区域右键：**
- 复制
- 粘贴
- 全选
- ─────
- 搜索
- 清屏
- ─────
- 水平分屏
- 垂直分屏
- ─────
- 打开 SFTP
- 会话日志
- 连接信息

**侧边栏连接项右键：**
- 连接
- 在新窗口连接
- ─────
- 编辑
- 复制连接
- 删除
- ─────
- 移动到分组
- 添加到收藏
- ─────
- 打开 SFTP
- 复制主机地址

### 3.5 响应式与自适应

- 窗口最小尺寸：960×600
- 侧边栏可折叠/展开，拖拽调整宽度
- SFTP 面板高度可拖拽调整
- 支持全屏模式（macOS 原生全屏）
- 多显示器支持：标签可拖拽到新窗口

---

## 四、技术架构设计

### 4.1 技术选型

| 层级 | 技术选择 | 理由 |
|------|---------|------|
| 框架 | **Tauri 2.x** | Rust 后端 + Web 前端，包体小（~10MB），性能好，跨平台 |
| 前端 | **Vue 3 + TypeScript** | 生态成熟，类型安全，开发效率高 |
| UI 库 | **TDesign Vue Next** | 开箱即用的企业级组件 |
| 终端 | **xterm.js** | 业界标准 Web 终端仿真器 |
| 文件编辑 | **Monaco Editor** | VSCode 同款编辑器，功能强大 |
| SSH 协议 | **russh**（Rust） | 纯 Rust SSH2 实现，性能好，安全 |
| 本地存储 | **SQLite + SQLCipher** | 轻量嵌入式数据库 + 透明加密 |
| 加密 | **ring / aes-gcm** | Rust 加密库，安全审计完备 |
| 构建 | **Vite 5** | 前端构建工具，HMR 快 |
| 包管理 | **pnpm** | 快速，磁盘占用小 |

### 4.2 为什么选择 Tauri 而非 Electron

| 对比项 | Tauri | Electron |
|--------|-------|----------|
| 安装包大小 | ~10-15MB | ~80-150MB |
| 内存占用 | ~50-100MB | ~200-500MB |
| 后端语言 | Rust（高性能、内存安全） | Node.js |
| 安全模型 | 权限细粒度控制 | 相对宽松 |
| 启动速度 | 快 | 较慢 |
| 跨平台 | macOS/Windows/Linux | macOS/Windows/Linux |
| 生态成熟度 | 较新但活跃 | 成熟 |

**结论**：对于 SSH 工具这种对性能和包体有要求的应用，Tauri 是更优选择。

### 4.3 系统架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (WebView)                         │
│  ┌───────────┬────────────┬────────────┬───────────────────┐    │
│  │  Vue App  │  xterm.js  │  Monaco    │  SFTP FileManager │    │
│  │  (TDesign)│  Terminal   │  Editor    │  (Custom Vue)     │    │
│  └─────┬─────┴─────┬──────┴─────┬──────┴─────────┬─────────┘    │
│        │           │            │                │               │
│        └───────────┴────────────┴────────────────┘               │
│                          │ Tauri IPC (invoke/events)             │
├──────────────────────────┼──────────────────────────────────────┤
│                    Rust Backend (Tauri Core)                      │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │                    Command Layer                           │    │
│  │  ┌──────────┬────────────┬───────────┬────────────────┐  │    │
│  │  │Connection│  Terminal   │   SFTP    │   Settings     │  │    │
│  │  │ Manager  │  Manager   │  Manager  │   Manager      │  │    │
│  │  └────┬─────┴─────┬──────┴─────┬─────┴───────┬────────┘  │    │
│  ├───────┼───────────┼────────────┼─────────────┼────────────┤    │
│  │       │    Service Layer       │             │            │    │
│  │  ┌────┴─────┐ ┌───┴──────┐ ┌──┴────────┐ ┌─┴─────────┐ │    │
│  │  │SSH Client│ │ Session  │ │  File     │ │ Crypto    │ │    │
│  │  │ (russh)  │ │ Pool     │ │  Transfer │ │ Service   │ │    │
│  │  └────┬─────┘ └───┬──────┘ └──┬────────┘ └─┬─────────┘ │    │
│  ├───────┼───────────┼────────────┼────────────┼────────────┤    │
│  │       │    Data Layer          │            │            │    │
│  │  ┌────┴──────────────────┐  ┌──┴────────────┴─────────┐ │    │
│  │  │  SQLite + SQLCipher   │  │  File System (密钥/日志) │ │    │
│  │  └───────────────────────┘  └────────────────────────── ┘ │    │
│  └──────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### 4.4 核心模块详细设计

#### 4.4.1 连接管理器（Connection Manager）

```rust
// 数据模型
struct Connection {
    id: Uuid,
    name: String,
    host: String,
    port: u16,                    // 默认 22
    username: String,
    auth_method: AuthMethod,
    group_id: Option<Uuid>,
    tags: Vec<String>,
    color: Option<String>,
    charset: String,              // 默认 UTF-8
    keepalive_interval: u32,     // 秒
    startup_command: Option<String>,
    proxy_jump: Option<Uuid>,    // 跳板机连接ID
    created_at: DateTime,
    updated_at: DateTime,
}

enum AuthMethod {
    Password { password: EncryptedString },
    PublicKey { key_path: PathBuf, passphrase: Option<EncryptedString> },
    Agent,
    KeyboardInteractive,
}

struct ConnectionGroup {
    id: Uuid,
    name: String,
    parent_id: Option<Uuid>,     // 支持多级
    sort_order: i32,
    color: Option<String>,
    icon: Option<String>,
}
```

#### 4.4.2 SSH 会话管理

```rust
// 会话池管理
struct SessionPool {
    sessions: HashMap<Uuid, SshSession>,
    max_sessions: usize,
}

struct SshSession {
    id: Uuid,
    connection_id: Uuid,
    session: russh::client::Handle,
    channel: russh::Channel,
    status: SessionStatus,
    created_at: Instant,
    last_active: Instant,
}

enum SessionStatus {
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
}

// 自动重连策略
struct ReconnectPolicy {
    max_retries: u32,            // 默认 3
    retry_interval: Duration,    // 默认 5s
    backoff_factor: f64,         // 指数退避因子
}
```

#### 4.4.3 SFTP 文件传输

```rust
// 传输任务模型
struct TransferTask {
    id: Uuid,
    session_id: Uuid,
    transfer_type: TransferType,
    local_path: PathBuf,
    remote_path: String,
    file_size: u64,
    transferred: AtomicU64,      // 已传输字节
    status: TransferStatus,
    created_at: Instant,
}

enum TransferType {
    Upload,
    Download,
}

enum TransferStatus {
    Queued,
    InProgress,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

// 传输管理器
struct TransferManager {
    queue: VecDeque<TransferTask>,
    active_transfers: Vec<TransferTask>,
    max_concurrent: usize,       // 最大并发传输数，默认 3
    history: Vec<TransferTask>,
}
```

#### 4.4.4 加密存储

```rust
// 主密码派生
fn derive_key(master_password: &str, salt: &[u8]) -> Key {
    // Argon2id 密钥派生
    argon2id(master_password, salt, iterations=3, memory=65536)
}

// 数据加密
fn encrypt(plaintext: &[u8], key: &Key) -> EncryptedData {
    // AES-256-GCM 加密
    let nonce = generate_random_nonce();
    let ciphertext = aes_256_gcm_encrypt(plaintext, key, nonce);
    EncryptedData { nonce, ciphertext }
}
```

### 4.5 前端架构

```
src/
├── main.ts                     # 应用入口
├── App.vue                     # 根组件
├── assets/                     # 静态资源
├── styles/                     # 全局样式
│   ├── variables.css           # CSS 变量（主题色）
│   └── themes/                 # 主题文件
├── components/                 # 通用组件
│   ├── layout/                 # 布局组件
│   │   ├── Sidebar.vue
│   │   ├── TabBar.vue
│   │   └── StatusBar.vue
│   ├── terminal/               # 终端相关
│   │   ├── TerminalView.vue
│   │   ├── TerminalTabs.vue
│   │   └── SplitPane.vue
│   ├── sftp/                   # SFTP相关
│   │   ├── FileExplorer.vue
│   │   ├── FileList.vue
│   │   ├── TransferQueue.vue
│   │   └── FileEditor.vue
│   ├── connection/             # 连接管理
│   │   ├── ConnectionForm.vue
│   │   ├── ConnectionTree.vue
│   │   └── GroupManager.vue
│   └── common/                 # 公共组件
│       ├── SearchInput.vue
│       ├── ContextMenu.vue
│       └── ConfirmDialog.vue
├── composables/                # 组合式函数
│   ├── useSSH.ts              # SSH 连接逻辑
│   ├── useSFTP.ts             # SFTP 操作逻辑
│   ├── useTerminal.ts         # 终端管理逻辑
│   └── useTheme.ts            # 主题切换逻辑
├── stores/                     # Pinia 状态管理
│   ├── connection.ts          # 连接数据
│   ├── session.ts             # 会话状态
│   ├── transfer.ts            # 传输队列
│   └── settings.ts            # 全局设置
├── views/                      # 页面视图
│   ├── MainView.vue           # 主界面
│   ├── SettingsView.vue       # 设置页
│   └── AboutView.vue          # 关于页
├── services/                   # Tauri IPC 调用封装
│   ├── ssh.ts
│   ├── sftp.ts
│   ├── storage.ts
│   └── crypto.ts
├── types/                      # TypeScript 类型定义
│   ├── connection.ts
│   ├── session.ts
│   └── transfer.ts
└── utils/                      # 工具函数
    ├── format.ts
    ├── validators.ts
    └── constants.ts
```

### 4.6 IPC 通信设计

前后端通过 Tauri 的 `invoke` (命令调用) 和 `events` (事件流) 通信：

```typescript
// 命令调用（请求-响应模式）
invoke('create_connection', { connection: ConnectionData })
invoke('connect', { connectionId: string })
invoke('disconnect', { sessionId: string })
invoke('sftp_list_dir', { sessionId: string, path: string })
invoke('sftp_upload', { sessionId: string, localPath: string, remotePath: string })

// 事件监听（流式数据）
listen('terminal-output', (event) => { /* 终端数据流 */ })
listen('transfer-progress', (event) => { /* 传输进度 */ })
listen('session-status-change', (event) => { /* 连接状态变化 */ })
listen('sftp-event', (event) => { /* SFTP 操作事件 */ })
```

### 4.7 数据存储设计

#### 4.7.1 SQLite 数据表

```sql
-- 连接表
CREATE TABLE connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER DEFAULT 22,
    username TEXT NOT NULL,
    auth_method TEXT NOT NULL,        -- JSON
    group_id TEXT,
    tags TEXT,                        -- JSON array
    color TEXT,
    charset TEXT DEFAULT 'UTF-8',
    keepalive_interval INTEGER DEFAULT 60,
    startup_command TEXT,
    proxy_jump_id TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (group_id) REFERENCES groups(id),
    FOREIGN KEY (proxy_jump_id) REFERENCES connections(id)
);

-- 分组表
CREATE TABLE groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id TEXT,
    sort_order INTEGER DEFAULT 0,
    color TEXT,
    icon TEXT,
    FOREIGN KEY (parent_id) REFERENCES groups(id)
);

-- 密钥表
CREATE TABLE ssh_keys (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    key_type TEXT NOT NULL,          -- rsa/ed25519/ecdsa
    private_key_path TEXT,
    public_key TEXT,
    passphrase TEXT,                 -- 加密存储
    created_at TEXT NOT NULL
);

-- 快捷命令表
CREATE TABLE snippets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    category TEXT,
    description TEXT,
    sort_order INTEGER DEFAULT 0
);

-- 连接历史表
CREATE TABLE connection_history (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    connected_at TEXT NOT NULL,
    disconnected_at TEXT,
    duration INTEGER,               -- 秒
    FOREIGN KEY (connection_id) REFERENCES connections(id)
);

-- 设置表
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL              -- JSON
);
```

#### 4.7.2 文件系统存储

```
~/.easyssh/                         # 应用数据目录
├── config/
│   ├── settings.json              # 非敏感全局配置
│   └── keybindings.json           # 快捷键配置
├── data/
│   ├── easyssh.db                 # SQLCipher 加密数据库
│   └── easyssh.db.bak            # 数据库备份
├── keys/                          # 密钥文件存储
│   ├── id_rsa_server1
│   └── id_ed25519_server2
├── logs/                          # 会话日志
│   ├── 2026-05-08_server-a.log
│   └── 2026-05-08_server-b.log
├── themes/                        # 自定义主题
│   └── my-theme.json
└── temp/                          # 临时文件（编辑中的远程文件）
    └── tmp_xxxx.conf
```

### 4.8 安全架构

```
┌─────────────────────────────────────┐
│          应用层安全                   │
│  • 输入验证和清洗                     │
│  • 最小权限原则                       │
│  • 敏感数据内存擦除                   │
├─────────────────────────────────────┤
│          数据层安全                   │
│  • SQLCipher 数据库加密              │
│  • AES-256-GCM 字段级加密            │
│  • Argon2id 密钥派生                 │
│  • 安全随机数生成                     │
├─────────────────────────────────────┤
│          传输层安全                   │
│  • SSH 协议加密通道                   │
│  • Host Key 验证（TOFU 模型）        │
│  • 支持强加密算法优先                 │
├─────────────────────────────────────┤
│          系统层安全                   │
│  • Tauri 权限沙箱                    │
│  • 进程隔离                          │
│  • 系统密钥链集成（macOS Keychain）   │
└─────────────────────────────────────┘
```

---

## 五、开发计划（建议）

### 5.1 里程碑规划

| 阶段 | 周期 | 目标 |
|------|------|------|
| **Phase 0**: 项目初始化 | 1 周 | Tauri + Vue3 脚手架，CI/CD 流水线 |
| **Phase 1**: 核心连接 | 3 周 | SSH 连接（密码+密钥），基本终端 |
| **Phase 2**: 连接管理 | 2 周 | 分组、搜索、导入/导出 |
| **Phase 3**: 终端增强 | 2 周 | 分屏、主题、快捷命令、日志 |
| **Phase 4**: SFTP | 3 周 | 双栏浏览器、上传下载、传输队列 |
| **Phase 5**: 安全加固 | 2 周 | 主密码、加密存储、密钥管理 |
| **Phase 6**: 打磨发布 | 2 周 | UI 打磨、性能优化、打包发布 |

**总计**：约 15 周（3.5 个月）

### 5.2 技术风险与对策

| 风险 | 影响 | 对策 |
|------|------|------|
| xterm.js 与 Tauri WebView 兼容性 | 高 | 早期验证 PoC，备选 WebView2 |
| russh 库功能不完善 | 中 | 备选 libssh2 Rust binding |
| 跨平台 UI 一致性 | 中 | 严格使用 CSS 变量 + 多平台 CI 测试 |
| 大文件传输性能 | 中 | 分块传输 + 内存映射文件 |
| SQLCipher 集成复杂度 | 低 | 使用 `rusqlite` + 编译期链接 |

---

## 六、附录

### 6.1 参考技术文档

- [Tauri 2.0 官方文档](https://v2.tauri.app/)
- [russh - Rust SSH 库](https://github.com/warp-tech/russh)
- [xterm.js 文档](https://xtermjs.org/)
- [SQLCipher](https://www.zetetic.net/sqlcipher/)
- [TDesign Vue Next](https://tdesign.tencent.com/vue-next/)

### 6.2 名词解释

| 术语 | 说明 |
|------|------|
| SSH | Secure Shell，加密的远程登录协议 |
| SFTP | SSH File Transfer Protocol，基于 SSH 的文件传输协议 |
| ProxyJump | SSH 跳板机，通过中间服务器连接目标机器 |
| SSH Agent | 系统级 SSH 密钥代理服务 |
| TOFU | Trust On First Use，首次信任模型 |
| Keep-Alive | 保活心跳包，防止连接超时断开 |
| SQLCipher | SQLite 的加密扩展 |

---

> 📝 **文档状态**：初稿，待评审
>
> **下一步**：确认技术选型和核心功能优先级后，进入 Phase 0 开发
