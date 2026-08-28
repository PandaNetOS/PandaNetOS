//! 通信协议定义
//!
//! 统一生态内组件间通信的 API 路径、消息结构与数据模型。
//! 遵循 [`docs/standards/api.md`] 与 [`docs/standards/node-protocol.md`] 标准。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// API 路径前缀
pub const API_PREFIX: &str = "/api/v1";

/// 统一 API 路径定义
pub mod paths {
    /// API 路径前缀
    pub const PREFIX: &str = super::API_PREFIX;

    // ===== 节点 =====
    /// 节点注册
    pub const AGENT_REGISTER: &str = "/api/v1/agent/register";
    /// 节点 WebSocket 实时通道
    pub const AGENT_WS: &str = "/api/v1/agent/ws";
    /// 节点心跳
    pub const NODE_HEARTBEAT: &str = "/api/v1/nodes/{id}/heartbeat";
    /// 节点列表
    pub const NODES: &str = "/api/v1/nodes";
    /// 节点详情
    pub const NODE_DETAIL: &str = "/api/v1/nodes/{id}";
    /// 节点能力修改
    pub const NODE_CAPABILITIES: &str = "/api/v1/nodes/{id}/capabilities";
    /// 清理离线节点
    pub const NODES_OFFLINE: &str = "/api/v1/nodes/offline";
    /// 节点配置文件（YAML）
    pub const NODE_CONFIG_YAML: &str = "/api/v1/nodes/{id}/config.yaml";

    // ===== 总览 =====
    /// 系统总览
    pub const OVERVIEW: &str = "/api/v1/overview";

    // ===== 任务 =====
    /// 任务列表
    pub const TASKS: &str = "/api/v1/tasks";
    /// 任务详情
    pub const TASK_DETAIL: &str = "/api/v1/tasks/{id}";
    /// 触发任务
    pub const TASK_TRIGGER: &str = "/api/v1/tasks/{id}/trigger";
    /// 批量任务操作
    pub const TASKS_BATCH: &str = "/api/v1/tasks/batch";

    // ===== 调度 =====
    /// 待领取调度
    pub const DISPATCHES_PENDING: &str = "/api/v1/dispatches/pending";
    /// 领取任务
    pub const DISPATCH_CLAIM: &str = "/api/v1/dispatches/claim";
}

/// 节点注册请求（与能力协商标准 5.1 对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterReq {
    /// 节点唯一 ID（None 则服务端生成）
    #[serde(default)]
    pub node_id: Option<Uuid>,
    /// 主机名
    pub hostname: String,
    /// 平台（linux/windows/macos）
    pub platform: String,
    /// 架构（x86_64/aarch64）
    pub arch: String,
    /// 组件版本号
    pub version: String,
    /// 自定义标签
    #[serde(default)]
    pub labels: Vec<String>,
    /// 最大并发任务数
    #[serde(default)]
    pub max_concurrent: Option<u32>,
    /// 最大带宽上限 bps
    #[serde(default)]
    pub max_bandwidth_bps: Option<u64>,
    /// 通用能力参数（JSON，灵活扩展）
    #[serde(default)]
    pub capabilities: Option<serde_json::Value>,
}

/// 节点注册响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResp {
    /// 节点 ID
    pub node_id: Uuid,
    /// 轮询间隔（秒）
    pub poll_interval_secs: u64,
    /// 主控监听地址
    pub master_listen: String,
    /// 节点注册后的状态（online/pending，可选字段，兼容旧主控不返回状态）
    #[serde(default)]
    pub status: Option<String>,
}

/// 节点心跳请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatReq {
    /// 节点 ID
    pub node_id: Uuid,
    /// 当前活跃任务数
    #[serde(default)]
    pub active_tasks: u32,
    /// 累计下载字节数
    #[serde(default)]
    pub bytes_downloaded: u64,
    /// 当前总速度 bps
    #[serde(default)]
    pub speed_bps: u64,
    /// 节点状态（online/busy/offline）
    #[serde(default = "default_online")]
    pub status: String,
}

fn default_online() -> String {
    "online".to_string()
}

/// 能力修改请求（合并模式，能力协商标准 5.3 对应）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateNodeCapabilitiesReq {
    /// 最大并发任务数（None 则不修改）
    #[serde(default)]
    pub max_concurrent: Option<u32>,
    /// 最大带宽上限 bps（None 则不修改）
    #[serde(default)]
    pub max_bandwidth_bps: Option<u64>,
    /// 通用能力参数合并（未提供则不修改）
    #[serde(default)]
    pub capabilities: Option<serde_json::Value>,
}

/// WebSocket 主控→节点消息（ServerMsg）
///
/// 由 PK 主控通过 WebSocket 下发给 SPDE 节点。
/// 与 [`ClientMsg`] 构成生态内 WS 实时通信的权威协议。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    /// 配置变更，节点应重新拉取配置
    ConfigChanged,
    /// 新任务，节点应拉取最新任务列表
    NewTask,
    /// 心跳保活，节点应回 Pong
    Ping,
    /// 节点已被删除，节点应立即暂停所有任务并重新注册
    NodeDeleted,
    /// 删除本地文件（旧协议兼容）
    DeleteFile {
        /// 文件名
        filename: String,
        /// 保存路径
        #[serde(default)]
        save_path: Option<String>,
    },
}

/// WebSocket 节点→主控消息（ClientMsg）
///
/// 由 SPDE 节点通过 WebSocket 上报给 PK 主控。
/// 与 [`ServerMsg`] 构成生态内 WS 实时通信的权威协议。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    /// 实时状态上报
    Status {
        /// 活跃任务数
        active_tasks: u32,
        /// 累计下载字节数
        bytes_downloaded: u64,
        /// 是否忙碌
        #[serde(default)]
        busy: bool,
        /// 当前总速度 bps
        #[serde(default)]
        total_speed_bps: u64,
        /// 最近错误
        #[serde(default)]
        last_error: Option<String>,
    },
    /// 任务开始
    TaskStarted {
        /// 调度 ID
        dispatch_id: Uuid,
    },
    /// 任务进度
    TaskProgress {
        /// 调度 ID
        dispatch_id: Uuid,
        /// 任务名称
        task_name: String,
        /// 进度百分比（0-100）
        percent: f64,
        /// 已下载字节数
        downloaded_bytes: u64,
        /// 总大小字节数
        total_size: u64,
        /// 速度 bps
        speed_bps: u64,
        /// 活跃连接数
        active_connections: u32,
        /// 已运行秒数
        elapsed_secs: f64,
    },
    /// 任务完成报告
    TaskReport {
        /// 调度 ID
        dispatch_id: Option<Uuid>,
        /// 任务 ID
        task_id: Option<Uuid>,
        /// 任务名称
        task_name: String,
        /// 下载 URL
        url: String,
        /// 文件名
        filename: String,
        /// 文件大小
        file_size: u64,
        /// 已下载字节数
        downloaded_bytes: u64,
        /// 已运行秒数
        elapsed_secs: f64,
        /// 平均速度 Mbps
        avg_speed_mbps: f64,
        /// 状态
        status: String,
        /// 成功分片数
        success_chunks: u64,
        /// 失败分片数
        failed_chunks: u64,
        /// 错误消息
        #[serde(default)]
        error_msg: Option<String>,
    },
    /// 心跳应答（保活）
    Pong,
    /// 节点注册（旧协议兼容）
    Register {
        /// 节点 ID
        node_id: Uuid,
        /// 主机名
        hostname: String,
        /// 平台
        platform: String,
        /// 架构
        arch: String,
        /// 版本
        version: String,
    },
    /// 心跳（旧协议兼容）
    Heartbeat {
        /// 节点 ID
        node_id: Uuid,
        /// 活跃任务数
        active_tasks: u32,
        /// 累计下载字节数
        bytes_downloaded: u64,
    },
}

/// 下载任务配置（主控下发）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchConfig {
    /// 调度 ID
    pub dispatch_id: Uuid,
    /// 任务名称
    pub task_name: String,
    /// 下载 URL
    pub url: String,
    /// 保存路径
    pub save_path: String,
    /// 每文件连接数
    #[serde(default)]
    pub connections_per_file: u32,
    /// 重试次数
    #[serde(default)]
    pub retry_times: u32,
    /// 超时秒数
    #[serde(default)]
    pub timeout_secs: u32,
    /// 是否跳过 TLS 验证
    #[serde(default)]
    pub skip_tls_verify: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_req_round_trip() {
        let req = RegisterReq {
            node_id: None,
            hostname: "node-1".to_string(),
            platform: "linux".to_string(),
            arch: "x86_64".to_string(),
            version: "0.6.1".to_string(),
            labels: vec!["fast".to_string()],
            max_concurrent: Some(4),
            max_bandwidth_bps: Some(104_857_600),
            capabilities: Some(serde_json::json!({ "resume": true })),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: RegisterReq = serde_json::from_str(&json).unwrap();
        assert_eq!(back.hostname, "node-1");
        assert_eq!(back.max_concurrent, Some(4));
        assert_eq!(back.max_bandwidth_bps, Some(104_857_600));
    }

    #[test]
    fn ws_message_tagged_enum() {
        let msg = ClientMsg::TaskProgress {
            dispatch_id: Uuid::new_v4(),
            task_name: "t".to_string(),
            percent: 50.0,
            downloaded_bytes: 100,
            total_size: 200,
            speed_bps: 1024,
            active_connections: 4,
            elapsed_secs: 1.0,
        };
        let json = serde_json::to_string(&msg).unwrap();
        // 应使用 snake_case 的 type 标签
        assert!(json.contains("\"type\":\"task_progress\""));
    }

    #[test]
    fn server_msg_wire_format() {
        let msg = ServerMsg::Ping;
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"ping\""));
        // DeleteFile 的 save_path 应序列化为 snake_case
        let msg = ServerMsg::DeleteFile {
            filename: "f".to_string(),
            save_path: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"save_path\""));
    }

    #[test]
    fn update_capabilities_defaults() {
        let req = UpdateNodeCapabilitiesReq::default();
        assert!(req.max_concurrent.is_none());
        assert!(req.capabilities.is_none());
    }
}
