use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

/// A tantivy Tokenizer that delegates to jieba for Chinese text segmentation.
/// For non-Chinese text, jieba still produces reasonable word-boundary tokens.
#[derive(Clone)]
pub struct JiebaTokenizer {
    inner: jieba_rs::Jieba,
}

impl JiebaTokenizer {
    pub fn new() -> Self {
        Self {
            inner: jieba_rs::Jieba::new(),
        }
    }
}

impl Tokenizer for JiebaTokenizer {
    type TokenStream<'a> = JiebaTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> JiebaTokenStream<'a> {
        let tokens = self
            .inner
            .tokenize(text, jieba_rs::TokenizeMode::Search, true);
        JiebaTokenStream {
            tokens,
            index: 0,
            current: Token::default(),
        }
    }
}

pub struct JiebaTokenStream<'a> {
    tokens: Vec<jieba_rs::Token<'a>>,
    index: usize,
    current: Token,
}

impl TokenStream for JiebaTokenStream<'_> {
    fn advance(&mut self) -> bool {
        if self.index >= self.tokens.len() {
            return false;
        }
        let t = &self.tokens[self.index];
        self.current = Token {
            offset_from: t.start,
            offset_to: t.end,
            position: self.index,
            text: t.word.to_lowercase(),
            position_length: 1,
        };
        self.index += 1;
        true
    }

    fn token(&self) -> &Token {
        &self.current
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.current
    }
}
