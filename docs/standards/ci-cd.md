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
