#[cfg(all(not(feature = "peg"), not(feature = "pest"), feature = "nom"))]
pub type Tokenizer<'t> = simple_implementation::Tokenizer<'t>;

#[cfg(any(feature = "peg", feature = "pest"))]
pub type Tokenizer<'t> = crate::grammar::KodeptParser<'t>;

#[cfg(all(feature = "peg", feature = "trace"))]
pub type TracedTokenizer<'t> = crate::grammar::peg::Tokenizer<'t, true>;

pub type LazyTokenizer<'t> = simple_implementation::Tokenizer<'t>;

mod simple_implementation {
    use std::iter::FusedIterator;

    use tracing::error;

    use kodept_core::code_point::CodePoint;
    use kodept_core::structure::span::Span;

    use crate::lexer::Token;
    use crate::parse_token;
    use crate::token_match::TokenMatch;

    pub struct Tokenizer<'t> {
        buffer: &'t str,
        pos: usize,
    }

    impl<'t> Tokenizer<'t> {
        #[must_use]
        #[inline]
        pub const fn new(reader: &'t str) -> Self {
            Self {
                buffer: reader,
                pos: 0,
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

            let mut token_match = parse_token(&self.buffer[self.pos..], self.buffer).unwrap_or_else(|e| {
                error!(
                    input = &self.buffer[self.pos..self.pos + 10],
                    "Cannot parse token: {e:#?}"
                );
                TokenMatch::new(Token::Unknown, Span::new(CodePoint::single_point(self.pos)))
            });

            token_match.span.point.offset = self.pos;
            self.pos += token_match.span.point.length;

            Some(token_match)
        }
    }
    
    impl FusedIterator for Tokenizer<'_> {}
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
