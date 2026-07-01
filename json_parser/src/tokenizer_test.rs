use crate::tokenizer::tokenize;
use crate::types::Token;
use crate::types::Token::*;

// ── 单字面量 ──

#[test]
fn test_single_true() {
    let tokens = tokenize("true").unwrap();
    assert_eq!(tokens, vec![True]);
}

#[test]
fn test_single_false() {
    assert_eq!(tokenize("false").unwrap(), vec![False]);
}

#[test]
fn test_single_null() {
    assert_eq!(tokenize("null").unwrap(), vec![Null]);
}

// ── 数字 ──

#[test]
fn test_integer() {
    assert_eq!(tokenize("42").unwrap(), vec![Number(42.0)]);
}

#[test]
fn test_float() {
    assert_eq!(tokenize("3.14").unwrap(), vec![Number(3.14)]);
}

// ── 字符串 ──

#[test]
fn test_simple_string() {
    assert_eq!(
        tokenize(r#""hello""#).unwrap(),
        vec![String("hello".into())]
    );
}

// ── 数组 ──

#[test]
fn test_array() {
    assert_eq!(
        tokenize("[1, true]").unwrap(),
        vec![LeftBracket, Number(1.0), Comma, True, RightBracket,]
    );
}

// ── 错误 ──

#[test]
fn test_unexpected_char() {
    assert!(tokenize("@").is_err());
}
