# API 规范

## 基础约定

### API 版本

所有 API 路径以版本号前缀开头：

```
/api/v1/...
```

版本号变更规则：
- 不兼容变更 → 主版本号 +1（/api/v2）
- 向后兼容新增 → 不变更版本号

### 基础 URL

```
http://{host}:{port}/api/v1
```

### 请求方法

| 方法 | 用途 | 幂等 |
|------|------|------|
| GET | 查询资源 | ✅ |
| POST | 创建资源 / 执行操作 | ❌ |
| PUT | 全量更新资源 | ✅ |
| PATCH | 部分更新资源 | ❌ |
| DELETE | 删除资源 | ✅ |

### 请求头

```
Content-Type: application/json
Authorization: Bearer <token>  # 可选，需要认证的接口
```

## 统一响应格式

### 成功响应

```json
{
  "code": 0,
  "data": { ... },
  "message": "ok"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| code | int | 业务状态码，0 表示成功 |
| data | any | 响应数据，对象或数组 |
| message | string | 提示信息 |

### 错误响应

```json
{
  "code": "TASK_NOT_FOUND",
  "message": "任务不存在: 550e8400-e29b-41d4-a716-446655440000",
  "details": null
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| code | string | 错误码，格式 `{DOMAIN}_{REASON}` |
| message | string | 错误信息（人类可读） |
| details | object/null | 错误详情（可选） |

### HTTP 状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 成功 |
| 201 | 创建成功 |
| 204 | 成功无返回内容 |
| 400 | 请求参数错误 |
| 401 | 未认证 |
| 403 | 无权限 |
| 404 | 资源不存在 |
| 409 | 状态冲突 |
| 429 | 请求过于频繁 |
| 500 | 服务器内部错误 |
| 503 | 服务不可用 |

## 分页规范

### 请求参数

```
GET /api/v1/tasks?page=1&page_size=20
```

| 参数 | 类型 | 默认 | 说明 |
|------|------|------|------|
| page | int | 1 | 页码，从 1 开始 |
| page_size | int | 20 | 每页大小，最大 100 |

### 响应格式

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

## 过滤与排序

### 过滤

```
GET /api/v1/tasks?status=running&node_id=xxx
```

- 过滤参数直接作为 query 参数
- 支持多值过滤：`?status=running&status=pending`
- 支持范围过滤：`?created_at_from=2026-01-01&created_at_to=2026-01-31`

### 排序

```
GET /api/v1/tasks?sort=created_at&order=desc
```

| 参数 | 说明 |
|------|------|
| sort | 排序字段 |
| order | asc / desc，默认 desc |

## 时间格式

所有时间使用 **RFC3339** 格式，UTC 时区：

```
2026-08-27T10:30:00Z
```

## ID 格式

所有资源 ID 使用 **UUID v4**：

```
550e8400-e29b-41d4-a716-446655440000
```

## 字节单位

- 内部存储和 API 传输使用**字节（bytes）**整数
- 前端展示时转换为人类可读格式（KB/MB/GB）

## API 路径命名

- 使用**小写字母 + 连字符**（kebab-case）
- 资源名使用复数
- 子资源嵌套不超过 2 层

```
✅ /api/v1/tasks
✅ /api/v1/tasks/{id}
✅ /api/v1/nodes/{id}/config
✅ /api/v1/dispatches/pending
❌ /api/v1/Task
❌ /api/v1/getTask
❌ /api/v1/nodes/{id}/config/detail/extra
```

## 操作类接口

非 CRUD 操作使用动词后缀：

```
POST /api/v1/tasks/{id}/trigger    # 触发任务
POST /api/v1/nodes/{id}/heartbeat  # 心跳上报
POST /api/v1/dispatches/claim      # 领取任务
DELETE /api/v1/nodes/offline       # 清理离线节点
```

## 批量操作

批量操作使用 POST + 数组：

```
POST /api/v1/tasks/batch
{
  "ids": ["uuid1", "uuid2"],
  "action": "delete"
}
```

或使用 query 参数批量删除：

```
DELETE /api/v1/nodes?status=offline
```

## 错误处理示例

### 资源不存在

```json
// HTTP 404
{
  "code": "TASK_NOT_FOUND",
  "message": "任务不存在: 550e8400-e29b-41d4-a716-446655440000"
}
```

### 参数错误

```json
// HTTP 400
{
  "code": "INVALID_PARAM",
  "message": "参数校验失败",
  "details": {
    "url": "URL 格式不正确",
    "name": "名称不能为空"
  }
}
```

### 状态冲突

```json
// HTTP 409
{
  "code": "DISPATCH_ALREADY_CLAIMED",
  "message": "任务已被其他节点领取"
}
```
