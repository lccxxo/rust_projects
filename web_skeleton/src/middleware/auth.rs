//! 鉴权中间件示例。
//!
//! 这是一个演示性的 Bearer Token 校验：检查 `Authorization: Bearer <token>`
//! 是否存在。实际项目应替换为 JWT 校验、会话查询等，并把解析出的用户
//! 身份通过 `req.extensions_mut().insert(...)` 传递给后续 handler。
//!
//! 挂载方式（对需要保护的路由）：
//! ```ignore
//! use axum::middleware;
//! router.layer(middleware::from_fn(crate::middleware::auth::require_auth))
//! ```

use axum::{extract::Request, middleware::Next, response::Response};

use crate::error::AppError;

/// 要求请求携带合法的 Bearer Token，否则返回 401。
///
/// 骨架里未挂载到任何路由，属于示例代码，故允许 dead_code。
#[allow(dead_code)]
pub async fn require_auth(req: Request, next: Next) -> Result<Response, AppError> {
    let ok = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer ") && v.len() > "Bearer ".len())
        .unwrap_or(false);

    if ok {
        // 放行到下一环（其他中间件或最终 handler）。
        Ok(next.run(req).await)
    } else {
        Err(AppError::Unauthorized)
    }
}
