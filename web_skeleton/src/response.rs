//! 统一响应结构。
//!
//! 所有成功响应统一包裹为 `{ code, message, data }`，
//! 前端只需按固定结构解析。错误响应见 [`crate::error`]。

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// 统一的成功响应包装。
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    /// 业务状态码，0 表示成功。
    pub code: i32,
    /// 提示信息。
    pub message: String,
    /// 业务数据。
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 构造成功响应。
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// 已创建资源（201）的响应。
pub struct Created<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse for Created<T> {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, Json(ApiResponse::ok(self.0))).into_response()
    }
}
