use std::fmt;

use crate::Token::{LeftBrace, LeftBracket};

// 解析出来的词元类型
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Comma,          // ,
    Colon,          // :
    True,           // true
    False,          // false
    Null,           // null
    String(String), // "..." 存储解码后的内容
    Number(f64),    // 123, -123, 1.23, etc
}

#[derive(Debug, Clone, Copy)]
struct Position {
    line: usize, // 从1开始
    col: usize,  // 从1开始 每个字符算1
}

impl Position {
    pub fn new() -> Self {
        Position { line: 1, col: 1 }
    }

    // 每消费一字符后 改变位置
    fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }
}

// 错误解析
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub pos: Position,
}

impl ParseError {
    pub fn new(msg: impl Into<String>, pos: Position) -> Self {
        ParseError {
            message: msg.into(),
            pos,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}, col {}: {}",
            self.pos.line, self.pos.col, self.message
        )
    }
}

struct CharIter<'a> {
    chars: std::str::Chars<'a>, // 迭代器
    peeked: Option<char>,       // 往前一个字符 相当于一个字符缓冲区
    pos: Position,              // 当前位置
    save: Position,             // 暂存的位置 用于错误打印
}

impl<'a> CharIter<'a> {
    fn new(input: &'a str) -> Self {
        CharIter {
            chars: input.chars(),
            peeked: None,
            pos: Position::new(),
            save: Position::new(),
        }
    }

    // 保存当前位置快照
    fn save_pos(&mut self) {
        self.save = self.pos;
    }

    // 将暂存的位置构造ParseError
    fn error(&self, msg: impl Into<String>) -> ParseError {
        ParseError {
            message: msg.into(),
            pos: self.save,
        }
    }

    // 保存下一个字符到缓存中
    fn peek(&mut self) -> Option<char> {
        if self.peeked.is_none() {
            self.peeked = self.chars.next();
        }
        self.peeked
    }

    // 消费下一个字符 改变位置
    fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.peeked.take() {
            self.pos.advance(ch);
            return Some(ch);
        }

        // 如果缓存没有 则从迭代器读取
        self.chars.next().map(|ch| {
            self.pos.advance(ch);
            ch
        })
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut iter = CharIter::new(input);
    let mut tokens = Vec::new();

    while let Some(ch) = iter.peek() {
        match ch {
            // 解析空白字符 直接跳过
            ' ' | '\t' | '\n' => {
                iter.next();
            }

            // 解析无用符号 入tokens
            '{' => {
                iter.next();
                tokens.push(LeftBrace);
            }
            '}' => {
                iter.next();
                tokens.push(Token::RightBrace);
            }
            '[' => {
                iter.next();
                tokens.push(LeftBracket);
            }
            ']' => {
                iter.next();
                tokens.push(Token::RightBracket);
            }
            ',' => {
                iter.next();
                tokens.push(Token::Comma);
            }
            ':' => {
                iter.next();
                tokens.push(Token::Colon);
            }

            // 解析关键字
            '"' => {
                iter.save_pos();
                let s = read_string(&mut iter)?;
                tokens.push(Token::String(s));
            }

            // ── 非法字符 ──
            other => {
                iter.save_pos();
                iter.next();
                return Err(iter.error(format!("unexpected character '{}'", other)));
            }
        }
    }
    Ok(tokens)
}

fn read_string(iter: &mut CharIter) -> Result<String, ParseError> {
    // 解析字符串前的"
    match iter.next() {
        Some('"') => {}
        Some(ch) => return Err(iter.error(format!("expected '\"', found '{}'", ch))),
        None => return Err(iter.error("unexpected end of input")),
    }

    // 将字符串内容解析到result中
    let mut result = String::new();

    loop {
        match iter.next() {
            Some('"') => return Ok(result),

            Some('\\') => {
                let escaped = read_escape(iter)?;
                result.push(escaped);
            }

            Some(ch) if (ch as u32) <= 0x1F => {
                return Err(iter.error(format!(
                    "unescaped control character U+{:04X} in string",
                    ch as u32
                )));
            }

            Some(ch) => {
                result.push(ch);
            }

            None => return Err(iter.error("unterminated string literal")),
        }
    }
}

/// 消费一个转义序列（'\\' 已被消费，现在是转义字符本身）
fn read_escape(iter: &mut CharIter) -> Result<char, ParseError> {
    match iter.next() {
        Some('"') => Ok('"'),
        Some('\\') => Ok('\\'),
        Some('/') => Ok('/'),
        Some('b') => Ok('\x08'), // 退格
        Some('f') => Ok('\x0C'), // 换页
        Some('n') => Ok('\n'),   // 换行
        Some('r') => Ok('\r'),   // 回车
        Some('t') => Ok('\t'),   // 制表符

        // Unicode 转义 \uXXXX
        Some('u') => read_unicode_escape(iter),

        Some(ch) => Err(iter.error(format!("invalid escape sequence \\\\{}", ch))),
        None => Err(iter.error("unexpected end of input in escape sequence")),
    }
}

/// 消费 4 位十六进制数字，返回对应字符
fn read_unicode_escape(iter: &mut CharIter) -> Result<char, ParseError> {
    let mut code = 0u32;

    for _ in 0..4 {
        match iter.next() {
            Some(ch @ '0'..='9') => {
                code = code * 16 + (ch as u32 - '0' as u32);
            }
            Some(ch @ 'a'..='f') => {
                code = code * 16 + (ch as u32 - 'a' as u32 + 10);
            }
            Some(ch @ 'A'..='F') => {
                code = code * 16 + (ch as u32 - 'A' as u32 + 10);
            }
            Some(ch) => {
                return Err(iter.error(format!("invalid hex digit '{}' in unicode escape", ch)));
            }
            None => {
                return Err(iter.error("unexpected end of input in unicode escape"));
            }
        }
    }

    match char::from_u32(code) {
        Some(ch) => Ok(ch),
        None => Err(iter.error(format!("invalid unicode code point U+{:04X}", code))),
    }
}
