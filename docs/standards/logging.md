# 日志标准

## 日志库

所有项目统一使用 `tracing` + `tracing-subscriber`：

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

标准库提供 `pandanetos::logging` 模块统一初始化：

```rust
use pandanetos::prelude::*;

fn main() {
    logging::init();
    // ...
}
```

## 日志级别

| 级别 | 用途 | 示例 |
|------|------|------|
| `ERROR` | 错误事件，可能导致功能异常 | 下载失败、连接断开 |
| `WARN` | 警告事件，不影响主流程 | 重试、配置缺失使用默认值 |
| `INFO` | 重要业务事件 | 任务开始/完成、节点注册、心跳 |
| `DEBUG` | 调试信息，开发阶段使用 | 分片下载进度、连接建立 |
| `TRACE` | 最详细的追踪信息 | 字节级操作、函数调用栈 |

### 级别使用规范

- 生产环境默认级别：`INFO`
- 通过环境变量 `RUST_LOG` 调整：`RUST_LOG=debug`
- 禁止在循环中使用 `INFO` 及以上级别（高频日志用 `DEBUG`/`TRACE`）
- 错误日志必须包含错误上下文（哪个任务、哪个节点、什么操作）

## 日志格式

### 开发环境

人类可读格式，带颜色：

```
2026-08-27T10:30:00Z  INFO spde::downloader: 任务开始 task_id=xxx filename=ubuntu.iso
2026-08-27T10:30:01Z  WARN spde::downloader: 分片重试 chunk=2 retry=1 error=timeout
2026-08-27T10:31:00Z  INFO spde::downloader: 任务完成 task_id=xxx speed=12.4MB/s
```

### 生产环境

JSON 格式（可选，通过配置开启）：

```json
{"timestamp":"2026-08-27T10:30:00Z","level":"INFO","module":"spde::downloader","message":"任务开始","task_id":"xxx","filename":"ubuntu.iso"}
```

## 结构化字段

使用 `tracing` 的结构化字段，禁止字符串拼接：

```rust
// ✅ 正确：结构化字段
info!(task_id = %task.id, filename = %task.filename, "任务开始");

// ❌ 错误：字符串拼接
info!("任务开始: {} {}", task.id, task.filename);
```

### 常用字段

| 字段 | 说明 |
|------|------|
| `task_id` | 任务标识 |
| `dispatch_id` | 调度实例标识 |
| `node_id` | 节点标识 |
| `filename` | 文件名 |
| `url` | 下载地址 |
| `error` | 错误信息 |
| `speed_mbps` | 速度（MB/s） |
| `elapsed_secs` | 耗时（秒） |

## 日志输出

- 默认输出到 **stdout**
- 不写入文件（由外部日志收集系统处理，如 Docker logs、systemd journal）
- 日志不包含敏感信息（Token、密码、完整 URL 中的查询参数）

## 性能规范

- 高频日志（每秒 > 100 条）必须检查级别后再构造字段：

```rust
if tracing::enabled!(tracing::Level::DEBUG) {
    debug!(chunk = i, downloaded = bytes, "分片进度");
}
```

- 禁止在日志中做昂贵计算（如格式化大文件、序列化完整对象）
- `TRACE` 级别日志在 release 构建中可通过编译期移除

## 错误日志规范

错误日志必须包含：

1. 错误发生的上下文（哪个任务/节点/操作）
2. 错误信息（使用 `{:#}` 打印完整错误链）
3. 后续动作（重试/跳过/终止）

```rust
error!(
    task_id = %task.id,
    error = %format!("{:#}", e),
    "下载失败，已达最大重试次数，任务标记为失败"
);
```
