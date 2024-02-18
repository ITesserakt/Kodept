use std::str::Utf8Error;

use nom_supreme::error::ErrorTree;
use thiserror::Error;

use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::Span;

use crate::lexer::{token, Token};
use crate::token_match::TokenMatch;

#[derive(Error, Debug)]
pub enum TokenizeError {
    #[error("Error while tokenizing: {0}")]
    Parse(#[from] ErrorTree<String>),
    #[error("Error while reading: {0}")]
    IO(#[from] std::io::Error),
    #[error("Found not utf-8 byte at {pos}", pos = _0.valid_up_to())]
    Utf8(#[from] Utf8Error),
}

pub struct Tokenizer<'t> {
    buffer: &'t str,
    pos: usize,
    row: u32,
    col: u16,
}

impl<'t> Tokenizer<'t> {
    #[must_use]
    pub const fn new(reader: &'t str) -> Self {
        Self {
            buffer: reader,
            pos: 0,
            row: 1,
            col: 1,
        }
    }

    pub fn into_vec(self) -> Vec<TokenMatch<'t>> {
        let mut vec = self.collect::<Vec<_>>();
        vec.shrink_to_fit();
        vec
    }
}

impl<'t> Iterator for Tokenizer<'t> {
    type Item = TokenMatch<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer[self.pos..].is_empty() {
            return None;
        }

        let (rest, token) = match token(&self.buffer[self.pos..]) {
            Ok(x) => x,
            _ => ("", Token::Unknown),
        };

        let matched_length = self.buffer.len() - rest.len() - self.pos;
        let original_text: &str = &self.buffer[self.pos..self.pos + matched_length];
        let span: TokenMatch =
            TokenMatch::new(token, Span::new(CodePoint::new(matched_length, self.pos)));

        for ch in original_text.chars() {
            if ch == '\n' {
                self.row += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        self.pos += matched_length;

        Some(span)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use kodept_core::code_point::CodePoint;

    use crate::lexer::{
        Identifier::*, Ignore::*, Keyword::*, MathOperator::*, Operator::*, Symbol::*,
    };
    use crate::tokenizer::Tokenizer;

    #[test]
    fn test_tokenizer_simple() {
        let input = " fun foo(x: Int, y: Int) => \n  x + y";
        let tokenizer = Tokenizer::new(input);
        let spans: Vec<_> = tokenizer.collect();

        assert_eq!(spans.len(), 26);
        assert_eq!(
            spans.iter().map(|it| it.token.clone()).collect::<Vec<_>>(),
            vec![
                Whitespace.into(),
                Fun.into(),
                Whitespace.into(),
                Identifier("foo").into(),
                LParen.into(),
                Identifier("x").into(),
                Colon.into(),
                Whitespace.into(),
                Type("Int").into(),
                Comma.into(),
                Whitespace.into(),
                Identifier("y").into(),
                Colon.into(),
                Whitespace.into(),
                Type("Int").into(),
                RParen.into(),
                Whitespace.into(),
                Flow.into(),
                Whitespace.into(),
                Newline.into(),
                Whitespace.into(),
                Identifier("x").into(),
                Whitespace.into(),
                Math(Plus).into(),
                Whitespace.into(),
                Identifier("y").into()
            ]
        );
        assert_eq!(spans.get(20).unwrap().span.point, CodePoint::new(2, 29))
    }
}
