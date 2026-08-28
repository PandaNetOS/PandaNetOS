//! 统一响应格式
//!
//! 所有 API 响应都应使用此模块定义的格式，确保前后端契约一致。
//! 格式遵循 [`docs/standards/api.md`] 标准。

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// 默认页码（从 1 开始）
pub const DEFAULT_PAGE: u32 = 1;
/// 默认每页大小
pub const DEFAULT_PAGE_SIZE: u32 = 20;
/// 每页大小上限
pub const MAX_PAGE_SIZE: u32 = 100;

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
///
/// `details` 为可选字段，序列化时缺失不输出，与文档中"可选"语义一致。
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    /// 错误码（字符串格式，如 TASK_NOT_FOUND）
    pub code: String,
    /// 错误信息
    pub message: String,
    /// 错误详情（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    /// 构造错误响应
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// 附加错误详情
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// 关联的 HTTP 状态码（依据错误码标准映射）
    pub fn http_status(&self) -> u16 {
        crate::error::error_code_http_status(self.code.as_str())
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
    /// 构造分页结果
    ///
    /// `page_size` 为 0 时按空分页处理，`total_pages` 为 0。
    pub fn new(items: Vec<T>, total: u64, page: u32, page_size: u32) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            total.div_ceil(page_size as u64)
        };
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }

    /// 空分页结果
    pub fn empty() -> Self {
        Self::new(Vec::new(), 0, DEFAULT_PAGE, DEFAULT_PAGE_SIZE)
    }

    /// 从查询参数构造（自动规范化 page_size）
    pub fn from_query(items: Vec<T>, total: u64, query: &PageQuery) -> Self {
        Self::new(items, total, query.page, query.effective_page_size())
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
    DEFAULT_PAGE
}

fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

impl PageQuery {
    /// 处理后的每页大小（0 视为默认，超过上限截断为 MAX_PAGE_SIZE）
    pub fn effective_page_size(&self) -> u32 {
        match self.page_size {
            0 => DEFAULT_PAGE_SIZE,
            n if n > MAX_PAGE_SIZE => MAX_PAGE_SIZE,
            n => n,
        }
    }

    /// 计算 SQL OFFSET（使用规范化后的 page_size）
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1) as u64) * self.effective_page_size() as u64
    }

    /// 计算 SQL LIMIT（规范化后的每页大小）
    pub fn limit(&self) -> u32 {
        self.effective_page_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{codes, CoreError};

    #[test]
    fn api_response_ok() {
        let resp = ApiResponse::ok(42u32);
        assert_eq!(resp.code, 0);
        assert_eq!(resp.data, 42);
        assert_eq!(resp.message, "ok");
    }

    #[test]
    fn api_response_ok_with_msg() {
        let resp = ApiResponse::ok_with_msg(vec![1u32, 2], "done");
        assert_eq!(resp.message, "done");
    }

    #[test]
    fn api_error_http_status() {
        let err = ApiError::new(codes::TASK_NOT_FOUND, "任务不存在");
        assert_eq!(err.http_status(), 404);
        assert!(err.details.is_none());
    }

    #[test]
    fn api_error_from_core_error() {
        let core = CoreError::NotFound("任务不存在".into());
        let api: ApiError = core.into();
        assert_eq!(api.code, codes::NOT_FOUND);
        assert_eq!(api.http_status(), 404);
    }

    #[test]
    fn api_error_details_serialization() {
        let err = ApiError::new(codes::INVALID_PARAM, "参数错误")
            .with_details(serde_json::json!({ "url": ["不能为空"] }));
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"details\""));

        let plain = serde_json::to_string(&ApiError::new(codes::INVALID_PARAM, "x")).unwrap();
        assert!(!plain.contains("details"));
    }

    #[test]
    fn page_result_total_pages() {
        let p = PageResult::new(vec![1u32, 2], 5, 1, 2);
        assert_eq!(p.total_pages, 3);

        let p0 = PageResult::new(Vec::<u32>::new(), 0, 1, 0);
        assert_eq!(p0.total_pages, 0);
    }

    #[test]
    fn page_result_empty_and_from_query() {
        let empty = PageResult::<u32>::empty();
        assert!(empty.items.is_empty());
        assert_eq!(empty.page, DEFAULT_PAGE);
        assert_eq!(empty.page_size, DEFAULT_PAGE_SIZE);

        let q = PageQuery {
            page: 2,
            page_size: 10,
        };
        let from_q = PageResult::from_query(vec![3u32], 25, &q);
        assert_eq!(from_q.page, 2);
        assert_eq!(from_q.page_size, 10);
        assert_eq!(from_q.total_pages, 3);
    }

    #[test]
    fn page_query_normalization() {
        // page_size = 0 → 默认 20
        let q0 = PageQuery {
            page: 1,
            page_size: 0,
        };
        assert_eq!(q0.effective_page_size(), DEFAULT_PAGE_SIZE);
        assert_eq!(q0.offset(), 0);
        assert_eq!(q0.limit(), DEFAULT_PAGE_SIZE);

        // page_size 超过上限 → 截断为 100
        let q_big = PageQuery {
            page: 3,
            page_size: 500,
        };
        assert_eq!(q_big.effective_page_size(), MAX_PAGE_SIZE);
        assert_eq!(q_big.offset(), 200);
        assert_eq!(q_big.limit(), MAX_PAGE_SIZE);

        // page = 0 → saturating_sub 防下溢
        let q_zero = PageQuery {
            page: 0,
            page_size: 20,
        };
        assert_eq!(q_zero.offset(), 0);
    }

    #[test]
    fn page_query_deserialize_with_defaults() {
        let q: PageQuery = serde_json::from_str(r#"{"page":2}"#).unwrap();
        assert_eq!(q.page, 2);
        assert_eq!(q.page_size, DEFAULT_PAGE_SIZE);

        let empty: PageQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(empty.page, DEFAULT_PAGE);
        assert_eq!(empty.page_size, DEFAULT_PAGE_SIZE);
    }
}
