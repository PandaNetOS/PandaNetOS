# 代码质量标准

## 格式化

所有代码必须通过 `cargo fmt`：

```bash
cargo fmt --all -- --check
```

### 格式化规则

- 缩进：4 空格
- 行宽：100 字符
- 大括号：同行（K&R 风格）
- 导入分组：标准库 → 外部 crate → 内部模块，空行分隔

```rust
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;
use tokio::net::TcpStream;

use crate::error::CoreError;
use crate::protocol::ClientMsg;
```

## Lint

所有代码必须通过 `cargo clippy`，警告视为错误：

```bash
cargo clippy --all-targets -- -D warnings
```

### 工作区统一配置

在根 `Cargo.toml` 中配置：

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "allow"
```

### 必须修复的 Clippy 警告

- `unwrap_used` — 禁止使用 `unwrap()`，除非有注释证明安全
- `expect_used` — 禁止使用 `expect()`，除非有注释证明安全
- `todo` — 禁止遗留 `todo!()`
- `dbg_macro` — 禁止遗留 `dbg!()`
- `print_stdout` / `print_stderr` — 禁止使用 `println!`/`eprintln!`（用 `tracing` 替代）

### 允许的例外

必须在行内标注并说明理由：

```rust
#[allow(clippy::unwrap_used)]
// 安全：此处值在上面已校验非空
let value = config.get("key").unwrap();
```

## unsafe 代码

- **标准库禁止 `unsafe`**（`unsafe_code = "forbid"`）
- 业务项目如确需使用，必须：
  1. 单独封装在模块中
  2. 每个 `unsafe` 块有 `// SAFETY:` 注释说明为什么安全
  3. 通过代码评审

```rust
// SAFETY: 指针来自已校验的有效内存区域，长度在范围内
unsafe {
    ptr::copy(src, dst, len);
}
```

## 文档

### 公开项必须有文档

所有 `pub` 结构体、枚举、函数、trait 必须有文档注释：

```rust
/// 下载任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 等待调度
    Pending,
    /// 已被节点领取
    Acked,
    /// 正在下载
    Running,
    /// 下载成功
    Success,
    /// 下载失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 创建一个新的下载任务
///
/// # 参数
/// - `url`: 下载地址
/// - `filename`: 本地保存文件名
///
/// # 错误
/// 返回 `CoreError::InvalidParam` 当 URL 格式不合法
pub fn create_task(url: &str, filename: &str) -> Result<Task> {
    // ...
}
```

### 文档注释规范

- 使用 `///` 三斜杠
- 第一行是一句话摘要
- 空行后可加详细说明
- 参数、错误、示例使用 `# 参数`、`# 错误`、`# 示例` 标题

## 测试

### 单元测试

- 每个模块末尾使用 `#[cfg(test)] mod tests`
- 核心逻辑（错误码映射、序列化、配置解析）必须有测试
- 测试函数命名：`test_{场景}_{预期结果}`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_param_maps_to_400() {
        let err = CoreError::InvalidParam("test".into());
        assert_eq!(err.http_status(), 400);
    }
}
```

### 测试覆盖率

- 标准库核心模块（error、response、protocol）覆盖率 ≥ 80%
- 业务项目核心逻辑覆盖率 ≥ 60%
- 不追求 100% 覆盖率，重点覆盖边界条件和错误路径

### 集成测试

- 放在 `tests/` 目录
- 测试公共 API，不测试私有实现
- 每个测试独立，不依赖执行顺序

## 错误处理

### 禁止

- 禁止忽略 `Result`（必须 `?` 或显式处理）
- 禁止使用 `unwrap()` / `expect()`（除非有安全证明）
- 禁止吞掉错误（`let _ = result;`）

### 推荐

- 使用 `?` 传播错误
- 库函数返回 `pandanetos::Result<T>`（即 `Result<T, CoreError>`）
- 二进制入口使用 `anyhow::Result`
- 错误上下文使用 `.with_context(|| ...)`

## 依赖管理

### 依赖选择

- 优先使用生态成熟的 crate（下载量 > 100 万/月）
- 禁止引入功能重叠的多个 crate（如同时用 `reqwest` 和 `hyper` 做 HTTP）
- 新依赖必须在 PR 中说明理由

### 版本锁定

- 所有依赖在根 `Cargo.toml` 的 `[workspace.dependencies]` 中统一管理
- 使用精确版本或 caret 版本（`^1.2`），禁止使用 `*`
- `Cargo.lock` 必须提交

### 定期更新

- 每月检查依赖更新
- 重大版本升级需单独 PR，不与功能混合

## 提交前检查清单

提交代码前必须确认：

- [ ] `cargo fmt --all -- --check` 通过
- [ ] `cargo clippy --all-targets -- -D warnings` 通过
- [ ] `cargo test` 全部通过
- [ ] 无遗留 `println!` / `dbg!` / `todo!()`
- [ ] 新增公开项有文档注释
- [ ] 核心逻辑有单元测试
- [ ] README/文档已同步更新
- [ ] 无硬编码敏感信息（Token、密码、密钥）
