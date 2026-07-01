#[cfg(test)]
mod tests {
    use crate::parse;
    use crate::tokenize;
    use crate::types::{JsonValue, Token};
    use std::collections::HashMap;

    // ── Lexer 测试 ──

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("").unwrap();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_tokenize_whitespace() {
        let tokens = tokenize("  \t\n\r  ").unwrap();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_tokenize_literals() {
        let tokens = tokenize("true false null").unwrap();
        assert_eq!(tokens, vec![Token::True, Token::False, Token::Null,]);
    }

    #[test]
    fn test_tokenize_simple_string() {
        let tokens = tokenize(r#""hello""#).unwrap();
        assert_eq!(tokens, vec![Token::String("hello".to_string()),]);
    }

    #[test]
    fn test_tokenize_string_with_escapes() {
        let tokens = tokenize(r#""line1\nline2\tindented""#).unwrap();
        assert_eq!(
            tokens,
            vec![Token::String("line1\nline2\tindented".to_string()),]
        );
    }

    #[test]
    fn test_tokenize_unicode_escape() {
        let tokens = tokenize(r#""\u0041\u0042\u0043""#).unwrap();
        assert_eq!(tokens, vec![Token::String("ABC".to_string()),]);
    }

    #[test]
    fn test_tokenize_numbers() {
        assert_eq!(tokenize("42").unwrap(), vec![Token::Number(42.0)]);
        assert_eq!(tokenize("-3.14").unwrap(), vec![Token::Number(-3.14)]);
        assert_eq!(tokenize("1.5e10").unwrap(), vec![Token::Number(1.5e10)]);
        assert_eq!(tokenize("2.5e-3").unwrap(), vec![Token::Number(2.5e-3)]);
    }

    #[test]
    fn test_tokenize_number_leading_zero() {
        assert!(tokenize("01").is_err());
        assert!(tokenize("-01").is_err());
        // 0 本身合法
        assert!(tokenize("0").is_ok());
    }

    #[test]
    fn test_tokenize_invalid_number_trailing_dot() {
        assert!(tokenize("1.").is_err());
    }

    #[test]
    fn test_tokenize_invalid_number_empty_exponent() {
        assert!(tokenize("1e").is_err());
    }

    #[test]
    fn test_tokenize_unterminated_string() {
        assert!(tokenize(r#""hello"#).is_err());
    }

    #[test]
    fn test_tokenize_invalid_escape() {
        assert!(tokenize(r#""\x""#).is_err());
    }

    #[test]
    fn test_tokenize_control_char_in_string() {
        // 原始换行符在字符串中是无效的（必须转义为 \n）
        let input = "\"\n\"";
        assert!(tokenize(input).is_err());
    }

    #[test]
    fn test_tokenize_keyword_partial_match() {
        // "trueX" 不是合法 token
        assert!(tokenize("trueX").is_err());
    }

    // ── Parser 测试 ──

    #[test]
    fn test_parse_null() {
        let value = parse("null").unwrap();
        assert_eq!(value, JsonValue::Null);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse("false").unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse("-3.14").unwrap(), JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse(r#""hello""#).unwrap(),
            JsonValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_parse_empty_array() {
        assert_eq!(parse("[]").unwrap(), JsonValue::Array(vec![]));
    }

    #[test]
    fn test_parse_simple_array() {
        let value = parse("[1, true, null]").unwrap();
        assert_eq!(
            value,
            JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Bool(true),
                JsonValue::Null,
            ])
        );
    }

    #[test]
    fn test_parse_nested_array() {
        let value = parse("[1, [2, [3]]]").unwrap();
        assert_eq!(
            value,
            JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Array(vec![
                    JsonValue::Number(2.0),
                    JsonValue::Array(vec![JsonValue::Number(3.0),]),
                ]),
            ])
        );
    }

    #[test]
    fn test_parse_empty_object() {
        let value = parse("{}").unwrap();
        assert_eq!(value, JsonValue::Object(HashMap::new()));
    }

    #[test]
    fn test_parse_simple_object() {
        let value = parse(r#"{"name": "Alice", "age": 30}"#).unwrap();
        match value {
            JsonValue::Object(map) => {
                assert_eq!(
                    map.get("name"),
                    Some(&JsonValue::String("Alice".to_string()))
                );
                assert_eq!(map.get("age"), Some(&JsonValue::Number(30.0)));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_parse_duplicate_keys() {
        let value = parse(r#"{"a": 1, "a": 2}"#).unwrap();
        match value {
            JsonValue::Object(map) => {
                assert_eq!(map.get("a"), Some(&JsonValue::Number(2.0)));
            }
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn test_parse_trailing_comma_array() {
        assert!(parse("[1,]").is_err());
    }

    #[test]
    fn test_parse_trailing_comma_object() {
        assert!(parse(r#"{"a":1,}"#).is_err());
    }

    #[test]
    fn test_parse_missing_comma() {
        assert!(parse("[1 2]").is_err());
    }

    #[test]
    fn test_parse_unexpected_token() {
        assert!(parse("{").is_err());
    }

    #[test]
    fn test_parse_trailing_garbage() {
        assert!(parse("1 2").is_err());
    }

    #[test]
    fn test_parse_empty_input() {
        assert!(parse("").is_err());
    }
}
