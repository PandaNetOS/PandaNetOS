# 项目结构标准

## 仓库布局

所有 pandanetos 生态项目遵循统一的仓库布局：

```
{project}/
├── Cargo.toml              # 项目元数据与依赖
├── Cargo.lock              # 依赖锁定（必须提交）
├── Cross.toml              # 交叉编译配置（如涉及多平台）
├── build.rs                # 构建脚本（注入构建信息，可选）
├── README.md               # 项目自述（必填章节见下文）
├── CHANGELOG.md            # 变更日志（可选，推荐）
├── .github/
│   └── workflows/          # CI/CD 流水线
├── src/
│   ├── lib.rs              # 库入口（如项目同时是库）
│   ├── main.rs             # 二进制入口
│   ├── bin/                # 多个二进制时使用
│   └── ...
└── tests/                  # 集成测试（可选）
```

## 命名规范

### 仓库名

- 全小写，使用连字符 `-` 分隔
- 示例：`spde`、`pk`、`pcdn-keeper`、`runtime-rust`

### Rust 模块 / 文件

- 模块名：`snake_case`
- 文件名：`snake_case.rs`
- 结构体 / 枚举 / trait：`PascalCase`
- 函数 / 变量 / 常量：函数和变量 `snake_case`，常量 `SCREAMING_SNAKE_CASE`
- 禁止使用拼音命名

### Git 分支

- `main`：主分支，保护分支，仅通过 PR 合并
- `feature/{描述}`：功能分支
- `fix/{描述}`：修复分支
- `release/{版本号}`：发布分支

### Commit Message

格式：`{type}: {简短描述}`

| type | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | 修复 |
| `docs` | 文档变更 |
| `refactor` | 重构（不改变功能） |
| `perf` | 性能优化 |
| `test` | 测试相关 |
| `chore` | 构建/工具/依赖变更 |
| `ci` | CI/CD 相关 |

示例：
```
feat(download): 支持 FTP 协议下载
fix: 修复断点续传时分片状态丢失
docs: 补充标准库路径约定
```

## 标准库路径约定（强制）

所有依赖 `pandanetos` 标准库的项目，必须遵循以下目录布局：

```
<workspace>/
├── PandaNetOS/              # 标准库仓库（必须与各项目同级）
│   └── crates/pandanetos/
├── pk/                      # 主控台
├── spde/                    # 下载节点
└── pcdn-keeper/             # Docker 封装
```

各项目 `Cargo.toml` 中统一使用 **path 依赖**：

```toml
[dependencies]
pandanetos = { path = "../PandaNetOS/crates/pandanetos" }
```

### 约束

- **禁止**使用 git 依赖进行本地开发（CI/CD 场景除外）
- **禁止**在项目内维护私有协议常量、私有错误码或私有响应格式
- **禁止**修改标准库路径，所有项目必须与 `PandaNetOS` 仓库同级
- 克隆项目后必须同时克隆 `PandaNetOS/PandaNetOS` 到同级目录，否则 `cargo build` 失败

### CI/CD 例外

GitHub Actions 发布构建中，可自动 checkout 标准库并修正 path 依赖，或使用 git 依赖：

```toml
[dependencies]
pandanetos = { git = "https://github.com/PandaNetOS/PandaNetOS", branch = "main" }
```

## README 必填章节（强制）

所有生态项目的 `README.md` 必须包含以下章节，顺序如下：

### 1. 项目标题与一句话简介

```markdown
# {项目名}

> {一句话描述项目定位}
```

### 2. 生态定位

必须声明隶属 PandaNetOS 生态，引用标准库仓库：

```markdown
## 生态定位

本项目隶属 **PandaNetOS 生态项目群**，以生态权威标准库 [PandaNetOS](https://github.com/PandaNetOS/PandaNetOS) 为准绳。
```

### 3. 标准库路径约定

必须写明目录布局与 path 依赖写法：

```markdown
### 标准库路径约定

本项目强制依赖生态共享标准库 `pandanetos`，使用 **path 依赖**，目录布局固定：

\`\`\`
<workspace>/
├── PandaNetOS/              # 标准库仓库（必须与本项目同级）
│   └── crates/pandanetos/
└── {project}/               # 本仓库
    └── Cargo.toml           # pandanetos = { path = "../PandaNetOS/crates/pandanetos" }
\`\`\`

\`Cargo.toml\` 中的依赖声明：

\`\`\`toml
[dependencies]
pandanetos = { path = "../PandaNetOS/crates/pandanetos" }
\`\`\`

> 克隆本仓库后，需同时克隆 \`PandaNetOS/PandaNetOS\` 到同级目录，否则 \`cargo build\` 会因找不到 path 依赖而失败。
```

### 4. 版本与平台

| 项 | 说明 |
|----|------|
| 当前版本 | 语义化版本 `vMAJOR.MINOR.PATCH` |
| 发布通道 | GitHub Actions Tag 自动构建 |
| 平台矩阵 | 支持的操作系统与架构 |

### 5. 快速开始

必须包含编译与运行的最小命令：

```bash
cargo build --release
./target/release/{binary}
```

### 6. 配置体系

列出所有配置项、类型、默认值、说明。使用表格格式。

### 7. 构建与发布

说明本地编译方式、交叉编译配置、CI/CD 触发条件。

### 8. 许可证

统一使用 MIT。

## 可选章节

- 核心特性（功能列表）
- 目录结构（工作目录说明）
- CLI 命令
- 通信协议
- 数据记录格式
- 故障排查
