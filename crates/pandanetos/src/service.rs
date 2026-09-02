//! 服务注册与发现
//!
//! 生态内 Agent 间点对点通信的服务注册中心协议。
//! PK 主控维护服务注册表，Agent 启动时注册自己的 serve 模式地址与能力，
//! 其他 Agent 通过查询端点或 WebSocket 事件获取可用服务列表。
//!
//! 遵循 [`docs/standards/service-discovery.md`] 标准。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── 能力标识常量 ───

/// Tracker 发现能力
pub const CAP_TRACKER: &str = "tracker";
/// DHT 发现能力
pub const CAP_DHT: &str = "dht";
/// PEX 发现能力
pub const CAP_PEX: &str = "pex";
/// Peer 缓存能力
pub const CAP_CACHE: &str = "cache";
/// 宣告做种能力
pub const CAP_ANNOUNCE: &str = "announce";
/// 优先级排序能力
pub const CAP_PRIORITY_SORTING: &str = "priority_sorting";
/// 结果去重能力
pub const CAP_DEDUP: &str = "dedup";
/// 健康检查能力
pub const CAP_HEALTH_CHECK: &str = "health_check";

// ─── 服务健康状态 ───

/// 服务健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceHealth {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 未知（刚注册，尚未心跳）
    Unknown,
}

impl ServiceHealth {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Unhealthy => "unhealthy",
            Self::Unknown => "unknown",
        }
    }
}

impl Default for ServiceHealth {
    fn default() -> Self {
        Self::Unknown
    }
}

// ─── 服务变更类型 ───

/// 服务变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceChangeType {
    /// 服务上线
    Up,
    /// 服务下线
    Down,
    /// 服务信息更新
    Updated,
}

impl ServiceChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Updated => "updated",
        }
    }
}

// ─── 服务注册信息 ───

/// 已注册服务的 Agent 信息
///
/// PK 主控在 Agent 注册时记录其 serve 模式地址与能力，
/// 供其他 Agent 查询并建立点对点连接。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAgentInfo {
    /// Agent 唯一 ID
    pub agent_id: Uuid,
    /// 节点名称（hostname 或自定义名）
    pub name: String,
    /// Agent 类型（如 spde / pdc / pcdn-keeper）
    pub agent_type: String,
    /// serve 模式监听地址
    pub host: String,
    /// serve 模式监听端口
    pub port: u16,
    /// 能力标识列表（如 ["tracker", "dht", "pex", "cache"]）
    pub capabilities: Vec<String>,
    /// 健康状态
    #[serde(default)]
    pub health: ServiceHealth,
    /// 当前负载（0.0 - 1.0）
    #[serde(default)]
    pub load: f32,
    /// 区域/机房标识（用于区域感知调度）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// 组件版本号
    pub version: String,
    /// 最后心跳时间（RFC3339）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_heartbeat: Option<String>,
}

impl ServiceAgentInfo {
    /// 构造服务注册信息
    pub fn new(
        agent_id: Uuid,
        name: impl Into<String>,
        agent_type: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        capabilities: Vec<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            agent_id,
            name: name.into(),
            agent_type: agent_type.into(),
            host: host.into(),
            port,
            capabilities,
            health: ServiceHealth::Unknown,
            load: 0.0,
            region: None,
            version: version.into(),
            last_heartbeat: None,
        }
    }

    /// 是否具备指定能力
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// 是否健康
    pub fn is_healthy(&self) -> bool {
        matches!(self.health, ServiceHealth::Healthy)
    }

    /// 构造基础 URL（http://host:port）
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

// ─── 服务查询响应 ───

/// 服务查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceQueryResponse {
    /// 符合条件的 Agent 列表
    pub agents: Vec<ServiceAgentInfo>,
    /// 总数
    pub total: usize,
}

impl ServiceQueryResponse {
    pub fn new(agents: Vec<ServiceAgentInfo>) -> Self {
        let total = agents.len();
        Self { agents, total }
    }

    pub fn empty() -> Self {
        Self {
            agents: Vec::new(),
            total: 0,
        }
    }
}

// ─── 服务变更事件（WebSocket 推送）───

/// 服务变更事件
///
/// PK 主控通过 WebSocket 向所有订阅 Agent 推送服务上下线/更新事件，
/// Agent 收到后更新本地服务缓存，无需轮询。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceChangedEvent {
    /// 变更的 Agent ID
    pub agent_id: Uuid,
    /// 变更类型
    pub change_type: ServiceChangeType,
    /// Agent 类型
    pub agent_type: String,
    /// 能力标识列表
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// serve 地址（下线事件可能为 None）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// serve 端口（下线事件可能为 None）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// 区域
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// 健康状态
    #[serde(default)]
    pub health: ServiceHealth,
    /// 负载
    #[serde(default)]
    pub load: f32,
}

impl ServiceChangedEvent {
    /// 构造服务上线事件
    pub fn up(info: &ServiceAgentInfo) -> Self {
        Self {
            agent_id: info.agent_id,
            change_type: ServiceChangeType::Up,
            agent_type: info.agent_type.clone(),
            capabilities: info.capabilities.clone(),
            host: Some(info.host.clone()),
            port: Some(info.port),
            region: info.region.clone(),
            health: info.health,
            load: info.load,
        }
    }

    /// 构造服务下线事件
    pub fn down(agent_id: Uuid, agent_type: impl Into<String>) -> Self {
        Self {
            agent_id,
            change_type: ServiceChangeType::Down,
            agent_type: agent_type.into(),
            capabilities: Vec::new(),
            host: None,
            port: None,
            region: None,
            health: ServiceHealth::Unhealthy,
            load: 0.0,
        }
    }

    /// 构造服务更新事件
    pub fn updated(info: &ServiceAgentInfo) -> Self {
        Self {
            agent_id: info.agent_id,
            change_type: ServiceChangeType::Updated,
            agent_type: info.agent_type.clone(),
            capabilities: info.capabilities.clone(),
            host: Some(info.host.clone()),
            port: Some(info.port),
            region: info.region.clone(),
            health: info.health,
            load: info.load,
        }
    }
}

// ─── 服务查询参数 ───

/// 服务查询过滤参数
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ServiceQuery {
    /// 按能力过滤（如 "tracker"）
    #[serde(default)]
    pub capability: Option<String>,
    /// 按 Agent 类型过滤（如 "pdc"）
    #[serde(default)]
    pub agent_type: Option<String>,
    /// 按健康状态过滤（默认只返回 healthy）
    #[serde(default)]
    pub health: Option<String>,
    /// 按区域过滤
    #[serde(default)]
    pub region: Option<String>,
    /// 页码（从 1 开始）
    #[serde(default = "default_page")]
    pub page: u32,
    /// 每页大小
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    50
}

impl ServiceQuery {
    /// 是否只看健康的
    pub fn only_healthy(&self) -> bool {
        match &self.health {
            Some(h) => h == "healthy",
            None => true, // 默认只返回健康的
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_agent_info_round_trip() {
        let info = ServiceAgentInfo::new(
            Uuid::new_v4(),
            "pdc-node-01",
            "pdc",
            "10.0.0.5",
            6881,
            vec![
                CAP_TRACKER.to_string(),
                CAP_DHT.to_string(),
                CAP_CACHE.to_string(),
            ],
            "0.1.0",
        );
        assert!(info.has_capability("tracker"));
        assert!(!info.has_capability("pex"));
        assert_eq!(info.base_url(), "http://10.0.0.5:6881");

        let json = serde_json::to_string(&info).unwrap();
        let back: ServiceAgentInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, info.agent_id);
        assert_eq!(back.port, 6881);
        assert_eq!(back.capabilities.len(), 3);
    }

    #[test]
    fn service_changed_event_variants() {
        let info = ServiceAgentInfo::new(
            Uuid::new_v4(),
            "pdc-1",
            "pdc",
            "127.0.0.1",
            6881,
            vec![CAP_TRACKER.to_string()],
            "0.1.0",
        );

        let up = ServiceChangedEvent::up(&info);
        assert_eq!(up.change_type, ServiceChangeType::Up);
        assert!(up.host.is_some());

        let down = ServiceChangedEvent::down(info.agent_id, "pdc");
        assert_eq!(down.change_type, ServiceChangeType::Down);
        assert!(down.host.is_none());

        let updated = ServiceChangedEvent::updated(&info);
        assert_eq!(updated.change_type, ServiceChangeType::Updated);
    }

    #[test]
    fn service_query_defaults() {
        let q: ServiceQuery = serde_json::from_str("{}").unwrap();
        assert!(q.capability.is_none());
        assert_eq!(q.page, 1);
        assert_eq!(q.page_size, 50);
        assert!(q.only_healthy());
    }

    #[test]
    fn capability_constants() {
        assert_eq!(CAP_TRACKER, "tracker");
        assert_eq!(CAP_DHT, "dht");
        assert_eq!(CAP_PEX, "pex");
        assert_eq!(CAP_CACHE, "cache");
        assert_eq!(CAP_ANNOUNCE, "announce");
    }
}
