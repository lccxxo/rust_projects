//! 语法分析器:把 `Vec<Token>` 解析成 [`JsonValue`] AST。
//!
//! ## 入口
//!
//! [`parse`] —— 从 `&str` 一站式得到 [`JsonValue`],内部会先调 [`tokenize`]。
//!
//! ## 算法
//!
//! 经典**递归下降** + 1 token lookahead:
//!
//! - 用 `std::iter::Peekable<std::slice::Iter<Token>>` 当 token 流(借用,不复制)
//! - 每个语法规则对应一个方法,匹配失败就 `?` 抛 [`ParseError`]
//! - 顶层 [`parse`] 调 `parse_value` 解析根值,再校验没有"根值后的多余 token"
//!
//! ## 语法规则(简化版 BNF)
//!
//! ```text
//!   value   := object | array | string | number | true | false | null
//!   object  := "{" [ member ("," member)* ] "}"
//!   member  := string ":" value
//!   array   := "[" [ value ("," value)* ] "]"
//! ```
//!
//! ## 数据流
//!
//! ```text
//!   &str ──tokenize──► Vec<Token> ──► Parser::new(&tokens) ──► parse_value ──► JsonValue
//! ```
//!
//! ## 错误恢复策略
//!
//! 本 parser **不做错误恢复**:遇到任何语法错误就立刻 `return Err(ParseError)`。
//! 对教育/练手项目而言,清晰的早 fail 比容错继续更有价值。

use std::collections::HashMap;

use crate::error::{ParseError, Position};
use crate::tokenizer::tokenize;
use crate::types::{JsonValue, Token};

/// 语法分析器状态。
///
/// 内部维护一个 `Peekable<Iter<Token>>`,借用的 `&[Token]` 不持有所有权,
/// 因此 `Parser` 生命周期 `'a` 和输入切片同步。
struct Parser<'a> {
    /// 最后一次"被成功消费的 token"的位置(目前只在 `expect` 报错时使用)
    last_pos: Position,
    /// 预留:当前 token 的位置(为后续精确错误信息做基础设施)
    current_token_pos: Position,
    /// Token 流的可 peek 迭代器
    tokens: std::iter::Peekable<std::slice::Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    /// 用 `&[Token]` 切片构造一个 parser。
    fn new(tokens: &'a [Token]) -> Self {
        Parser {
            last_pos: Position::new(),
            current_token_pos: Position::new(),
            tokens: tokens.iter().peekable(),
        }
    }

    /// **预读**下一个 token,不消费。
    ///
    /// 返回 `Option<&&Token>`(双引用:外层是 `peekable` 内部缓冲的引用,内层是 `slice` 的 `&Token`)。
    fn peek(&mut self) -> Option<&&Token> {
        self.tokens.peek()
    }

    /// **消费**下一个 token。
    fn next(&mut self) -> Option<&Token> {
        self.tokens.next()
    }

    /// 断言下一个 token 等于 `expected`,消费它,否则报错。
    ///
    /// 报错信息包含期望值 + 实际值,例如:
    /// `expected: ',', got: ']'`
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        match self.tokens.next() {
            Some(t) if *t == expected => Ok(()),
            Some(t) => Err(ParseError::new(
                format!(
                    "expected: {}, got: {}",
                    expected.name(),
                    t.name()
                ),
                self.last_pos,
            )),
            None => Err(ParseError::new(
                format!("expected {}, got end of input", expected.name()),
                self.last_pos,
            )),
        }
    }

    /// 检查下一个 token **是否匹配** `expected`,**不消费**。
    ///
    /// 内部用 [`Option::is_some_and`](std::option::Option::is_some_and) 做 `Some(t) && *t == expected`。
    fn check(&mut self, expected: &Token) -> bool {
        self.tokens.peek().is_some_and(|t| *t == expected)
    }

    /// 按下一个 token 分派到对应的解析函数。
    ///
    /// 入口方法,被 [`parse`] 调用。
    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        match self.peek() {
            // 对象 ──► parse_object
            Some(Token::LeftBrace) => self.parse_object(),

            // 数组 ──► parse_array
            Some(Token::LeftBracket) => self.parse_array(),

            // 字符串字面量:消费并 clone
            Some(Token::String(_)) => {
                if let Some(Token::String(s)) = self.next() {
                    Ok(JsonValue::String(s.clone()))
                } else {
                    unreachable!()
                }
            }

            // 数字字面量:消费并解引用
            Some(Token::Number(_)) => {
                if let Some(Token::Number(n)) = self.next() {
                    Ok(JsonValue::Number(*n))
                } else {
                    unreachable!()
                }
            }

            // 三个关键字
            Some(Token::True) => { self.next(); Ok(JsonValue::Bool(true)) }
            Some(Token::False) => { self.next(); Ok(JsonValue::Bool(false)) }
            Some(Token::Null) => { self.next(); Ok(JsonValue::Null) }

            // 非法首 token
            Some(other) => Err(ParseError::new(
                format!("unexpected token {} at start of value", other.name()),
                Position { line: 1, col: 1 },
            )),
            None => Err(ParseError::new(
                "unexpected end of input, expected a value".to_string(),
                Position { line: 1, col: 1 },
            )),
        }
    }

    /// 解析 JSON 对象 `{...}`。
    ///
    /// 语法:`"{" [ member ("," member)* ] "}"`
    ///
    /// 流程:
    /// 1. 吃掉 `{`
    /// 2. 如果下一个就是 `}`,返回空对象
    /// 3. 循环:解析一个 member → 吃掉 `,`(如果下一个不是 `}`)→ 重复
    /// 4. 见到 `}` 收尾
    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        // 1. 吃掉左大括号
        self.expect(Token::LeftBrace)?;

        let mut map = HashMap::new();

        // 2. 空对象 `{}` 早退
        if self.check(&Token::RightBrace) {
            self.next();
            return Ok(JsonValue::Object(map));
        }

        // 3. 循环解析 member
        loop {
            let (key, value) = self.parse_member()?;
            map.insert(key, value);

            // 见到 `}` 收尾
            if self.check(&Token::RightBrace) {
                self.next();
                break;
            }

            // member 之间必须有 `,`
            self.expect(Token::Comma)?;

            // 4. 防止 `{"a":1,}` 这种尾随逗号
            if self.check(&Token::RightBrace) {
                return Err(ParseError::new(
                    "trailing comma in object".to_string(),
                    Position { line: 1, col: 1 },
                ));
            }
        }

        Ok(JsonValue::Object(map))
    }

    /// 解析 JSON 数组 `[...]`。
    ///
    /// 语法:`"[" [ value ("," value)* ] "]"`
    ///
    /// 流程和 `parse_object` 几乎对称,只是 value 是任意 JSON 值(递归调 `parse_value`)。
    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        // 1. 吃掉左中括号
        self.expect(Token::LeftBracket)?;

        let mut items = Vec::new();

        // 2. 空数组 `[]` 早退
        if self.check(&Token::RightBracket) {
            self.next();
            return Ok(JsonValue::Array(items));
        }

        // 3. 循环解析 value
        loop {
            items.push(self.parse_value()?);

            // 见到 `]` 收尾
            if self.check(&Token::RightBracket) {
                self.next();
                break;
            }

            // value 之间必须有 `,`
            self.expect(Token::Comma)?;

            // 4. 防止 `[1,2,3,]` 这种尾随逗号
            if self.check(&Token::RightBracket) {
                return Err(ParseError::new(
                    "trailing comma in array".to_string(),
                    Position { line: 1, col: 1 },
                ));
            }
        }

        Ok(JsonValue::Array(items))
    }

    /// 解析 object 中的 `key: value` 对。
    ///
    /// 流程:
    /// 1. 吃一个字符串 token 作为 key
    /// 2. 吃 `,`
    /// 3. 递归调 `parse_value` 解析 value
    fn parse_member(&mut self) -> Result<(String, JsonValue), ParseError> {
        // 1. 吃 key(必须是字符串)
        let key = match self.next() {
            Some(Token::String(s)) => s.clone(),
            Some(t) => {
                return Err(ParseError::new(
                    format!("expected string key in object, got {}", t.name()),
                    Position { line: 1, col: 1 },
                ));
            }
            None => {
                return Err(ParseError::new(
                    "unexpected end of input, expected object key".to_string(),
                    Position { line: 1, col: 1 },
                ));
            }
        };

        // 2. 吃中间的 `:`
        self.expect(Token::Colon)?;

        // 3. 递归解析 value
        let value = self.parse_value()?;

        Ok((key, value))
    }
}

/// 顶层解析入口:从 `&str` 直接得到 [`JsonValue`]。
///
/// 这是 crate **对外的主 API**,底层会自动调 [`tokenize`] 词法分析。
///
/// ## 流程
///
/// 1. `tokenize(input)` 得到 `Vec<Token>`
/// 2. 用 `&tokens` 构造 `Parser`
/// 3. 调 `parse_value()` 解析根值
/// 4. **检查根值后没有多余 token**(防 `{...}garbage` 这种输入)
///
/// ## 错误
///
/// 词法错误、语法错误、尾随数据、根值缺失 都会返回 [`ParseError`]。
///
/// ## 示例
///
/// ```
/// use json_parser::{parse, JsonValue};
///
/// let v = parse(r#"{"name": "lccxxo", "n": 42}"#).unwrap();
/// if let JsonValue::Object(map) = v {
///     assert_eq!(map.get("n"), Some(&JsonValue::Number(42.0)));
/// } else {
///     panic!("expected object");
/// }
/// ```
pub fn parse(input: &str) -> Result<JsonValue, ParseError> {
    // 1. 词法分析
    let tokens = tokenize(input)?;

    // 2. 构造 parser
    let mut parser = Parser::new(&tokens);

    // 3. 解析根值
    let value = parser.parse_value()?;

    // 4. 根值后必须没有多余 token
    if parser.peek().is_some() {
        return Err(ParseError::new(
            "unexpected data after root value".to_string(),
            Position { line: 1, col: 1 },
        ));
    }

    Ok(value)
}
