//! 统一错误类型与错误码
//!
//! 所有项目的错误都应基于此模块定义，确保错误码格式统一。

use thiserror::Error;

/// 统一错误码格式：{DOMAIN}_{REASON}，全大写下划线
pub type ErrorCode = &'static str;

/// 通用错误码
pub mod codes {
    // 通用
    pub const INTERNAL_ERROR: ErrorCode = "INTERNAL_ERROR";
    pub const INVALID_PARAM: ErrorCode = "INVALID_PARAM";
    pub const UNAUTHORIZED: ErrorCode = "UNAUTHORIZED";
    pub const FORBIDDEN: ErrorCode = "FORBIDDEN";
    pub const NOT_FOUND: ErrorCode = "NOT_FOUND";
    pub const CONFLICT: ErrorCode = "CONFLICT";
    pub const RATE_LIMITED: ErrorCode = "RATE_LIMITED";
    pub const SERVICE_UNAVAILABLE: ErrorCode = "SERVICE_UNAVAILABLE";

    // 任务领域
    pub const TASK_NOT_FOUND: ErrorCode = "TASK_NOT_FOUND";
    pub const TASK_INVALID_STATE: ErrorCode = "TASK_INVALID_STATE";
    pub const TASK_DOWNLOAD_FAILED: ErrorCode = "TASK_DOWNLOAD_FAILED";
    pub const TASK_ALREADY_EXISTS: ErrorCode = "TASK_ALREADY_EXISTS";

    // 节点领域
    pub const NODE_NOT_FOUND: ErrorCode = "NODE_NOT_FOUND";
    pub const NODE_OFFLINE: ErrorCode = "NODE_OFFLINE";
    pub const NODE_ALREADY_REGISTERED: ErrorCode = "NODE_ALREADY_REGISTERED";
    pub const NODE_HEARTBEAT_TIMEOUT: ErrorCode = "NODE_HEARTBEAT_TIMEOUT";

    // 调度领域
    pub const DISPATCH_NOT_FOUND: ErrorCode = "DISPATCH_NOT_FOUND";
    pub const DISPATCH_ALREADY_CLAIMED: ErrorCode = "DISPATCH_ALREADY_CLAIMED";
    pub const NO_AVAILABLE_NODE: ErrorCode = "NO_AVAILABLE_NODE";

    // 下载领域
    pub const DOWNLOAD_UNSUPPORTED_PROTOCOL: ErrorCode = "DOWNLOAD_UNSUPPORTED_PROTOCOL";
    pub const DOWNLOAD_CONNECTION_FAILED: ErrorCode = "DOWNLOAD_CONNECTION_FAILED";
    pub const DOWNLOAD_TIMEOUT: ErrorCode = "DOWNLOAD_TIMEOUT";
    pub const DOWNLOAD_CHECKSUM_MISMATCH: ErrorCode = "DOWNLOAD_CHECKSUM_MISMATCH";
    pub const DOWNLOAD_DISK_FULL: ErrorCode = "DOWNLOAD_DISK_FULL";
}

/// 核心错误类型
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("内部错误: {0}")]
    Internal(String),

    #[error("参数错误: {0}")]
    InvalidParam(String),

    #[error("未授权")]
    Unauthorized,

    #[error("禁止访问")]
    Forbidden,

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("状态冲突: {0}")]
    Conflict(String),

    #[error("请求过于频繁")]
    RateLimited,

    #[error("服务不可用: {0}")]
    ServiceUnavailable(String),

    #[error("外部错误: {0}")]
    External(#[from] anyhow::Error),
}

impl CoreError {
    /// 获取错误码
    pub fn code(&self) -> ErrorCode {
        match self {
            CoreError::Internal(_) => codes::INTERNAL_ERROR,
            CoreError::InvalidParam(_) => codes::INVALID_PARAM,
            CoreError::Unauthorized => codes::UNAUTHORIZED,
            CoreError::Forbidden => codes::FORBIDDEN,
            CoreError::NotFound(_) => codes::NOT_FOUND,
            CoreError::Conflict(_) => codes::CONFLICT,
            CoreError::RateLimited => codes::RATE_LIMITED,
            CoreError::ServiceUnavailable(_) => codes::SERVICE_UNAVAILABLE,
            CoreError::External(_) => codes::INTERNAL_ERROR,
        }
    }

    /// 获取 HTTP 状态码
    pub fn http_status(&self) -> u16 {
        match self {
            CoreError::Internal(_) | CoreError::External(_) => 500,
            CoreError::InvalidParam(_) => 400,
            CoreError::Unauthorized => 401,
            CoreError::Forbidden => 403,
            CoreError::NotFound(_) => 404,
            CoreError::Conflict(_) => 409,
            CoreError::RateLimited => 429,
            CoreError::ServiceUnavailable(_) => 503,
        }
    }
}

/// 统一 Result 类型
pub type Result<T> = std::result::Result<T, CoreError>;
