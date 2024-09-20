use crate::common::RLTProducer;
use crate::lalrpop::compatibility::CompatIter;
use crate::lexer::Token;
use crate::token_stream::TokenStream;
use derive_more::Constructor;
use kodept_core::structure::rlt::RLT;
use lalrpop_util::ParseError;
use std::convert::Infallible;

#[derive(Constructor)]
pub struct Parser;

impl RLTProducer for Parser {
    type Error<'t> = ParseError<usize, Token<'t>, Infallible>;

    fn parse_stream<'t>(&self, input: TokenStream<'t>) -> Result<RLT, Self::Error<'t>> {
        super::kodept::ProgramParser::new().parse(CompatIter::new(input))
    }
}
