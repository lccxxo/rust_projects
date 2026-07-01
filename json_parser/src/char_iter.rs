//! 字符遍历器：将 &str 包装为支持 peek/next 的迭代器。
//!
//! - `peek()` — 预读下一个字符但不消费
//! - `next()` — 消费并返回下一个字符，自动更新 Position
//! - `save_pos()` + `error()` — 快照当前位置，构造 ParseError

use crate::error::{ParseError,Position};

pub struct CharIter<'a> {
    chars: std::str::Chars<'a>, // 迭代器
    peeked: Option<char>,       // 往前一个字符 相当于一个字符缓冲区
    pos: Position,              // 当前位置
    save: Position,             // 暂存的位置 用于错误打印
}

impl<'a> CharIter<'a> {
    pub fn new(input: &'a str) -> Self {
        CharIter {
            chars: input.chars(),
            peeked: None,
            pos: Position::new(),
            save: Position::new(),
        }
    }

    pub fn save_pos(&mut self) {
        self.save = self.pos;
    }

    pub fn error(&self, msg: impl Into<String>) -> ParseError {
        ParseError {
            message: msg.into(),
            pos: self.save,
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        if self.peeked.is_none() {
            self.peeked = self.chars.next();
        }
        self.peeked
    }

    pub fn next(&mut self) -> Option<char> {
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