use crate::common::ErrorAdapter;
use crate::error::{ErrorLocation, Original, ParseError, ParseErrors};
use crate::lexer::Token;
use crate::nom::TokenVerificationError;
use crate::token_stream::TokenStream;
use derive_more::Constructor;
use itertools::Itertools;
use kodept_core::code_point::CodePoint;
use nom_supreme::error::{BaseErrorKind, ErrorTree, Expectation};
use std::borrow::Cow;
use std::collections::VecDeque;

trait ExpectedError {
    fn expected(&self) -> Cow<'static, str>;
}

#[derive(Debug, Constructor)]
struct BaseError<'s, O, E> {
    location: O,
    kind: BaseErrorKind<&'s str, E>,
}

impl<'s, O, E> BaseError<'s, O, E>
where
    E: ExpectedError,
{
    fn into_expected(self) -> Cow<'static, str> {
        match self.kind {
            BaseErrorKind::Expected(Expectation::Something) => Cow::Borrowed("something"),
            BaseErrorKind::Expected(expectation) => Cow::Owned(expectation.to_string()),
            BaseErrorKind::Kind(kind) => Cow::Owned(kind.description().to_string()),
            BaseErrorKind::External(ext) => ext.expected(),
        }
    }
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

impl<A> ErrorAdapter<A, &str> for ErrorTree<&str>
where
    for<'a> &'a str: Original<A>,
    for<'a> A: From<&'a str>,
{
    fn adapt(self, _: &str, position: usize) -> ParseErrors<A> {
        let mut current_errors = VecDeque::from([self]);
        let mut base_errors = vec![];

        loop {
            match current_errors.pop_front() {
                None => break,
                Some(ErrorTree::Base { location, kind }) => {
                    base_errors.push(BaseError::new(location, kind))
                }
                Some(ErrorTree::Stack { base, .. }) => current_errors.push_back(*base),
                Some(ErrorTree::Alt(es)) => current_errors.extend(es),
            }
        }

        let parse_errors = base_errors
            .into_iter()
            .chunk_by(|it| it.location)
            .into_iter()
            .map(|(key, group)| {
                let actual = A::from(&key[0..=1]);
                let expected = group.map(|it| it.into_expected()).collect();
                let location = ErrorLocation::new(position, CodePoint::single_point(position as u32));

                ParseError::new(expected, actual, location)
            })
            .collect();
        ParseErrors::new(parse_errors)
    }
}

impl<'t> ErrorAdapter<Token<'t>, TokenStream<'t>> for super::parser::ParseError<'t> {
    fn adapt(self, _: TokenStream<'t>, position: usize) -> ParseErrors<Token<'t>> {
        let mut current_errors = VecDeque::from([self]);
        let mut base_errors = vec![];

        loop {
            match current_errors.pop_front() {
                None => break,
                Some(super::parser::ParseError::Base { location, kind }) => {
                    base_errors.push(BaseError::new(location, kind))
                }
                Some(super::parser::ParseError::Stack { base, .. }) => {
                    current_errors.push_back(*base)
                }
                Some(super::parser::ParseError::Alt(es)) => current_errors.extend(es),
            }
        }

        let parse_errors = base_errors
            .into_iter()
            .chunk_by(|it| it.location)
            .into_iter()
            .map(|(key, group)| {
                let actual = key.slice[0].token;
                let expected = group.map(|it| it.into_expected()).collect();
                let location = ErrorLocation::new(position, CodePoint::single_point(position as u32));

                ParseError::new(expected, actual, location)
            })
            .collect();
        ParseErrors::new(parse_errors)
    }
}
