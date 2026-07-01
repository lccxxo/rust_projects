use std::collections::HashMap;

use crate::ParseError;
use crate::error::Position;
use crate::tokenizer::tokenize;
use crate::types::{JsonValue, Token};

struct Parser<'a> {
    last_pos: Position, // 最后一次解析的位置
    tokens: std::iter::Peekable<std::slice::Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Parser {
            last_pos: Position::new(),
            tokens: tokens.iter().peekable(),
        }
    }

    /// 预存下一个token 不消费
    fn peek(&mut self) -> Option<&&Token> {
        self.tokens.peek()
    }

    /// 消费下一个token
    fn next(&mut self) -> Option<&Token> {
        self.tokens.next()
    }

    /// 断言下一个token是指定的token 如果不是则返回错误
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        match self.tokens.next() {
            Some(t) if *t == expected => Ok(()),
            Some(t) => Err(ParseError::new(
                format!(
                    "expected: {:?}, got: {:?}",
                    token_name(&expected),
                    token_name(t)
                ),
                self.last_pos,
            )),
            None => Err(ParseError::new(
                format!("expected {:?}, got end of input", token_name(&expected)),
                self.last_pos,
            )),
        }
    }

    /// 检查下一个token是否匹配 不消费
    fn check(&mut self, expected: &Token) -> bool {
        self.tokens.peek().is_some_and(|t| *t == expected)
    }

    /// 解析值
    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        match self.peek() {
            Some(Token::LeftBrace) => self.parse_object(),
            Some(Token::LeftBracket) => self.parse_array(),
            Some(Token::String(_)) => {
                if let Some(Token::String(s)) = self.next() {
                    Ok(JsonValue::String(s.clone()))
                } else {
                    unreachable!()
                }
            }
            Some(Token::Number(_)) => {
                if let Some(Token::Number(n)) = self.next() {
                    Ok(JsonValue::Number(*n))
                } else {
                    unreachable!()
                }
            }
            Some(Token::True) => {
                self.next();
                Ok(JsonValue::Bool(true))
            }
            Some(Token::False) => {
                self.next();
                Ok(JsonValue::Bool(false))
            }
            Some(Token::Null) => {
                self.next();
                Ok(JsonValue::Null)
            }
            Some(other) => Err(ParseError::new(
                format!("unexpected token {} at start of value", token_name(other)),
                // 简化：实际应存 token 对应源码位置
                Position { line: 1, col: 1 },
            )),
            None => Err(ParseError::new(
                "unexpected end of input, expected a value".to_string(),
                Position { line: 1, col: 1 },
            )),
        }
    }

    /// 解析 {}
    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        // 是否为期望的左括号 如果不是则返回错误
        self.expect(Token::LeftBrace)?;

        let mut map = HashMap::new();

        // 如果下一个token是右括号 则返回空对象
        if self.check(&Token::RightBrace) {
            self.next();
            return Ok(JsonValue::Object(map));
        }

        loop {
            let (key, value) = self.parse_member()?;
            map.insert(key, value);

            if self.check(&Token::RightBrace) {
                self.next();
                break;
            }

            // 必须有逗号
            self.expect(Token::Comma)?;

            // 逗号后不能直接 '}' — 非法尾随逗号
            if self.check(&Token::RightBrace) {
                return Err(ParseError::new(
                    "trailing comma in object".to_string(),
                    Position { line: 1, col: 1 },
                ));
            }
        }

        Ok(JsonValue::Object(map))
    }

    /// 解析数组
    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect(Token::LeftBracket)?;

        let mut items = Vec::new();

        // 如果下一个token是右括号 则返回空数组
        if self.check(&Token::RightBracket) {
            self.next();
            return Ok(JsonValue::Array(items));
        }

        loop {
            // 解析数组中的值
            items.push(self.parse_value()?);

            // 如果下一个token是右括号 则返回数组
            if self.check(&Token::RightBracket) {
                self.next();
                break;
            }

            // 如果下一个token不是逗号 则返回错误
            self.expect(Token::Comma)?;

            // 防止 [1,2,3,] 这种情况出现
            if self.check(&Token::RightBracket) {
                return Err(ParseError::new(
                    "trailing comma in array".to_string(),
                    Position { line: 1, col: 1 },
                ));
            }
        }

        Ok(JsonValue::Array(items))
    }

    /// 解析key: value
    fn parse_member(&mut self) -> Result<(String, JsonValue), ParseError> {
        // 解析key 一定是字符串 如果不是则返回错误
        let key = match self.next() {
            Some(Token::String(s)) => s.clone(),
            Some(t) => {
                return Err(ParseError::new(
                    format!("expected string key in object, got {}", token_name(t)),
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

        // 解析中间的 :
        self.expect(Token::Colon)?;

        // 解析value
        let value = self.parse_value()?;

        Ok((key, value))
    }
}

/// 根据类型获取token名称
fn token_name(t: &Token) -> &'static str {
    match t {
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

/// 解析的入口函数
pub fn parse(input: &str) -> Result<JsonValue, ParseError> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(&tokens);
    let value = parser.parse_value()?;

    if parser.peek().is_some() {
        return Err(ParseError::new(
            "unexpected data after root value".to_string(),
            // 无法精确定位
            Position { line: 1, col: 1 },
        ));
    }

    Ok(value)
}
