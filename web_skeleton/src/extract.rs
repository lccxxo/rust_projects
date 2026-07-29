//! 自定义提取器。
//!
//! [`ValidatedJson`] 在 axum 提取 JSON 的基础上自动执行 validator 校验，
//! 让 handler 拿到的一定是已校验过的数据。

use axum::{
    extract::{FromRequest, Request},
    Json,
};
use validator::Validate;

use crate::error::AppError;

/// 提取 JSON 请求体并执行校验。
///
/// 用法：`async fn handler(ValidatedJson(payload): ValidatedJson<CreateXxx>)`
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 先按 JSON 解析，解析失败归为 400。
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(e.body_text()))?;

        // 再执行字段校验，失败归为 422。
        value.validate()?;

        Ok(ValidatedJson(value))
    }
}
