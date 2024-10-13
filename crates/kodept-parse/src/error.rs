use std::borrow::Cow;

use derive_more::Constructor;

use kodept_core::code_point::CodePoint;

use crate::lexer::PackedToken;
use crate::token_stream::PackedTokenStream;

#[derive(Debug, Constructor)]
pub struct ErrorLocation {
    pub in_stream: usize,
    pub in_code: CodePoint,
}

type StaticStr = Cow<'static, str>;

#[derive(Debug)]
pub enum ParseError<A> {
    ExpectedInstead {
        expected: Vec<StaticStr>,
        actual: A,
        location: ErrorLocation,
        hints: Vec<StaticStr>,
    },
    ExpectedNotEOF {
        expected: Vec<StaticStr>,
        location: ErrorLocation,
        hints: Vec<StaticStr>,
    },
}

#[derive(Debug, Constructor)]
pub struct ParseErrors<A> {
    errors: Vec<ParseError<A>>,
}

pub trait Original<Actual> {
    fn point_pos(&self, point: impl Into<CodePoint>) -> Option<usize>;
    fn actual(&self, point: impl Into<CodePoint>) -> Option<Actual>;
}

impl<A> ParseError<A> {
    pub fn expected(expected: Vec<StaticStr>, actual: A, location: ErrorLocation) -> Self {
        Self::ExpectedInstead {
            expected,
            actual,
            location,
            hints: vec![],
        }
    }
    
    pub fn unexpected_eof(expected: Vec<StaticStr>, location: ErrorLocation) -> Self {
        Self::ExpectedNotEOF {
            expected,
            location,
            hints: vec![],
        }
    }
    
    pub fn with_hints(mut self, hint: Cow<'static, str>) -> Self {
        let hints = match self {
            ParseError::ExpectedInstead { ref mut hints, .. } => hints,
            ParseError::ExpectedNotEOF { ref mut hints, .. } => hints,
        };
        hints.push(hint);
        self
    }
}

impl<A> IntoIterator for ParseErrors<A> {
    type Item = ParseError<A>;
    type IntoIter = std::vec::IntoIter<ParseError<A>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'t> Original<PackedToken> for PackedTokenStream<'t> {
    fn point_pos(&self, point: impl Into<CodePoint>) -> Option<usize> {
        let point = point.into();
        self.iter().position(|it| it.point == point)
    }

    fn actual(&self, point: impl Into<CodePoint>) -> Option<PackedToken> {
        let pos = self.point_pos(point)?;
        Some(self[pos].token)
    }
}

impl<'a, S: From<&'a str>> Original<S> for &'a str {
    fn point_pos(&self, point: impl Into<CodePoint>) -> Option<usize> {
        let point = point.into();
        Some(point.offset as usize)
    }

    fn actual(&self, point: impl Into<CodePoint>) -> Option<S> {
        let point = point.into();
        Some(S::from(self.get(point.as_range())?))
    }
}

impl<A> ParseError<A> {
    pub fn map<B>(self, f: impl FnOnce(A) -> B) -> ParseError<B> {
        match self {
            ParseError::ExpectedInstead {
                expected,
                actual,
                location,
                hints,
            } => ParseError::ExpectedInstead {
                expected,
                actual: f(actual),
                location,
                hints,
            },
            ParseError::ExpectedNotEOF {
                expected,
                hints,
                location,
            } => ParseError::ExpectedNotEOF {
                expected,
                hints,
                location,
            },
        }
    }
}

impl<A> ParseErrors<A> {
    pub fn map<B>(self, mut f: impl FnMut(A) -> B) -> ParseErrors<B> {
        ParseErrors::new(self.into_iter().map(move |it| it.map(&mut f)).collect())
    }
}
