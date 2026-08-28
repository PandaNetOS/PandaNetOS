//! 日志初始化工具
//!
//! 基于 `tracing_subscriber` 提供统一的结构化日志初始化，遵循
//! [`docs/standards/logging.md`] 日志标准。

use std::io;

/// 初始化日志（默认级别：info）
///
/// 可通过环境变量 `RUST_LOG` 覆盖日志级别，例如：
/// `RUST_LOG=debug`、`RUST_LOG=pk=warn,spde=debug`。
///
/// 重复调用不会重复安装（tracing 全局订阅器只能安装一次）。
pub fn init() {
    init_with_level("info");
}

/// 按指定级别初始化日志
pub fn init_with_level(default_level: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_writer(io::stdout)
        .try_init();
}

/// 初始化测试日志（仅测试用，屏蔽 info 以下级别避免噪音）
#[cfg(test)]
pub fn init_test_logger() {
    let filter = tracing_subscriber::EnvFilter::new("debug");
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(io::stderr)
        .try_init();
}
