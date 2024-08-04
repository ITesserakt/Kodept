use ::nom::IResult;
use derive_more::Constructor;
use nom_supreme::error::GenericErrorTree;
use nom_supreme::final_parser::final_parser;
use kodept_core::structure::rlt::RLT;

use crate::common::RLTProducer;
use crate::nom::TokenVerificationError;
use crate::token_stream::TokenStream;

type ParseError<'t> =
    GenericErrorTree<TokenStream<'t>, &'static str, &'static str, TokenVerificationError>;
type ParseResult<'t, O> = IResult<TokenStream<'t>, O, ParseError<'t>>;

mod block_level;
mod code_flow;
mod expression;
mod file;
mod function;
mod literal;
mod operator;
mod parameter;
mod term;
mod top_level;
mod r#type;
mod utils;

#[derive(Constructor)]
pub struct Parser;

impl RLTProducer for Parser {
    type Error<'t> = ParseError<'t>;

    fn parse_rlt<'t>(&self, input: TokenStream<'t>) -> Result<RLT, Self::Error<'t>> {
        let file = final_parser(file::grammar)(input)?;
        Ok(RLT(file))
    }
}

mod macros {
    // TODO: Make it const as early as possible
    macro_rules! function {
        () => {{
            const fn f() {}
            let name = std::any::type_name_of_val(&f);
            &name[..name.len() - 3]
        }};
    }

    macro_rules! match_token {
        ($pat:pat_param) => {{
            nom::error::context(
                stringify!($pat),
                nom::combinator::verify($crate::nom::parser::utils::any_not_ignored_token, move |t| {
                    matches!(&t.token, $pat)
                }),
            )
        }};
    }

    macro_rules! match_any_token {
        ($pat:pat_param) => {{
            nom::error::context(
                stringify!($pat),
                nom::combinator::verify($crate::nom::parser::utils::any_token, move |t| {
                    matches!(&t.token, $pat)
                }),
            )
        }};
    }

    #[cfg(test)]
    macro_rules! assert_parses_to {
        ($parser:ident, $input:expr, $expectation:pat_param) => {
            match $parser($input) {
                Err(::nom::Err::Error(e) | ::nom::Err::Failure(e)) => {
                    panic!("{}", ::nom::error::convert_error($input, e));
                }
                Err(e) => {
                    panic!("Failed to parse {:?}", e)
                }
                Ok((_, candidate_val)) => {
                    if !matches!(&candidate_val, $expectation) {
                        panic!(
                            "Failed to parse to expected value\n\
                        Got:      {:?}",
                            &candidate_val
                        )
                    }
                }
            }
        };
    }
    
    pub(crate) use {function, match_any_token, match_token};
    #[cfg(test)]
    pub(crate) use assert_parses_to;
}
