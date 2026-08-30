# WebSocket 协议

## 连接

### 连接地址

```
ws(s)://{host}:{port}/api/v1/agent/ws?node_id={node_id}
```

### 连接参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `node_id` | string | 是 | 节点唯一标识（注册时获取） |
| `token` | string | 否 | Bearer Token（如主控启用鉴权） |

### 心跳机制

- 服务端每 **30 秒**发送 `ping`
- 客户端收到 `ping` 后必须在 **10 秒**内回复 `pong`
- 连续 3 次未收到 `pong`，服务端主动断开连接
- 客户端断线后 **3 秒**自动重连，指数退避（最大 30 秒）

## 消息格式

所有消息为 JSON，使用 `type` 字段区分消息类型：

```json
{
  "type": "message_type",
  "field1": "value1",
  "field2": "value2"
}
```

`type` 字段使用 `snake_case`。

## 服务端 → 客户端消息

### ping

心跳探测。

```json
{ "type": "ping" }
```

### config_changed

配置变更通知，触发客户端重新拉取配置。

```json
{ "type": "config_changed" }
```

客户端收到后应调用 `GET /api/v1/nodes/{node_id}/config.yaml` 重新拉取最新配置。

### new_task

新任务通知（可选，部分实现使用 config_changed 统一通知）。

```json
{
  "type": "new_task",
  "task_id": "uuid",
  "dispatch_id": "uuid"
}
```

### node_deleted

节点被删除通知，客户端应停止运行并清理本地状态。

```json
{ "type": "node_deleted" }
```

### delete_file

删除指定文件指令。

```json
{
  "type": "delete_file",
  "filename": "path/to/file"
}
```

## 客户端 → 服务端消息

### pong

心跳响应。

```json
{ "type": "pong" }
```

### status

节点状态上报，每 **10 秒**发送一次。

| 字段 | 类型 | 说明 |
|------|------|------|
| `active_tasks` | int | 当前活跃任务数 |
| `bytes_downloaded` | int | 累计下载字节数 |
| `busy` | bool | 是否忙碌（活跃任务数 > 0） |
| `last_error` | string/null | 最近一次错误信息 |

```json
{
  "type": "status",
  "active_tasks": 2,
  "bytes_downloaded": 1073741824,
  "busy": true,
  "last_error": null
}
```

### task_started

任务开始通知。

| 字段 | 类型 | 说明 |
|------|------|------|
| `dispatch_id` | string | 调度实例标识 |

```json
{
  "type": "task_started",
  "dispatch_id": "uuid"
}
```

### task_progress

任务进度上报（可选，高频场景使用）。

| 字段 | 类型 | 说明 |
|------|------|------|
| `dispatch_id` | string | 调度实例标识 |
| `downloaded_bytes` | int | 已下载字节数 |
| `file_size` | int | 文件总大小 |
| `speed_mbps` | float | 当前速度（MB/s） |

### task_report

任务完成/失败详细回报。

| 字段 | 类型 | 说明 |
|------|------|------|
| `dispatch_id` | string | 调度实例标识 |
| `task_id` | string | 任务标识 |
| `task_name` | string | 任务名称 |
| `url` | string | 下载地址 |
| `filename` | string | 本地文件名 |
| `file_size` | int | 文件总大小（字节） |
| `downloaded_bytes` | int | 实际下载字节数 |
| `elapsed_secs` | float | 耗时（秒） |
| `avg_speed_mbps` | float | 平均速度（MB/s） |
| `status` | string | `success` / `failed` / `skipped` |
| `success_chunks` | int | 成功分片数 |
| `failed_chunks` | int | 失败分片数 |
| `error_msg` | string/null | 失败原因（成功为 null） |

```json
{
  "type": "task_report",
  "dispatch_id": "uuid",
  "task_id": "uuid",
  "task_name": "Ubuntu 22.04 ISO",
  "url": "https://example.com/ubuntu.iso",
  "filename": "ubuntu.iso",
  "file_size": 1572864000,
  "downloaded_bytes": 1572864000,
  "elapsed_secs": 120.5,
  "avg_speed_mbps": 12.4,
  "status": "success",
  "success_chunks": 8,
  "failed_chunks": 0,
  "error_msg": null
}
```

## 兼容说明

以下消息类型为旧协议兼容，新实现不应使用，标记为 `deprecated`：

- `register` — 使用 HTTP `POST /api/v1/agent/register` 替代
- `heartbeat` — 使用 HTTP `POST /api/v1/agent/heartbeat` 替代
