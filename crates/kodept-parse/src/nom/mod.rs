use derive_more::{Constructor, Display, Error};
use nom::IResult;
use nom_supreme::error::ErrorTree;

mod error;
mod lexer;
mod parser;

pub(crate) use lexer::Lexer;
pub(crate) use parser::Parser;

type TokenizationError<'t> = ErrorTree<&'t str>;
type TResult<'t, O> = IResult<&'t str, O, TokenizationError<'t>>;

#[derive(Error, Debug, Constructor, Display)]
#[display("Expected `{expected}`")]
pub struct TokenVerificationError {
    pub expected: &'static str,
}
