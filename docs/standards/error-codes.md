# 错误码标准

## 错误码格式

```
{DOMAIN}_{REASON}
```

- 全大写字母 + 下划线
- DOMAIN：领域前缀（如 TASK、NODE、DISPATCH、DOWNLOAD）
- REASON：具体原因（如 NOT_FOUND、ALREADY_EXISTS、TIMEOUT）

## HTTP 状态码映射

| 错误类型 | HTTP 状态码 | 说明 |
|----------|------------|------|
| 参数错误 | 400 | INVALID_PARAM |
| 未认证 | 401 | UNAUTHORIZED |
| 无权限 | 403 | FORBIDDEN |
| 资源不存在 | 404 | *_NOT_FOUND |
| 状态冲突 | 409 | *_ALREADY_*、*_INVALID_STATE |
| 请求频繁 | 429 | RATE_LIMITED |
| 内部错误 | 500 | INTERNAL_ERROR |
| 服务不可用 | 503 | SERVICE_UNAVAILABLE |

## 通用错误码

| 错误码 | HTTP | 说明 |
|--------|------|------|
| `INTERNAL_ERROR` | 500 | 内部错误 |
| `INVALID_PARAM` | 400 | 参数错误 |
| `UNAUTHORIZED` | 401 | 未认证 |
| `FORBIDDEN` | 403 | 无权限 |
| `NOT_FOUND` | 404 | 资源不存在（通用） |
| `CONFLICT` | 409 | 状态冲突（通用） |
| `RATE_LIMITED` | 429 | 请求过于频繁 |
| `SERVICE_UNAVAILABLE` | 503 | 服务不可用 |

## 任务领域（TASK_*）

| 错误码 | HTTP | 说明 |
|--------|------|------|
| `TASK_NOT_FOUND` | 404 | 任务不存在 |
| `TASK_ALREADY_EXISTS` | 409 | 任务已存在 |
| `TASK_INVALID_STATE` | 409 | 任务状态非法（如对已完成任务执行开始） |
| `TASK_DOWNLOAD_FAILED` | 500 | 任务下载失败 |
| `TASK_DISABLED` | 409 | 任务已禁用 |

## 节点领域（NODE_*）

| 错误码 | HTTP | 说明 |
|--------|------|------|
| `NODE_NOT_FOUND` | 404 | 节点不存在 |
| `NODE_ALREADY_REGISTERED` | 409 | 节点已注册 |
| `NODE_OFFLINE` | 409 | 节点离线 |
| `NODE_HEARTBEAT_TIMEOUT` | 408 | 节点心跳超时 |
| `NODE_VERSION_TOO_OLD` | 409 | 节点版本过低，不兼容 |

## 调度领域（DISPATCH_*）

| 错误码 | HTTP | 说明 |
|--------|------|------|
| `DISPATCH_NOT_FOUND` | 404 | 调度记录不存在 |
| `DISPATCH_ALREADY_CLAIMED` | 409 | 任务已被其他节点领取 |
| `DISPATCH_INVALID_STATE` | 409 | 调度状态非法 |
| `NO_AVAILABLE_NODE` | 503 | 没有可用节点 |
| `DISPATCH_EXPIRED` | 409 | 调度已过期 |

## 下载领域（DOWNLOAD_*）

| 错误码 | HTTP | 说明 |
|--------|------|------|
| `DOWNLOAD_UNSUPPORTED_PROTOCOL` | 400 | 不支持的下载协议 |
| `DOWNLOAD_CONNECTION_FAILED` | 502 | 下载连接失败 |
| `DOWNLOAD_TIMEOUT` | 504 | 下载超时 |
| `DOWNLOAD_CHECKSUM_MISMATCH` | 409 | 下载校验和不匹配 |
| `DOWNLOAD_DISK_FULL` | 507 | 磁盘空间不足 |
| `DOWNLOAD_PARTIAL_CONTENT` | 206 | 部分内容（断点续传） |
| `DOWNLOAD_RANGE_NOT_SATISFIABLE` | 416 | 请求范围不满足 |

## 工作流领域（WORKFLOW_*）

| 错误码 | HTTP | 说明 |
|--------|------|------|
| `WORKFLOW_NOT_FOUND` | 404 | 工作流不存在 |
| `WORKFLOW_ALREADY_EXISTS` | 409 | 工作流已存在 |
| `WORKFLOW_DISABLED` | 409 | 工作流已禁用 |
| `WORKFLOW_INVALID_CRON` | 400 | Cron 表达式无效 |
| `WORKFLOW_TRIGGER_FAILED` | 500 | 工作流触发失败 |

## 配置领域（CONFIG_*）

| 错误码 | HTTP | 说明 |
|--------|------|------|
| `CONFIG_INVALID` | 400 | 配置无效 |
| `CONFIG_NOT_FOUND` | 404 | 配置项不存在 |
| `CONFIG_READ_ONLY` | 409 | 配置只读，不可修改 |

## 认证领域（AUTH_*）

| 错误码 | HTTP | 说明 |
|--------|------|------|
| `AUTH_TOKEN_INVALID` | 401 | Token 无效 |
| `AUTH_TOKEN_EXPIRED` | 401 | Token 已过期 |
| `AUTH_TOKEN_REVOKED` | 401 | Token 已撤销 |
| `AUTH_PERMISSION_DENIED` | 403 | 权限不足 |

## 错误响应示例

### 任务不存在

```json
// HTTP 404
{
  "code": "TASK_NOT_FOUND",
  "message": "任务不存在: 550e8400-e29b-41d4-a716-446655440000"
}
```

### 参数校验失败

```json
// HTTP 400
{
  "code": "INVALID_PARAM",
  "message": "参数校验失败",
  "details": {
    "url": ["URL 格式不正确", "不能为空"],
    "name": ["长度不能超过 100"]
  }
}
```

### 任务已被领取

```json
// HTTP 409
{
  "code": "DISPATCH_ALREADY_CLAIMED",
  "message": "任务已被节点 a98d87c3 领取",
  "details": {
    "claimed_by": "a98d87c3-...",
    "claimed_at": "2026-08-27T10:30:00Z"
  }
}
```

## Rust 中定义错误码

```rust
use pandanetos::error::{CoreError, ErrorCode, codes};

// 领域错误
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("任务不存在: {0}")]
    NotFound(Uuid),
    #[error("任务状态非法: 当前={current}, 期望={expected}")]
    InvalidState { current: String, expected: String },
}

impl TaskError {
    pub fn code(&self) -> ErrorCode {
        match self {
            TaskError::NotFound(_) => codes::TASK_NOT_FOUND,
            TaskError::InvalidState { .. } => codes::TASK_INVALID_STATE,
        }
    }
}
```
