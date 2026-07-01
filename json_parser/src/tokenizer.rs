//! 词法分析器(Lexer):把 JSON 字符串切分为 `Token` 流。
//!
//! ## 入口
//!
//! [`tokenize`] —— 从 `&str` 得到 `Result<Vec<Token>, ParseError>`。
//!
//! ## 内部模块
//!
//! 词法分析按字符类型分派到不同的读取函数:
//!
//! | 字符类别 | 读取函数 | 产出 |
//! |---|---|---|
//! | 空白(` `/`\t`/`\n`) | 直接 `next()` 跳过 | —— |
//! | `{` `}` `[` `]` `,` `:` | 直接 `next()` 消费 | 对应标点 `Token` |
//! | `"` | `read_string` | `Token::String(String)`(已解码转义) |
//! | `-` / 数字 | `read_number` | `Token::Number(f64)` |
//! | `t` / `f` / `n` | `read_keyword` | `Token::True` / `False` / `Null` |
//! | 其他 | 报错 | `ParseError` |
//!
//! ## 数据流
//!
//! ```text
//!   &str  ──►  CharIter::new  ──►  while let Some(ch) = peek()  ──►  match ch 分派
//!                                                                        │
//!                          ┌─────────────────────────────────────────────┤
//!                          ▼                                             ▼
//!                   read_string / read_number / read_keyword        直接产标点 Token
//!                          │
//!                          ▼
//!                  写入 tokens: Vec<Token>
//! ```
//!
//! ## 数字状态机
//!
//! `read_number` 内部用一个 5 状态的有限状态机解析数字字面量:
//!
//! ```text
//!   Start ─(-)─► Integer ─(.)─► Fraction ─(e/E)─► ExponentSign ─(digit)─► Exponent ──► Done
//!                  │                │                  │                    │
//!                  └──(e/E)─────────┴──(e/E)───────────┘                    │
//!                                                                           └─(其他)─► Done
//! ```
//!
//! 校验项:
//! - 禁止前导零(`01` 非法)
//! - 小数点后必须有数字
//! - 指数符号后必须有数字

use crate::char_iter::CharIter;
use crate::error::ParseError;
use crate::types::Token;

/// 把 JSON 字符串切成 [`Token`] 流。
///
/// ## 流程
///
/// 1. 包装 `&str` 为 [`CharIter`]
/// 2. 循环 `peek` 下一个非空字符
/// 3. 根据首字符分派到对应处理逻辑
/// 4. 把产出的 `Token` push 进结果 `Vec`
///
/// ## 错误
///
/// - 遇到非合法字符 → `ParseError`(`unexpected character 'X'`)
///
/// ## 示例
///
/// ```
/// use json_parser::{tokenize, Token};
///
/// let tokens = tokenize("[1, \"hi\", true]").unwrap();
/// assert_eq!(tokens, vec![
///     Token::LeftBracket,
///     Token::Number(1.0),
///     Token::Comma,
///     Token::String("hi".into()),
///     Token::Comma,
///     Token::True,
///     Token::RightBracket,
/// ]);
/// ```
pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut iter = CharIter::new(input);
    let mut tokens = Vec::new();

    // 主循环:不停 peek 下一个非空字符,直到流结束
    while let Some(ch) = iter.peek() {
        match ch {
            // ── 空白字符:直接跳过 ──
            ' ' | '\t' | '\n' | '\r' => {
                iter.next();
            }

            // ── 标点符号:消耗字符,产出对应 Token ──
            '{' => { iter.next(); tokens.push(Token::LeftBrace); }
            '}' => { iter.next(); tokens.push(Token::RightBrace); }
            '[' => { iter.next(); tokens.push(Token::LeftBracket); }
            ']' => { iter.next(); tokens.push(Token::RightBracket); }
            ',' => { iter.next(); tokens.push(Token::Comma); }
            ':' => { iter.next(); tokens.push(Token::Colon); }

            // ── 字符串:吃 "..." 含转义 ──
            '\"' => {
                iter.save_pos();
                let s = read_string(&mut iter)?;
                tokens.push(Token::String(s));
            }

            // ── 数字:状态机解析 ──
            '-' | '0'..='9' => {
                iter.save_pos();
                let n = read_number(&mut iter)?;
                tokens.push(Token::Number(n));
            }

            // ── 关键字:三选一 ──
            't' => {
                iter.save_pos();
                read_keyword(&mut iter, "true")?;
                tokens.push(Token::True);
            }
            'f' => {
                iter.save_pos();
                read_keyword(&mut iter, "false")?;
                tokens.push(Token::False);
            }
            'n' => {
                iter.save_pos();
                read_keyword(&mut iter, "null")?;
                tokens.push(Token::Null);
            }

            // ── 非法字符:报错并退出 ──
            other => {
                iter.save_pos();
                iter.next();
                return Err(iter.error(format!("unexpected character '{}'", other)));
            }
        }
    }
    Ok(tokens)
}

/// 解析双引号包裹的字符串字面量,**包括转义解码**。
///
/// 入口:已经 peek 到首个 `"`,函数会消费它。
/// 出口:消费闭合 `"`,返回解码后的 [`String`]。
///
/// 支持的转义:`\"` `\\` `\/` `\b` `\f` `\n` `\r` `\t` `\uXXXX`。
/// 不允许出现未转义的控制字符(U+0000 ~ U+001F)。
fn read_string(iter: &mut CharIter) -> Result<String, ParseError> {
    // 吃掉开头的 "
    match iter.next() {
        Some('\"') => {}
        Some(ch) => return Err(iter.error(format!("expected '\\\"', found '{}'", ch))),
        None => return Err(iter.error("unexpected end of input")),
    }

    let mut result = String::new();

    // 循环吃字符,直到遇到闭合 "
    loop {
        match iter.next() {
            // 遇到闭合 ":字符串结束
            Some('\"') => return Ok(result),

            // 转义:吃掉 \\ 后处理下一个字符
            Some('\\') => {
                let escaped = read_escape(iter)?;
                result.push(escaped);
            }

            // 禁止未转义的控制字符(JSON 安全要求)
            Some(ch) if (ch as u32) <= 0x1F => {
                return Err(iter.error(format!(
                    "unescaped control character U+{:04X} in string",
                    ch as u32
                )));
            }

            // 普通字符
            Some(ch) => {
                result.push(ch);
            }

            // 流到结尾还没找到闭合 "
            None => return Err(iter.error("unterminated string literal")),
        }
    }
}

/// 处理 `\\` 后面的单个字符(转义序列的第二位)。
///
/// 对应到 RFC 8259 的 StringEscapeProduction。
fn read_escape(iter: &mut CharIter) -> Result<char, ParseError> {
    match iter.next() {
        // 简单转义
        Some('\"') => Ok('\"'),
        Some('\\') => Ok('\\'),
        Some('/') => Ok('/'),
        Some('b') => Ok('\x08'),  // 退格
        Some('f') => Ok('\x0C'),  // 换页
        Some('n') => Ok('\n'),     // 换行
        Some('r') => Ok('\r'),     // 回车
        Some('t') => Ok('\t'),     // 制表符

        // Unicode 转义:\\uXXXX
        Some('u') => read_unicode_escape(iter),

        // 非法转义字符
        Some(ch) => Err(iter.error(format!("invalid escape sequence \\\\{}", ch))),
        None => Err(iter.error("unexpected end of input in escape sequence")),
    }
}

/// 解析 `\\uXXXX` 形式的 Unicode 转义,返回一个 char。
///
/// 简化:不做 surrogate pair 配对,直接把 U+XXXX 转成 `char`。
/// 大部分 BMP 内的字符能正确处理;超出 BMP 的代理对会原样输出两个独立的 char。
fn read_unicode_escape(iter: &mut CharIter) -> Result<char, ParseError> {
    let mut code = 0u32;

    // 吃 4 个十六进制位
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

    // 验证码点合法
    match char::from_u32(code) {
        Some(ch) => Ok(ch),
        None => Err(iter.error(format!("invalid unicode code point U+{:04X}", code))),
    }
}

/// 解析数字字面量(整数 / 小数 / 负数 / 科学计数法),用 5 状态 FSM 实现。
///
/// 状态机:
///
/// | 当前状态 | 见到 | 下一状态 | 副作用 |
/// |---|---|---|---|
/// | `Start` | `-` | `Integer` | 记 `-` |
/// | `Start` | `0..9` | `Integer` | —— |
/// | `Integer` | `0..9` | `Integer` | 记数字,校验前导零 |
/// | `Integer` | `.` | `Fraction` | 记 `.` |
/// | `Integer` | `e/E` | `ExponentSign` | 记 `e` |
/// | `Integer` | 其他 | `Done` | 退出 |
/// | `Fraction` | `0..9` | `Fraction` | 记数字 |
/// | `Fraction` | `e/E` | `ExponentSign` | 记 `e` |
/// | `Fraction` | 其他 | `Done` | 必须至少有一个数字 |
/// | `ExponentSign` | `+/-` | `Exponent` | 记符号 |
/// | `ExponentSign` | `0..9` | `Exponent` | —— |
/// | `Exponent` | `0..9` | `Exponent` | 记数字 |
/// | `Exponent` | 其他 | `Done` | 必须至少有一个数字 |
///
/// 出口:用 `text.parse::<f64>()` 把累积字符串转成 `f64`。
fn read_number(iter: &mut CharIter) -> Result<f64, ParseError> {
    let mut text = String::new();

    // FSM 状态枚举
    enum State {
        Start,         // 起点,准备接收符号或首位数字
        Integer,       // 整数部分
        Fraction,      // 小数部分(已见过 .)
        ExponentSign,  // 指数符号(已见过 e/E)
        Exponent,      // 指数数字
        Done,          // 结束
    }

    let mut state = State::Start;

    loop {
        let ch = iter.peek();

        match state {
            // ── 起点:接受 - 或首位数字 ──
            State::Start => match ch {
                Some('-') => {
                    text.push('-');
                    iter.next();
                    state = State::Integer;
                }
                Some('0'..='9') => {
                    state = State::Integer;
                }
                _ => {
                    return Err(iter.error("expected digit or '-'"));
                }
            },

            // ── 整数部分:记数字,可跳到 Fraction / ExponentSign ──
            State::Integer => match ch {
                Some(ch @ '0'..='9') => {
                    // 禁止前导零(但允许 -0、0、0.x、0e1 等合法情形)
                    if text.ends_with('0')
                        && (text == "0" || text == "-0")
                        && text != "e"
                        && text != "E"
                        && text != "."
                    {
                        return Err(iter.error("leading zeros are not allowed"));
                    }
                    text.push(ch);
                    iter.next();
                }
                Some('.') => {
                    text.push('.');
                    iter.next();
                    state = State::Fraction;
                }
                Some('e') | Some('E') => {
                    // 统一存成小写 e
                    text.push('e');
                    iter.next();
                    state = State::ExponentSign;
                }
                _ => state = State::Done,
            },

            // ── 小数部分:记数字,可跳到 ExponentSign ──
            State::Fraction => match ch {
                Some(ch @ '0'..='9') => {
                    text.push(ch);
                    iter.next();
                }
                Some('e') | Some('E') => {
                    text.push('e');
                    iter.next();
                    state = State::ExponentSign;
                }
                _ => {
                    // 小数点后必须有数字
                    if text.ends_with('.') {
                        return Err(
                            iter.error("expected at least one digit after decimal point")
                        );
                    }
                    state = State::Done;
                }
            }

            // ── 指数符号位:接受 + / - / 数字 ──
            State::ExponentSign => match ch {
                Some(c @ '-') | Some(c @ '+') => {
                    text.push(c);
                    iter.next();
                    state = State::Exponent;
                }
                Some('0'..='9') => {
                    state = State::Exponent;
                }
                _ => {
                    return Err(iter.error("expected digit or '+' or '-' after 'e'"));
                }
            }

            // ── 指数数字:记数字,见其他则结束(但必须至少 1 位) ──
            State::Exponent => match ch {
                Some(ch @ '0'..='9') => {
                    text.push(ch);
                    iter.next();
                }
                _ => {
                    // 检查指数部分至少有一个数字
                    let last = text.chars().last().unwrap();
                    if last == 'e' || last == 'E' || last == '+' || last == '-' {
                        return Err(iter.error("expected at least one digit in exponent"));
                    }
                    state = State::Done;
                }
            }

            // ── 结束:跳出循环 ──
            State::Done => break,
        }
    }

    // 把累积字符串交给 Rust 标准库解析 f64
    text.parse::<f64>()
        .map_err(|_| iter.error(format!("invalid number literal: '{}'", text)))
}

/// 解析关键字字面量(`true` / `false` / `null`)。
///
/// 1. 逐字符比对 `expected`,任何不匹配就报错
/// 2. 关键字结束后,必须看到非字母数字字符(`trueX` 非法)
fn read_keyword(iter: &mut CharIter, expected: &str) -> Result<(), ParseError> {
    for expected_ch in expected.chars() {
        match iter.next() {
            Some(ch) if ch == expected_ch => continue,
            Some(ch) => {
                return Err(iter.error(format!(
                    "unexpected character '{}' (expected keyword '{}')",
                    ch, expected
                )));
            }
            None => {
                return Err(iter.error(format!("unexpected end of input, expected '{}'", expected)));
            }
        }
    }

    // 关键字后不能紧跟字母数字
    if let Some(ch) = iter.peek() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            return Err(iter.error(format!(
                "unexpected character '{}' after keyword '{}'",
                ch, expected
            )));
        }
    }

    Ok(())
}
