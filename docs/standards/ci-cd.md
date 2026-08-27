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

#### 发版前校验清单（三大类，全部强制）

发版前必须完成以下三类校验，全部通过才能发版。

---

**一、代码校验（静态审查）**

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
- [ ] 无硬编码的敏感信息（token、密码、密钥）
- [ ] 错误处理完整（`?` 传播、`unwrap` 已确认安全）

**前端代码检查：**
- [ ] JavaScript 语法检查通过（`node -c app.js`）
- [ ] HTML 表头列数与渲染函数列数一致
- [ ] 所有点击事件已正确绑定
- [ ] 所有 API 调用路径与后端路由一致
- [ ] CSS 样式类名与 HTML 一致
- [ ] 无 XSS 风险（用户输入已转义）
- [ ] 无硬编码的敏感信息

**配置/文档检查：**
- [ ] Cargo.toml 版本号已更新
- [ ] Cargo.lock 已更新并提交
- [ ] README/文档已同步更新
- [ ] 配置文件示例已更新（如有新增配置项）

---

**二、构建验证（强制，必须通过才能发版）**

- [ ] 本地 `cargo check` 通过（无编译错误、无类型错误）
- [ ] 本地 `cargo build --release` 通过（发布模式编译成功）
- [ ] 无未使用导入、无未使用变量警告（或已确认可忽略）
- [ ] 前端资源已正确嵌入（如使用 include_str!/rust-embed）
- [ ] CI 构建配置正确（workflow 文件无语法错误）
- [ ] 依赖版本兼容（无冲突、无 yanked 版本）
- [ ] 交叉编译配置正确（如涉及多平台）

> **重要**：本地构建验证是发版前校验的**必须项**，不允许跳过。
> 开发环境必须安装 Rust 工具链（rustup + stable），发版前必须运行
> `cargo check` 和 `cargo build --release` 验证编译通过。
> 仅靠代码审查或 CI 验证是不够的，CI 构建失败会浪费时间且污染 tag 历史。
>
> **构建失败处理**：任何项目 `cargo build --release` 失败，都不允许打 tag 发版，
> 必须先修复编译错误，重新运行构建验证通过后才能发版。

---

**三、功能验证（运行时验证）**

**单元测试：**
- [ ] `cargo test` 全部通过（无失败、无忽略的关键测试）
- [ ] 新增功能有对应的单元测试
- [ ] 测试覆盖率不降低（关键路径有测试覆盖）

**集成测试/手动验证：**
- [ ] 新增功能的完整流程可用（注册→使用→修改→删除）
- [ ] 边界情况处理（空值、异常、并发、超时）
- [ ] 向后兼容（旧版本数据/配置能正常读取）
- [ ] 向前兼容（新版本字段旧版本能忽略）
- [ ] 关键路径手动验证通过（如：节点注册、任务下发、进度上报）
- [ ] 错误场景验证（如：网络断开、节点离线、任务失败）

**性能验证（如涉及性能改动）：**
- [ ] 无明显性能退化（编译时间、运行时间、内存占用）
- [ ] 并发场景下无死锁、无竞态条件
- [ ] 资源使用合理（无内存泄漏、无文件句柄泄漏）

**安全验证：**
- [ ] 无 SQL 注入风险（参数化查询）
- [ ] 无路径遍历风险（用户输入路径已校验）
- [ ] 认证/授权逻辑正确（如涉及）
- [ ] 敏感信息不写入日志

---

> **三类校验的关系**：
> - 代码校验是基础（发现语法、逻辑、规范问题）
> - 构建验证是门槛（不通过不能发版）
> - 功能验证是保障（确保改动确实可用，没有引入回归）
> 三者缺一不可，全部通过才能进入发版流程。

#### 校验结果报告

校验完成后必须输出校验确认报告，格式：

```
## 发版前校验报告

### 一、代码校验

| 检查项 | 项目A | 项目B | 备注 |
|--------|-------|-------|------|
| 数据库表结构与迁移 | ✅/❌ | ✅/❌ | |
| SQL 参数数量匹配 | ✅/❌ | ✅/❌ | |
| API 路由注册 | ✅/❌ | ✅/❌ | |
| 结构体字段完整 | ✅/❌ | ✅/❌ | |
| 状态机逻辑正确 | ✅/❌ | ✅/❌ | |
| 前端 JS 语法 | ✅/❌ | - | |
| 前端列数匹配 | ✅/❌ | - | |
| 无硬编码敏感信息 | ✅/❌ | ✅/❌ | |
| 版本号已更新 | ✅/❌ | ✅/❌ | |

### 二、构建验证（强制门槛）

| 项目 | cargo check | cargo build --release | cargo test | 备注 |
|------|------------|----------------------|------------|------|
| 项目A | ✅/❌ | ✅/❌ | ✅/❌ | |
| 项目B | ✅/❌ | ✅/❌ | ✅/❌ | |

### 三、功能验证

| 验证项 | 结果 | 备注 |
|--------|------|------|
| 新增功能完整流程 | ✅/❌ | |
| 边界情况处理 | ✅/❌ | |
| 向后兼容 | ✅/❌ | |
| 向前兼容 | ✅/❌ | |
| 关键路径手动验证 | ✅/❌ | |
| 错误场景验证 | ✅/❌ | |
| 无性能退化 | ✅/❌ | |
| 无安全风险 | ✅/❌ | |

**结论：代码校验通过/未通过，构建验证通过/未通过，功能验证通过/未通过，可/不可发版。**

> **发版门槛**：三类校验必须全部通过，才能进入发版流程。
> 任何一个项目 `cargo build --release` 失败，或关键功能验证失败，
> 都不允许打 tag 发版，必须先修复后重新校验。
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
| ❌ 跳过本地构建验证 | 发版前必须运行 cargo check + cargo build --release，验证通过才能发版 |
| ❌ 本地构建失败仍发版 | 任何项目 cargo build 失败都不允许打 tag，必须先修复 |

### 6. 版本号规范

遵循语义化版本（SemVer）：

| 版本类型 | 版本号变化 | 说明 |
|---------|-----------|------|
| 破坏性变更 | vX+1.0.0 | 不兼容的 API 变更 |
| 功能新增 | vX.Y+1.0 | 向后兼容的功能新增 |
| Bug修复 | vX.Y.Z+1 | 向后兼容的问题修复 |

**预发版本必须与改动类型匹配。**
