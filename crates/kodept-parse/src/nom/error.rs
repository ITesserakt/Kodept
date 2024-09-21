use crate::common::ErrorAdapter;
use crate::error::{ErrorLocation, Original, ParseError, ParseErrors};
use crate::lexer::PackedToken;
use crate::nom::TokenVerificationError;
use crate::token_stream::PackedTokenStream;
use derive_more::Constructor;
use itertools::Itertools;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
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

impl<'a, A> ErrorAdapter<A, &'a str> for ErrorTree<&'a str>
where
    &'a str: Original<A>,
    A: From<&'a str>,
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
                let location =
                    ErrorLocation::new(position, CodePoint::single_point(position as u32));

                ParseError::new(expected, actual, location)
            })
            .collect();
        ParseErrors::new(parse_errors)
    }
}

impl<'t> ErrorAdapter<PackedToken, PackedTokenStream<'t>> for super::parser::ParseError<'t> {
    fn adapt(self, _: PackedTokenStream<'t>, position: usize) -> ParseErrors<PackedToken> {
        use super::parser::ParseError as NomParseError;

        let mut current_errors = VecDeque::from([self]);
        let mut base_errors = vec![];

        loop {
            match current_errors.pop_front() {
                None => break,
                Some(NomParseError::Base { location, kind }) => {
                    base_errors.push(BaseError::new(location, kind))
                }
                Some(NomParseError::Stack { base, contexts }) => current_errors.push_back(*base),
                Some(NomParseError::Alt(es)) => current_errors.extend(es),
            }
        }

        let parse_errors = base_errors
            .into_iter()
            .chunk_by(|it| it.location)
            .into_iter()
            .map(|(key, group)| {
                let actual = key[0];
                let expected = group.map(|it| it.into_expected()).collect();
                let location = ErrorLocation::new(position, key.location());

                ParseError::new(expected, actual.token, location)
            })
            .collect();
        ParseErrors::new(parse_errors)
    }
}
