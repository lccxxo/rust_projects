//! 健康检查端点，供负载均衡 / K8s 探针使用。

use axum::Json;
use serde_json::{json, Value};

/// GET /health —— 返回服务存活状态。
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
