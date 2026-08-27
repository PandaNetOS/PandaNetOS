# PandaNetOS

> pandanetpl 生态的统一架构与标准仓库

PandaNetOS 是 pandanetpl 项目群的**架构标准、通信协议、共享代码**的统一管理仓库。所有子项目（pk、spde、pcdn-keeper 等）都必须遵循本仓库定义的标准。

## 生态项目

| 项目 | 角色 | 仓库 |
|------|------|------|
| **pk** | 主控台（Control Plane）- 任务下发、节点管理、实时监控 | [PandaNetOS/pk](https://github.com/PandaNetOS/pk) |
| **spde** | 下载节点（Data Plane）- 多协议下载、带宽榨取、进度上报 | [PandaNetOS/spde](https://github.com/PandaNetOS/spde) |
| **pcdn-keeper** | Docker 镜像 - 封装 pk + spde 的一体化部署 | [PandaNetOS/pcdn-keeper](https://github.com/PandaNetOS/pcdn-keeper) |
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

Rust 共享标准库，所有项目共同依赖：

```toml
[dependencies]
pandanetos = { git = "https://github.com/PandaNetOS/PandaNetOS" }
```

包含模块：
- `error` - 统一错误类型、错误码
- `response` - 统一响应格式、分页
- `protocol` - 通信协议定义（API 路径、DTO、WS 消息）
- `domain` - 领域模型、扩展点 trait
- `config` - 配置加载工具
- `logging` - 日志初始化
- `time` - 时间工具
- `utils` - 通用工具函数

## 版本管理

- 语义化版本：`vMAJOR.MINOR.PATCH`
- 所有项目版本独立，但通信协议变更需同步升级
- 打 tag 自动触发 CI 构建和发版

## 贡献指南

1. 新功能先在本仓库提 Issue 讨论标准
2. 标准确定后再在各子项目实现
3. 所有代码必须通过 `cargo fmt` 和 `cargo clippy`
4. 遵循本仓库定义的所有标准

## License

MIT
