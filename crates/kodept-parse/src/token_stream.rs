use kodept_core::code_point::CodePoint;
use std::fmt::{Debug, Display, Formatter};
use std::iter::FusedIterator;

use nom::{InputIter, InputLength, InputTake, Needed, UnspecializedInput};
use nom_supreme::final_parser::RecreateContext;

use crate::lexer::traits::ToRepresentation;
use crate::lexer::{Identifier, Ignore, Literal, Token};
use crate::token_match::TokenMatch;
use kodept_core::structure::Located;

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct TokenStream<'t> {
    pub(crate) slice: &'t [TokenMatch<'t>],
}

impl<'t> TokenStream<'t> {
    #[must_use]
    #[inline]
    pub fn iter(&self) -> TokenStreamIterator {
        TokenStreamIterator {
            stream: *self,
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
            [x] => Some(*x),
            _ => None,
        }
    }

    pub fn token_iter(&self) -> impl Iterator<Item = &Token> {
        self.slice.iter().map(|it| &it.token)
    }

    pub fn len(&self) -> usize {
        self.slice.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }
    
    pub fn iter_indices(&self) -> TokenStreamIndices {
        TokenStreamIndices {
            stream: *self,
            position: 0,
        }
    }
    
    pub fn iter_elements(&self) -> TokenStreamIterator {
        TokenStreamIterator {
            stream: *self,
            position: 0,
        }
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
            stream: *self,
            position: 0,
        }
    }

    fn iter_elements(&self) -> Self::IterElem {
        TokenStreamIterator {
            stream: *self,
            position: 0,
        }
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.slice.iter().position(|it| predicate(*it))
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
        self.len()
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
        let size = self.slice.last().expect("Unreachable").span.point.offset - offset + 1;
        // FIXME: wrong size
        let mut output = " ".repeat((size * 4) as usize);
        for token_match in self.iter() {
            let index = (token_match.span.point.offset - offset) as usize;
            let len = token_match.span.point.length as usize;
            
            output.replace_range(
                index..index + len,
                match token_match.token {
                    Token::Ignore(Ignore::Newline) => "\n",
                    Token::Ignore(Ignore::Whitespace) => "",
                    Token::Ignore(Ignore::MultilineComment(x)) => x,
                    Token::Ignore(Ignore::Comment(x)) => x,
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
                    Token::Unknown => "?",
                },
            )
        }
        write!(f, "{}", output)
    }
}

impl Located for TokenStream<'_> {
    fn location(&self) -> CodePoint {
        let len = self.slice.iter().map(|it| it.span.point.length).sum();
        
        match self.slice { 
            [x, ..] => CodePoint::new(len, x.span.point.offset),
            [] => CodePoint::new(0, 0)
        }
    }
}
