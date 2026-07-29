//! 应用共享状态。
//!
//! 通过 axum 的 `State` 提取器注入到各 handler。
//! 内部字段用 `Arc` 包裹，`Clone` 只增加引用计数，开销极小。

use std::sync::Arc;

use crate::config::AppConfig;
use crate::repository::{user::InMemoryUserRepository, user::UserRepository};

/// 全局共享状态。可按需在此添加数据库连接池、缓存客户端等。
#[derive(Clone)]
pub struct AppState {
    /// 应用配置。
    pub config: Arc<AppConfig>,
    /// 用户仓储（trait 对象，便于替换实现，如日后换成数据库版）。
    pub users: Arc<dyn UserRepository>,
}

impl AppState {
    /// 使用内存实现构建状态。
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
            users: Arc::new(InMemoryUserRepository::new()),
        }
    }
}
