# CI/CD 标准

## 构建矩阵

所有 Rust 项目统一支持以下目标平台：

| OS | 架构 | Target | 编译方式 |
|----|------|--------|---------|
| Windows | x86_64 | `x86_64-pc-windows-msvc` | cargo |
| macOS | x86_64 | `x86_64-apple-darwin` | cargo |
| macOS | aarch64 | `aarch64-apple-darwin` | cargo |
| Linux | x86_64 | `x86_64-unknown-linux-musl` | cross |
| Linux | aarch64 | `aarch64-unknown-linux-musl` | cross |

### 为什么 Linux 用 musl + cross

- **musl**：静态链接，单文件可执行，不依赖系统 libc，任意 Linux 发行版直接运行
- **cross**：在 Docker 容器中交叉编译，工具链完整，避免手动安装 musl-tools 等依赖

## 缓存策略

### sccache（non-cross 目标）

Windows 和 macOS 使用 sccache 加速编译：

- **缓存后端**：本地磁盘（不使用 GHA 缓存服务，避免服务不可用导致构建失败）
- **持久化**：通过 `actions/cache` 保存 sccache 缓存目录
- **缓存 key**：`sccache-{target}-{Cargo.lock hash}`

sccache 缓存目录：

| OS | 路径 |
|----|------|
| Linux | `~/.cache/sccache` |
| macOS | `~/Library/Caches/Mozilla.sccache` |
| Windows | `~/AppData/Local/Mozilla/sccache/cache` |

### Swatinem/rust-cache（所有目标）

所有目标都使用 `Swatinem/rust-cache` 缓存 cargo target 目录：

- 缓存整个 target 目录，包括依赖编译结果
- 自动基于 Cargo.lock 生成缓存 key
- cross 目标也能缓存（通过配置共享 target 目录）

### Cargo.toml 编译优化

所有项目的 `Cargo.toml` 统一添加：

```toml
[profile.release]
codegen-units = 256  # 增加并行编译单元，加快编译速度
lto = false           # 禁用链接时优化，加快编译（发布版可按需开启）
```

## CI 工作流标准

### 触发条件

```yaml
on:
  push:
    tags:
      - 'v*'          # 打 tag 触发发布
  workflow_dispatch:   # 支持手动触发
```

### 工作流结构

```
build (矩阵构建)
  ├── Windows x86_64
  ├── macOS x86_64
  ├── macOS aarch64
  ├── Linux x86_64 musl (cross)
  └── Linux aarch64 musl (cross)
        ↓
release (创建 GitHub Release)
        ↓
dispatch (触发下游项目构建)
```

### fail-fast

```yaml
strategy:
  fail-fast: false  # 一个平台失败不影响其他平台
```

## 发版流程

### 版本号

- 语义化版本：`vMAJOR.MINOR.PATCH`
- 示例：`v0.6.1`、`v1.0.0`

### 发版步骤

1. 更新 `Cargo.toml` 中的 `version`
2. 提交变更：`git commit -m "chore: 版本号 x.y.z -> x.y.w"`
3. 打 tag：`git tag v0.6.1`
4. 推送：`git push origin main --tags`
5. CI 自动构建并创建 GitHub Release

### Release 内容

- 名称：`{项目名} {版本号}`，如 `SPDE v0.6.1`
- 发布说明：自动生成（`generate_release_notes: true`）
- 附件：各平台二进制文件

## 二进制命名规范

```
{项目名}-{平台}-{架构}.{扩展名}
```

| 项目 | Windows | macOS x86_64 | macOS aarch64 | Linux x86_64 musl | Linux aarch64 musl |
|------|---------|-------------|--------------|-------------------|--------------------|
| spde | `spde-x86_64-windows.exe` | `spde-x86_64-macos` | `spde-aarch64-macos` | `spde-x86_64-linux-musl` | `spde-aarch64-linux-musl` |
| pk | `pk-x86_64-windows.exe` | `pk-x86_64-macos` | `pk-aarch64-macos` | `pk-x86_64-linux-musl` | `pk-aarch64-linux-musl` |

## 代码质量检查

### 提交前检查

所有提交必须通过：

```bash
cargo fmt --check    # 代码格式检查
cargo clippy -- -D warnings  # 代码 lint，警告视为错误
cargo test           # 测试通过
```

### CI 中检查

在 PR 合并前运行检查工作流：

```yaml
name: CI Check
on: [pull_request, push]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
```

## Docker 镜像标准

### 基础镜像

- Linux musl 静态二进制 → `FROM scratch` 或 `FROM alpine`
- 最小化镜像体积

### 镜像标签

```
{组织}/{项目名}:{版本号}
{组织}/{项目名}:latest
```

示例：
- `pandanetos/pcdn-keeper:0.6.1`
- `pandanetos/pcdn-keeper:latest`

### 多架构构建

使用 `docker buildx` 构建多架构镜像：

- `linux/amd64`
- `linux/arm64`

## 依赖更新

- 定期运行 `cargo update` 更新依赖
- 重大版本升级需测试后再合并
- 安全漏洞依赖立即升级

## 参考 workflow 模板

完整的 release workflow 参考：

```yaml
name: Release
on:
  push:
    tags: ['v*']
  workflow_dispatch:
permissions:
  contents: write
jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: project-x86_64-windows.exe
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            artifact: project-x86_64-linux-musl
            cross: true
          - os: ubuntu-latest
            target: aarch64-unknown-linux-musl
            artifact: project-aarch64-linux-musl
            cross: true
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: project-x86_64-macos
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: project-aarch64-macos
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install sccache
        if: ${{ !matrix.cross }}
        uses: mozilla-actions/sccache-action@v0.0.6
      - name: Cache sccache
        if: ${{ !matrix.cross }}
        uses: actions/cache@v4
        with:
          path: |
            ~/.cache/sccache
            ~/Library/Caches/Mozilla.sccache
            ~/AppData/Local/Mozilla/sccache/cache
          key: sccache-${{ matrix.target }}-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            sccache-${{ matrix.target }}-
      - name: Cache Rust build
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}
      - name: Install cross
        if: matrix.cross
        uses: taiki-e/install-action@cross
      - name: Build with cross
        if: matrix.cross
        run: cross build --release --target ${{ matrix.target }}
      - name: Build with sccache
        if: ${{ !matrix.cross }}
        env:
          RUSTC_WRAPPER: sccache
        run: cargo build --release --target ${{ matrix.target }}
      - name: Package
        shell: bash
        run: |
          bin="target/${{ matrix.target }}/release/project"
          [ "${{ matrix.os }}" = "windows-latest" ] && bin="${bin}.exe"
          cp "$bin" "${{ matrix.artifact }}"
          [ "${{ matrix.os }}" != "windows-latest" ] && chmod +x "${{ matrix.artifact }}"
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: ${{ matrix.artifact }}
  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          merge-multiple: true
      - uses: softprops/action-gh-release@v2
        with:
          name: Project ${{ github.ref_name }}
          generate_release_notes: true
          files: project-*
```

## 发版流程标准

### 1. 发版申请（强制）

**任何项目发版前必须先提发版申请，经确认后才能操作。**

发版申请格式：

```
## 发版申请

| 项目 | 当前版本 | 预发版本 | 版本类型 |
|------|---------|---------|---------|
| 项目A | vX.X.X | vX.X.X | 功能新增/Bug修复/破坏性变更 |
| 项目B | vX.X.X | vX.X.X | ... |

**发版顺序**：项目A → 项目B → ...

### 本轮改动（按项目分）

#### 项目A
1. 改动1
2. 改动2
...

#### 项目B
1. 改动1
...
```

**发版申请必须包含**：
- 项目名称
- 当前版本号
- 预发版本号
- 版本类型（功能新增/ Bug修复 / 破坏性变更）
- 发版顺序
- 本轮改动（按项目分类罗列）

### 2. 发版前查错确认（强制）

发版前必须对本轮所有改动进行全面查错，确保功能完整、能正常构建。

#### 查错检查清单

**后端代码检查：**
- [ ] 数据库表结构与迁移逻辑一致（新增列允许 NULL 或有默认值）
- [ ] 所有 SQL 语句参数数量与 `params!` 匹配
- [ ] 所有 API 路由已正确注册
- [ ] 所有 handler 函数签名与路由提取器匹配
- [ ] 结构体字段完整，新增字段用 `Option<T>` / `#[serde(default)]`
- [ ] 枚举值序列化与数据库存储一致
- [ ] 状态机转换逻辑正确（状态判断、状态转换条件）
- [ ] 无未定义变量、无变量顺序错误
- [ ] 导入完整，无未使用导入

**前端代码检查：**
- [ ] JavaScript 语法检查通过（`node -c`）
- [ ] HTML 表头列数与渲染函数列数一致
- [ ] 所有点击事件已正确绑定
- [ ] 所有 API 调用路径与后端路由一致
- [ ] CSS 样式类名与 HTML 一致

**功能完整性检查：**
- [ ] 新增功能的完整流程可用（注册→使用→修改→删除）
- [ ] 边界情况处理（空值、异常、并发）
- [ ] 向后兼容（旧版本数据/配置能正常读取）
- [ ] 向前兼容（新版本字段旧版本能忽略）

**构建验证：**
- [ ] 本地编译通过（如环境允许）
- [ ] CI 构建配置正确
- [ ] 依赖版本兼容

#### 查错结果报告

查错完成后必须输出查错确认报告，格式：

```
## 查错确认报告

### 项目A检查项（全部通过 ✅ / 存在问题 ❌）

| 检查项 | 结果 |
|--------|------|
| 数据库表结构与迁移 | ✅ |
| SQL 参数数量匹配 | ✅ |
| ... | ... |

### 项目B检查项（全部通过 ✅ / 存在问题 ❌）
...

**结论：本次改动功能完整/存在问题，可/不可正常构建。**
```

### 3. 发版顺序规范

**依赖关系决定发版顺序：**
- 被依赖的项目先发版
- 依赖方后发版
- 自动触发的项目最后（如 pcdn-keeper 由 pk/spde 发版自动触发）

**示例顺序：**
```
spde（被依赖）→ pk（依赖spde）→ pcdn-keeper（自动触发）
```

### 4. 发版后验证

发版完成后必须验证：
- [ ] GitHub Release 已创建，包含所有平台二进制
- [ ] 镜像已推送到所有配置的仓库（GHCR/Docker Hub/阿里云等）
- [ ] 下游项目自动触发构建（如配置了 repository_dispatch）
- [ ] 版本号正确，无遗漏
- [ ] Release Notes 生成正确

### 5. 禁止事项

| 禁止行为 | 说明 |
|---------|------|
| ❌ 未提发版申请直接发版 | 任何发版必须先申请，经确认后操作 |
| ❌ 未查错直接发版 | 发版前必须完成查错确认 |
| ❌ 发版申请信息不全 | 必须包含项目、当前版本、预发版本、改动清单 |
| ❌ 跳过发版顺序 | 必须按依赖关系顺序发版 |
| ❌ 发版后不验证 | 发版后必须验证 Release、镜像、下游触发 |
| ❌ 查错不完整 | 必须按检查清单逐项检查，输出查错报告 |

### 6. 版本号规范

遵循语义化版本（SemVer）：

| 版本类型 | 版本号变化 | 说明 |
|---------|-----------|------|
| 破坏性变更 | vX+1.0.0 | 不兼容的 API 变更 |
| 功能新增 | vX.Y+1.0 | 向后兼容的功能新增 |
| Bug修复 | vX.Y.Z+1 | 向后兼容的问题修复 |

**预发版本必须与改动类型匹配。**
