//! pandanetos 统一标准库
//!
//! 所有 pandanetos 生态项目共同依赖的标准库。包含：
//! - [`error`]：统一错误类型与错误码
//! - [`response`]：统一响应格式与分页
//! - [`protocol`]：通信协议定义（API 路径、DTO、WS 消息）
//! - [`domain`]：领域模型与扩展点 trait
//! - [`capability`]：自描述能力清单（Capability Manifest）
//! - [`config`]：配置加载工具
//! - [`logging`]：结构化日志初始化
//! - [`time`]：时间工具
//! - [`utils`]：通用工具函数

pub mod capability;
pub mod config;
pub mod domain;
pub mod error;
pub mod logging;
pub mod protocol;
pub mod response;
pub mod time;
pub mod utils;

pub use capability::{CapabilityManifest, ComponentRole};
pub use error::{CoreError, ErrorCode, Result};
pub use protocol::{ClientMsg, ServerMsg};
pub use response::{ApiError, ApiResponse, PageQuery, PageResult};
pub use time::now_rfc3339;
