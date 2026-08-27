# 数据格式标准

## JSON 字段命名

- 使用 **snake_case**（小写字母 + 下划线）
- 禁止使用 camelCase 或 kebab-case

```json
✅ { "task_id": "xxx", "node_name": "node-1", "created_at": "..." }
❌ { "taskId": "xxx", "nodeName": "node-1", "createdAt": "..." }
```

## 时间格式

### 标准格式

所有时间使用 **RFC3339 / ISO 8601** 格式，UTC 时区，带 `Z` 后缀：

```
2026-08-27T10:30:00Z
2026-08-27T10:30:00.123Z  // 可选毫秒
```

### 禁止格式

```
❌ 2026-08-27 10:30:00          // 无时区，空格分隔
❌ 2026/08/27 10:30:00          // 斜杠分隔
❌ 1693122600                     // Unix 时间戳（仅内部使用，不对外）
❌ 2026-08-27T10:30:00+08:00    // 带时区偏移（统一用 UTC）
```

### 前端展示

前端可根据用户本地时区转换展示，但 API 传输必须用 UTC。

## 字节与数据量

### 存储与传输

- 内部存储和 API 传输使用**字节（bytes）**整数
- 字段名以 `_bytes` 结尾

```json
{
  "file_size_bytes": 1073741824,
  "downloaded_bytes": 536870912,
  "speed_bps": 10485760
}
```

### 前端展示

前端转换为人类可读格式：

| 单位 | 换算 |
|------|------|
| B | 1 byte |
| KB | 1024 bytes |
| MB | 1024 KB |
| GB | 1024 MB |
| TB | 1024 GB |

速度单位：`KB/s`、`MB/s`、`GB/s`

## ID 格式

### 资源 ID

所有资源 ID 使用 **UUID v4**，字符串格式（小写）：

```
550e8400-e29b-41d4-a716-446655440000
```

### 字段命名

| 资源 | 字段名 |
|------|--------|
| 任务 | `task_id` |
| 节点 | `node_id` |
| 调度 | `dispatch_id` |
| 工作流 | `workflow_id` |
| 运行记录 | `run_id` |

## 枚举值

- 使用 **snake_case** 小写
- 状态枚举使用过去式或状态名

```json
✅ "status": "running"
✅ "state": "pending"
❌ "status": "Running"
❌ "status": "RUNNING"
```

### 任务状态

| 值 | 说明 |
|----|------|
| `pending` | 待下发 |
| `acked` | 已确认领取 |
| `running` | 执行中 |
| `success` | 成功 |
| `failed` | 失败 |
| `cancelled` | 已取消 |

### 节点状态

| 值 | 说明 |
|----|------|
| `online` | 在线 |
| `offline` | 离线 |

## 布尔值

使用 JSON `true` / `false`，不使用 0/1 或字符串：

```json
✅ { "enabled": true }
❌ { "enabled": 1 }
❌ { "enabled": "true" }
```

## 数字

- 整数使用 `integer`
- 浮点数使用 `number`，保留精度由业务决定
- 百分比使用 0-100 的浮点数，字段名以 `_percent` 结尾

```json
{
  "retry_count": 3,
  "progress_percent": 75.5,
  "speed_bps": 10485760.5
}
```

## 空值与缺失

- 明确的空值使用 `null`
- 可选字段缺失时不返回该字段（不要返回空字符串）
- 数组为空时返回 `[]`，不返回 `null`

```json
✅ { "name": "test", "description": null, "tags": [] }
❌ { "name": "test", "description": "", "tags": null }
❌ { "name": "test" }  // 如果 description 是必填，不能缺失
```

## 分页响应

```json
{
  "code": 0,
  "data": {
    "items": [ ... ],
    "total": 100,
    "page": 1,
    "page_size": 20,
    "total_pages": 5
  },
  "message": "ok"
}
```

## 列表排序

- 时间相关列表默认按时间倒序（最新在前）
- 可通过 `sort` + `order` 参数自定义

## 大数字处理

- 超过 2^53 的数字使用字符串传输，避免 JavaScript 精度丢失
- 如：`"total_bytes": "9223372036854775807"`

## 密码与敏感信息

- 密码、Token 等敏感信息**永不返回**
- 如需返回，使用脱敏格式：`"password": "******"`
- API Token 只在创建时返回一次，后续查询不返回

## 示例：完整的任务对象

```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "iPhone iOS 15.3 固件",
  "filename": "iPhone12,1_15.3_19D50_Restore.ipsw",
  "url": "https://updates-http.cdn-apple.com/...",
  "enabled": true,
  "file_size_bytes": 6442450944,
  "created_at": "2026-08-27T09:00:00Z",
  "updated_at": "2026-08-27T09:30:00Z",
  "note": null,
  "tags": ["ios", "firmware"],
  "config": {
    "connections_per_file": 8,
    "retry_times": 3,
    "timeout_secs": 1800,
    "max_concurrent": 4,
    "save_path": "/tmp",
    "skip_tls_verify": false,
    "dry_run": false
  }
}
```

## 示例：节点实时状态

```json
{
  "node_id": "a98d87c3-4b5d-4e6f-8a9b-0c1d2e3f4a5b",
  "hostname": "node-1",
  "platform": "linux-x86_64",
  "version": "0.6.1",
  "status": "online",
  "total_speed_bps": 104857600,
  "active_task_count": 2,
  "last_heartbeat_at": "2026-08-27T10:30:00Z",
  "active_tasks_progress": [
    {
      "dispatch_id": "11111111-...",
      "task_name": "iPhone iOS 15.3 固件",
      "percent": 75.5,
      "downloaded_bytes": 4865392640,
      "total_size_bytes": 6442450944,
      "speed_bps": 52428800,
      "active_connections": 8,
      "elapsed_secs": 120.5,
      "updated_at": "2026-08-27T10:30:00Z"
    }
  ]
}
```
