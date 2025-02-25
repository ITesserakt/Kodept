use derive_more::Constructor;
use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError;
use crate::common::{EagerTokensProducer, TokenProducer};
use crate::token_match::TokenMatch;

#[derive(Constructor, Copy, Clone)]
pub struct Lexer;

impl TokenProducer for Lexer {
    type Error<'t> = ParseError<usize, Token<'t>, &'static str>;

    fn parse_string<'t>(&self, whole_input: &'t str, position: usize) -> Result<TokenMatch<'t>, Self::Error<'t>> {
        let input = &whole_input[position..];
        let parser = super::lexer_implementation::TokenParser::new();
        parser.parse(input)
    }
}

impl EagerTokensProducer for Lexer {
    type Error<'t> = ParseError<usize, Token<'t>, &'static str>;

    fn parse_string<'t>(&self, input: &'t str) -> Result<Vec<TokenMatch<'t>>, Self::Error<'t>> {
        let parser = super::lexer_implementation::TokensParser::new();
        parser.parse(input)
    }
}
