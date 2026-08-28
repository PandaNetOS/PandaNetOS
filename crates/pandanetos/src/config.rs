//! 配置加载工具
//!
//! 提供统一的配置加载方式，支持环境变量覆盖与默认值合并。
//! 优先级：环境变量 > 配置文件 > 默认值。

use std::path::Path;

use serde::de::DeserializeOwned;

/// 默认的文件名（不带扩展名，自动探测 yaml/yml/json/toml）
pub const DEFAULT_CONFIG_NAME: &str = "config";

/// 从配置文件和环境变量加载配置
///
/// 优先级：环境变量 > 配置文件 > 默认值。
/// 环境变量前缀通过 `prefix` 参数指定，分隔符为 `__`。
///
/// # 示例
/// ```ignore
/// let config: AppConfig = pandanetos::config::load("config", "PK")?;
/// ```
pub fn load<T: DeserializeOwned>(config_file: &str, env_prefix: &str) -> crate::error::Result<T> {
    let mut builder =
        config::Config::builder().add_source(config::File::with_name(config_file).required(false));

    if !env_prefix.is_empty() {
        builder = builder.add_source(
            config::Environment::with_prefix(env_prefix)
                .separator("__")
                .try_parsing(true),
        );
    }

    build_and_deserialize(builder)
}

/// 从配置文件、环境变量和默认值合并加载配置
///
/// `defaults` 作为最低优先级，保证缺失字段也能得到合理默认值。
/// 优先级：环境变量 > 配置文件 > 默认值。
///
/// # 示例
/// ```ignore
/// #[derive(Deserialize)]
/// struct AppConfig { max_concurrent: u32, timeout_secs: u32 }
///
/// let config: AppConfig = pandanetos::config::load_with_defaults(
///     "config",
///     "PK",
///     serde_json::json!({ "max_concurrent": 4, "timeout_secs": 30 }),
/// )?;
/// ```
pub fn load_with_defaults<T: DeserializeOwned>(
    config_file: &str,
    env_prefix: &str,
    defaults: serde_json::Value,
) -> crate::error::Result<T> {
    let defaults_config = config::Config::try_from(&defaults)
        .map_err(|e| crate::error::CoreError::InvalidParam(format!("默认配置无效: {e}")))?;
    let mut builder = config::Config::builder()
        .add_source(defaults_config)
        .add_source(config::File::with_name(config_file).required(false));

    if !env_prefix.is_empty() {
        builder = builder.add_source(
            config::Environment::with_prefix(env_prefix)
                .separator("__")
                .try_parsing(true),
        );
    }

    build_and_deserialize(builder)
}

/// 构建并反序列化配置
fn build_and_deserialize<T: DeserializeOwned>(
    builder: config::ConfigBuilder<config::builder::DefaultState>,
) -> crate::error::Result<T> {
    let config = builder
        .build()
        .map_err(|e| crate::error::CoreError::Internal(format!("配置加载失败: {e}")))?;

    config
        .try_deserialize()
        .map_err(|e| crate::error::CoreError::InvalidParam(format!("配置解析失败: {e}")))
}

/// 检查配置文件是否存在（支持 yaml/yml/json/toml）
pub fn config_exists(config_file: &str) -> bool {
    Path::new(&format!("{config_file}.yaml")).exists()
        || Path::new(&format!("{config_file}.yml")).exists()
        || Path::new(&format!("{config_file}.json")).exists()
        || Path::new(&format!("{config_file}.toml")).exists()
}
