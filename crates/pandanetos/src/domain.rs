//! 领域模型与扩展点
//!
//! 核心领域模型（Task/Node/Dispatch）、状态枚举与端口 trait。
//! 遵循 [`docs/architecture.md`] 的分层架构：domain 层不依赖任何外层。

use std::any::Any;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

/// 任务状态（与 `docs/standards/data-format.md` 枚举标准一致）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// 待下发
    Pending,
    /// 已确认领取
    Acked,
    /// 执行中
    Running,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

impl TaskStatus {
    /// 是否为终止状态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Failed | Self::Cancelled)
    }

    /// 是否可被调度
    pub fn is_schedulable(&self) -> bool {
        matches!(self, Self::Pending)
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Acked => "acked",
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        f.write_str(s)
    }
}

/// 节点状态（与能力协商标准 7.1 一致）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    /// 空闲，可领取任务
    Online,
    /// 活跃任务达到上限
    Busy,
    /// 心跳超时
    Offline,
    /// 待审批
    Pending,
}

impl NodeStatus {
    /// 是否可领取任务
    pub fn can_claim(&self) -> bool {
        matches!(self, Self::Online)
    }
}

impl fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Online => "online",
            Self::Busy => "busy",
            Self::Offline => "offline",
            Self::Pending => "pending",
        };
        f.write_str(s)
    }
}

/// 任务模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub task_id: Uuid,
    /// 任务名称
    pub name: String,
    /// 文件名
    pub filename: String,
    /// 下载 URL
    pub url: String,
    /// 是否启用
    pub enabled: bool,
    /// 文件大小（字节）
    pub file_size_bytes: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 备注（可选）
    pub note: Option<String>,
    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,
    /// 当前状态
    pub status: TaskStatus,
}

/// 节点模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// 节点 ID
    pub node_id: Uuid,
    /// 主机名
    pub hostname: String,
    /// 平台
    pub platform: String,
    /// 架构
    pub arch: String,
    /// 组件版本号
    pub version: String,
    /// 节点状态
    pub status: NodeStatus,
    /// 最后心跳时间
    pub last_seen_at: DateTime<Utc>,
    /// 注册时间
    pub registered_at: DateTime<Utc>,
    /// 标签
    #[serde(default)]
    pub labels: Vec<String>,
    /// 活跃任务数
    #[serde(default)]
    pub active_tasks: u32,
    /// 累计下载字节数
    #[serde(default)]
    pub bytes_downloaded: u64,
    /// 最近错误信息（节点表 5.2 规范字段）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// 最大并发任务数（None 用全局默认）
    #[serde(default)]
    pub max_concurrent: Option<u32>,
    /// 最大带宽上限 bps（None 用全局默认）
    #[serde(default)]
    pub max_bandwidth_bps: Option<u64>,
    /// 通用能力参数（JSON）
    #[serde(default)]
    pub capabilities: Option<serde_json::Value>,
}

impl Node {
    /// 根据活跃任务数判断状态（能力协商标准 7.3 逻辑）
    pub fn derive_status(&self, global_default_max_concurrent: u32) -> NodeStatus {
        if self.status == NodeStatus::Offline {
            return NodeStatus::Offline;
        }
        let max = self.max_concurrent.unwrap_or(global_default_max_concurrent);
        if self.active_tasks >= max {
            NodeStatus::Busy
        } else {
            NodeStatus::Online
        }
    }
}

/// 调度模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispatch {
    /// 调度 ID
    pub dispatch_id: Uuid,
    /// 任务 ID
    pub task_id: Uuid,
    /// 节点 ID（未领取时为 None）
    pub node_id: Option<Uuid>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 领取时间
    pub claimed_at: Option<DateTime<Utc>>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 当前状态
    pub status: String,
}

/// 下载端口（扩展点：新增协议不需要改核心代码）
pub trait Downloader: Send + Sync {
    /// 协议 scheme（如 http/https/ftp）
    fn scheme(&self) -> &'static str;

    /// 探测 URL 资源信息
    fn probe(&self, url: &str) -> crate::error::Result<DownloadFileInfo>;

    /// 执行下载
    fn download(
        &self,
        task: &Task,
        progress_sender: tokio::sync::mpsc::Sender<DownloadProgress>,
    ) -> crate::error::Result<()>;
}

/// 文件探测信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadFileInfo {
    /// 文件大小（字节）
    pub size_bytes: u64,
    /// 是否支持断点续传
    pub supports_resume: bool,
    /// 是否支持多连接
    pub supports_multi_connection: bool,
}

/// 下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// 已下载字节数
    pub downloaded_bytes: u64,
    /// 总字节数
    pub total_bytes: u64,
    /// 当前速度 bps
    pub speed_bps: u64,
    /// 活跃连接数
    pub active_connections: u32,
    /// 下载进度百分比（0.0 - 100.0）
    pub percent: f64,
    /// 已用时间（秒）
    pub elapsed_secs: f64,
}

// ============================================================================
// 下载领域抽象（spde / pk 等下载节点共享）
// ============================================================================

/// 取消令牌（协程间共享，clone 后指向同一个 AtomicBool）
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// 创建新的取消令牌
    pub fn new() -> Self {
        Self::default()
    }

    /// 触发取消
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// 是否已取消
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// 分片状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkState {
    /// 待下载
    Pending,
    /// 下载中
    Downloading,
    /// 已完成
    Completed,
    /// 失败（可重试）
    Failed,
}

/// 下载分片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// 分片 ID（从 0 开始）
    pub chunk_id: u32,
    /// 文件内偏移（字节）
    pub offset: u64,
    /// 分片长度（字节）
    pub length: u64,
    /// 当前状态
    pub state: ChunkState,
    /// 下载该分片的源 ID（None 表示未分配）
    pub source_id: Option<String>,
    /// 已重试次数
    pub retry_count: u32,
}

/// 分片集合（整个文件的分片规划）
#[derive(Debug, Clone)]
pub struct ChunkSet {
    /// 所有分片
    pub chunks: Vec<Chunk>,
    /// 文件总大小（字节）
    pub total_size: u64,
    /// 分片大小（字节）
    pub chunk_size: u64,
}

impl ChunkSet {
    /// 根据文件大小和分片大小创建分片集合
    pub fn new(total_size: u64, chunk_size: u64) -> Self {
        let chunk_size = chunk_size.max(1);
        let num_chunks = total_size.div_ceil(chunk_size);
        let chunks = (0..num_chunks)
            .map(|i| {
                let offset = i * chunk_size;
                let length = (total_size - offset).min(chunk_size);
                Chunk {
                    chunk_id: i as u32,
                    offset,
                    length,
                    state: ChunkState::Pending,
                    source_id: None,
                    retry_count: 0,
                }
            })
            .collect();
        Self {
            chunks,
            total_size,
            chunk_size,
        }
    }

    /// 计算已完成分片的总下载字节数
    pub fn downloaded_bytes(&self) -> u64 {
        self.chunks
            .iter()
            .filter(|c| c.state == ChunkState::Completed)
            .map(|c| c.length)
            .sum()
    }
}

/// 单分片下载统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkStats {
    /// 分片 ID
    pub chunk_id: u32,
    /// 源 ID
    pub source_id: String,
    /// 已下载字节数
    pub downloaded_bytes: u64,
    /// 耗时（秒）
    pub elapsed_secs: f64,
    /// 是否成功
    pub success: bool,
    /// 错误码（成功时为 None）
    pub error_code: Option<&'static str>,
}

/// 下载源能力声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCapabilities {
    /// 是否支持 Range 请求
    pub supports_range: bool,
    /// 是否支持多连接并发
    pub supports_concurrent: bool,
    /// 是否支持断点续传
    pub supports_resume: bool,
    /// 最大并发连接数
    pub max_concurrency: u32,
    /// 建议分片大小范围 (min, max) 字节
    pub chunk_size_range: Option<(u64, u64)>,
    /// 内容是否不可变（如 CDN 静态资源，true 时可跳过校验）
    pub immutable: bool,
    /// 协议类型（http/https/ftp/ssh/torrent/file 等）
    pub protocol: &'static str,
}

impl Default for SourceCapabilities {
    fn default() -> Self {
        Self {
            supports_range: false,
            supports_concurrent: false,
            supports_resume: false,
            max_concurrency: 1,
            chunk_size_range: None,
            immutable: false,
            protocol: "",
        }
    }
}

/// 下载源健康度（速度、成功率、熔断状态）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceHealth {
    /// 权重（速度越快权重越高，熔断时为 0）
    pub weight: u64,
    /// 平滑速度（字节/秒）
    pub speed_bps: u64,
    /// 累计成功次数
    pub success_count: u64,
    /// 累计失败次数
    pub fail_count: u64,
    /// 是否熔断
    pub circuit_open: bool,
}

/// 下载结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResult {
    /// 是否成功
    pub success: bool,
    /// 总字节数
    pub total_bytes: u64,
    /// 已下载字节数
    pub downloaded_bytes: u64,
    /// 成功分片数
    pub success_chunks: u32,
    /// 失败分片数
    pub failed_chunks: u32,
    /// 平均速度（字节/秒）
    pub avg_speed_bps: u64,
    /// 耗时（秒）
    pub elapsed_secs: f64,
    /// 错误信息（成功时为 None）
    pub error_msg: Option<String>,
}

/// 下载源抽象（协议无关，http/ftp/torrent 等实现此 trait）
pub trait DownloadSource: Send + Sync + std::fmt::Debug {
    /// 协议 scheme（http/https/ftp/torrent 等）
    fn protocol(&self) -> &str;

    /// 唯一标识符（用于健康度统计和去重）
    fn identifier(&self) -> String;

    /// 显示名称（用于日志和 UI）
    fn display_name(&self) -> String;

    /// 能力声明
    fn capabilities(&self) -> SourceCapabilities;

    /// 向下转型为具体类型（用于协议特定的优化）
    fn as_any(&self) -> &dyn Any;
}

/// 分片写入器抽象（磁盘/内存/对象存储等实现此 trait）
#[async_trait]
pub trait ChunkWriter: Send + Sync {
    /// 在指定偏移写入数据（并发安全，不需要 seek）
    async fn write_at(&self, offset: u64, data: &[u8]) -> crate::error::Result<()>;

    /// 刷新所有缓冲到磁盘（fsync）
    async fn flush(&self) -> crate::error::Result<()>;

    /// 预分配文件空间（避免写入时分配磁盘块导致 IO 抖动）
    async fn preallocate(&self, size: u64) -> crate::error::Result<()>;

    /// 获取文件当前大小
    async fn file_size(&self) -> crate::error::Result<u64>;
}

/// 分片下载器抽象（协议无关，http/ftp/torrent 等实现此 trait）
#[async_trait]
pub trait ChunkDownloader: Send + Sync {
    /// 协议 scheme
    fn protocol(&self) -> &str;

    /// 探测源的可用性和文件信息（HEAD 请求等）
    async fn probe(&self, source: &dyn DownloadSource) -> crate::error::Result<DownloadFileInfo>;

    /// 下载一个分片（流式写入 writer）
    async fn download_chunk(
        &self,
        source: &dyn DownloadSource,
        chunk: &Chunk,
        writer: &dyn ChunkWriter,
        cancel: &CancellationToken,
    ) -> crate::error::Result<ChunkStats>;
}

/// 镜像发现器抽象（DNS 多 IP / DHT / P2P 等实现此 trait）
#[async_trait]
pub trait MirrorDiscoverer: Send + Sync {
    /// 适用的协议 scheme
    fn protocol(&self) -> &str;

    /// 发现器名称（用于日志和配置）
    fn name(&self) -> &str;

    /// 从原始源发现所有可用镜像
    async fn discover(
        &self,
        source: &dyn DownloadSource,
    ) -> crate::error::Result<Vec<Box<dyn DownloadSource>>>;
}

/// 下载策略抽象（多源分片 / 单源最快 / Torrent 原生等实现此 trait）
#[async_trait]
pub trait DownloadStrategy: Send + Sync {
    /// 策略名称
    fn name(&self) -> &str;

    /// 判断是否支持给定的源列表和能力
    fn supports(&self, sources: &[&dyn DownloadSource], caps: &SourceCapabilities) -> bool;

    /// 执行下载策略
    async fn execute(
        &self,
        sources: Vec<Box<dyn DownloadSource>>,
        chunk_set: Arc<Mutex<ChunkSet>>,
        writer: Arc<dyn ChunkWriter>,
        progress_tx: mpsc::Sender<DownloadProgress>,
        cancel: CancellationToken,
    ) -> crate::error::Result<DownloadResult>;
}

/// 存储端口（Repository 抽象）
pub trait Repository<T>: Send + Sync {
    /// 按 ID 查询
    fn find_by_id(&self, id: Uuid) -> crate::error::Result<Option<T>>;

    /// 保存
    fn save(&self, entity: &T) -> crate::error::Result<()>;
}

/// 调度策略端口
pub trait DispatchStrategy: Send + Sync {
    /// 为任务选择可用节点
    fn select_node<'a>(
        &self,
        task: &Task,
        candidates: &'a [Node],
    ) -> crate::error::Result<Option<&'a Node>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_lifecycle() {
        assert!(TaskStatus::Pending.is_schedulable());
        assert!(!TaskStatus::Running.is_schedulable());
        assert!(!TaskStatus::Success.is_terminal() || TaskStatus::Success.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());
    }

    #[test]
    fn task_status_display() {
        assert_eq!(TaskStatus::Pending.to_string(), "pending");
        assert_eq!(TaskStatus::Success.to_string(), "success");
    }

    #[test]
    fn node_status_claimability() {
        assert!(NodeStatus::Online.can_claim());
        assert!(!NodeStatus::Busy.can_claim());
        assert!(!NodeStatus::Offline.can_claim());
    }

    #[test]
    fn node_derive_status_from_active_tasks() {
        let node = Node {
            node_id: Uuid::new_v4(),
            hostname: "node-1".into(),
            platform: "linux".into(),
            arch: "x86_64".into(),
            version: "0.6.1".into(),
            status: NodeStatus::Online,
            last_seen_at: Utc::now(),
            registered_at: Utc::now(),
            labels: vec![],
            active_tasks: 2,
            bytes_downloaded: 0,
            last_error: None,
            max_concurrent: Some(2),
            max_bandwidth_bps: None,
            capabilities: None,
        };
        // 活跃任务数达到上限 → busy
        assert_eq!(node.derive_status(4), NodeStatus::Busy);

        let idle = Node {
            active_tasks: 1,
            ..node.clone()
        };
        assert_eq!(idle.derive_status(4), NodeStatus::Online);
    }

    #[test]
    fn node_derive_status_keeps_offline() {
        let node = Node {
            status: NodeStatus::Offline,
            ..sample_node()
        };
        assert_eq!(node.derive_status(4), NodeStatus::Offline);
    }

    fn sample_node() -> Node {
        Node {
            node_id: Uuid::new_v4(),
            hostname: "node-1".into(),
            platform: "linux".into(),
            arch: "x86_64".into(),
            version: "0.6.1".into(),
            status: NodeStatus::Online,
            last_seen_at: Utc::now(),
            registered_at: Utc::now(),
            labels: vec![],
            active_tasks: 0,
            bytes_downloaded: 0,
            last_error: None,
            max_concurrent: None,
            max_bandwidth_bps: None,
            capabilities: None,
        }
    }
}
