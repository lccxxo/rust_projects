//! JSON 解析器：从零实现的 JSON parser，零外部依赖。
//!
//! ## 架构
//! ```text
//! &str → Tokenizer → Vec<Token> → Parser → JsonValue
//! ```
//!
//! ## 模块
//! - `error`    — Position / ParseError
//! - `types`    — Token / JsonValue
//! - `char_iter` — 字符遍历器
//! - `tokenizer` — 词法分析
//! - `parser`    — 语法分析（待实现）
//! - `tests`     — 单元测试

// 子模块声明
mod error;
mod types;
mod char_iter;
mod tokenizer;
