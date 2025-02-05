use derive_more::{Constructor, Display, Error};
use nom::error::{ContextError, ErrorKind, ParseError};

mod compatibility;
mod error;
mod lexer;
mod parser;

pub(crate) use lexer::Lexer;
pub(crate) use parser::Parser;

#[derive(Debug)]
pub struct VerboseError<T> {
    errors: Vec<(T, VerboseErrorKind)>
}

#[derive(Debug)]
enum VerboseErrorKind {
    Context(&'static str),
    Char(char),
    Nom(ErrorKind)
}

impl<T> ParseError<T> for VerboseError<T> {
    fn from_error_kind(input: T, kind: ErrorKind) -> Self {
        Self {
            errors: vec![(input, VerboseErrorKind::Nom(kind))],
        }
    }

    fn append(input: T, kind: ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, VerboseErrorKind::Nom(kind)));
        other
    }

    fn from_char(input: T, value: char) -> Self {
        Self { errors: vec![(input, VerboseErrorKind::Char(value))] }
    }

    fn or(mut self, other: Self) -> Self {
        self.errors.extend(other.errors);
        self
    }
}

impl<T> ContextError<T> for VerboseError<T> {
    fn add_context(input: T, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, VerboseErrorKind::Context(ctx)));
        other
    }
}

type TError<'t> = VerboseError<&'t str>;

#[derive(Error, Debug, Constructor, Display)]
#[display("Expected `{expected}`")]
pub struct TokenVerificationError {
    pub expected: &'static str,
}

trait TParser<'t, O = &'t str>:
    nom::Parser<&'t str, Output = O, Error = TError<'t>>
{
}
impl<'t, O, P: nom::Parser<&'t str, Output = O, Error = TError<'t>>> TParser<'t, O>
    for P
{
}
