//! JSON 解析器:从零实现的 JSON parser,零外部依赖。
//!
//! ## 架构
//! ```text
//! &str → Tokenizer → Vec<Token> → Parser → JsonValue
//! ```
//!
//! ## 模块
//! - `error`     — Position / ParseError
//! - `types`     — Token / JsonValue
//! - `char_iter` — 字符遍历器
//! - `tokenizer` — 词法分析(`tokenize` 入口)
//! - `tests`     — 单元测试
//!
//! ## CLI 用法(bin/json_parser)
//! ```text
//! 运行后输入 JSON 字符串,回车后打印 token 流。
//! 输入 exit / quit 退出。
//! ```

// 子模块声明 —— 这里改成 pub,让二进制 crate 也能 use
pub mod char_iter;
pub mod error;
pub mod parser;
pub mod tokenizer;
pub mod types;

// 重新导出常用 API,让外部用 json_parser::tokenize 而不是 json_parser::tokenizer::tokenize
pub use error::ParseError;
pub use parser::parse;
pub use tokenizer::tokenize;
pub use types::Token;

// 测试模块
#[cfg(test)]
mod test;
