//! 统一错误类型。
//!
//! 所有 handler / service 返回 `Result<T, AppError>`，
//! `AppError` 实现了 [`IntoResponse`]，出错时自动转成带状态码的 JSON。
//! 配合 `?` 运算符，错误可在各层间自动向上传播。

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// 应用统一错误。
///
/// 部分变体（Unauthorized/Forbidden）是给鉴权等场景预留的示例，
/// 骨架里暂未触发，故允许 dead_code。
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 资源不存在（404）。
    #[error("资源不存在: {0}")]
    NotFound(String),

    /// 请求参数非法（400）。
    #[error("请求参数错误: {0}")]
    BadRequest(String),

    /// 参数校验失败（422），来自 validator。
    #[error("参数校验失败")]
    Validation(#[from] validator::ValidationErrors),

    /// 未认证（401）。
    #[error("未认证")]
    Unauthorized,

    /// 无权限（403）。
    #[error("无权限访问")]
    Forbidden,

    /// 冲突，如唯一键重复（409）。
    #[error("资源冲突: {0}")]
    Conflict(String),

    /// 内部错误（500）。使用 anyhow 兜底任意底层错误。
    #[error("内部服务器错误")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// 映射到 HTTP 状态码与业务码。
    fn parts(&self) -> (StatusCode, i32) {
        match self {
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, 40000),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, 40100),
            AppError::Forbidden => (StatusCode::FORBIDDEN, 40300),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, 40400),
            AppError::Conflict(_) => (StatusCode::CONFLICT, 40900),
            AppError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, 42200),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50000),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.parts();

        // 内部错误记录详细日志，但不把细节暴露给客户端。
        if let AppError::Internal(ref e) = self {
            tracing::error!(error = %e, "内部错误");
        }

        // 校验错误附带字段级详情。
        let details = match &self {
            AppError::Validation(errs) => Some(json!(errs)),
            _ => None,
        };

        let body = json!({
            "code": code,
            "message": self.to_string(),
            "data": details,
        });

        (status, Json(body)).into_response()
    }
}

/// 便捷别名。
pub type AppResult<T> = Result<T, AppError>;
