//! 仓储层：定义数据访问的抽象接口与实现。
//!
//! 每种资源定义一个 `trait`（如 [`user::UserRepository`]），
//! 业务层只依赖 trait，不关心底层是内存、SQLite 还是 Postgres。
//! 更换存储只需提供一个新的 trait 实现，上层代码无需改动。

pub mod user;
