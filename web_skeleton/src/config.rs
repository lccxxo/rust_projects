//! 应用配置：从环境变量读取，带默认值。
//!
//! 生产环境通过环境变量或 `.env` 文件注入；这里只做最小实现，
//! 需要更复杂的分层配置（多环境 profile、配置文件合并）时可引入 `config` crate。

use std::env;

/// 全局应用配置。
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 监听地址，默认 127.0.0.1
    pub host: String,
    /// 监听端口，默认 3000
    pub port: u16,
    /// 日志等级，默认 info
    pub log_level: String,
    /// 请求超时秒数，默认 30
    pub request_timeout_secs: u64,
    /// 请求体大小上限（字节），默认 2MB
    pub body_limit_bytes: usize,
}

impl AppConfig {
    /// 从环境变量加载配置，缺省时使用默认值。
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: parse_env("APP_PORT", 3000)?,
            log_level: env::var("APP_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            request_timeout_secs: parse_env("APP_REQUEST_TIMEOUT_SECS", 30)?,
            body_limit_bytes: parse_env("APP_BODY_LIMIT_BYTES", 2 * 1024 * 1024)?,
        })
    }
}

/// 解析某个环境变量为目标类型，缺失或为空时返回默认值。
fn parse_env<T>(key: &str, default: T) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(v) if !v.trim().is_empty() => v
            .trim()
            .parse::<T>()
            .map_err(|e| anyhow::anyhow!("环境变量 {key} 解析失败: {e}")),
        _ => Ok(default),
    }
}
