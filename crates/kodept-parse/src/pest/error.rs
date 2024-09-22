use std::borrow::Cow;

use kodept_core::code_point::CodePoint;
use pest::error::InputLocation;
use pest::RuleType;

use crate::common::ErrorAdapter;
use crate::error::{ErrorLocation, Original, ParseError, ParseErrors};

impl<A, O, E> ErrorAdapter<A, O> for pest::error::Error<E>
where
    O: Original<A>,
    E: RuleType,
{
    fn adapt(self, original_input: O, position: usize) -> ParseErrors<A> {
        let point = CodePoint::single_point(position as u32);
        let actual = original_input.actual(point);
        let location = match self.location {
            InputLocation::Pos(pos) => ErrorLocation::new(pos, CodePoint::single_point(pos as u32)),
            InputLocation::Span((start, end)) => {
                ErrorLocation::new(start, CodePoint::new((end - start) as u32, start as u32))
            }
        };
        let expected = Cow::Owned(self.variant.message().to_string());
        ParseErrors::new(vec![match actual {
            None => ParseError::unexpected_eof(vec![expected], location),
            Some(actual) => ParseError::expected(vec![expected], actual, location),
        }])
    }
}
