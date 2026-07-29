//! 用户相关的领域实体与 DTO。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// 用户领域实体（对应「存储层」的一条记录）。
#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

/// 创建用户的请求 DTO，带字段校验规则。
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 1, max = 50, message = "用户名长度需在 1-50 之间"))]
    pub name: String,

    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
}

/// 更新用户的请求 DTO（字段可选，仅更新提供的字段）。
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(length(min = 1, max = 50, message = "用户名长度需在 1-50 之间"))]
    pub name: Option<String>,

    #[validate(email(message = "邮箱格式不正确"))]
    pub email: Option<String>,
}
