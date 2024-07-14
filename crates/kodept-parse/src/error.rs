use std::borrow::Cow;

use derive_more::{Constructor};

use crate::lexer::Token;
use crate::token_stream::TokenStream;
use kodept_core::code_point::CodePoint;

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
        point_pos(*self, point.into())
    }

    fn actual(&self, point: impl Into<CodePoint>) -> Token<'t> {
        let pos = self.point_pos(point);
        self.slice[pos].token
    }
}

impl<'s> Original<&'s str> for &'s str {
    fn point_pos(&self, point: impl Into<CodePoint>) -> usize {
        let point = point.into();
        point.offset
    }

    fn actual(&self, point: impl Into<CodePoint>) -> &'s str {
        let point = point.into();
        &self[point.offset..point.offset + point.length]
    }
}

impl Original<String> for &str {
    fn point_pos(&self, point: impl Into<CodePoint>) -> usize {
        let point = point.into();
        point.offset
    }

    fn actual(&self, point: impl Into<CodePoint>) -> String {
        let point = point.into();
        self[point.offset..point.offset + point.length].to_string()
    }
}

#[cfg(feature = "peg")]
impl<A, O: Original<A>, P: Into<crate::grammar::compatibility::Position> + Clone>
    From<(peg::error::ParseError<P>, O)> for ParseErrors<A>
{
    fn from((error, stream): (peg::error::ParseError<P>, O)) -> Self {
        let expected = error.expected.tokens().map(Cow::Borrowed).collect();
        let pos = stream.point_pos(error.location.clone().into());
        let actual = stream.actual(error.location.clone().into());
        let location = ErrorLocation::new(pos, CodePoint::from(error.location.into()));

        ParseErrors {
            errors: vec![ParseError::new(expected, actual, location)],
        }
    }
}

#[inline(always)]
fn point_pos(stream: TokenStream, point: CodePoint) -> usize {
    stream
        .slice
        .iter()
        .position(|it| it.span.point == point)
        .unwrap()
}

#[cfg(feature = "nom")]
pub mod default {
    use std::borrow::Cow;
    use std::collections::VecDeque;
    use derive_more::Constructor;
    use itertools::Itertools;
    use nom_supreme::error::{BaseErrorKind, Expectation};
    use kodept_core::code_point::CodePoint;
    use crate::error::{point_pos, ErrorLocation, ParseError, ParseErrors};
    use crate::lexer::Token;
    use crate::parser::nom::TokenVerificationError;
    use crate::token_stream::TokenStream;
    use crate::TokenizationError;

    fn take_first_token(stream: TokenStream) -> Token {
        stream
            .slice
            .iter()
            .next()
            .map_or(Token::Unknown, |it| it.token)
    }

    trait ExpectedError {
        fn expected(&self) -> Cow<'static, str>;
    }

    impl ExpectedError for TokenVerificationError {
        fn expected(&self) -> Cow<'static, str> {
            Cow::Borrowed(self.expected)
        }
    }

    impl<T: ?Sized + ToString> ExpectedError for Box<T> {
        fn expected(&self) -> Cow<'static, str> {
            Cow::Owned(self.to_string())
        }
    }

    #[derive(Debug, Constructor)]
    struct BaseError<'s, O, E> {
        location: O,
        kind: BaseErrorKind<&'s str, E>
    }

    impl<'s, O, E> BaseError<'s, O, E>
    where
        E: ExpectedError
    {
        pub fn into_expected(self) -> Cow<'static, str> {
            match self.kind {
                BaseErrorKind::Expected(Expectation::Something) => Cow::Borrowed("something"),
                BaseErrorKind::Expected(expectation) => Cow::Owned(expectation.to_string()),
                BaseErrorKind::Kind(kind) => Cow::Owned(kind.description().to_string()),
                BaseErrorKind::External(ext) => ext.expected(),
            }
        }
    }

    impl<'t, 's> From<(crate::ParseError<'t>, TokenStream<'s>)> for ParseErrors<Token<'t>> {
        fn from((error, stream): (crate::ParseError<'t>, TokenStream<'s>)) -> Self {
            let mut current_errors = VecDeque::from([error]);
            let mut base_errors = vec![];
            loop {
                match current_errors.pop_front() {
                    Some(crate::ParseError::Base { location, kind }) => {
                        base_errors.push(BaseError::new(location, kind))
                    }
                    Some(crate::ParseError::Stack { base, .. }) => current_errors.push_back(*base),
                    Some(crate::ParseError::Alt(vec)) => current_errors.extend(vec),
                    None => break,
                }
            }

            let parse_errors = base_errors
                .into_iter()
                .chunk_by(|it| it.location)
                .into_iter()
                .map(|(key, group)| {
                    let point = key.slice[0].span.point;
                    let pos = point_pos(stream, point);
                    let error_loc = ErrorLocation::new(pos, point);
                    let actual = take_first_token(key);
                    let expected = group.map(|it| it.into_expected()).collect();
                    ParseError::new(expected, actual, error_loc)
                })
                .collect();
            ParseErrors {
                errors: parse_errors,
            }
        }
    }

    impl<'t, 's> From<(TokenizationError<'t>, &'s str)> for ParseErrors<&'t str> {
        fn from((error, stream): (TokenizationError<'t>, &'s str)) -> Self {
            let mut current_errors = VecDeque::from([error]);
            let mut base_errors = vec![];
            loop {
                match current_errors.pop_front() {
                    None => break,
                    Some(TokenizationError::Base { location, kind }) => base_errors.push(BaseError::new(location, kind)),
                    Some(TokenizationError::Stack { base, .. }) => current_errors.push_back(*base),
                    Some(TokenizationError::Alt(vec)) => current_errors.extend(vec)
                }
            }

            let parse_errors = base_errors
                .into_iter()
                .chunk_by(|it| it.location)
                .into_iter()
                .map(|(key, group)| {
                    let pos = stream.find(key).unwrap_or_default();
                    let error_loc = ErrorLocation::new(pos, CodePoint::single_point(pos));
                    let actual = &key[0..=1];
                    let expected = group.map(|it| it.into_expected()).collect();
                    ParseError::new(expected, actual, error_loc)
                }).collect();
            ParseErrors { errors: parse_errors }
        }
    }
}
