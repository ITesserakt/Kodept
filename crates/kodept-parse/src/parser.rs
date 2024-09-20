use crate::common::{ErrorAdapter, RLTProducer};
use crate::error::{Original, ParseErrors};
use crate::token_stream::TokenStream;

pub type NomParser = crate::nom::Parser;
pub type PegParser<const TRACE: bool> = crate::peg::Parser<TRACE>;

#[cfg(feature = "lalrpop")]
pub type LaLRPop = crate::lalrpop::Parser;

pub fn parse_from_top<'t, A, E, P, O>(input: TokenStream<'t>, parser: P) -> Result<O, ParseErrors<A>>
where
    P: RLTProducer<O, Error<'t> = E>,
    E: ErrorAdapter<A, TokenStream<'t>>,
    TokenStream<'t>: Original<A>,
{
    match parser.parse_stream(input) {
        Ok(x) => Ok(x),
        Err(e) => Err(e.adapt(input, 0)),
    }
}
