use crate::common::{ErrorAdapter, RLTProducer};
use crate::error::{Original, ParseErrors};
use crate::token_stream::{PackedTokenStream};

pub type NomParser = crate::nom::Parser;
pub type PegParser<const TRACE: bool> = crate::peg::Parser<TRACE>;

pub fn parse_from_top<'t, A, E, P, O>(input: PackedTokenStream<'t>, parser: P) -> Result<O, ParseErrors<A>>
where
    P: RLTProducer<O, Error<'t> = E> + 't,
    E: ErrorAdapter<A, PackedTokenStream<'t>>,
    O: 't,
    PackedTokenStream<'t>: Original<A>,
{
    match parser.parse_stream(&input) {
        Ok(x) => Ok(x),
        Err(e) => Err(e.adapt(input, 0)),
    }
}
