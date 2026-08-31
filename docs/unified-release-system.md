# PandaNetOS 统一发版系统

## 概述

PandaNetOS 提供一套统一的、可扩展的发版工作流，所有项目通过引用标准库的 Reusable Workflow 实现发版流程的统一管理。

## 架构

```
各个仓库（spde、pk、pcdn-keeper等）
    ↓ 引用（15行调用文件）
PandaNetOS 标准库
├── reusable-release-rust-binary.yml    # Rust 二进制项目
├── reusable-release-docker-image.yml   # Docker 镜像项目
├── reusable-release-rust-library.yml   # Rust 库项目
├── reusable-release-frontend.yml       # 前端项目
└── release.yml                          # 统一发版入口（自动检测类型）
```

## 支持的项目类型

### 1. Rust 二进制项目（spde、pk）

**特性**：
- 多平台构建矩阵（Windows、Linux musl x86_64/aarch64、macOS x86_64/aarch64）
- sccache 编译缓存
- cross 交叉编译
- PandaNetOS 标准库自动集成
- 自动创建 GitHub Release
- 可选通知下游项目重建

**调用示例**：
```yaml
name: Release
on:
  push:
    tags: ['v*']
permissions:
  contents: write
jobs:
  release:
    uses: PandaNetOS/PandaNetOS/.github/workflows/reusable-release-rust-binary.yml@main
    with:
      binary_name: spde
      project_name: SPDE
      dispatch_downstream: true
      downstream_repo: pcdn-keeper
    secrets: inherit
```

### 2. Docker 镜像项目（pcdn-keeper）

**特性**：
- 多架构构建（linux/amd64, linux/arm64）
- Docker Buildx + QEMU
- 支持推送到 Docker Hub 和 GHCR
- 自动版本标签（semver、latest）
- 构建缓存

**调用示例**：
```yaml
name: Release
on:
  push:
    tags: ['v*']
permissions:
  contents: read
  packages: write
jobs:
  release:
    uses: PandaNetOS/PandaNetOS/.github/workflows/reusable-release-docker-image.yml@main
    with:
      image_name: pcdn-keeper
      platforms: linux/amd64,linux/arm64
      push_to_ghcr: true
    secrets: inherit
```

### 3. Rust 库项目（PandaNetOS）

**特性**：
- 发布前自动验证（测试、clippy、格式）
- 可选发布到 crates.io
- 自动创建 GitHub Release

### 4. 前端项目

**特性**：
- npm 构建
- 可选部署到 GitHub Pages
- 自动创建 GitHub Release

## 统一发版入口

所有仓库可以直接使用统一的 `release.yml`，系统会自动检测项目类型并调用对应的工作流：

```yaml
name: Release
on:
  push:
    tags: ['v*']
jobs:
  release:
    uses: PandaNetOS/PandaNetOS/.github/workflows/release.yml@main
    secrets: inherit
```

## 版本管理策略

- `@main`：跟踪最新版本，适合开发中的项目
- `@v1`：锁定大版本，适合生产项目
- `@v1.0.0`：锁定具体版本，最稳定

## 扩展新的项目类型

1. 在 PandaNetOS 创建 `reusable-release-{type}.yml`
2. 在 `release.yml` 中增加类型检测和调用
3. 更新本文档

## 优势

1. **修改一处，全局生效**：优化构建、增加平台、修改规则只需要改标准库
2. **消除重复代码**：每个仓库从 150+ 行减少到 15 行
3. **统一质量标准**：所有项目使用相同的构建配置、缓存策略、安全设置
4. **易于扩展**：新增项目类型只需要新增一个 Reusable Workflow
5. **版本锁定**：支持按版本引用，稳定性可控
