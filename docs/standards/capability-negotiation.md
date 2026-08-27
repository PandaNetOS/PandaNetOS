# 能力协商与版本兼容标准

> 所有 PandaNetOS 生态项目必须遵循本标准，确保组件间独立迭代、版本解耦、向前/向后兼容。

## 1. 设计原则

### 1.1 核心原则

| 原则 | 说明 |
|------|------|
| **能力驱动而非版本驱动** | 根据对方上报的能力决定行为，不根据版本号硬编码逻辑 |
| **版本解耦** | 各组件独立迭代版本，不需要严格版本对齐 |
| **向前兼容** | 新版本组件的新能力，旧版本组件不认识时透传，不报错 |
| **向后兼容** | 旧版本组件的接口，新版本组件必须支持 |
| **渐进式升级** | 可以先升级一端，新能力自动生效；再升级另一端，开始控制新能力 |
| **容错降级** | 版本不匹配时，认识的部分正常工作，不认识的用默认值，核心功能不受影响 |

### 1.2 版本不匹配场景

```
场景A: spde 版本高，pk 版本低
  → spde 上报新能力，pk 不认识 → 透传，spde 自己用默认值
  → pk 不报错，节点正常注册和工作

场景B: pk 版本高，spde 版本低
  → pk 下发新配置，spde 不认识 → 忽略，用自己的默认值
  → spde 不报错，正常工作

场景C: 部分节点升级，部分不升级
  → 升级的节点有新能力，没升级的没有
  → pk 根据每个节点上报的能力分别处理
```

## 2. 能力注册机制

### 2.1 注册时机

组件（如 spde 节点）在**首次启动/注册时**上报自身所有能力参数。

### 2.2 能力参数结构

使用通用 JSON 字段 `capabilities` 存储所有能力参数，**灵活扩展，不需要每次加新能力都改表结构/改结构体**。

```json
{
  "spde_version": "0.6.1",
  "supported_protocols": ["http", "https", "ftp", "bt"],
  "features": {
    "resume": true,
    "multi_connection": true,
    "retry": true,
    "proxy": true,
    "dry_run": true
  },
  "hardware": {
    "cpu_cores": 8,
    "memory_gb": 16
  },
  "config_defaults": {
    "max_concurrent": 4,
    "connections_per_file": 16,
    "retry_times": 3,
    "timeout_secs": 30,
    "resume": true
  },
  "performance_limits": {
    "max_bandwidth_bps": 104857600
  }
}
```

### 2.3 独立字段 vs 通用字段

| 类型 | 字段 | 说明 |
|------|------|------|
| **独立字段** | `max_concurrent`, `max_bandwidth_bps` | pk 需要直接查询/控制的核心参数，建独立列，方便 SQL 查询和索引 |
| **通用字段** | `capabilities` (JSON) | 其他所有能力参数，灵活扩展，不需要改表结构 |

**规则**：pk 需要在列表中展示、需要 SQL 查询、需要频繁修改的参数 → 独立字段；其他 → 通用 JSON 字段。

## 3. 能力协商流程

### 3.1 注册流程

```
spde 启动
  ↓
读取本地配置，构建 capabilities JSON
  ↓
调用 POST /api/v1/agent/register
  请求体包含: node_id, hostname, platform, arch, version, labels,
              max_concurrent, max_bandwidth_bps, capabilities(JSON)
  ↓
pk 接收注册
  ↓
存储: 独立字段(max_concurrent等) + capabilities(JSON)
  ↓
pk 不做强类型解析，不认识的字段透传保存
  ↓
返回注册成功
```

### 3.2 能力修改流程（合并模式）

pk 修改节点能力参数时，**必须使用合并模式，不能全量覆盖**：

```
pk 收到修改请求 PUT /api/v1/nodes/{id}/capabilities
  ↓
读取节点当前的 capabilities(JSON)
  ↓
合并:
  - pk 认识的字段(max_concurrent, max_bandwidth_bps) → 覆盖
  - 请求中额外的 capabilities 字段 → 合并到 JSON
  - pk 不认识的字段 → 保留原值，不修改
  ↓
保存合并后的结果
  ↓
返回成功
```

**禁止**：直接用请求体全量覆盖 capabilities，会丢失 spde 上报的其他能力参数。

### 3.3 任务下发流程

pk 给节点下发任务配置时：

```
pk 构建任务配置
  ↓
合并:
  - pk 认识的配置项 → 用 pk 的值
  - pk 不认识的配置项 → 用节点 capabilities 中的默认值
  - 节点也没有的 → 用全局默认值
  ↓
下发给节点
  ↓
节点接收配置
  ↓
节点认识的字段 → 用下发的值
节点不认识的字段 → 忽略，用自己的默认值
```

## 4. 版本兼容策略

### 4.1 接口兼容规则

| 规则 | 说明 |
|------|------|
| **新增字段必须可选** | 所有新增字段用 `Option<T>` / `#[serde(default)]`，旧版本不发送也不报错 |
| **新增字段必须有默认值** | 接收方收到不认识的字段，用合理的默认值 |
| **不删除已有字段** | 只能新增，不能删除或重命名已有字段 |
| **不修改已有字段语义** | 已有字段的含义不能变，需要新语义就加新字段 |
| **枚举值只能新增** | 状态枚举只能新增值，不能删除或修改已有值的含义 |

### 4.2 数据库兼容规则

| 规则 | 说明 |
|------|------|
| **新增列必须允许 NULL** | `ALTER TABLE ADD COLUMN` 新增的列必须允许 NULL 或有默认值 |
| **使用通用 JSON 字段扩展** | 优先用 `capabilities` JSON 字段扩展，避免频繁改表结构 |
| **启动时自动迁移** | 应用启动时检测并执行 `ALTER TABLE ADD COLUMN`，忽略已存在的错误 |
| **不删除已有列** | 只能新增列，不能删除或重命名已有列 |

### 4.3 配置兼容规则

| 规则 | 说明 |
|------|------|
| **配置文件向后兼容** | 新版本能读取旧版本的配置文件，缺失字段用默认值 |
| **配置文件向前兼容** | 旧版本能读取新版本的配置文件，不认识的字段忽略 |
| **配置合并而非覆盖** | 修改配置时合并，不全量覆盖 |

## 5. 数据结构规范

### 5.1 注册请求体规范

所有组件注册请求必须包含：

```rust
pub struct RegisterReq {
    // 基础标识
    pub node_id: Option<Uuid>,           // 节点唯一ID，None则服务端生成
    pub hostname: String,                 // 主机名
    pub platform: String,                 // 平台 (linux/windows/macos)
    pub arch: String,                     // 架构 (x86_64/aarch64)
    pub version: String,                  // 组件版本号
    pub labels: Vec<String>,              // 自定义标签

    // 核心能力参数（独立字段，pk 可直接查询/控制）
    pub max_concurrent: Option<u32>,      // 最大并发任务数
    pub max_bandwidth_bps: Option<u64>,   // 最大带宽上限 bps

    // 通用能力参数（JSON，灵活扩展）
    pub capabilities: Option<serde_json::Value>,
}
```

### 5.2 节点表规范

```sql
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    hostname TEXT,
    platform TEXT,
    arch TEXT,
    version TEXT,
    status TEXT,                        -- online/busy/offline/pending
    last_seen TEXT,
    registered_at TEXT,
    labels TEXT,
    active_tasks INTEGER,
    bytes_downloaded INTEGER,
    last_error TEXT,
    -- 核心能力参数（独立列）
    max_concurrent INTEGER,
    max_bandwidth_bps INTEGER,
    -- 通用能力参数（JSON，灵活扩展）
    capabilities TEXT
);
```

### 5.3 能力修改请求体规范

```rust
pub struct UpdateNodeCapabilitiesReq {
    // pk 认识的核心参数（可选，None则不修改）
    #[serde(default)]
    pub max_concurrent: Option<u32>,
    #[serde(default)]
    pub max_bandwidth_bps: Option<u64>,

    // 通用能力参数合并（pk 不认识的字段透传）
    #[serde(default)]
    pub capabilities: Option<serde_json::Value>,
}
```

## 6. API 规范

### 6.1 注册接口

```
POST /api/v1/agent/register
Content-Type: application/json

请求体: RegisterReq（见 5.1）
响应体: { node_id, poll_interval_secs, master_listen }
```

### 6.2 能力查询接口

```
GET /api/v1/nodes
响应体: Node[]（包含 max_concurrent, max_bandwidth_bps, capabilities）
```

### 6.3 能力修改接口

```
PUT /api/v1/nodes/{id}/capabilities
Content-Type: application/json

请求体: UpdateNodeCapabilitiesReq（见 5.3）
行为: 合并模式，只修改请求中指定的字段，其他保留原值
响应体: { success: true }
```

## 7. 状态机规范

### 7.1 节点状态

| 状态 | 说明 | 能领取任务 |
|------|------|-----------|
| `online` | 空闲，可领取任务 | ✅ |
| `busy` | 活跃任务达到 max_concurrent 上限 | ❌ |
| `offline` | 离线，心跳超时 | ❌ |
| `pending` | 待审批，被删除后再次注册 | ❌ |

### 7.2 状态转换

```
注册 → online（首次注册自动通过）
注册 → pending（被删除过的节点再次注册）
pending → online（用户点同意）
pending → pending（用户点拒绝，保持 pending，可随时再同意）
online → busy（活跃任务数 >= max_concurrent）
busy → online（活跃任务数 < max_concurrent）
任何状态 → offline（心跳超时）
```

### 7.3 busy 状态判断

节点心跳时，根据 `active_tasks` 和节点的 `max_concurrent` 判断：

```rust
let max_concurrent = node_max_concurrent.unwrap_or(global_default_max_concurrent);
let new_status = if active_tasks >= max_concurrent { "busy" } else { "online" };
```

**优先用节点自定义的 max_concurrent，没有则用全局默认值。**

## 8. 实现检查清单

新增能力参数时，检查以下各项：

- [ ] 注册请求体新增字段用 `Option<T>` / `#[serde(default)]`
- [ ] 节点表新增列允许 NULL 或有默认值
- [ ] 启动时自动执行 `ALTER TABLE ADD COLUMN` 迁移
- [ ] list_nodes 查询读取新字段
- [ ] agent_register 保存新字段
- [ ] update_capabilities 用合并模式，不全量覆盖
- [ ] 前端展示新字段（如需要）
- [ ] 前端编辑新字段（如需要）
- [ ] 文档更新本标准

## 9. 反模式（禁止）

| 反模式 | 说明 | 正确做法 |
|--------|------|---------|
| ❌ 根据版本号硬编码逻辑 | `if version >= "0.6.0" { ... }` | 根据 capabilities 判断能力 |
| ❌ 全量覆盖 capabilities | 修改时直接用请求体覆盖 | 合并模式，只改指定字段 |
| ❌ 新增字段必填 | 新增字段不加 `Option` / `default` | 所有新增字段可选，有默认值 |
| ❌ 删除/重命名已有字段 | 破坏向后兼容 | 只能新增字段 |
| ❌ 不认识的字段报错 | 收到不认识的字段就报错 | 忽略/透传，用默认值 |
| ❌ 频繁改表结构 | 每个新能力都 ALTER TABLE | 优先用 capabilities JSON 字段 |
| ❌ 要求版本严格对齐 | 必须 pk 和 spde 同版本才能工作 | 版本不匹配也能工作，降级运行 |
