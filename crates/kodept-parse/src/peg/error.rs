use std::borrow::Cow;

use kodept_core::code_point::CodePoint;

use crate::common::ErrorAdapter;
use crate::error::{ErrorLocation, Original, ParseError, ParseErrors};
use crate::peg::compatibility::Position;

impl<A, O, P> ErrorAdapter<A, O> for peg::error::ParseError<P>
where
    O: Original<A>,
    P: Into<Position>
{
    fn adapt(self, original_input: O, position: usize) -> ParseErrors<A> {
        let expected = self.expected.tokens().map(Cow::Borrowed).collect();
        let loc: CodePoint = P::into(self.location).into();
        let actual = original_input.actual(loc);
        let location = ErrorLocation::new(position, loc);

        ParseErrors::new(vec![ParseError::new(expected, actual, location)])
    }
}
