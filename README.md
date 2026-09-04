# PandaNetOS

> pandanetos 生态的统一架构与标准仓库

PandaNetOS 是 pandanetos 项目群的**架构标准、通信协议、共享代码**的统一管理仓库。所有子项目（pk、spde、pcdn-keeper 等）都必须遵循本仓库定义的标准。

## 功能特性

- **统一架构标准**：四层架构、依赖规则、模块划分，所有项目遵循同一套架构规范
- **统一通信协议**：RESTful API、WebSocket 消息格式、节点通信协议，确保组件间无缝对接
- **统一数据标准**：JSON 字段命名、时间格式、字节单位、错误码，消除数据格式歧义
- **统一工程标准**：配置加载、结构化日志、CI/CD 流程、代码质量检查，提升开发效率
- **Rust 共享标准库**：`pandanetos` crate，包含错误类型、响应格式、协议定义、领域模型、工具函数
- **统一发版系统**：Reusable Workflows，支持 Rust 二进制、Docker 镜像、Rust 库、前端四种项目类型
- **Tag Guard 发版校验**：自动校验 tag 格式、版本一致性、CI 状态，失败自动删除 tag
- **合规检查脚本**：10 项合规检查，强制所有项目遵循统一标准

## 标准库路径约定

各项目仓库与 `PandaNetOS` 仓库必须放在**同一父目录**下：

```
<workspace>/
├── PandaNetOS/              # 本仓库（标准库）
│   └── crates/pandanetos/
├── pk/                      # 主控台
├── spde/                    # 下载节点
├── PeerDiscoveryCenter/    # Peer 发现中心（Tracker + DHT + PEX）
└── pcdn-keeper/             # Docker 封装
```

各项目 `Cargo.toml` 中统一使用 path 依赖：

```toml
[dependencies]
pandanetos = { path = "../PandaNetOS/crates/pandanetos" }
```

> **禁止**在项目内维护私有协议常量、私有错误码或私有响应格式；全部复用 `pandanetos`。

## 快速开始

### 环境要求

- Rust 1.75+
- Git
- 各项目仓库与 PandaNetOS 放在同一父目录

### 安装

克隆 PandaNetOS 标准库到本地：

```bash
cd <workspace>
git clone https://github.com/PandaNetOS/PandaNetOS.git
```

### 使用示例

在你的 Rust 项目中添加依赖：

```toml
[dependencies]
pandanetos = { path = "../PandaNetOS/crates/pandanetos" }
```

然后在代码中导入：

```rust
use pandanetos::prelude::*;
```

## 生态项目

| 项目 | 角色 | 仓库 |
|------|------|------|
| **pk** | 主控台（Control Plane）- 任务下发、节点管理、实时监控 | [pandamelive/pk](https://github.com/pandamelive/pk) |
| **spde** | 下载节点（Data Plane）- 多协议下载、带宽榨取、进度上报 | [pandamelive/spde](https://github.com/pandamelive/spde) |
| **pcdn-keeper** | Docker 镜像 - 封装 pk + spde 的一体化部署 | [pandamelive/pcdn-keeper](https://github.com/pandamelive/pcdn-keeper) |
| **runtime-rust** | Rust 运行时（Runtime）- 承载生态组件的标准 Rust 编译环境 | [pandamelive/runtime-rust](https://github.com/pandamelive/runtime-rust) |
| **PeerDiscoveryCenter** | Peer 发现中心 - BitTorrent Peer 统一发现（Tracker + DHT + PEX） | [pandamelive/PeerDiscoveryCenter](https://github.com/pandamelive/PeerDiscoveryCenter) |
| **PandaNetOS** | 架构标准 + 共享库 | 当前仓库 |

## 系统架构

```
┌─────────────────────────────────────────────────────────┐
│                      用户 / 第三方系统                      │
└──────────────────────────┬──────────────────────────────┘
                           │ HTTP API / WebSocket
┌──────────────────────────▼──────────────────────────────┐
│                     pk (主控台)                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │ 任务管理  │ │ 节点管理  │ │ 调度引擎  │ │ 实时监控  │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
│  ┌────────────────────────────────────────────────────┐  │
│  │              SQLite (任务/节点/调度记录)              │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────┬──────────────────────────────┘
                           │ 任务下发 (config.yaml)
                           │ 心跳/进度上报 (HTTP + WS)
          ┌────────────────┼────────────────┐
          │                │                │
┌─────────▼──────┐ ┌──────▼───────┐ ┌─────▼────────┐
│  spde 节点 1    │ │  spde 节点 2  │ │  spde 节点 N  │
│  (多协议下载)   │ │  (多协议下载)  │ │  (多协议下载)  │
└────────────────┘ └──────────────┘ └──────────────┘
```

## 标准规范

### 架构标准
- [分层架构](docs/architecture.md) - 四层架构、依赖规则、模块划分
- [项目结构标准](docs/standards/project-structure.md) - 目录结构、命名规范

### 通信标准
- [API 规范](docs/standards/api.md) - RESTful API、统一响应、分页、错误码
- [能力协商与版本兼容](docs/standards/capability-negotiation.md) - 组件能力注册、能力协商、版本解耦、向前/向后兼容
- [自描述能力清单](docs/standards/capability-manifest.md) - 每个构建版本生成说明书，运行时上报，其他程序一看就知道怎么调用
- [WebSocket 协议](docs/standards/websocket.md) - 消息格式、心跳、重连
- [节点通信协议](docs/standards/node-protocol.md) - 注册、心跳、任务领取、进度上报

### 数据标准
- [数据格式标准](docs/standards/data-format.md) - JSON 字段命名、时间格式、字节单位
- [错误码标准](docs/standards/error-codes.md) - 统一错误码定义、HTTP 状态码映射

### 工程标准
- [配置标准](docs/standards/config.md) - 配置加载、环境变量命名、默认值
- [日志标准](docs/standards/logging.md) - 结构化日志、级别、字段约定
- [CI/CD 标准](docs/standards/ci-cd.md) - 构建矩阵、缓存策略、发版流程
- [代码质量标准](docs/standards/code-quality.md) - fmt、clippy、测试

## 共享库

### `pandanetos` crate

Rust 共享标准库，所有项目共同依赖。

#### 包含模块

- `error` - 统一错误类型、错误码（7 大领域，含 HTTP 状态码映射）
- `response` - 统一响应格式、分页（`ApiResponse`/`ApiError`/`PageResult`）
- `protocol` - 通信协议定义（API 路径常量、DTO、WebSocket 消息）
- `domain` - 领域模型（Task/Node/Dispatch）、扩展点 trait（Downloader/Repository/DispatchStrategy）
- `capability` - 自描述能力清单（Capability Manifest）
- `config` - 配置加载工具（YAML + 环境变量覆盖）
- `logging` - 结构化日志初始化（tracing + env-filter）
- `time` - 时间工具（RFC3339 UTC 统一格式）
- `utils` - 通用工具函数（字节格式化、UUID 校验）

#### 一行导入

```rust
use pandanetos::prelude::*;
```

## 开发指南

### 构建

```bash
cargo build --release
```

### 测试

```bash
cargo test --all
```

### 合规检查

```bash
bash scripts/check_compliance.sh .
```

## 贡献指南

1. 新功能先在本仓库提 Issue 讨论标准
2. 标准确定后再在各子项目实现
3. 所有代码必须通过 `cargo fmt` 和 `cargo clippy`
4. 遵循本仓库定义的所有标准
5. 提交前运行合规检查，确保所有检查项通过

## 版本管理

- 语义化版本：`vMAJOR.MINOR.PATCH`
- 所有项目版本独立，但通信协议变更需同步升级
- 打 tag 自动触发 CI 构建和发版
- Tag Guard 自动校验 tag 格式、版本一致性和 CI 状态

## 许可证

MIT License
