use std::borrow::Cow;

use derive_more::Constructor;

use kodept_core::code_point::CodePoint;

use crate::lexer::Token;
use crate::token_stream::TokenStream;

#[derive(Debug, Constructor)]
pub struct ErrorLocation {
    pub in_stream: usize,
    pub in_code: CodePoint,
}

#[derive(Debug, Constructor)]
pub struct ParseError<A> {
    pub expected: Vec<Cow<'static, str>>,
    pub actual: A,
    pub location: ErrorLocation,
}

#[derive(Debug, Constructor)]
pub struct ParseErrors<A> {
    errors: Vec<ParseError<A>>,
}

pub trait Original<Actual> {
    fn point_pos(&self, point: impl Into<CodePoint>) -> usize;
    fn actual(&self, point: impl Into<CodePoint>) -> Actual;
}

impl<A> IntoIterator for ParseErrors<A> {
    type Item = ParseError<A>;
    type IntoIter = std::vec::IntoIter<ParseError<A>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'t> Original<Token<'t>> for TokenStream<'t> {
    fn point_pos(&self, point: impl Into<CodePoint>) -> usize {
        let stream = *self;
        let point1 = point.into();
        stream
            .slice
            .iter()
            .position(|it| it.span.point == point1)
            .unwrap()
    }

    fn actual(&self, point: impl Into<CodePoint>) -> Token<'t> {
        let pos = self.point_pos(point);
        self.slice[pos].token
    }
}

impl<'a, S: From<&'a str>> Original<S> for &'a str {
    fn point_pos(&self, point: impl Into<CodePoint>) -> usize {
        let point = point.into();
        point.offset
    }

    fn actual(&self, point: impl Into<CodePoint>) -> S {
        let point = point.into();
        S::from(&self[point.offset..point.offset + point.length])
    }
}

impl<A> ParseError<A> {
    pub fn map<B>(self, f: impl FnOnce(A) -> B) -> ParseError<B> {
        ParseError {
            expected: self.expected,
            actual: f(self.actual),
            location: self.location,
        }
    }
}

impl<A> ParseErrors<A> {
    pub fn map<B>(self, mut f: impl FnMut(A) -> B) -> ParseErrors<B> {
        ParseErrors::new(self.into_iter().map(move |it| it.map(&mut f)).collect())
    }
}
