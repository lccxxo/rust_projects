//! 应用组装：把路由、共享状态、全局中间件拼成最终的 `Router`。
//!
//! 中间件采用 `tower` 的洋葱模型，`ServiceBuilder` 中越靠前的层越靠外：
//! 请求按「从上到下」进入，响应按「从下到上」返回。

use std::time::Duration;

use axum::{extract::DefaultBodyLimit, http::StatusCode, response::IntoResponse, Json, Router};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::routes;
use crate::state::AppState;

/// 构建完整的应用路由。
pub fn build_router(state: AppState) -> Router {
    let body_limit = state.config.body_limit_bytes;
    let timeout = Duration::from_secs(state.config.request_timeout_secs);

    // 全局中间件栈（对所有路由生效）。
    let middleware = ServiceBuilder::new()
        // 为每个请求生成唯一 x-request-id，便于日志串联。
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // 把 request-id 透传到响应头。
        .layer(PropagateRequestIdLayer::x_request_id())
        // 结构化访问日志。
        .layer(TraceLayer::new_for_http())
        // 请求超时保护，超时返回 408。
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            timeout,
        ))
        // 跨域，演示用放开；生产应收敛到白名单。
        .layer(CorsLayer::permissive());

    Router::new()
        // 系统路由：/health
        .merge(routes::system_routes())
        // 业务路由：/api/...
        .nest("/api", routes::api_routes())
        // 兜底 404。
        .fallback(not_found)
        .layer(middleware)
        // 请求体大小上限，防止过大 body 打爆内存（axum 原生层）。
        .layer(DefaultBodyLimit::max(body_limit))
        .with_state(state)
}

/// 未匹配任何路由时的统一 404 响应。
async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "code": 40400, "message": "接口不存在", "data": null })),
    )
}
