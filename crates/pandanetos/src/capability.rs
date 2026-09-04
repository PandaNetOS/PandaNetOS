//! 自描述能力清单（Capability Manifest）
//!
//! 遵循 [`docs/standards/capability-manifest.md`] 标准：
//! 每个构建版本生成自己的能力清单（说明书），运行时上报给主控端。
//!
//! 顶层字段：
//! - `manifest_version`：清单格式版本号（当前 `"1.0"`）
//! - `basic`：基本信息（名称/版本/描述/角色）
//! - `capabilities`：能力清单（协议/功能特性/任务控制/硬件/编译特性）
//! - `configurable_params`：可配置参数（类型/范围/默认值/单位/描述）
//! - `api_interfaces`：API 接口定义
//! - `status_report`：状态上报字段
//! - `communication`：通信能力
//! - `build_info`：构建信息（Rust 版本/构建时间/Git commit/目标三元组）

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 能力清单格式版本号
pub const MANIFEST_VERSION: &str = "1.0";

/// 组件角色（与能力清单标准 2.2 一致）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentRole {
    /// 主控面
    ControlPlane,
    /// 数据面
    DataPlane,
    /// 边车
    Sidecar,
    /// 监控
    Monitor,
}

impl ComponentRole {
    /// 角色的 JSON 字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ControlPlane => "control_plane",
            Self::DataPlane => "data_plane",
            Self::Sidecar => "sidecar",
            Self::Monitor => "monitor",
        }
    }
}

/// 基本信息（能力清单 2.2）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicInfo {
    /// 程序名称（如 spde、pk、pcdn-keeper、PeerDiscoveryCenter）
    pub name: String,
    /// 语义化版本号（如 0.6.2）
    pub version: String,
    /// 程序功能描述
    pub description: String,
    /// 角色
    pub role: String,
    /// 当前运行模式（agent/standalone/cli，可选）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_mode: Option<String>,
}

impl BasicInfo {
    /// 构造基本信息
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        role: ComponentRole,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            role: role.as_str().to_string(),
            current_mode: None,
        }
    }

    /// 设置运行模式
    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.current_mode = Some(mode.into());
        self
    }
}

/// 功能特性分组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Capabilities {
    /// 支持的协议列表
    pub protocols: Vec<String>,
    /// 功能特性
    pub features: BTreeMap<String, bool>,
    /// 任务控制能力
    pub task_control: BTreeMap<String, bool>,
    /// 硬件能力
    pub hardware: BTreeMap<String, serde_json::Value>,
    /// 编译特性
    pub compile_features: Vec<String>,
}

impl Capabilities {
    /// 是否支持指定协议
    pub fn supports_protocol(&self, protocol: &str) -> bool {
        self.protocols.iter().any(|p| p == protocol)
    }

    /// 是否具备指定特性
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

/// 可配置参数描述（能力清单 2.4）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurableParam {
    /// 数据类型（u32/u64/f64/bool/string/enum）
    pub r#type: String,
    /// 默认值
    pub default: serde_json::Value,
    /// 最小值（数值类型，可选）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// 最大值（数值类型，可选）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// 枚举可选值（enum 类型，可选）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<serde_json::Value>>,
    /// 单位（可选）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// 参数说明
    pub description: String,
}

impl ConfigurableParam {
    /// 构造数值型参数
    pub fn number(
        type_name: &str,
        default: f64,
        min: Option<f64>,
        max: Option<f64>,
        unit: Option<&str>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            r#type: type_name.to_string(),
            default: serde_json::json!(default),
            min,
            max,
            r#enum: None,
            unit: unit.map(String::from),
            description: description.into(),
        }
    }

    /// 构造布尔型参数
    pub fn boolean(default: bool, description: impl Into<String>) -> Self {
        Self {
            r#type: "bool".to_string(),
            default: serde_json::json!(default),
            min: None,
            max: None,
            r#enum: None,
            unit: None,
            description: description.into(),
        }
    }

    /// 构造字符串/枚举型参数
    pub fn string(
        default: impl Into<String>,
        choices: Option<Vec<&str>>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            r#type: "string".to_string(),
            default: serde_json::json!(default.into()),
            min: None,
            max: None,
            r#enum: choices
                .map(|choices| choices.into_iter().map(|c| serde_json::json!(c)).collect()),
            unit: None,
            description: description.into(),
        }
    }
}

/// API 接口描述（能力清单 2.5）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInterface {
    /// HTTP 方法（GET/POST/PUT/DELETE）
    pub method: String,
    /// API 路径
    pub path: String,
    /// 接口说明
    pub description: String,
    /// 请求体结构（字段名+类型，可选）
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub request: BTreeMap<String, String>,
    /// 响应体结构（可选）
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub response: BTreeMap<String, String>,
    /// 是否需要认证（默认 false）
    #[serde(default)]
    pub auth_required: bool,
}

impl ApiInterface {
    /// 构造 API 接口描述
    pub fn new(
        method: impl Into<String>,
        path: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            description: description.into(),
            request: BTreeMap::new(),
            response: BTreeMap::new(),
            auth_required: false,
        }
    }

    /// 添加请求字段
    pub fn with_request_field(mut self, name: &str, type_name: &str) -> Self {
        self.request.insert(name.to_string(), type_name.to_string());
        self
    }

    /// 添加响应字段
    pub fn with_response_field(mut self, name: &str, type_name: &str) -> Self {
        self.response
            .insert(name.to_string(), type_name.to_string());
        self
    }

    /// 设置需要认证
    pub fn with_auth(mut self) -> Self {
        self.auth_required = true;
        self
    }
}

/// 状态上报字段（能力清单 2.6）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StatusReport {
    /// 节点级上报字段
    pub node_level: Vec<String>,
    /// 任务级上报字段
    pub task_level: Vec<String>,
}

/// 通信能力（能力清单 2.7）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Communication {
    /// 是否支持 WebSocket
    pub websocket: bool,
    /// 是否支持 HTTP API
    pub http_api: bool,
    /// 是否支持心跳
    pub heartbeat: bool,
    /// 心跳间隔（秒）
    pub heartbeat_interval_secs: u64,
    /// WebSocket 重连间隔（秒）
    pub websocket_reconnect_secs: u64,
}

impl Default for Communication {
    fn default() -> Self {
        Self {
            websocket: false,
            http_api: false,
            heartbeat: false,
            heartbeat_interval_secs: 10,
            websocket_reconnect_secs: 3,
        }
    }
}

/// 构建信息（能力清单 2.8）
///
/// 由 `build.rs` 注入的环境变量填充，缺失时回退为 `"unknown"`（向前兼容）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BuildInfo {
    /// Rust 编译器版本
    pub rust_version: String,
    /// 构建模式（debug/release）
    pub build_profile: String,
    /// 构建时间（ISO 8601）
    pub build_time: String,
    /// Git commit hash
    pub git_commit: String,
    /// Git 分支名
    pub git_branch: String,
    /// 编译目标三元组（如 x86_64-unknown-linux-musl）
    pub target_triple: String,
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            rust_version: option_env!("RUSTC_VERSION")
                .unwrap_or("unknown")
                .to_string(),
            build_profile: option_env!("BUILD_PROFILE")
                .unwrap_or("unknown")
                .to_string(),
            build_time: option_env!("BUILD_TIME").unwrap_or("unknown").to_string(),
            git_commit: option_env!("GIT_COMMIT").unwrap_or("unknown").to_string(),
            git_branch: option_env!("GIT_BRANCH").unwrap_or("unknown").to_string(),
            target_triple: option_env!("TARGET_TRIPLE")
                .unwrap_or("unknown")
                .to_string(),
        }
    }
}

/// 运行时构建信息（从 cargo 环境变量动态获取）
pub fn current_build_info() -> BuildInfo {
    BuildInfo {
        rust_version: build_info_or_unknown("RUSTC_VERSION"),
        build_profile: build_info_or_unknown("BUILD_PROFILE"),
        build_time: build_info_or_unknown("BUILD_TIME"),
        git_commit: build_info_or_unknown("GIT_COMMIT"),
        git_branch: build_info_or_unknown("GIT_BRANCH"),
        target_triple: build_info_or_unknown("TARGET_TRIPLE"),
    }
}

fn build_info_or_unknown(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| "unknown".to_string())
}

/// 能力清单（完整结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CapabilityManifest {
    /// 清单格式版本号
    pub manifest_version: String,
    /// 基本信息
    pub basic: BasicInfo,
    /// 能力清单
    #[serde(default)]
    pub capabilities: Capabilities,
    /// 可配置参数
    #[serde(default)]
    pub configurable_params: BTreeMap<String, ConfigurableParam>,
    /// API 接口定义
    #[serde(default)]
    pub api_interfaces: BTreeMap<String, ApiInterface>,
    /// 状态上报字段
    #[serde(default)]
    pub status_report: StatusReport,
    /// 通信能力
    #[serde(default)]
    pub communication: Communication,
    /// 构建信息
    #[serde(default)]
    pub build_info: BuildInfo,
}

impl CapabilityManifest {
    /// 构造空清单（仅基本信息 + 当前构建信息）
    pub fn new(basic: BasicInfo) -> Self {
        Self {
            manifest_version: MANIFEST_VERSION.to_string(),
            basic,
            capabilities: Capabilities::default(),
            configurable_params: BTreeMap::new(),
            api_interfaces: BTreeMap::new(),
            status_report: StatusReport::default(),
            communication: Communication::default(),
            build_info: current_build_info(),
        }
    }

    /// 设置能力
    pub fn with_capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// 添加可配置参数
    pub fn with_configurable_param(mut self, name: &str, param: ConfigurableParam) -> Self {
        self.configurable_params.insert(name.to_string(), param);
        self
    }

    /// 添加 API 接口
    pub fn with_api_interface(mut self, name: &str, interface: ApiInterface) -> Self {
        self.api_interfaces.insert(name.to_string(), interface);
        self
    }

    /// 设置状态上报
    pub fn with_status_report(mut self, report: StatusReport) -> Self {
        self.status_report = report;
        self
    }

    /// 设置通信能力
    pub fn with_communication(mut self, communication: Communication) -> Self {
        self.communication = communication;
        self
    }

    /// 是否为数据面角色
    pub fn is_data_plane(&self) -> bool {
        self.basic.role == ComponentRole::DataPlane.as_str()
    }
}

/// 生成完整的运行时能力清单（标准 3.2 约定函数签名）
///
/// 遵循 [`docs/standards/capability-manifest.md`]：返回完整的能力清单 JSON，
/// 供运行时上报与 `--manifest` CLI 命令输出。
pub fn build_capability_manifest(manifest: &CapabilityManifest) -> serde_json::Value {
    serde_json::to_value(manifest).unwrap_or_else(|_| {
        serde_json::json!({
            "manifest_version": MANIFEST_VERSION,
            "basic": {
                "name": manifest.basic.name,
                "version": manifest.basic.version,
            },
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> CapabilityManifest {
        CapabilityManifest::new(BasicInfo::new(
            "spde",
            "0.6.2",
            "多协议下载节点",
            ComponentRole::DataPlane,
        ))
        .with_capabilities(Capabilities {
            protocols: vec!["http".into(), "https".into(), "ftp".into()],
            features: BTreeMap::from([("resume".into(), true), ("retry".into(), true)]),
            task_control: BTreeMap::from([("pause".into(), true), ("cancel".into(), true)]),
            hardware: BTreeMap::from([
                ("cpu_cores".into(), serde_json::json!(8)),
                ("memory_gb".into(), serde_json::json!(16)),
            ]),
            compile_features: vec!["ftp".into()],
        })
        .with_configurable_param(
            "max_concurrent",
            ConfigurableParam::number(
                "u32",
                4.0,
                Some(1.0),
                Some(256.0),
                Some("tasks"),
                "最大并发任务数",
            ),
        )
        .with_api_interface(
            "agent_register",
            ApiInterface::new("POST", "/api/v1/agent/register", "节点注册")
                .with_request_field("node_id", "uuid")
                .with_auth(),
        )
    }

    #[test]
    fn manifest_serializes_to_documented_json() {
        let m = sample_manifest();
        let json = serde_json::to_value(&m).unwrap();
        assert_eq!(json["manifest_version"], "1.0");
        assert_eq!(json["basic"]["name"], "spde");
        assert_eq!(json["basic"]["role"], "data_plane");
        assert_eq!(json["capabilities"]["protocols"][0], "http");
        assert_eq!(json["configurable_params"]["max_concurrent"]["max"], 256.0);
        assert_eq!(
            json["api_interfaces"]["agent_register"]["auth_required"],
            true
        );
    }

    #[test]
    fn supports_protocol_and_feature() {
        let m = sample_manifest();
        assert!(m.capabilities.supports_protocol("http"));
        assert!(!m.capabilities.supports_protocol("torrent"));
        assert!(m.capabilities.has_feature("resume"));
        assert!(!m.capabilities.has_feature("dry_run"));
    }

    #[test]
    fn role_detection() {
        let m = sample_manifest();
        assert!(m.is_data_plane());
    }

    #[test]
    fn configurable_param_variants() {
        let bool_param = ConfigurableParam::boolean(true, "是否启用重试");
        assert_eq!(bool_param.r#type, "bool");
        assert_eq!(bool_param.default, serde_json::json!(true));

        let str_param = ConfigurableParam::string("auto", Some(vec!["auto", "manual"]), "调度模式");
        assert_eq!(str_param.r#type, "string");
        assert_eq!(
            str_param.r#enum,
            Some(vec![serde_json::json!("auto"), serde_json::json!("manual")])
        );
    }

    #[test]
    fn build_info_defaults_are_tolerated() {
        // 即使在未注入环境变量时，也应生成可用清单
        let m = CapabilityManifest::new(BasicInfo::new(
            "pk",
            "0.2.0",
            "主控台",
            ComponentRole::ControlPlane,
        ));
        assert_eq!(m.manifest_version, "1.0");
        assert!(!m.build_info.rust_version.is_empty());
    }
}
