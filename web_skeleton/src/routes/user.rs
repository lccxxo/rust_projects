//! 用户路由与 handler。
//!
//! handler 保持「薄」：解析请求 → 调 service → 包装响应。

use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use uuid::Uuid;

use crate::error::AppResult;
use crate::extract::ValidatedJson;
use crate::models::user::{CreateUser, UpdateUser, User};
use crate::response::{ApiResponse, Created};
use crate::services::user as service;
use crate::state::AppState;

/// 用户模块路由表。
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(remove))
}

/// GET /api/users —— 列出用户。
async fn list(State(state): State<AppState>) -> AppResult<ApiResponse<Vec<User>>> {
    let users = service::list_users(&state).await?;
    Ok(ApiResponse::ok(users))
}

/// POST /api/users —— 创建用户。
async fn create(
    State(state): State<AppState>,
    ValidatedJson(input): ValidatedJson<CreateUser>,
) -> AppResult<Created<User>> {
    let user = service::create_user(&state, input).await?;
    Ok(Created(user))
}

/// GET /api/users/{id} —— 获取单个用户。
async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<User>> {
    let user = service::get_user(&state, id).await?;
    Ok(ApiResponse::ok(user))
}

/// PUT /api/users/{id} —— 更新用户。
async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(input): ValidatedJson<UpdateUser>,
) -> AppResult<ApiResponse<User>> {
    let user = service::update_user(&state, id, input).await?;
    Ok(ApiResponse::ok(user))
}

/// DELETE /api/users/{id} —— 删除用户。
async fn remove(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<ApiResponse<()>> {
    service::delete_user(&state, id).await?;
    Ok(ApiResponse::ok(()))
}
