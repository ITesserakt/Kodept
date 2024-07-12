use extend::ext;
#[cfg(feature = "nom")]
use nom::IResult;
#[cfg(feature = "nom")]
use nom_supreme::error::{ErrorTree, GenericErrorTree};

#[cfg(feature = "nom")]
use crate::parser::nom::TokenVerificationError;
#[cfg(feature = "nom")]
use crate::token_stream::TokenStream;

pub mod lexer;
pub mod parser;
pub mod token_match;
pub mod token_stream;
pub mod tokenizer;

pub mod grammar;
pub mod error;

#[cfg(feature = "nom")]
pub type TokenizationError<'t> = ErrorTree<&'t str>;
#[cfg(feature = "nom")]
pub type ParseError<'t> =
    GenericErrorTree<TokenStream<'t>, &'static str, &'static str, TokenVerificationError>;
#[cfg(feature = "nom")]
pub type TokenizationResult<'t, O> = IResult<&'t str, O, TokenizationError<'t>>;
#[cfg(feature = "nom")]
pub type ParseResult<'t, O> = IResult<TokenStream<'t>, O, ParseError<'t>>;

type Span<'t> = &'t str;

#[ext]
impl<T> Option<T> {
    #[inline]
    fn map_into<U: From<T>>(self) -> Option<U> {
        self.map(|x| x.into())
    }
}
