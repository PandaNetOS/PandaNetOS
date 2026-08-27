//! 配置加载工具
//!
//! 提供统一的配置加载方式，支持环境变量覆盖。

use std::path::Path;

use serde::de::DeserializeOwned;

/// 从配置文件和环境变量加载配置
///
/// 优先级：环境变量 > 配置文件 > 默认值
/// 环境变量前缀通过 `prefix` 参数指定，分隔符为 `__`
///
/// # 示例
/// ```ignore
/// let config: AppConfig = pandanetpl_core::config::load("config", "PK")?;
/// ```
pub fn load<T: DeserializeOwned>(
    config_file: &str,
    env_prefix: &str,
) -> crate::error::Result<T> {
    let mut builder = config::Config::builder()
        .add_source(config::File::with_name(config_file).required(false));

    if !env_prefix.is_empty() {
        builder = builder.add_source(
            config::Environment::with_prefix(env_prefix)
                .separator("__")
                .try_parsing(true),
        );
    }

    let config = builder
        .build()
        .map_err(|e| crate::error::CoreError::Internal(format!("配置加载失败: {e}")))?;

    config
        .try_deserialize()
        .map_err(|e| crate::error::CoreError::Internal(format!("配置解析失败: {e}")))
}

/// 检查配置文件是否存在
pub fn config_exists(config_file: &str) -> bool {
    Path::new(&format!("{config_file}.yaml")).exists()
        || Path::new(&format!("{config_file}.yml")).exists()
        || Path::new(&format!("{config_file}.json")).exists()
        || Path::new(&format!("{config_file}.toml")).exists()
}
