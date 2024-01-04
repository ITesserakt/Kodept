use extend::ext;
use nom::IResult;
use nom_supreme::error::{ErrorTree, GenericErrorTree};

use crate::parser::nom::TokenVerificationError;
use crate::token_stream::TokenStream;

pub mod lexer;
pub mod parser;
pub mod token_match;
pub mod token_stream;
pub mod tokenizer;

pub type TokenizationError<'t> = ErrorTree<&'t str>;
pub type ParseError<'t> =
    GenericErrorTree<TokenStream<'t>, &'static str, &'static str, TokenVerificationError>;
pub type TokenizationResult<'t, O> = IResult<&'t str, O, TokenizationError<'t>>;
pub type ParseResult<'t, O> = IResult<TokenStream<'t>, O, ParseError<'t>>;

type Span<'t> = &'t str;

#[ext]
impl<T> Option<T> {
    #[inline]
    fn map_into<U: From<T>>(self) -> Option<U> {
        self.map(|x| x.into())
    }
}
