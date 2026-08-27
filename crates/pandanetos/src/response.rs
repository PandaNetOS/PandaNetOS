//! 统一响应格式
//!
//! 所有 API 响应都应使用此模块定义的格式，确保前后端契约一致。

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, ErrorCode};

/// 统一成功响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 业务状态码，0 表示成功，非 0 表示错误码
    pub code: i32,
    /// 响应数据
    pub data: T,
    /// 提示信息
    pub message: String,
}

impl<T> ApiResponse<T> {
    /// 成功响应
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            data,
            message: "ok".to_string(),
        }
    }

    /// 成功响应（带自定义消息）
    pub fn ok_with_msg(data: T, message: impl Into<String>) -> Self {
        Self {
            code: 0,
            data,
            message: message.into(),
        }
    }
}

/// 统一错误响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    /// 错误码（字符串格式，如 TASK_NOT_FOUND）
    pub code: ErrorCode,
    /// 错误信息
    pub message: String,
    /// 错误详情（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        Self::new(err.code(), err.to_string())
    }
}

/// 分页结果
#[derive(Debug, Serialize, Deserialize)]
pub struct PageResult<T> {
    /// 数据列表
    pub items: Vec<T>,
    /// 总数
    pub total: u64,
    /// 当前页码（从 1 开始）
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
    /// 总页数
    pub total_pages: u64,
}

impl<T> PageResult<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, page_size: u32) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            (total + page_size as u64 - 1) / page_size as u64
        };
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

/// 分页查询参数
#[derive(Debug, Deserialize)]
pub struct PageQuery {
    /// 页码（从 1 开始，默认 1）
    #[serde(default = "default_page")]
    pub page: u32,
    /// 每页大小（默认 20，最大 100）
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

impl PageQuery {
    /// 计算 SQL OFFSET
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1) as u64) * self.page_size as u64
    }

    /// 计算 SQL LIMIT（限制最大 100）
    pub fn limit(&self) -> u32 {
        self.page_size.min(100)
    }
}
