use cfg_if::cfg_if;

use kodept_core::structure::rlt::RLT;

use crate::common::{ErrorAdapter, RLTProducer};
use crate::error::{Original, ParseErrors};
use crate::lexer::Token;
use crate::token_stream::TokenStream;

cfg_if! {
    if #[cfg(feature = "peg")] {
        pub type DefaultParser = PegParser<false>;
    } else if #[cfg(feature = "nom")] {
        pub type DefaultParser = NomParser;
    } else {
        compile_error!("Either feature `peg` or `nom` must be enabled for this crate");
    }
}

#[cfg(feature = "nom")]
pub type NomParser = crate::nom::Parser;
#[cfg(feature = "peg")]
pub type PegParser<const TRACE: bool> = crate::peg::Parser<TRACE>;

pub fn parse_from_top<'t, A, E, P>(input: TokenStream<'t>, parser: P) -> Result<RLT, ParseErrors<A>>
where
    P: RLTProducer<Error<'t> = E>,
    E: ErrorAdapter<A, TokenStream<'t>>,
    TokenStream<'t>: Original<A>,
{
    match parser.parse_rlt(input) {
        Ok(x) => Ok(x),
        Err(e) => Err(e.adapt(input, 0)),
    }
}

pub fn default_parse_from_top(input: TokenStream) -> Result<RLT, ParseErrors<Token>>
{
    parse_from_top(input, DefaultParser::new())
}
