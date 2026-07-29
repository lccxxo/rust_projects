//! 用户仓储：接口定义 + 内存实现。
//!
//! 生产环境可新增 `SqlxUserRepository` 实现同一 trait，
//! 在 [`crate::state::AppState::new`] 中替换即可，业务层无感知。

use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::user::{CreateUser, UpdateUser, User};

/// 用户数据访问接口。
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// 列出所有用户。
    async fn list(&self) -> AppResult<Vec<User>>;
    /// 按 ID 查找，不存在返回 `NotFound`。
    async fn find(&self, id: Uuid) -> AppResult<User>;
    /// 创建用户，邮箱重复返回 `Conflict`。
    async fn create(&self, input: CreateUser) -> AppResult<User>;
    /// 更新用户指定字段。
    async fn update(&self, id: Uuid, input: UpdateUser) -> AppResult<User>;
    /// 删除用户。
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

/// 基于 `RwLock<HashMap>` 的内存实现，适合开发 / 测试 / 脚手架演示。
pub struct InMemoryUserRepository {
    inner: RwLock<HashMap<Uuid, User>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn list(&self) -> AppResult<Vec<User>> {
        let map = self.inner.read().expect("锁未被毒化");
        let mut users: Vec<User> = map.values().cloned().collect();
        // 按创建时间排序，保证输出稳定。
        users.sort_by_key(|u| u.created_at);
        Ok(users)
    }

    async fn find(&self, id: Uuid) -> AppResult<User> {
        let map = self.inner.read().expect("锁未被毒化");
        map.get(&id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("用户 {id} 不存在")))
    }

    async fn create(&self, input: CreateUser) -> AppResult<User> {
        let mut map = self.inner.write().expect("锁未被毒化");

        // 邮箱唯一性检查。
        if map.values().any(|u| u.email == input.email) {
            return Err(AppError::Conflict(format!("邮箱 {} 已存在", input.email)));
        }

        let user = User {
            id: Uuid::new_v4(),
            name: input.name,
            email: input.email,
            created_at: Utc::now(),
        };
        map.insert(user.id, user.clone());
        Ok(user)
    }

    async fn update(&self, id: Uuid, input: UpdateUser) -> AppResult<User> {
        let mut map = self.inner.write().expect("锁未被毒化");

        // 若要改邮箱，先检查是否与他人冲突。
        if let Some(ref email) = input.email {
            if map.values().any(|u| u.email == *email && u.id != id) {
                return Err(AppError::Conflict(format!("邮箱 {email} 已存在")));
            }
        }

        let user = map
            .get_mut(&id)
            .ok_or_else(|| AppError::NotFound(format!("用户 {id} 不存在")))?;

        if let Some(name) = input.name {
            user.name = name;
        }
        if let Some(email) = input.email {
            user.email = email;
        }
        Ok(user.clone())
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let mut map = self.inner.write().expect("锁未被毒化");
        map.remove(&id)
            .ok_or_else(|| AppError::NotFound(format!("用户 {id} 不存在")))?;
        Ok(())
    }
}
