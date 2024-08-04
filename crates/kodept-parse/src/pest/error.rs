use std::borrow::Cow;

use pest::error::InputLocation;
use pest::RuleType;
use kodept_core::code_point::CodePoint;

use crate::common::ErrorAdapter;
use crate::error::{ErrorLocation, Original, ParseError, ParseErrors};

impl<A, O, E> ErrorAdapter<A, O> for pest::error::Error<E>
where
    O: Original<A>,
    E: RuleType
{
    fn adapt(self, original_input: O, position: usize) -> ParseErrors<A> {
        let point = CodePoint::single_point(position);
        let actual = original_input.actual(point);
        let location = match self.location {
            InputLocation::Pos(pos) => ErrorLocation::new(pos, CodePoint::single_point(pos)),
            InputLocation::Span((start, end)) => {
                ErrorLocation::new(start, CodePoint::new(end - start, start))
            }
        };
        let expected = Cow::Owned(self.variant.message().to_string());
        ParseErrors::new(vec![ParseError::new(vec![expected], actual, location)])
    }
}
