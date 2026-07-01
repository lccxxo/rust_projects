//! 字符遍历器:把 `&str` 包装为支持 `peek` / `next` 的迭代器。
//!
//! 普通的 `std::str::Chars` 只能**消费**字符,不能预读。但手写 parser 经常需要"看一眼下一个字符是什么,
//! 决定走哪条分支,然后再消费它"——这就是经典的 **LL(1) lookahead** 模式。
//!
//! `CharIter` 在 `Chars` 之上加了一层 1 个字符的 lookahead 缓冲,实现 `peek` / `next` 配对调用:
//!
//! - [`CharIter::peek`] —— 预读下一字符,**不消费**
//! - [`CharIter::next`] —— 消费并返回下一字符
//! - [`CharIter::save_pos`] + [`CharIter::error`] —— 快照当前位置,用于构造带位置信息的错误
//!
//! ## 关键不变量
//!
//! 1. `peek` 是幂等的:连续调多次只算一次 `Chars::next`
//! 2. `peek` 之后调 `next` 一定拿到刚才 `peek` 看到的那个字符
//! 3. `next` 之后,`pos` 自动推进(用 [`Position::advance`])
//!
//! ## 状态机
//!
//! ```text
//!                  peek() 命中缓存         peek() 缓存为空
//!                  ────────────────         ────────────────
//!   peeked == Some   ──► 返回 *peeked        │
//!                                            └──► chars.next() → 写入 peeked → 返回
//!   peeked == None   ──► chars.next() ──► 缓存
//! ```
//!
//! ## 完整示例
//!
//! ```
//! use json_parser::char_iter::CharIter;
//!
//! let mut it = CharIter::new("ab");
//! assert_eq!(it.peek(), Some('a'));  // 预读 a,不消费
//! assert_eq!(it.peek(), Some('a'));  // 还是 a,缓存命中
//! assert_eq!(it.next(), Some('a'));  // 消费 a
//! assert_eq!(it.next(), Some('b'));  // 消费 b
//! assert_eq!(it.next(), None);        // 流结束
//! ```

use crate::error::{ParseError, Position};

/// 带 lookahead 的字符流迭代器。
///
/// 内部由两层组成:
/// - `chars`:`std::str::Chars`,真正的字符源
/// - `peeked`:1 个字符的缓存,存 `peek` 看到但还没消费的那个字符
///
/// `'a` 生命周期对应借用的输入字符串切片——`CharIter` 不会复制字符串内容,
/// 也不持有超出输入生命周期的引用。
pub struct CharIter<'a> {
    /// 底层字符迭代器,消耗到最后就是 `None`。
    chars: std::str::Chars<'a>,
    /// 预读缓存,`Some(c)` 表示下一个 `next` 会返回 `c` 而不去碰 `chars`。
    peeked: Option<char>,
    /// 当前已经**消费到的**位置(`pos` 总是指向"下一个要返回的字符"的位置)
    pos: Position,
    /// 通过 [`CharIter::save_pos`] 暂存的位置,用于构造错误时报告"出问题的那一格"的坐标
    save: Position,
}

impl<'a> CharIter<'a> {
    /// 从字符串切片创建字符流,初始位置 `(1, 1)`。
    pub fn new(input: &'a str) -> Self {
        CharIter {
            chars: input.chars(),
            peeked: None,
            pos: Position::new(),
            save: Position::new(),
        }
    }

    /// 把当前 `pos` 快照到 `save`。
    ///
    /// 一般调用顺序:看到可能要出错的 token → `save_pos()` → 继续往下试 → 真错了就用 `error(msg)` 报错。
    /// 这样错误信息会指向**触发错误的起点**,而不是"出错之后看到的位置"。
    pub fn save_pos(&mut self) {
        self.save = self.pos;
    }

    /// 用 `save` 位置构造一个 [`ParseError`],消息是 `msg.into()`。
    ///
    /// 必须在 [`CharIter::save_pos`] 之后调用,否则位置信息会过期。
    pub fn error(&self, msg: impl Into<String>) -> ParseError {
        ParseError {
            message: msg.into(),
            pos: self.save,
        }
    }

    /// **预读**下一字符,不消费。
    ///
    /// 第一次调用会触发 `chars.next()` 并把结果缓存到 `peeked`;
    /// 之后再次调用直接返回缓存,不会移动流。
    ///
    /// 行为对照:
    /// - `Chars::peek`(标准库没有)—— `CharIter` 自己实现的版本
    /// - `Bytes::peek`(标准库 `Peekable` 适配器)——语义一致
    ///
    /// 流结束时返回 `None`。
    pub fn peek(&mut self) -> Option<char> {
        if self.peeked.is_none() {
            self.peeked = self.chars.next();
        }
        self.peeked
    }

    /// **消费**下一字符,推进 `pos`。
    ///
    /// 优先返回 `peeked` 缓存(并清空它),否则从 `chars` 拉一个。
    /// 任何情况都会调用 [`Position::advance`] 更新位置。
    ///
    /// 流结束时返回 `None`。
    pub fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.peeked.take() {
            self.pos.advance(ch);
            return Some(ch);
        }

        // 缓存没有 → 从 chars 拉一个,顺便推进位置
        self.chars.next().map(|ch| {
            self.pos.advance(ch);
            ch
        })
    }

    /// 当前 `pos` 的只读访问(主要用于测试和错误恢复)。
    pub fn position(&self) -> Position {
        self.pos
    }
}
