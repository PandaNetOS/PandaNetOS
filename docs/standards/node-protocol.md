# 节点通信协议

## 概述

**Agent** 与主控（pk）之间通过 **HTTP REST + WebSocket** 双通道通信：

- **HTTP**：注册、心跳、配置拉取、结果回报
- **WebSocket**：实时指令下发、状态推送

所有 API 路径前缀：`/api/v1`

### 适用范围

本协议适用于**所有接入 pk 的 Agent**，不限于下载节点。当前已接入：

| Agent | 职责 | 说明 |
|-------|------|------|
| `spde` | 下载执行 | 下载节点，领取下载任务并回写结果 |
| `PeerDiscoveryCenter` | Peer 发现 | 提供 Tracker + DHT + PEX 的 peer 发现能力 |

未来新增 Agent 遵循同一协议即可接入，主控侧无需改造。

## 节点注册

### 请求

```
POST /api/v1/agent/register
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `hostname` | string | 是 | 主机名 |
| `platform` | string | 是 | 操作系统（linux / windows / darwin） |
| `arch` | string | 是 | 架构（x86_64 / aarch64） |
| `version` | string | 是 | spde 版本号 |
| `capabilities` | object | 否 | 能力清单（Capability Manifest） |

```json
{
  "hostname": "node-01",
  "platform": "linux",
  "arch": "x86_64",
  "version": "1.1.1",
  "capabilities": { ... }
}
```

### 响应

```json
{
  "code": 0,
  "data": {
    "node_id": "550e8400-e29b-41d4-a716-446655440000",
    "heartbeat_interval_secs": 10
  },
  "message": "ok"
}
```

- `node_id`：节点永久唯一标识，节点应持久化保存
- `heartbeat_interval_secs`：心跳间隔（秒）

## 心跳

### 请求

```
POST /api/v1/agent/heartbeat
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `node_id` | string | 是 | 节点标识 |
| `active_tasks` | int | 是 | 活跃任务数 |
| `bytes_downloaded` | int | 是 | 累计下载字节数 |
| `busy` | bool | 是 | 是否忙碌 |
| `last_error` | string/null | 否 | 最近错误 |

```json
{
  "node_id": "550e8400-e29b-41d4-a716-446655440000",
  "active_tasks": 2,
  "bytes_downloaded": 1073741824,
  "busy": true,
  "last_error": null
}
```

### 响应

心跳响应中可携带待领取的调度任务：

```json
{
  "code": 0,
  "data": {
    "pending_dispatches": [
      {
        "dispatch_id": "uuid",
        "task_id": "uuid",
        "task_name": "Ubuntu 22.04 ISO",
        "url": "https://example.com/ubuntu.iso",
        "filename": "ubuntu.iso",
        "config": { ... }
      }
    ]
  },
  "message": "ok"
}
```

### 超时判定

- 主控超过 `heartbeat_timeout_secs`（默认 45 秒）未收到心跳，标记节点为离线
- 离线节点不再分配新任务

## 配置拉取

### 请求

```
GET /api/v1/nodes/{node_id}/config.yaml
```

### 响应

返回 YAML 格式的节点配置，包含：

- `agent`：Agent 模式配置（master、heartbeat_interval）
- `global`：全局配置（max_concurrent、connections_per_file 等）
- `output`：输出配置
- `proxy`：代理配置
- `controller`：主控地址与 Token
- `direct_tasks`：直接任务列表

## 任务领取

### 待领取列表

```
GET /api/v1/dispatches/pending?node_id={node_id}
```

### 领取任务

```
POST /api/v1/dispatches/{dispatch_id}/claim
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `node_id` | string | 是 | 节点标识 |

领取成功后任务状态变为 `acked`，其他节点不可再领取。

## 结果回报

### 请求

```
POST /api/v1/agent/report
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `node_id` | string | 是 | 节点标识 |
| `dispatch_id` | string | 是 | 调度实例标识 |
| `task_id` | string | 是 | 任务标识 |
| `status` | string | 是 | `success` / `failed` / `skipped` |
| `file_size` | int | 否 | 文件总大小 |
| `downloaded_bytes` | int | 否 | 实际下载字节数 |
| `elapsed_secs` | float | 否 | 耗时 |
| `avg_speed_mbps` | float | 否 | 平均速度 |
| `success_chunks` | int | 否 | 成功分片数 |
| `failed_chunks` | int | 否 | 失败分片数 |
| `error_msg` | string/null | 否 | 失败原因 |

## 能力上报

### 更新节点能力

```
POST /api/v1/nodes/{node_id}/capabilities
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `capabilities` | object | 是 | Capability Manifest |

节点（Agent）启动时或能力变更时上报，主控据此进行调度决策。

各类 Agent 上报的能力内容不同，例如：

- **spde**：下载协议、下载特性、任务控制、可配置参数、硬件信息
- **PeerDiscoveryCenter**：peer 发现机制（tracker / dht / pex）、缓存策略、健康检查状态、并发与超时参数

## 鉴权

当主控配置 `token` 非空时，所有 API 请求需携带：

```
Authorization: Bearer {token}
```

WebSocket 连接通过 URL 参数 `?token={token}` 传递。

## 错误响应

所有错误使用统一格式：

```json
{
  "code": "TASK_NOT_FOUND",
  "message": "任务不存在",
  "details": null
}
```

错误码定义见 [错误码标准](error-codes.md)。
