//! 错误与位置追踪：定义 Position（行列号）和 ParseError（带位置的解析错误）。
//!
//! 所有 Tokenizer/Parser 的错误都通过 ParseError 返回，
//! 携带源码中的行列信息，方便定位问题。

use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    line: usize, // 从1开始
    col: usize,  // 从1开始 每个字符算1
}

impl Position {
    pub fn new() -> Self {
        Position { line: 1, col: 1 }
    }

    pub fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }
}

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
