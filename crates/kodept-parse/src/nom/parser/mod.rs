use derive_more::Constructor;
use kodept_core::structure::rlt::RLT;
use ::nom::IResult;
use nom_supreme::error::GenericErrorTree;
use nom_supreme::final_parser::final_parser;

use crate::common::RLTProducer;
use crate::nom::TokenVerificationError;
use crate::token_stream::PackedTokenStream;

pub(in crate::nom) type ParseError<'t> =
    GenericErrorTree<PackedTokenStream<'t>, &'static str, &'static str, TokenVerificationError>;
type ParseResult<'t, O> = IResult<PackedTokenStream<'t>, O, ParseError<'t>>;

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

#[derive(Constructor, Debug)]
pub struct Parser;

impl RLTProducer for Parser {
    type Error<'t> = ParseError<'t>;

    fn parse_stream<'t>(&self, input: &PackedTokenStream<'t>) -> Result<RLT, Self::Error<'t>> {
        let file = final_parser(file::grammar)(*input)?;
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
                nom::combinator::verify(
                    $crate::nom::parser::utils::any_not_ignored_token,
                    move |t| matches!(&t.token, $pat),
                ),
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

    pub(crate) use {function, match_any_token, match_token};
}
