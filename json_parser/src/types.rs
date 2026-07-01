//! 核心数据结构:`Token` 与 `JsonValue`。
//!
//! 这两个枚举是整个 crate 流转的两类"货物":
//!
//! ```text
//!   CharIter  ──tokenize──►  Vec<Token>   ──parse──►  JsonValue
//!   字符流       词法分析      Token 流         语法分析   AST
//! ```
//!
//! ## `Token`(词法分析产物)
//!
//! 词法分析把 `&str` 切成一串 `Token`。每种 JSON 字面量 / 标点对应一个变体。
//!
//! - 标点:`LeftBrace` / `RightBrace` / `LeftBracket` / `RightBracket` / `Comma` / `Colon`
//! - 关键字:`True` / `False` / `Null`
//! - 字面量:`String(String)`(已解码转义)/ `Number(f64)`
//!
//! 注意:`String` 变体里的字符串**已经处理过** `\\n` `\\t` `\\uXXXX` 等转义,
//! 拿到的是解码后的 Rust `String`,可以直接用。
//!
//! ## `JsonValue`(语法分析产物)
//!
//! 语法分析把 `Vec<Token>` 变成 `JsonValue` AST。
//!
//! - `Null` / `Bool(bool)` / `Number(f64)` / `String(String)` —— 标量
//! - `Array(Vec<JsonValue>)` —— 有序列表
//! - `Object(HashMap<String, JsonValue>)` —— 键值映射(注意:key 顺序不保留)
//!
//! ## 示例:从 `&str` 到 `JsonValue`
//!
//! ```
//! use json_parser::{parse, JsonValue, Token};
//!
//! let v = parse(r#"{"a": 1, "b": [true, null]}"#).unwrap();
//! match v {
//!     JsonValue::Object(map) => {
//!         assert!(map.contains_key("a"));
//!         assert!(map.contains_key("b"));
//!     }
//!     _ => panic!("expected object"),
//! }
//! ```

use std::collections::HashMap;

// ── Token:词法分析输出 ──
//
// 每个变体对应 JSON 语法里的一个最小单元。
// 变体内部不再嵌套枚举,直接用 `String` / `f64` 存字面量值。
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// 关键字 `true`
    True,
    /// 关键字 `false`
    False,
    /// 关键字 `null`
    Null,
    /// 字符串字面量(已经过转义解码),如 `"hello\\nworld"`
    String(String),
    /// 数字字面量(用 `f64` 存,可能有精度损失——JSON 标准是 IEEE 754 double)
    Number(f64),
}

impl Token {
    /// 给日志/调试用的简短名字,带引号或字面类型名。
    ///
    /// 主要给 parser 报错用:
    /// `expected: ',', got: ']'` 这种。
    pub fn name(&self) -> &'static str {
        match self {
            Token::LeftBrace => "'{'",
            Token::RightBrace => "'}'",
            Token::LeftBracket => "'['",
            Token::RightBracket => "']'",
            Token::Comma => "','",
            Token::Colon => "':'",
            Token::True => "'true'",
            Token::False => "'false'",
            Token::Null => "'null'",
            Token::String(_) => "string",
            Token::Number(_) => "number",
        }
    }
}

// ── JsonValue:语法分析输出,JSON 文档的内存表示 ──
//
// 用 `HashMap` 存对象键值对。**重要**:这意味着 key 的插入顺序不会保留。
// 如果顺序敏感,后续可以把 `Object` 换成 `Vec<(String, JsonValue)>` 或 `IndexMap`。
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// JSON `null`
    Null,
    /// JSON `true` / `false`
    Bool(bool),
    /// JSON 数字(整数 / 小数 / 科学计数法,统一存为 `f64`)
    Number(f64),
    /// JSON 字符串(已经过转义解码)
    String(String),
    /// JSON 数组,有序
    Array(Vec<JsonValue>),
    /// JSON 对象,**键值对无序**(底层 `HashMap`)
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    /// 便捷判断:是否为 `Null`
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// 便捷判断:是否为 `Bool`
    pub fn is_bool(&self) -> bool {
        matches!(self, JsonValue::Bool(_))
    }

    /// 便捷判断:是否为 `Number`
    pub fn is_number(&self) -> bool {
        matches!(self, JsonValue::Number(_))
    }

    /// 便捷判断:是否为 `String`
    pub fn is_string(&self) -> bool {
        matches!(self, JsonValue::String(_))
    }

    /// 便捷判断:是否为 `Array`
    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    /// 便捷判断:是否为 `Object`
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }
}
