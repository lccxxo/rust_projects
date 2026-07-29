//! 用户业务逻辑。
//!
//! 当前逻辑较薄，直接转调仓储；实际项目中权限判断、
//! 事件通知、跨资源事务等业务规则都应写在这一层。

use uuid::Uuid;

use crate::error::AppResult;
use crate::models::user::{CreateUser, UpdateUser, User};
use crate::state::AppState;

/// 列出所有用户。
pub async fn list_users(state: &AppState) -> AppResult<Vec<User>> {
    state.users.list().await
}

/// 按 ID 获取用户。
pub async fn get_user(state: &AppState, id: Uuid) -> AppResult<User> {
    state.users.find(id).await
}

/// 创建用户。
pub async fn create_user(state: &AppState, input: CreateUser) -> AppResult<User> {
    state.users.create(input).await
}

/// 更新用户。
pub async fn update_user(state: &AppState, id: Uuid, input: UpdateUser) -> AppResult<User> {
    state.users.update(id, input).await
}

/// 删除用户。
pub async fn delete_user(state: &AppState, id: Uuid) -> AppResult<()> {
    state.users.delete(id).await
}
