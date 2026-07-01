//! 词法分析器（Lexer）：将 JSON 字符串切分为 Token 流。
//!
//! ## 入口
//! - `tokenize(&str) -> Result<Vec<Token>, ParseError>`
//!
//! ## 内部辅助函数
//! - `read_string`   — 解析字符串字面量，处理转义
//! - `read_escape`   — 解析单个转义序列（\n \t \\ 等）
//! - `read_unicode_escape` — 解析 \uXXXX
//! - `read_number`   — 状态机解析数字（支持负数、小数、科学计数法）
//! - `read_keyword`  — 解析 true / false / null

use crate::char_iter::CharIter;
use crate::error::ParseError;
use crate::types::Token;

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
                tokens.push(Token::LeftBrace);
            }
            '}' => {
                iter.next();
                tokens.push(Token::RightBrace);
            }
            '[' => {
                iter.next();
                tokens.push(Token::LeftBracket);
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

            // 解析字符串
            '"' => {
                iter.save_pos();
                let s = read_string(&mut iter)?;
                tokens.push(Token::String(s));
            }
            // 解析数字
            '-' | '0'..='9' => {
                iter.save_pos();
                let n = read_number(&mut iter)?;
                tokens.push(Token::Number(n));
            }
            // 解析关键字 true false null
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

// 解析字符串
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

// 解析数字
fn read_number(iter: &mut CharIter) -> Result<f64, ParseError> {
    let mut text = String::new();

    enum State {
        Start,
        Integer,      // 数字
        Fraction,     // 小数
        ExponentSign, // 指数
        Exponent,     // 指数数字
        Done,
    }

    let mut state = State::Start;

    loop {
        // 预读下一位字符 不消费
        let ch = iter.peek();

        // 根据内部状态机的状态决定下一步的操作
        match state {
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

            State::Integer => match ch {
                Some(ch @ '0'..='9') => {
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
                    // 统一为小写的e
                    text.push('e');
                    iter.next();
                    state = State::ExponentSign;
                }

                _ => {
                    state = State::Done;
                }
            },

            State::Fraction => {
                match ch {
                    Some(ch @ '0'..='9') => {
                        text.push(ch);
                        iter.next();
                    }
                    Some('e') | Some('E') => {
                        // 统一为小写的e
                        text.push('e');
                        iter.next();
                        state = State::ExponentSign;
                    }
                    _ => {
                        // 小数部分必须至少有一个数字
                        if text.ends_with('.') {
                            return Err(
                                iter.error("expected at least one digit after decimal point")
                            );
                        }
                        state = State::Done;
                    }
                }
            }
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
            },
            State::Exponent => {
                match ch {
                    Some(ch @ '0'..='9') => {
                        text.push(ch);
                        iter.next();
                    }
                    _ => {
                        // 指数部分必须至少有一个数字
                        let last = text.chars().last().unwrap();
                        if last == 'e' || last == 'E' || last == '+' || last == '-' {
                            return Err(iter.error("expected at least one digit in exponent"));
                        }
                        state = State::Done;
                    }
                }
            }
            State::Done => {
                break;
            }
        }
    }

    text.parse::<f64>()
        .map_err(|_| iter.error(format!("invalid number literal: '{}'", text)))
}

// 解析关键字 true false null
fn read_keyword(iter: &mut CharIter, expected: &str) -> Result<(), ParseError> {
    for expected_ch in expected.chars() {
        match iter.next() {
            Some(ch) if ch == expected_ch => {
                continue;
            }
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

    // 额外检查：关键字后面不能紧跟字母数字（如 "trueX" 是非法的）
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
