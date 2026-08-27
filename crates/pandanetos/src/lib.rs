//! pandanetpl 统一标准库
//!
//! 所有 pandanetpl 生态项目共同依赖的标准库，包含：
//! - 统一错误类型与错误码
//! - 统一响应格式
//! - 通信协议定义（API 路径、DTO、WS 消息）
//! - 领域模型与扩展点 trait
//! - 配置、日志、时间等通用工具

pub mod error;
pub mod response;
pub mod config;
pub mod logging;
pub mod time;
pub mod protocol;
pub mod domain;
pub mod utils;

pub use error::{CoreError, ErrorCode, Result};
pub use response::{ApiResponse, ApiError, PageResult};
pub use time::now_rfc3339;
