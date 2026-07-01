//! # json_parser
//!
//! 从零实现的 JSON 解析器，**零外部依赖**，严格遵循 [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259) 规范。
//!
//! ## 整体架构
//!
//! 解析流程分两步,数据单向流动:
//!
//! ```text
//!   &str  ──►  CharIter  ──►  Tokenizer  ──►  Vec<Token>  ──►  Parser  ──►  JsonValue
//!   源码      字符流         词法分析          Token 流         语法分析      解析树
//! ```
//!
//! - **CharIter**:把 `&str` 包装成支持 `peek` / `next` 的字符流,带位置追踪
//! - **Tokenizer**:把字符流切成 `Token` 流,识别关键字、字符串、数字、转义
//! - **Parser**:基于 `Peekable<Iter<Token>>` 做递归下降,产出 `JsonValue` AST
//!
//! ## 公开 API 一览
//!
//! 模块用户只需要关心三件事:
//!
//! | 函数 / 类型 | 作用 | 位置 |
//! |---|---|---|
//! | [`parse`] | 顶层入口:从 `&str` 得到 `JsonValue` | `parser` 模块,本 crate re-export |
//! | [`tokenize`] | 词法分析:从 `&str` 得到 `Vec<Token>` | `tokenizer` 模块,本 crate re-export |
//! | [`Token`] / [`JsonValue`] | 核心数据结构 | `types` 模块,本 crate re-export |
//! | [`ParseError`] | 解析错误,带行列号 | `error` 模块,本 crate re-export |
//!
//! ## 快速上手
//!
//! ```
//! use json_parser::parse;
//!
//! let input = r#"{"name": "lccxxo", "age": 30, "tags": ["rust", "json"]}"#;
//! let value = parse(input).expect("valid JSON");
//! println!("{:#?}", value);
//! ```
//!
//! ## 错误处理
//!
//! 所有失败路径都返回 [`ParseError`],包含行列号 + 可读消息:
//!
//! ```
//! use json_parser::parse;
//!
//! let err = parse("[1, 2,]").unwrap_err();
//! assert_eq!(err.message, "trailing comma in array");
//! ```
//!
//! ## 模块组织
//!
//! - [`error`] —— [`Position`] / [`ParseError`]
//! - [`types`] —— [`Token`] / [`JsonValue`]
//! - [`char_iter`] —— [`CharIter`](char_iter::CharIter) 字符流
//! - [`tokenizer`] —— [`tokenize`] 词法分析
//! - [`parser`] —— [`parse`] 语法分析
//!
//! ## CLI 用法
//!
//! 项目根目录的 `src/main.rs` 是一个最小命令行包装:
//!
//! ```text
//! $ echo '{"name": "lccxxo"}' | cargo run
//! Object {
//!     "name": String("lccxxo"),
//! }
//! ```
//!
//! 非 0 退出码 = 解析失败,stderr 打印错误信息。
//!
//! ## 规范符合度
//!
//! 支持:对象、数组、字符串(完整转义 + `\\uXXXX` Unicode 转义)、数字(整数 / 小数 / 负数 / 科学计数法)、`true` / `false` / `null`、**禁止尾随逗号**、**禁止前导零**。
//!
//! 不支持(刻意简化):注释、流式解析、`\u` 高位代理对的配对解析(按字面 U+XXXX 输出)。

// ── 子模块声明 ──
// 这些 `pub mod` 让上层(包括 `src/main.rs`)能用 `json_parser::char_iter` 等路径。
pub mod char_iter;
pub mod error;
pub mod parser;
pub mod tokenizer;
pub mod types;

// ── 顶层 API 重新导出 ──
// 简化外部调用路径,使用者不用记子模块名:
//
//   ✅ json_parser::parse(input)
//   ❌ json_parser::parser::parse(input)
pub use error::{ParseError, Position};
pub use parser::parse;
pub use tokenizer::tokenize;
pub use types::{JsonValue, Token};

// 单元测试(只在 `cargo test` 时编译)
#[cfg(test)]
mod test;
