# 整体架构设计

## 架构原则

1. **控制面与数据面分离**：pk（控制面）负责任务调度和节点管理，spde（数据面）负责实际下载
2. **去中心化下载**：pk 不参与下载，只下发任务，spde 节点自主领取并执行
3. **可水平扩展**：spde 节点可任意增减，pk 自动感知并调度
4. **协议无关**：下载能力通过抽象层扩展，新增协议不改核心

## 分层架构

所有项目遵循四层架构：

```
┌─────────────────────────────────┐
│  API 层 (api/)                  │  HTTP/WS 入口，参数校验，响应格式化
│  - 路由注册                      │
│  - Handler                       │
│  - 请求/响应 DTO                 │
├─────────────────────────────────┤
│  服务层 (service/)              │  业务逻辑编排，事务控制
│  - 任务服务                      │
│  - 节点服务                      │
│  - 调度服务                      │
├─────────────────────────────────┤
│  领域层 (domain/)               │  核心模型、状态机、领域服务
│  - 模型 (Task, Node, Dispatch)  │
│  - 状态机                        │
│  - 端口 (trait 抽象)             │
├─────────────────────────────────┤
│  基础设施层 (infra/)            │  数据库、外部客户端、缓存
│  - Repository 实现               │
│  - Downloader 实现               │
│  - 外部 API 客户端               │
└─────────────────────────────────┘
```

### 依赖规则

- `api → service → domain`
- `infra → domain`（实现 domain 中定义的 trait）
- `domain` 不依赖任何外层
- 禁止跨层调用（如 api 直接调用 infra）

## 模块划分标准

```
src/
├── main.rs              # 入口，组装依赖，启动服务
├── lib.rs               # 库导出
├── config/              # 配置定义与加载
├── api/                 # API 层
│   ├── mod.rs
│   ├── routes.rs        # 路由注册
│   ├── handler/         # 请求处理
│   └── dto/             # 数据传输对象
├── service/             # 服务层
│   ├── mod.rs
│   ├── task_service.rs
│   ├── node_service.rs
│   └── dispatch_service.rs
├── domain/              # 领域层
│   ├── mod.rs
│   ├── model/           # 领域模型
│   ├── error.rs         # 领域错误
│   └── port.rs          # 端口 (trait)
├── infra/               # 基础设施层
│   ├── mod.rs
│   ├── repository/      # 数据库实现
│   ├── downloader/      # 下载器实现
│   └── client/          # 外部客户端
├── shared/              # 跨模块共享
│   ├── mod.rs
│   ├── utils.rs
│   └── constants.rs
└── tests/               # 集成测试
```

## pk 架构（主控台）

### 核心组件

| 组件 | 职责 |
|------|------|
| API Server | 提供 REST API 和 WebSocket，管理任务、节点、调度 |
| 调度引擎 | 根据策略将任务分配给节点（任一空闲/全部在线/指定节点） |
| 节点管理器 | 维护节点状态（在线/离线）、心跳检测、元数据管理 |
| 实时监控 | 通过 WebSocket 推送节点状态和下载进度 |
| 数据存储 | SQLite 存储任务、节点、调度记录、运行日志 |

### 数据流

```
用户创建任务 → API 层 → 任务服务 → 写入 DB
                                    ↓
调度引擎定时扫描 → 生成 dispatch → 写入 DB
                                    ↓
spde 节点轮询 /claim → 领取任务 → 更新 dispatch 状态
                                    ↓
spde 上报进度 → API 层 → 更新 DB → WebSocket 推送给前端
                                    ↓
spde 上报完成 → API 层 → 更新 dispatch 状态 → 记录运行日志
```

## spde 架构（下载节点）

### 核心组件

| 组件 | 职责 |
|------|------|
| 任务领取器 | 定期向 pk 轮询领取任务 |
| 下载抽象层 | 统一调度各协议下载器，管理并发和进度 |
| 协议下载器 | HTTP/HTTPS/FTP/SFTP/BitTorrent 等协议实现 |
| 进度上报器 | 实时向 pk 上报下载进度（WebSocket） |
| 心跳上报器 | 定期向 pk 上报节点状态和元数据 |
| 配置管理器 | 管理下载配置（并发数、超时、重试等） |

### 下载抽象层

```
┌─────────────────────────────────────────┐
│            DownloadScheduler             │  统一调度、并发控制、进度聚合
├─────────────────────────────────────────┤
│  Downloader trait (端口)                 │
│  - scheme() -> &str                      │
│  - download(task, progress_sender)       │
│  - probe(url) -> FileInfo                │
├──────┬──────┬──────┬──────┬────────────┤
│ HTTP │ FTP  │ SFTP │  BT  │  ... 扩展  │  协议实现
└──────┴──────┴──────┴──────┴────────────┘
```

### 数据流

```
启动 → 注册节点 → 心跳上报
         ↓
    定期轮询 /claim
         ↓
    领取任务 → 解析 URL → 选择下载器
         ↓
    下载抽象层调度 → 多连接并发下载
         ↓
    实时进度 → WebSocket 上报 pk
         ↓
    下载完成 → 校验 → HTTP 上报结果 → 继续轮询
```

## 通信架构

### 通信方式

| 方向 | 方式 | 用途 |
|------|------|------|
| pk → spde | HTTP 拉取（spde 主动） | 任务领取、配置获取 |
| spde → pk | HTTP POST | 心跳、任务结果上报 |
| spde → pk | WebSocket | 实时下载进度 |
| 用户 → pk | HTTP API | 任务管理、节点管理、调度 |
| pk → 用户 | WebSocket | 实时状态推送 |

### 为什么 spde 主动拉取任务

- 穿透 NAT：spde 可能在内网，pk 无法主动连接
- 解耦：pk 不需要维护 spde 连接状态
- 容错：spde 断线重连后自动继续领取任务

## 扩展点

### 新增下载协议

1. 在 `domain/port.rs` 中已有 `Downloader` trait
2. 新增 `infra/downloader/xxx.rs` 实现 trait
3. 在 `DownloaderRegistry` 中注册
4. 不改核心调度代码

### 新增调度策略

1. 在 `domain/` 中定义 `DispatchStrategy` trait
2. 新增策略实现
3. 在配置中选择策略

### 新增存储后端

1. 在 `domain/port.rs` 中定义 `Repository` trait
2. 新增实现（如 PostgreSQL、MySQL）
3. 依赖注入时选择实现
