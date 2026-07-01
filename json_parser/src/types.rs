//! Token 类型定义：JSON 词法分析输出的 Token 枚举。
//!
//! 包含：标点符号（{}[],:）、字面量（true/false/null）、
//! 字符串（已解码转义）、数字（f64）。
//!

// 用户输入的 JSON 字符串,经过 tokenizer 解析后生成的 Token 流。最终类型是Vec<Token>。
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Comma,        // ,
    Colon,        // :
    True,         // true
    False,        // false
    Null,         // null
    String(String),
    Number(f64),
}

// 由tokenizer.rs 生成的 Token 流,最终会被 Parser 解析成 JsonValue
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,                               // 空
    Bool(bool),                         // bool值
    Number(f64),                        // 数字
    String(String),                     // 字符串
    Array(Vec<JsonValue>),              // 数组
    Object(HashMap<String, JsonValue>), // 对象
}
