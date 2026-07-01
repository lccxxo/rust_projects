//! Token 类型定义：JSON 词法分析输出的 Token 枚举。
//!
//! 包含：标点符号（{}[],:）、字面量（true/false/null）、
//! 字符串（已解码转义）、数字（f64）。
//!
//! 后续 Day 2 会在此文件追加 JsonValue 枚举。

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    True,
    False,
    Null,
    String(String),
    Number(f64),
}
