//! 路由层：按业务模块拆分子路由，在此汇总。

pub mod health;
pub mod user;

use axum::Router;

use crate::state::AppState;

/// 汇总所有业务路由，统一挂在 `/api` 前缀下。
pub fn api_routes() -> Router<AppState> {
    Router::new().nest("/users", user::routes())
}

/// 系统级路由（健康检查等），不带 `/api` 前缀。
pub fn system_routes() -> Router<AppState> {
    Router::new().route("/health", axum::routing::get(health::health))
}
