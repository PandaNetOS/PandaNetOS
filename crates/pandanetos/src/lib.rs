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
//!
//! # 快速上手
//!
//! ```rust
//! use pandanetos::prelude::*;
//! ```

pub mod bittorrent;
pub mod capability;
pub mod config;
pub mod domain;
pub mod error;
pub mod logging;
pub mod protocol;
pub mod response;
pub mod service;
pub mod time;
pub mod utils;

// ---- 顶层 re-export（最常用类型，直接 `pandanetos::X` 可用）----
pub use bittorrent::{FileInfo, Infohash, MetadataInfo, PeerInfo, PeerSource};
pub use capability::{CapabilityManifest, ComponentRole};
pub use domain::{Dispatch, Node, NodeStatus, Task, TaskStatus};
pub use error::{CoreError, ErrorCode, Result};
pub use protocol::{ClientMsg, ServerMsg};
pub use response::{ApiError, ApiResponse, PageQuery, PageResult};
pub use service::{
    ServiceAgentInfo, ServiceChangeType, ServiceChangedEvent, ServiceHealth, ServiceQueryResponse,
};
pub use time::now_rfc3339;

/// 一站式导入：`use pandanetos::prelude::*;`
pub mod prelude {
    pub use crate::bittorrent::{FileInfo, Infohash, MetadataInfo, PeerInfo, PeerSource};
    pub use crate::capability::{
        BasicInfo, BuildInfo, Capabilities, CapabilityManifest, Communication, ComponentRole,
        ConfigurableParam, StatusReport,
    };
    pub use crate::domain::{
        CancellationToken, Chunk, ChunkDownloader, ChunkSet, ChunkState, ChunkStats, ChunkWriter,
        Dispatch, DownloadFileInfo, DownloadProgress, DownloadResult, DownloadSource,
        DownloadStrategy, MirrorDiscoverer, Node, NodeStatus, SourceCapabilities, SourceHealth,
        Task, TaskStatus,
    };
    pub use crate::error::{codes, CoreError, ErrorCode, Result};
    pub use crate::protocol::{
        paths, ClientMsg, DiscoverTask, DiscoveryResult, DiscoveryStarted, DispatchConfig,
        HeartbeatReq, PeerBrief, RegisterReq, RegisterResp, ServerMsg, UpdateNodeCapabilitiesReq,
        API_PREFIX,
    };
    pub use crate::response::{ApiError, ApiResponse, PageQuery, PageResult};
    pub use crate::service::{
        ServiceAgentInfo, ServiceChangeType, ServiceChangedEvent, ServiceHealth, ServiceQuery,
        ServiceQueryResponse, CAP_ANNOUNCE, CAP_CACHE, CAP_DEDUP, CAP_DHT, CAP_HEALTH_CHECK,
        CAP_PEX, CAP_PRIORITY_SORTING, CAP_TRACKER,
    };
    pub use crate::time::{format_rfc3339, now_millis, now_rfc3339, parse_rfc3339};
    pub use crate::utils::{format_bytes, format_speed, is_valid_uuid, new_uuid, parse_bytes};
}
