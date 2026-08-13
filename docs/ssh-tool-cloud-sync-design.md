# EasySSH — 云同步模块设计文档

> 模块代号：**Cloud Sync**
> 文档版本：v1.0
> 创建时间：2026-05-08

---

## 一、模块概述

### 1.1 功能定位

云同步模块为 EasySSH 提供跨设备数据同步能力，用户可以在多台 Mac/Windows 设备间无缝同步连接配置、分组、快捷命令、主题等数据，实现"一次配置，处处可用"。

### 1.2 同步范围

| 数据类型 | 是否同步 | 说明 |
|---------|---------|------|
| 连接配置 | ✅ | 主机、端口、用户名、认证方式等 |
| 密码/密钥密码短语 | ✅（加密） | 端到端加密，服务端不可读 |
| 连接分组 | ✅ | 分组结构和排序 |
| 快捷命令（Snippets） | ✅ | 全局命令片段 |
| 终端主题 | ✅ | 自定义主题配置 |
| 快捷键配置 | ✅ | 自定义快捷键方案 |
| 全局设置 | ✅ | 偏好设置 |
| 私钥文件 | ⚠️ 可选 | 用户可选择是否同步私钥文件（加密传输） |
| 会话日志 | ❌ | 仅本地保存 |
| 传输历史 | ❌ | 仅本地保存 |
| 连接历史 | ⚠️ 可选 | 用户可选择是否同步 |

---

## 二、功能需求说明

### 2.1 用户账号系统

#### 2.1.1 注册/登录方式

| 方式 | 优先级 | 说明 |
|------|--------|------|
| 邮箱 + 密码 | P0 | 基础注册登录 |
| GitHub OAuth | P0 | 第三方快捷登录 |
| Google OAuth | P1 | 第三方快捷登录 |
| 微信扫码 | P1 | 国内用户便捷 |
| Apple ID | P2 | macOS 用户体验 |

#### 2.1.2 账号功能

- 邮箱验证
- 密码找回（邮件重置链接）
- 两步验证（TOTP，如 Google Authenticator）
- 设备管理（查看已登录设备，远程登出）
- 账号注销

### 2.2 同步机制

#### 2.2.1 同步策略

| 策略 | 说明 |
|------|------|
| 实时同步 | 数据变更后自动上传（防抖 3 秒） |
| 手动同步 | 用户主动触发同步 |
| 启动同步 | 应用启动时自动拉取最新数据 |
| 离线缓冲 | 离线时变更暂存本地，恢复网络后自动同步 |

#### 2.2.2 冲突解决策略

```
场景：设备A 和 设备B 同时修改了同一个连接配置

策略：
1. 默认策略：最后写入胜出（Last Write Wins）
   - 基于时间戳比较，最新修改覆盖旧的
   
2. 冲突提示（可选开启）：
   - 检测到冲突时弹窗提示用户选择
   - 显示两个版本的差异
   - 用户选择保留哪个版本或合并

3. 字段级合并：
   - 同一连接的不同字段可分别合并
   - 如 A 改了端口，B 改了用户名，两者自动合并
```

#### 2.2.3 版本历史

- 每次同步保留变更快照
- 最近 30 天 / 最近 100 个版本（以先到为准）
- 支持回滚到任意历史版本
- 支持查看某个连接的变更历史

### 2.3 端到端加密（E2E Encryption）

#### 2.3.1 加密设计原则

```
核心原则：零知识架构（Zero-Knowledge）
- 服务端永远无法解密用户数据
- 加密/解密完全在客户端进行
- 即使服务端被攻破，用户数据依然安全
```

#### 2.3.2 加密流程

```
用户设置同步密码（Sync Password）
         │
         ▼
  Argon2id 派生密钥（Sync Key）
         │
         ▼
  本地数据 JSON 序列化
         │
         ▼
  AES-256-GCM 加密
         │
         ▼
  加密数据块上传到云端
         │
         ▼
  云端存储（不可读的密文）
```

#### 2.3.3 同步密码

- 用户首次开启同步时设置"同步密码"
- 同步密码 ≠ 账号密码（独立的加密密码）
- 同步密码不上传服务器，仅存在于本地
- 新设备登录时需要输入同步密码才能解密数据
- **忘记同步密码 = 无法恢复云端加密数据**（零知识代价）
- 支持设置恢复提示问题（可选）

### 2.4 设备管理

#### 2.4.1 多设备管理

| 功能 | 说明 |
|------|------|
| 设备列表 | 显示所有已登录设备（名称、系统、最后同步时间） |
| 设备命名 | 用户自定义设备别名 |
| 远程登出 | 撤销某设备的登录状态 |
| 同步状态 | 每台设备的同步进度和最后同步时间 |
| 设备数量限制 | 免费版最多 3 台，付费版不限 |

#### 2.4.2 选择性同步

- 用户可选择哪些分组/连接参与同步
- 某些连接可标记为"仅本地"（不同步）
- 私钥文件同步需用户明确开启

### 2.5 数据配额与计划

| 计划 | 同步设备数 | 存储配额 | 同步频率 | 版本历史 |
|------|-----------|---------|---------|---------|
| 免费版 | 3 台 | 10MB | 每 5 分钟 | 7 天 |
| 专业版 | 不限 | 100MB | 实时 | 30 天 |
| 团队版 | 不限 | 1GB/人 | 实时 | 90 天 |

### 2.6 团队协作（未来扩展）

- 团队空间：共享连接配置给团队成员
- 角色权限：管理员/成员/只读
- 共享分组：特定分组可设为团队共享
- 审计日志：记录谁在什么时候修改了什么
- 敏感信息脱敏：共享连接时密码不共享，成员需自行填写

---

## 三、交互设计说明

### 3.1 首次开启同步流程

```
┌─────────────────────────────────────────────────┐
│              开启云同步                            │
│                                                  │
│  ☁️  在多台设备之间无缝同步你的连接配置            │
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │  🔐 端到端加密保护                         │   │
│  │  服务端无法读取你的任何数据                   │   │
│  └──────────────────────────────────────────┘   │
│                                                  │
│  选择登录方式：                                   │
│                                                  │
│  [  📧 邮箱登录/注册  ]                          │
│  [  🐱 GitHub 登录    ]                          │
│  [  🔵 Google 登录    ]                          │
│                                                  │
└─────────────────────────────────────────────────┘
                    │
                    ▼ (登录成功后)
┌─────────────────────────────────────────────────┐
│          设置同步密码                              │
│                                                  │
│  同步密码用于加密你的云端数据                       │
│  ⚠️ 请牢记此密码，忘记将无法恢复数据              │
│                                                  │
│  同步密码: [••••••••••••]                        │
│  确认密码: [••••••••••••]                        │
│                                                  │
│  密码强度: ████████░░ 强                         │
│                                                  │
│  □ 设置恢复提示问题（推荐）                       │
│                                                  │
│  [  取消  ]              [  开启同步  ]           │
└─────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────┐
│          选择同步内容                              │
│                                                  │
│  ☑ 连接配置（主机、端口、用户名等）               │
│  ☑ 密码和密钥密码短语（加密存储）                 │
│  ☑ 连接分组和排序                                │
│  ☑ 快捷命令                                     │
│  ☑ 主题和外观设置                                │
│  ☑ 快捷键配置                                   │
│  □ 私钥文件（谨慎：大文件会占用配额）             │
│  □ 连接历史记录                                  │
│                                                  │
│  [  完成设置  ]                                  │
└─────────────────────────────────────────────────┘
```

### 3.2 新设备加入同步流程

```
新设备首次登录
      │
      ▼
┌─────────────────────────────────────────────────┐
│          检测到云端数据                            │
│                                                  │
│  ☁️  发现已有同步数据（来自 MacBook Pro）          │
│                                                  │
│  请输入同步密码以解密数据：                        │
│                                                  │
│  同步密码: [____________]                        │
│                                                  │
│  [忘记密码?]                                     │
│                                                  │
│  ────────── 或 ──────────                       │
│                                                  │
│  [ 作为全新设备开始（不拉取云端数据）]             │
│                                                  │
│  [  解密并同步  ]                                │
└─────────────────────────────────────────────────┘
```

### 3.3 同步状态指示

```
侧边栏底部:
┌──────────────────────┐
│ ☁️ 已同步 · 刚刚      │  ← 正常状态
│ 🔄 同步中...          │  ← 同步进行中
│ ⚠️ 同步失败 · 点击重试 │  ← 同步出错
│ 📴 离线 · 3项待同步    │  ← 离线状态
└──────────────────────┘
```

### 3.4 设置页面 - 同步选项卡

```
┌─────────────────────────────────────────────────────────────┐
│  设置 > 云同步                                                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  账号信息                                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  👤 avin@example.com              [登出]           │     │
│  │  📱 已登录设备: 3/3                [管理设备]       │     │
│  │  💾 存储已用: 2.3MB / 10MB                          │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  同步设置                                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  自动同步     [开启 ●───]                           │     │
│  │  同步频率     [实时 ▾]                              │     │
│  │  WiFi 才同步  [关闭 ───●]                           │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  同步内容                                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  ☑ 连接配置     ☑ 快捷命令     ☑ 主题设置          │     │
│  │  ☑ 密码信息     ☑ 快捷键       □ 私钥文件          │     │
│  │  ☑ 分组信息     □ 连接历史                          │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  安全                                                        │
│  ┌────────────────────────────────────────────────────┐     │
│  │  [修改同步密码]    [修改恢复问题]                    │     │
│  │  [查看版本历史]    [下载云端备份]                    │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  危险操作                                                    │
│  ┌────────────────────────────────────────────────────┐     │
│  │  [清除云端数据]    [解除所有设备绑定]                │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│  最后同步: 2026-05-08 11:28:35  ·  [立即同步]              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 3.5 冲突解决界面

```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️ 同步冲突                                                 │
│                                                              │
│  以下配置在多台设备上被同时修改，请选择保留版本：             │
│                                                              │
│  连接: Production Web Server                                 │
│  ┌──────────────────┬──────────────────────────────┐       │
│  │ 本地版本 (MacBook)│ 云端版本 (Windows PC)        │       │
│  │ 修改于 11:25      │ 修改于 11:23                │       │
│  ├──────────────────┼──────────────────────────────┤       │
│  │ 端口: 2222       │ 端口: 22                    │       │
│  │ 用户名: deploy   │ 用户名: root                │       │
│  │ 备注: 新端口      │ 备注: (无变更)              │       │
│  └──────────────────┴──────────────────────────────┘       │
│                                                              │
│  [保留本地版本]  [保留云端版本]  [手动合并]                  │
│                                                              │
│  □ 后续冲突自动使用"最新修改优先"策略                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 四、技术架构设计

### 4.1 云端服务架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         Client (EasySSH)                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Sync Engine                                             │    │
│  │  ├── Change Detector (监听本地数据变更)                    │    │
│  │  ├── Conflict Resolver (冲突检测与解决)                    │    │
│  │  ├── E2E Encryptor (端到端加密/解密)                      │    │
│  │  ├── Delta Calculator (增量计算)                          │    │
│  │  └── Offline Queue (离线队列)                             │    │
│  └──────────────────────────┬──────────────────────────────┘    │
│                              │ HTTPS + WebSocket                  │
└──────────────────────────────┼──────────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────────┐
│                        Cloud Services                              │
│                                                                    │
│  ┌────────────┐   ┌────────────┐   ┌────────────────────────┐   │
│  │  API       │   │  Auth      │   │  WebSocket             │   │
│  │  Gateway   │──▶│  Service   │   │  Server                │   │
│  │  (Nginx)   │   │  (JWT)     │   │  (实时推送)             │   │
│  └─────┬──────┘   └────────────┘   └────────────────────────┘   │
│        │                                                          │
│  ┌─────▼──────────────────────────────────────────────────────┐  │
│  │                   Sync API Service                          │  │
│  │  ├── /api/v1/sync/push    (上传变更)                        │  │
│  │  ├── /api/v1/sync/pull    (拉取变更)                        │  │
│  │  ├── /api/v1/sync/status  (同步状态)                        │  │
│  │  ├── /api/v1/devices      (设备管理)                        │  │
│  │  └── /api/v1/history      (版本历史)                        │  │
│  └─────┬──────────────────────────────────────────────────────┘  │
│        │                                                          │
│  ┌─────▼──────┐   ┌────────────┐   ┌────────────────────────┐   │
│  │ PostgreSQL │   │   Redis    │   │  Object Storage        │   │
│  │ (元数据)    │   │  (缓存/     │   │  (加密数据块)           │   │
│  │            │   │   会话)     │   │  (S3/COS/MinIO)       │   │
│  └────────────┘   └────────────┘   └────────────────────────┘   │
│                                                                    │
└──────────────────────────────────────────────────────────────────┘
```

### 4.2 同步协议设计

#### 4.2.1 增量同步协议

```
同步流程：

1. 客户端记录变更日志（Change Log）
   每次本地数据修改生成一条变更记录：
   {
     change_id: "uuid",
     entity_type: "connection",    // connection/group/snippet/settings
     entity_id: "uuid",
     action: "update",             // create/update/delete
     fields_changed: ["port", "username"],
     timestamp: 1715150000,
     device_id: "device-uuid"
   }

2. Push 阶段（上传变更）
   - 客户端收集自上次同步以来的所有变更
   - 加密变更数据
   - POST /api/v1/sync/push
   {
     device_id: "...",
     last_sync_version: 42,
     changes: [ encrypted_change_1, encrypted_change_2, ... ]
   }

3. Pull 阶段（拉取变更）
   - GET /api/v1/sync/pull?since_version=42
   - 服务端返回该版本之后的所有其他设备的变更
   - 客户端解密并应用变更

4. 冲突检测
   - 服务端基于 entity_id + timestamp 检测冲突
   - 返回冲突列表给客户端决策
```

#### 4.2.2 实时推送（WebSocket）

```typescript
// WebSocket 事件
interface SyncEvent {
  type: 'sync_available' | 'device_joined' | 'device_left' | 'force_logout';
  payload: {
    version: number;          // 最新版本号
    source_device: string;    // 来源设备
    summary: string;          // 变更摘要
  };
}

// 客户端收到 sync_available 事件后触发增量拉取
```

### 4.3 端到端加密实现

#### 4.3.1 密钥体系

```
Sync Password (用户输入)
       │
       ▼ Argon2id(password, salt, t=3, m=64MB)
Master Key (256-bit)
       │
       ├──▶ Data Encryption Key (DEK) — 加密实际数据
       │     └── 每次同步生成新的随机 DEK
       │
       └──▶ Key Encryption Key (KEK) — 加密 DEK
             └── 用 Master Key 派生
```

#### 4.3.2 加密数据格式

```rust
struct EncryptedSyncPayload {
    version: u8,                    // 加密协议版本
    salt: [u8; 32],                // Argon2 salt
    kek_nonce: [u8; 12],           // KEK 加密 DEK 的 nonce
    encrypted_dek: Vec<u8>,        // 加密后的 DEK
    dek_nonce: [u8; 12],           // DEK 加密数据的 nonce
    encrypted_data: Vec<u8>,       // 加密后的数据
    auth_tag: [u8; 16],           // GCM 认证标签
}
```

#### 4.3.3 客户端加密流程（Rust）

```rust
// 加密同步数据
fn encrypt_sync_data(plaintext: &[u8], sync_password: &str) -> Result<EncryptedSyncPayload> {
    // 1. 从同步密码派生 Master Key
    let salt = generate_random_bytes(32);
    let master_key = argon2id_derive(sync_password, &salt, 3, 65536)?;
    
    // 2. 生成随机 DEK
    let dek = generate_random_bytes(32);
    
    // 3. 用 Master Key 加密 DEK
    let kek_nonce = generate_random_bytes(12);
    let encrypted_dek = aes_256_gcm_encrypt(&dek, &master_key, &kek_nonce)?;
    
    // 4. 用 DEK 加密实际数据
    let dek_nonce = generate_random_bytes(12);
    let encrypted_data = aes_256_gcm_encrypt(plaintext, &dek, &dek_nonce)?;
    
    Ok(EncryptedSyncPayload { version: 1, salt, kek_nonce, encrypted_dek, dek_nonce, encrypted_data, .. })
}
```

### 4.4 云端服务技术栈

| 组件 | 技术选择 | 说明 |
|------|---------|------|
| API 服务 | **Rust (Axum)** 或 **Go (Gin)** | 高性能，低资源占用 |
| 认证 | **JWT + Refresh Token** | 无状态认证 |
| OAuth | GitHub / Google OAuth2 | 第三方登录 |
| 数据库 | **PostgreSQL** | 用户信息、同步元数据 |
| 缓存 | **Redis** | 会话、限流、WebSocket pub/sub |
| 对象存储 | **S3 / 腾讯云 COS / MinIO** | 存储加密数据块 |
| WebSocket | **Axum WebSocket** | 实时同步通知 |
| 部署 | **Docker + K8s** | 容器化部署 |
| CDN | **CloudFlare** | API 加速 + DDoS 防护 |

### 4.5 数据库设计（云端 PostgreSQL）

```sql
-- 用户表
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE,
    password_hash VARCHAR(255),         -- bcrypt
    oauth_provider VARCHAR(50),         -- github/google/wechat
    oauth_id VARCHAR(255),
    plan VARCHAR(20) DEFAULT 'free',    -- free/pro/team
    totp_secret VARCHAR(255),           -- 2FA
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 设备表
CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    device_name VARCHAR(255),
    os_type VARCHAR(20),               -- macos/windows
    os_version VARCHAR(50),
    app_version VARCHAR(20),
    last_sync_at TIMESTAMPTZ,
    last_active_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 同步版本表
CREATE TABLE sync_versions (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    device_id UUID REFERENCES devices(id),
    version BIGINT NOT NULL,
    change_summary JSONB,              -- 变更摘要（非敏感）
    data_key VARCHAR(500),             -- 对象存储中的 key
    data_size INTEGER,                 -- 字节数
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_sync_versions_user_version ON sync_versions(user_id, version);
CREATE INDEX idx_devices_user ON devices(user_id);

-- 用户配额使用表
CREATE TABLE usage_stats (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    device_count INTEGER DEFAULT 0,
    storage_used BIGINT DEFAULT 0,    -- 字节
    sync_count_today INTEGER DEFAULT 0,
    last_reset_date DATE DEFAULT CURRENT_DATE
);
```

### 4.6 API 接口设计

#### 4.6.1 认证接口

```
POST   /api/v1/auth/register          # 注册
POST   /api/v1/auth/login             # 登录
POST   /api/v1/auth/logout            # 登出
POST   /api/v1/auth/refresh           # 刷新 Token
POST   /api/v1/auth/oauth/github      # GitHub OAuth 回调
POST   /api/v1/auth/oauth/google      # Google OAuth 回调
POST   /api/v1/auth/forgot-password   # 忘记密码
POST   /api/v1/auth/reset-password    # 重置密码
POST   /api/v1/auth/verify-email      # 验证邮箱
POST   /api/v1/auth/2fa/enable        # 开启 2FA
POST   /api/v1/auth/2fa/verify        # 验证 2FA
```

#### 4.6.2 同步接口

```
POST   /api/v1/sync/push              # 上传变更
GET    /api/v1/sync/pull              # 拉取变更 (?since_version=N)
GET    /api/v1/sync/status            # 同步状态
POST   /api/v1/sync/full-upload       # 全量上传（首次/重置）
GET    /api/v1/sync/full-download     # 全量下载
GET    /api/v1/sync/history           # 版本历史列表
GET    /api/v1/sync/history/:version  # 获取某版本数据
POST   /api/v1/sync/rollback          # 回滚到某版本
DELETE /api/v1/sync/data              # 清除云端数据
```

#### 4.6.3 设备接口

```
GET    /api/v1/devices                # 设备列表
PUT    /api/v1/devices/:id            # 更新设备信息
DELETE /api/v1/devices/:id            # 移除设备（远程登出）
```

### 4.7 客户端同步引擎（Rust）

```rust
// 同步引擎核心结构
pub struct SyncEngine {
    config: SyncConfig,
    http_client: reqwest::Client,
    ws_client: Option<WebSocketClient>,
    change_log: ChangeLog,
    conflict_resolver: ConflictResolver,
    encryptor: E2EEncryptor,
    state: SyncState,
}

pub struct SyncConfig {
    server_url: String,
    sync_interval: Duration,        // 自动同步间隔
    auto_sync: bool,
    sync_on_start: bool,
    offline_queue_max: usize,
}

pub enum SyncState {
    Idle,
    Syncing,
    Offline { pending_changes: usize },
    Error { message: String, retries: u32 },
}

impl SyncEngine {
    /// 启动同步引擎
    pub async fn start(&mut self) -> Result<()>;
    
    /// 停止同步引擎
    pub async fn stop(&mut self) -> Result<()>;
    
    /// 手动触发同步
    pub async fn sync_now(&mut self) -> Result<SyncResult>;
    
    /// 记录本地变更
    pub fn record_change(&mut self, change: DataChange);
    
    /// 处理远程变更
    async fn apply_remote_changes(&mut self, changes: Vec<EncryptedChange>) -> Result<()>;
    
    /// 解决冲突
    async fn resolve_conflicts(&mut self, conflicts: Vec<Conflict>) -> Result<Vec<Resolution>>;
}
```

---

## 五、安全考量

### 5.1 传输安全
- 所有 API 通信强制 HTTPS（TLS 1.3）
- WebSocket 使用 WSS
- 证书固定（Certificate Pinning）防中间人

### 5.2 服务端安全
- 密码使用 bcrypt（cost=12）存储
- JWT 短期有效（15min），配合 Refresh Token（7天）
- API 限流（Rate Limiting）
- IP 白名单（可选）
- 所有输入验证和清洗

### 5.3 数据安全
- 零知识架构：服务端只存储密文
- 端到端加密：AES-256-GCM
- 密钥派生：Argon2id（抗 GPU/ASIC 攻击）
- 每次加密使用唯一随机 nonce
- 删除数据时安全擦除

---

## 六、监控与运维

- 服务健康检查端点：`GET /health`
- Prometheus 指标暴露
- 同步成功/失败率监控
- 用户存储配额预警
- 异常登录检测（新设备、异常 IP）
- 自动备份策略：每日全量 + WAL 归档
