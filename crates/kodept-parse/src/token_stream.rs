use std::fmt::{Debug, Display, Formatter};
use std::iter::FusedIterator;

use nom::{InputIter, InputLength, InputTake, Needed, UnspecializedInput};
use nom_supreme::final_parser::RecreateContext;

use kodept_core::code_point::CodePoint;

use crate::lexer::traits::ToRepresentation;
use crate::lexer::{Identifier, Literal, Token};
use crate::token_match::TokenMatch;

#[derive(Clone, Debug)]
pub struct TokenStream<'t> {
    slice: &'t [TokenMatch<'t>],
}

impl<'t> TokenStream<'t> {
    #[must_use]
    #[inline]
    pub fn iter(&self) -> TokenStreamIterator {
        TokenStreamIterator {
            stream: self.clone(),
            position: 0,
        }
    }

    #[must_use]
    pub const fn new(slice: &'t [TokenMatch<'t>]) -> Self {
        Self { slice }
    }

    #[must_use]
    pub fn into_token_match(self) -> Option<TokenMatch<'t>> {
        match self.slice {
            [x] => Some(x.clone()),
            _ => None,
        }
    }

    pub fn as_token_vec(&self) -> Vec<&Token> {
        self.slice.iter().map(|it| &it.token).collect()
    }
}

pub struct TokenStreamIterator<'t> {
    stream: TokenStream<'t>,
    position: usize,
}

pub struct TokenStreamIndices<'t> {
    stream: TokenStream<'t>,
    position: usize,
}

impl<'t> Iterator for TokenStreamIterator<'t> {
    type Item = TokenMatch<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position > self.stream.slice.len() {
            return None;
        }

        let token = self.stream.slice.get(self.position);
        self.position += 1;
        token.cloned()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.slice.iter().size_hint()
    }
}

impl<'t> FusedIterator for TokenStreamIterator<'t> {}

impl<'t> Iterator for TokenStreamIndices<'t> {
    type Item = (usize, TokenMatch<'t>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.position > self.stream.slice.len() {
            return None;
        }

        let token = self.stream.slice.get(self.position);
        self.position += 1;
        token.cloned().map(|it| (self.position - 1, it))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.slice.iter().size_hint()
    }
}

impl<'t> FusedIterator for TokenStreamIndices<'t> {}

impl<'t> InputIter for TokenStream<'t> {
    type Item = TokenMatch<'t>;
    type Iter = TokenStreamIndices<'t>;
    type IterElem = TokenStreamIterator<'t>;

    fn iter_indices(&self) -> Self::Iter {
        TokenStreamIndices {
            stream: self.clone(),
            position: 0,
        }
    }

    fn iter_elements(&self) -> Self::IterElem {
        TokenStreamIterator {
            stream: self.clone(),
            position: 0,
        }
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.slice.iter().position(|it| predicate(it.clone()))
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.slice.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.slice.len()))
        }
    }
}

impl<'t> UnspecializedInput for TokenStream<'t> {}

impl<'t> InputLength for TokenStream<'t> {
    fn input_len(&self) -> usize {
        self.slice.len()
    }
}

impl<'t> InputTake for TokenStream<'t> {
    fn take(&self, count: usize) -> Self {
        TokenStream {
            slice: &self.slice[..count],
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (first, second) = self.slice.split_at(count);
        (Self { slice: second }, Self { slice: first })
    }
}

impl<'t> RecreateContext<TokenStream<'t>> for CodePoint {
    fn recreate_context(original_input: TokenStream<'t>, tail: TokenStream<'t>) -> Self {
        if let Some((head, _)) = tail.slice.split_first() {
            head.span.point
        } else if let Some((last, _)) = original_input.slice.split_last() {
            last.span.point
        } else {
            CodePoint::single_point(0)
        }
    }
}

impl Display for TokenStream<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Some(offset) = self.slice.first().map(|it| it.span.point.offset) else {
            return write!(f, "");
        };
        let size = self.slice.last().expect("Unreachable").span.point.offset - offset;
        let mut output = " ".repeat(size);
        for token_match in self.iter().take(10) {
            let index = token_match.span.point.offset - offset;
            let len = token_match.span.point.length;

            output.replace_range(
                index..index + len,
                match token_match.token {
                    Token::Ignore(_) => " ",
                    Token::Keyword(x) => x.representation(),
                    Token::Symbol(x) => x.representation(),
                    Token::Identifier(x) => match x {
                        Identifier::Identifier(x) => x,
                        Identifier::Type(x) => x,
                    },
                    Token::Literal(x) => match x {
                        Literal::Binary(x) => x,
                        Literal::Octal(x) => x,
                        Literal::Hex(x) => x,
                        Literal::Floating(x) => x,
                        Literal::Char(x) => x,
                        Literal::String(x) => x,
                    },
                    Token::Operator(x) => x.representation(),
                    Token::Unknown => "<unknown>",
                },
            )
        }
        write!(f, "{}...", output.trim())
    }
}

#[cfg(test)]
mod tests {
    use nom::InputTake;

    use kodept_core::code_point::CodePoint;
    use kodept_core::structure::span::Span;

    use crate::lexer::Token;
    use crate::token_match::TokenMatch;
    use crate::token_stream::TokenStream;

    #[test]
    fn test_stream_take() {
        let slice = [TokenMatch::new(
            Token::Unknown,
            Span::new(CodePoint::default()),
        )];
        let stream = TokenStream::new(&slice);

        let (tail, body) = stream.take_split(0);
        assert_eq!(body.slice, &[]);
        assert_eq!(tail.slice, slice);
    }
}
