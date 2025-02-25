use crate::common::ErrorAdapter;
use crate::error::{ErrorLocation, Original, ParseError, ParseErrors};
use itertools::Itertools;
use kodept_core::code_point::CodePoint;
use lalrpop_util::ParseError::*;
use std::fmt::Display;

fn to_point(start: usize, end: usize) -> CodePoint {
    CodePoint::new((end - start) as u32, start as u32)
}

impl<'t, A, O, E, T> ErrorAdapter<A, O> for lalrpop_util::ParseError<usize, T, E>
where
    O: Original<A>,
    T: Display
{
    fn adapt(self, original_input: O, position: usize) -> ParseErrors<A> {
        let error = match self {
            InvalidToken { location } => {
                let point = CodePoint::single_point(location as u32);
                let actual = original_input.actual(point);
                let location = ErrorLocation::new(position, point);
                ParseError::new(vec![], actual, location)
            }
            UnrecognizedEof { location, expected } => {
                let point = CodePoint::single_point(location as u32);
                let actual = original_input.actual(point);
                let location = ErrorLocation::new(position, point);
                ParseError::new(expected.into_iter().map_into().collect(), actual, location)
            }
            UnrecognizedToken {
                token: (start, token, end),
                expected,
            } => {
                let point = to_point(start, end);
                let actual = original_input.actual(point);
                let location = ErrorLocation::new(position, point);
                ParseError::new(expected.into_iter().map_into().collect(), actual, location)
            }
            ExtraToken {
                token: (start, token, end),
            } => {
                let point = to_point(start, end);
                let actual = original_input.actual(point);
                let location = ErrorLocation::new(position, point);
                let expected = format!("not {token}");
                ParseError::new(vec![expected.into()], actual, location)
            }
            User { .. } => unreachable!(),
        };
        ParseErrors::new(vec![error])
    }
}
