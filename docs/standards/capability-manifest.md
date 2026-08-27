# 自描述能力清单标准（Capability Manifest）

> 所有 PandaNetOS 生态项目必须遵循本标准：每个构建版本生成自己的能力清单（说明书），运行时上报给主控端，其他程序一看就知道如何调用。

## 1. 设计原则

| 原则 | 说明 |
|------|------|
| **自描述** | 每个程序自带完整的能力说明书，不需要外部文档 |
| **版本绑定** | 能力清单与构建版本绑定，版本不同能力可能不同 |
| **运行时上报** | 程序启动/注册时主动上报能力清单 |
| **主控端展示** | 主控端（pk）存储并展示每个节点/组件的能力清单 |
| **向后兼容** | 新增能力字段用可选字段，旧版本主控端不认识时透传 |
| **机器可读** | JSON 格式，程序可解析，用于动态调度和能力协商 |

## 2. 能力清单结构

所有项目的能力清单必须包含以下顶层字段：

```json
{
  "manifest_version": "1.0",
  "basic": { ... },
  "capabilities": { ... },
  "configurable_params": { ... },
  "api_interfaces": { ... },
  "status_report": { ... },
  "communication": { ... },
  "build_info": { ... }
}
```

### 2.1 manifest_version

能力清单格式版本号，当前为 `"1.0"`。

### 2.2 basic（基本信息）

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| name | string | ✅ | 程序名称（如 spde、pk、pcdn-keeper） |
| version | string | ✅ | 语义化版本号（如 0.6.2） |
| description | string | ✅ | 程序功能描述 |
| role | string | ✅ | 角色（control_plane / data_plane / sidecar / monitor） |
| current_mode | string | ❌ | 当前运行模式（agent / standalone / cli） |

### 2.3 capabilities（能力清单）

程序支持的所有功能特性，按类别分组。每个能力用 `key: boolean` 或 `key: { ... }` 表示。

**标准分组：**

| 分组 | 说明 | 示例 |
|------|------|------|
| protocols | 支持的协议 | `["http", "https", "ftp", "ssh", "file", "torrent"]` |
| features | 功能特性 | `resume`, `multi_connection`, `retry`, `pause`, `cancel` |
| task_control | 任务控制能力 | `pause`, `resume`, `cancel`, `pause_all`, `cancel_all` |
| hardware | 硬件能力 | `cpu_cores`, `memory_gb`, `os`, `arch` |
| compile_features | 编译 feature | `ftp`, `torrent` 等条件编译特性 |

### 2.4 configurable_params（可配置参数）

主控端可下发覆盖的所有配置参数，每个参数必须包含完整的类型信息和范围：

```json
{
  "max_concurrent": {
    "type": "u32",
    "default": 4,
    "min": 1,
    "max": 256,
    "unit": "tasks",
    "description": "最大并发下载任务数"
  }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | ✅ | 数据类型（u32/u64/f64/bool/string/enum） |
| default | any | ✅ | 默认值 |
| min | number | ❌ | 最小值（数值类型） |
| max | number | ❌ | 最大值（数值类型） |
| enum | array | ❌ | 枚举可选值（enum 类型） |
| unit | string | ❌ | 单位（如 bytes/sec、tasks、MB） |
| description | string | ✅ | 参数说明 |

### 2.5 api_interfaces（API 接口定义）

程序暴露的所有 API 接口，用于其他程序调用：

```json
{
  "agent_register": {
    "method": "POST",
    "path": "/api/v1/agent/register",
    "description": "节点注册",
    "request": { ... },
    "response": { ... }
  }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| method | string | ✅ | HTTP 方法（GET/POST/PUT/DELETE） |
| path | string | ✅ | API 路径 |
| description | string | ✅ | 接口说明 |
| request | object | ❌ | 请求体结构（字段名+类型） |
| response | object | ❌ | 响应体结构 |
| auth_required | bool | ❌ | 是否需要认证（默认 false） |

### 2.6 status_report（状态上报字段）

程序上报的状态字段，用于主控端监控：

```json
{
  "node_level": ["active_tasks", "bytes_downloaded", "total_speed_bps"],
  "task_level": ["dispatch_id", "percent", "speed_bps", "downloaded_bytes"]
}
```

### 2.7 communication（通信能力）

程序支持的通信方式：

```json
{
  "websocket": true,
  "http_api": true,
  "heartbeat": true,
  "heartbeat_interval_secs": 10,
  "websocket_reconnect_secs": 3
}
```

### 2.8 build_info（构建信息）

构建时自动注入的元信息：

| 字段 | 类型 | 说明 |
|------|------|------|
| rust_version | string | Rust 编译器版本 |
| build_profile | string | 构建模式（debug/release） |
| build_time | string | 构建时间（ISO 8601） |
| git_commit | string | Git commit hash |
| git_branch | string | Git 分支名 |
| target_triple | string | 编译目标三元组（如 x86_64-unknown-linux-musl） |

## 3. 生成方式

### 3.1 build.rs 自动生成

每个项目必须在 `build.rs` 中生成构建信息，并通过环境变量注入到二进制中：

```rust
// build.rs 示例
fn main() {
    println!("cargo:rustc-env=BUILD_TIME={}", std::env::var("BUILD_TIME").unwrap_or_else(|_| "unknown".into()));
    println!("cargo:rustc-env=GIT_COMMIT={}", std::env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".into()));
    println!("cargo:rustc-env=GIT_BRANCH={}", std::env::var("GIT_BRANCH").unwrap_or_else(|_| "unknown".into()));
    println!("cargo:rustc-env=RUSTC_VERSION={}", rustc_version());
}
```

### 3.2 运行时函数生成

每个项目必须提供一个 `fn build_capability_manifest() -> serde_json::Value` 函数，运行时生成完整的能力清单。

### 3.3 CI 构建时生成静态文件

CI 构建时必须生成 `capability-manifest.json` 静态文件，随 Release 一起发布：

```yaml
- name: Generate capability manifest
  run: |
    ./target/release/spde --manifest > capability-manifest.json
- name: Upload manifest
  uses: actions/upload-artifact@v4
  with:
    name: capability-manifest
    path: capability-manifest.json
```

## 4. 上报方式

### 4.1 注册时上报

节点/组件注册时，必须在请求体中包含完整的能力清单：

```json
POST /api/v1/agent/register
{
  "node_id": "...",
  "hostname": "...",
  "capabilities": { ...完整能力清单... }
}
```

### 4.2 主控端存储

主控端（pk）必须存储每个节点/组件的能力清单，并在节点列表中展示：

- 节点详情页展示完整能力清单
- 节点列表展示关键能力（版本、支持协议、并发数等）
- 能力变更时更新存储

### 4.3 能力协商

主控端根据节点上报的能力清单，动态决定如何调度：

- 只调度节点支持的协议任务
- 只下发节点支持的配置参数
- 新能力主控端不认识时透传，不报错

## 5. 版本兼容规则

| 规则 | 说明 |
|------|------|
| 新增字段必须可选 | 所有新增能力字段用可选字段，旧版本主控端不认识时忽略 |
| 不删除已有字段 | 只能新增，不能删除或重命名已有能力字段 |
| 不修改已有字段语义 | 已有字段的含义不能变，需要新语义就加新字段 |
| 主控端透传不认识的字段 | 收到不认识的能力字段，原样保存，不解析不报错 |

## 6. 各项目实施要求

| 项目 | 能力清单内容 | 上报时机 |
|------|------------|---------|
| **spde** | 下载协议、下载特性、任务控制、可配置参数、硬件信息 | 注册时上报 |
| **pk** | API 接口、调度能力、节点管理能力、配置项 | 启动时生成，供其他组件查询 |
| **pcdn-keeper** | 镜像版本、包含组件版本、启动参数、环境变量 | 启动时上报给 pk |
| **未来项目** | 按本标准生成 | 按角色决定上报时机 |

## 7. CLI 命令

每个项目必须支持 `--manifest` 命令行参数，输出版本的能力清单 JSON：

```bash
$ spde --manifest
{
  "manifest_version": "1.0",
  "basic": { "name": "spde", "version": "0.6.2", ... },
  ...
}
```

## 8. 检查清单

每个项目发布前必须检查：

- [ ] `build.rs` 注入构建信息（RUSTC_VERSION、BUILD_TIME、GIT_COMMIT 等）
- [ ] 提供 `build_capability_manifest()` 函数
- [ ] 支持 `--manifest` CLI 命令输出能力清单
- [ ] 注册/启动时上报能力清单
- [ ] 能力清单包含所有 8 个顶层字段
- [ ] 所有可配置参数包含完整的类型/范围/默认值信息
- [ ] CI 构建时生成 `capability-manifest.json` 并随 Release 发布
- [ ] 新增能力字段用可选字段，向后兼容
