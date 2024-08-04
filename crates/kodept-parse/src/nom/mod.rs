use derive_more::Constructor;
use nom::IResult;
use nom_supreme::error::ErrorTree;
use thiserror::Error;

mod lexer;
mod error;
mod parser;

pub use lexer::Lexer;
pub use parser::Parser;

type TokenizationError<'t> = ErrorTree<&'t str>;
type TResult<'t, O> = IResult<&'t str, O, TokenizationError<'t>>;

#[derive(Error, Debug, Constructor)]
#[error("Expected `{expected}`")]
pub struct TokenVerificationError {
    pub expected: &'static str,
}