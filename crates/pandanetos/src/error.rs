//! 统一错误类型与错误码
//!
//! 所有项目的错误都应基于此模块定义，确保错误码格式统一。
//! 错误码格式遵循 [`docs/standards/error-codes.md`] 标准：`{DOMAIN}_{REASON}`。

use thiserror::Error;

/// 统一错误码格式：`{DOMAIN}_{REASON}`，全大写下划线
pub type ErrorCode = &'static str;

/// 通用错误码（与 `docs/standards/error-codes.md` 保持一致）
pub mod codes {
    use super::ErrorCode;

    /// 通用错误（HTTP 500）
    pub const INTERNAL_ERROR: ErrorCode = "INTERNAL_ERROR";
    /// 参数错误（HTTP 400）
    pub const INVALID_PARAM: ErrorCode = "INVALID_PARAM";
    /// 未认证（HTTP 401）
    pub const UNAUTHORIZED: ErrorCode = "UNAUTHORIZED";
    /// 无权限（HTTP 403）
    pub const FORBIDDEN: ErrorCode = "FORBIDDEN";
    /// 资源不存在，通用（HTTP 404）
    pub const NOT_FOUND: ErrorCode = "NOT_FOUND";
    /// 状态冲突，通用（HTTP 409）
    pub const CONFLICT: ErrorCode = "CONFLICT";
    /// 请求过于频繁（HTTP 429）
    pub const RATE_LIMITED: ErrorCode = "RATE_LIMITED";
    /// 服务不可用（HTTP 503）
    pub const SERVICE_UNAVAILABLE: ErrorCode = "SERVICE_UNAVAILABLE";

    // 认证领域（AUTH_*）
    /// Token 无效（HTTP 401）
    pub const AUTH_TOKEN_INVALID: ErrorCode = "AUTH_TOKEN_INVALID";
    /// Token 已过期（HTTP 401）
    pub const AUTH_TOKEN_EXPIRED: ErrorCode = "AUTH_TOKEN_EXPIRED";
    /// Token 已撤销（HTTP 401）
    pub const AUTH_TOKEN_REVOKED: ErrorCode = "AUTH_TOKEN_REVOKED";
    /// 权限不足（HTTP 403）
    pub const AUTH_PERMISSION_DENIED: ErrorCode = "AUTH_PERMISSION_DENIED";

    // 任务领域（TASK_*）
    /// 任务不存在（HTTP 404）
    pub const TASK_NOT_FOUND: ErrorCode = "TASK_NOT_FOUND";
    /// 任务状态非法，如对已完成任务执行开始（HTTP 409）
    pub const TASK_INVALID_STATE: ErrorCode = "TASK_INVALID_STATE";
    /// 任务下载失败（HTTP 500）
    pub const TASK_DOWNLOAD_FAILED: ErrorCode = "TASK_DOWNLOAD_FAILED";
    /// 任务已存在（HTTP 409）
    pub const TASK_ALREADY_EXISTS: ErrorCode = "TASK_ALREADY_EXISTS";
    /// 任务已禁用（HTTP 409）
    pub const TASK_DISABLED: ErrorCode = "TASK_DISABLED";

    // 节点领域（NODE_*）
    /// 节点不存在（HTTP 404）
    pub const NODE_NOT_FOUND: ErrorCode = "NODE_NOT_FOUND";
    /// 节点离线（HTTP 409）
    pub const NODE_OFFLINE: ErrorCode = "NODE_OFFLINE";
    /// 节点已注册（HTTP 409）
    pub const NODE_ALREADY_REGISTERED: ErrorCode = "NODE_ALREADY_REGISTERED";
    /// 节点心跳超时（HTTP 408）
    pub const NODE_HEARTBEAT_TIMEOUT: ErrorCode = "NODE_HEARTBEAT_TIMEOUT";
    /// 节点版本过低，不兼容（HTTP 409）
    pub const NODE_VERSION_TOO_OLD: ErrorCode = "NODE_VERSION_TOO_OLD";

    // 调度领域（DISPATCH_*）
    /// 调度记录不存在（HTTP 404）
    pub const DISPATCH_NOT_FOUND: ErrorCode = "DISPATCH_NOT_FOUND";
    /// 任务已被其他节点领取（HTTP 409）
    pub const DISPATCH_ALREADY_CLAIMED: ErrorCode = "DISPATCH_ALREADY_CLAIMED";
    /// 调度状态非法（HTTP 409）
    pub const DISPATCH_INVALID_STATE: ErrorCode = "DISPATCH_INVALID_STATE";
    /// 调度已过期（HTTP 409）
    pub const DISPATCH_EXPIRED: ErrorCode = "DISPATCH_EXPIRED";
    /// 没有可用节点（HTTP 503）
    pub const NO_AVAILABLE_NODE: ErrorCode = "NO_AVAILABLE_NODE";

    // 下载领域（DOWNLOAD_*）
    /// 不支持的下载协议（HTTP 400）
    pub const DOWNLOAD_UNSUPPORTED_PROTOCOL: ErrorCode = "DOWNLOAD_UNSUPPORTED_PROTOCOL";
    /// 下载连接失败（HTTP 502）
    pub const DOWNLOAD_CONNECTION_FAILED: ErrorCode = "DOWNLOAD_CONNECTION_FAILED";
    /// 下载超时（HTTP 504）
    pub const DOWNLOAD_TIMEOUT: ErrorCode = "DOWNLOAD_TIMEOUT";
    /// 下载校验和不匹配（HTTP 409）
    pub const DOWNLOAD_CHECKSUM_MISMATCH: ErrorCode = "DOWNLOAD_CHECKSUM_MISMATCH";
    /// 磁盘空间不足（HTTP 507）
    pub const DOWNLOAD_DISK_FULL: ErrorCode = "DOWNLOAD_DISK_FULL";
    /// 部分内容，断点续传（HTTP 206）
    pub const DOWNLOAD_PARTIAL_CONTENT: ErrorCode = "DOWNLOAD_PARTIAL_CONTENT";
    /// 请求范围不满足（HTTP 416）
    pub const DOWNLOAD_RANGE_NOT_SATISFIED: ErrorCode = "DOWNLOAD_RANGE_NOT_SATISFIED";

    // 工作流领域（WORKFLOW_*）
    /// 工作流不存在（HTTP 404）
    pub const WORKFLOW_NOT_FOUND: ErrorCode = "WORKFLOW_NOT_FOUND";
    /// 工作流已存在（HTTP 409）
    pub const WORKFLOW_ALREADY_EXISTS: ErrorCode = "WORKFLOW_ALREADY_EXISTS";
    /// 工作流已禁用（HTTP 409）
    pub const WORKFLOW_DISABLED: ErrorCode = "WORKFLOW_DISABLED";
    /// Cron 表达式无效（HTTP 400）
    pub const WORKFLOW_INVALID_CRON: ErrorCode = "WORKFLOW_INVALID_CRON";
    /// 工作流触发失败（HTTP 500）
    pub const WORKFLOW_TRIGGER_FAILED: ErrorCode = "WORKFLOW_TRIGGER_FAILED";

    // 配置领域（CONFIG_*）
    /// 配置无效（HTTP 400）
    pub const CONFIG_INVALID: ErrorCode = "CONFIG_INVALID";
    /// 配置项不存在（HTTP 404）
    pub const CONFIG_NOT_FOUND: ErrorCode = "CONFIG_NOT_FOUND";
    /// 配置只读，不可修改（HTTP 409）
    pub const CONFIG_READ_ONLY: ErrorCode = "CONFIG_READ_ONLY";
}

/// 核心错误类型
///
/// 覆盖所有生态项目共用的基础错误场景。领域项目可基于
/// [`CoreError`] 通过 `#[from]` 转换接入自己的领域错误。
#[derive(Debug, Error)]
pub enum CoreError {
    /// 内部错误，不应发生的异常（HTTP 500）
    #[error("内部错误: {0}")]
    Internal(String),

    /// 参数校验失败（HTTP 400）
    #[error("参数错误: {0}")]
    InvalidParam(String),

    /// 未认证，缺少或未提供凭据（HTTP 401）
    #[error("未授权")]
    Unauthorized,

    /// 已认证但无权限（HTTP 403）
    #[error("禁止访问")]
    Forbidden,

    /// 认证失败，凭据错误（HTTP 401）
    #[error("认证失败: {0}")]
    AuthFailed(String),

    /// 资源不存在（HTTP 404）
    #[error("未找到: {0}")]
    NotFound(String),

    /// 状态冲突，资源当前状态不允许该操作（HTTP 409）
    #[error("状态冲突: {0}")]
    Conflict(String),

    /// 请求过于频繁，触发限流（HTTP 429）
    #[error("请求过于频繁")]
    RateLimited,

    /// 服务不可用（HTTP 503）
    #[error("服务不可用: {0}")]
    ServiceUnavailable(String),

    /// 网络错误，连接/读取/写入失败
    #[error("网络错误: {0}")]
    Network(String),

    /// 超时错误，操作超过时限
    #[error("超时: {0}")]
    Timeout(String),

    /// IO 错误，文件读写失败
    #[error("IO错误: {0}")]
    IO(String),

    /// 认证错误，登录/凭据验证失败
    #[error("认证错误: {0}")]
    Auth(String),

    /// 未初始化错误，资源尚未准备好
    #[error("未初始化: {0}")]
    NotInitialized(String),

    /// 外部错误，来自下层依赖的未归类错误（HTTP 500）
    #[error("外部错误: {0}")]
    External(#[from] anyhow::Error),
}

/// 错误码到 HTTP 状态码的映射关系，与 `docs/standards/error-codes.md` 一致
pub fn error_code_http_status(code: &str) -> u16 {
    match code {
        // 400
        codes::INVALID_PARAM
        | codes::DOWNLOAD_UNSUPPORTED_PROTOCOL
        | codes::WORKFLOW_INVALID_CRON
        | codes::CONFIG_INVALID => 400,
        // 401
        codes::UNAUTHORIZED
        | codes::AUTH_TOKEN_INVALID
        | codes::AUTH_TOKEN_EXPIRED
        | codes::AUTH_TOKEN_REVOKED => 401,
        // 403
        codes::FORBIDDEN | codes::AUTH_PERMISSION_DENIED => 403,
        // 404
        codes::NOT_FOUND
        | codes::TASK_NOT_FOUND
        | codes::NODE_NOT_FOUND
        | codes::DISPATCH_NOT_FOUND
        | codes::WORKFLOW_NOT_FOUND
        | codes::CONFIG_NOT_FOUND => 404,
        // 408
        codes::NODE_HEARTBEAT_TIMEOUT => 408,
        // 409
        codes::CONFLICT
        | codes::TASK_ALREADY_EXISTS
        | codes::TASK_INVALID_STATE
        | codes::TASK_DISABLED
        | codes::NODE_ALREADY_REGISTERED
        | codes::NODE_OFFLINE
        | codes::NODE_VERSION_TOO_OLD
        | codes::DISPATCH_ALREADY_CLAIMED
        | codes::DISPATCH_INVALID_STATE
        | codes::DISPATCH_EXPIRED
        | codes::DOWNLOAD_CHECKSUM_MISMATCH
        | codes::WORKFLOW_ALREADY_EXISTS
        | codes::WORKFLOW_DISABLED
        | codes::CONFIG_READ_ONLY => 409,
        // 416
        codes::DOWNLOAD_RANGE_NOT_SATISFIED => 416,
        // 429
        codes::RATE_LIMITED => 429,
        // 500
        codes::INTERNAL_ERROR | codes::TASK_DOWNLOAD_FAILED | codes::WORKFLOW_TRIGGER_FAILED => 500,
        // 502
        codes::DOWNLOAD_CONNECTION_FAILED => 502,
        // 503
        codes::SERVICE_UNAVAILABLE | codes::NO_AVAILABLE_NODE => 503,
        // 504
        codes::DOWNLOAD_TIMEOUT => 504,
        // 507
        codes::DOWNLOAD_DISK_FULL => 507,
        // 206
        codes::DOWNLOAD_PARTIAL_CONTENT => 206,
        // 兜底
        _ => 500,
    }
}

impl CoreError {
    /// 获取错误码
    pub fn code(&self) -> ErrorCode {
        match self {
            CoreError::Internal(_) => codes::INTERNAL_ERROR,
            CoreError::InvalidParam(_) => codes::INVALID_PARAM,
            CoreError::Unauthorized => codes::UNAUTHORIZED,
            CoreError::Forbidden => codes::FORBIDDEN,
            CoreError::AuthFailed(_) => codes::AUTH_TOKEN_INVALID,
            CoreError::NotFound(_) => codes::NOT_FOUND,
            CoreError::Conflict(_) => codes::CONFLICT,
            CoreError::RateLimited => codes::RATE_LIMITED,
            CoreError::ServiceUnavailable(_) => codes::SERVICE_UNAVAILABLE,
            CoreError::Network(_) => codes::DOWNLOAD_CONNECTION_FAILED,
            CoreError::Timeout(_) => codes::DOWNLOAD_TIMEOUT,
            CoreError::IO(_) => codes::INTERNAL_ERROR,
            CoreError::Auth(_) => codes::AUTH_TOKEN_INVALID,
            CoreError::NotInitialized(_) => codes::INTERNAL_ERROR,
            CoreError::External(_) => codes::INTERNAL_ERROR,
        }
    }

    /// 获取 HTTP 状态码
    pub fn http_status(&self) -> u16 {
        match self {
            CoreError::Internal(_)
            | CoreError::External(_)
            | CoreError::IO(_)
            | CoreError::NotInitialized(_) => 500,
            CoreError::InvalidParam(_) => 400,
            CoreError::Unauthorized | CoreError::AuthFailed(_) | CoreError::Auth(_) => 401,
            CoreError::Forbidden => 403,
            CoreError::NotFound(_) => 404,
            CoreError::Conflict(_) => 409,
            CoreError::RateLimited => 429,
            CoreError::ServiceUnavailable(_) => 503,
            CoreError::Network(_) => 502,
            CoreError::Timeout(_) => 504,
        }
    }
}

impl From<&str> for CoreError {
    fn from(msg: &str) -> Self {
        CoreError::InvalidParam(msg.to_string())
    }
}

/// 统一 Result 类型
pub type Result<T> = std::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_mapping_matches_documented_codes() {
        // 与 error-codes.md 表格逐条对齐
        assert_eq!(error_code_http_status(codes::INVALID_PARAM), 400);
        assert_eq!(error_code_http_status(codes::UNAUTHORIZED), 401);
        assert_eq!(error_code_http_status(codes::AUTH_TOKEN_INVALID), 401);
        assert_eq!(error_code_http_status(codes::AUTH_TOKEN_EXPIRED), 401);
        assert_eq!(error_code_http_status(codes::AUTH_TOKEN_REVOKED), 401);
        assert_eq!(error_code_http_status(codes::FORBIDDEN), 403);
        assert_eq!(error_code_http_status(codes::AUTH_PERMISSION_DENIED), 403);
        assert_eq!(error_code_http_status(codes::NOT_FOUND), 404);
        assert_eq!(error_code_http_status(codes::TASK_NOT_FOUND), 404);
        assert_eq!(error_code_http_status(codes::NODE_NOT_FOUND), 404);
        assert_eq!(error_code_http_status(codes::DISPATCH_NOT_FOUND), 404);
        assert_eq!(error_code_http_status(codes::WORKFLOW_NOT_FOUND), 404);
        assert_eq!(error_code_http_status(codes::CONFIG_NOT_FOUND), 404);
        assert_eq!(error_code_http_status(codes::NODE_HEARTBEAT_TIMEOUT), 408);
        assert_eq!(error_code_http_status(codes::CONFLICT), 409);
        assert_eq!(error_code_http_status(codes::TASK_ALREADY_EXISTS), 409);
        assert_eq!(error_code_http_status(codes::TASK_INVALID_STATE), 409);
        assert_eq!(error_code_http_status(codes::TASK_DISABLED), 409);
        assert_eq!(error_code_http_status(codes::NODE_ALREADY_REGISTERED), 409);
        assert_eq!(error_code_http_status(codes::NODE_OFFLINE), 409);
        assert_eq!(error_code_http_status(codes::NODE_VERSION_TOO_OLD), 409);
        assert_eq!(error_code_http_status(codes::DISPATCH_ALREADY_CLAIMED), 409);
        assert_eq!(error_code_http_status(codes::DISPATCH_INVALID_STATE), 409);
        assert_eq!(error_code_http_status(codes::DISPATCH_EXPIRED), 409);
        assert_eq!(
            error_code_http_status(codes::DOWNLOAD_CHECKSUM_MISMATCH),
            409
        );
        assert_eq!(error_code_http_status(codes::WORKFLOW_ALREADY_EXISTS), 409);
        assert_eq!(error_code_http_status(codes::WORKFLOW_DISABLED), 409);
        assert_eq!(error_code_http_status(codes::CONFIG_READ_ONLY), 409);
        assert_eq!(
            error_code_http_status(codes::DOWNLOAD_RANGE_NOT_SATISFIED),
            416
        );
        assert_eq!(error_code_http_status(codes::RATE_LIMITED), 429);
        assert_eq!(error_code_http_status(codes::INTERNAL_ERROR), 500);
        assert_eq!(error_code_http_status(codes::TASK_DOWNLOAD_FAILED), 500);
        assert_eq!(error_code_http_status(codes::WORKFLOW_TRIGGER_FAILED), 500);
        assert_eq!(
            error_code_http_status(codes::DOWNLOAD_CONNECTION_FAILED),
            502
        );
        assert_eq!(error_code_http_status(codes::SERVICE_UNAVAILABLE), 503);
        assert_eq!(error_code_http_status(codes::NO_AVAILABLE_NODE), 503);
        assert_eq!(error_code_http_status(codes::DOWNLOAD_TIMEOUT), 504);
        assert_eq!(error_code_http_status(codes::DOWNLOAD_DISK_FULL), 507);
        assert_eq!(error_code_http_status(codes::DOWNLOAD_PARTIAL_CONTENT), 206);
    }

    #[test]
    fn core_error_code_and_http_status() {
        let err = CoreError::InvalidParam("bad".into());
        assert_eq!(err.code(), codes::INVALID_PARAM);
        assert_eq!(err.http_status(), 400);

        let err = CoreError::NotFound("task".into());
        assert_eq!(err.code(), codes::NOT_FOUND);
        assert_eq!(err.http_status(), 404);

        let err = CoreError::Unauthorized;
        assert_eq!(err.code(), codes::UNAUTHORIZED);
        assert_eq!(err.http_status(), 401);

        let err = CoreError::AuthFailed("token expired".into());
        assert_eq!(err.code(), codes::AUTH_TOKEN_INVALID);
        assert_eq!(err.http_status(), 401);
    }

    #[test]
    fn error_display_contains_chinese_message() {
        let err = CoreError::NotFound("任务不存在".into());
        assert!(err.to_string().contains("未找到"));
        assert!(err.to_string().contains("任务不存在"));
    }

    #[test]
    fn external_error_maps_to_internal() {
        let err = CoreError::External(anyhow::anyhow!("io error"));
        assert_eq!(err.code(), codes::INTERNAL_ERROR);
        assert_eq!(err.http_status(), 500);
    }

    #[test]
    fn from_str_converts_to_invalid_param() {
        let err: CoreError = "bad input".into();
        assert_eq!(err.code(), codes::INVALID_PARAM);
    }

    #[test]
    fn unknown_code_defaults_to_500() {
        assert_eq!(error_code_http_status("UNKNOWN_CODE_XYZ"), 500);
    }
}
