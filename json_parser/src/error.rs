//! 错误与位置追踪。
//!
//! 本模块定义两个核心类型:
//!
//! - [`Position`] —— 源码中的行列号,字符级别
//! - [`ParseError`] —— 解析错误,携带 [`Position`] + 可读消息
//!
//! 所有 tokenizer / parser 内部都通过 `iter.error(msg)` 或 `ParseError::new(msg, pos)` 构造错误,
//! 错误冒泡到顶层 [`parse`](crate::parse) 后,由调用者统一处理。
//!
//! ## 位置语义
//!
//! 行列号从 **1** 开始,与大多数编辑器和 `rustc` 保持一致。`advance` 在遇到换行符时把 `col` 重置为 1。
//!
//! ## 用法示例
//!
//! ```
//! use json_parser::error::{Position, ParseError};
//!
//! let pos = Position::new();
//! let err = ParseError::new("unexpected end of input", pos);
//! assert_eq!(format!("{}", err), "line 1, col 1: unexpected end of input");
//! ```

use std::fmt;

/// 源码中的行列号,字符级别。
///
/// 行号从 1 开始,列号从 1 开始。`\\n` 视为一个字符,但它会把 `col` 重置为 1 同时 `line += 1`。
///
/// `Position` 实现了 `Copy`,所以在 `ParseError` 里按值持有没有性能开销。
///
/// ## 字段
///
/// - `line`:行号,从 1 开始
/// - `col`:列号,从 1 开始
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// 行号,从 1 开始。
    pub line: usize,
    /// 列号,从 1 开始。每个字符算 1。
    pub col: usize,
}

impl Position {
    /// 构造一个指向文件开头的位置 `(1, 1)`。
    pub fn new() -> Self {
        Position { line: 1, col: 1 }
    }

    /// 消费一个字符后推进位置。
    ///
    /// - 遇到 `\\n` 时换行:`line += 1, col = 1`
    /// - 其他字符:`col += 1`
    pub fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析错误,带源码位置。
///
/// 由 tokenizer / parser 内部产生,通过 `?` 操作符冒泡到 [`parse`](crate::parse) 的调用方。
///
/// `ParseError` 实现了 `Display`,格式为 `line {line}, col {col}: {message}`,可以直接 `println!`。
#[derive(Debug, Clone)]
pub struct ParseError {
    /// 错误描述,如 `"unexpected character '@'"`、`"trailing comma in array"`。
    pub message: String,
    /// 错误发生的位置(行列号)。
    pub pos: Position,
}

impl ParseError {
    /// 用给定的消息和位置构造一个新错误。
    ///
    /// 消息接受 `impl Into<String>`,所以可以传 `&str`、`String`、`format_args!()` 等。
    pub fn new(msg: impl Into<String>, pos: Position) -> Self {
        ParseError {
            message: msg.into(),
            pos,
        }
    }
}

impl fmt::Display for ParseError {
    /// 输出格式:`line {line}, col {col}: {message}`
    ///
    /// 这是 `rustc` 风格的错误信息,可以直接被编辑器/IDE 解析跳转到对应行列。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}, col {}: {}",
            self.pos.line, self.pos.col, self.message
        )
    }
}

impl std::error::Error for ParseError {}
