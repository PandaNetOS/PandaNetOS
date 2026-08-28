//! 领域模型与扩展点
//!
//! 核心领域模型（Task/Node/Dispatch）、状态枚举与端口 trait。
//! 遵循 [`docs/architecture.md`] 的分层架构：domain 层不依赖任何外层。

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
