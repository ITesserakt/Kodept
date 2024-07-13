#[cfg(all(not(feature = "peg"), not(feature = "pest"), feature = "nom"))]
pub type Tokenizer<'t> = simple_implementation::Tokenizer<'t>;

#[cfg(any(feature = "peg", feature = "pest"))]
pub type Tokenizer<'t> = crate::grammar::KodeptParser<'t>;

#[cfg(all(feature = "peg", feature = "trace"))]
pub type TracedTokenizer<'t> = crate::grammar::peg::Tokenizer<'t, true>;

#[cfg(feature = "nom")]
pub type SimpleTokenizer<'t> = simple_implementation::Tokenizer<'t>;

#[cfg(feature = "nom")]
mod simple_implementation {
    use std::convert::Infallible;
    use crate::lexer::{Token, token};
    use crate::token_match::TokenMatch;
    use kodept_core::code_point::CodePoint;
    use kodept_core::structure::span::Span;
    use tracing::error;

    pub struct Tokenizer<'t> {
        buffer: &'t str,
        pos: usize,
    }

    impl<'t> Tokenizer<'t> {
        #[must_use]
        pub const fn new(reader: &'t str) -> Self {
            Self {
                buffer: reader,
                pos: 0,
            }
        }
        
        pub const fn try_new(reader: &'t str) -> Result<Self, Infallible> {
            Ok(Self::new(reader))
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

            let (rest, token) = token(&self.buffer[self.pos..]).unwrap_or_else(|e| {
                error!(
                    input = &self.buffer[self.pos..self.pos + 10],
                    "Cannot parse token: {e:#?}"
                );
                ("", Token::Unknown)
            });

            let matched_length = self.buffer.len() - rest.len() - self.pos;
            let span: TokenMatch =
                TokenMatch::new(token, Span::new(CodePoint::new(matched_length, self.pos)));

            self.pos += matched_length;

            Some(span)
        }
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
            spans.iter().map(|it| it.token).collect::<Vec<_>>(),
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
