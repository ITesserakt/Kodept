use extend::ext;
#[cfg(feature = "nom")]
use nom::IResult;
#[cfg(feature = "nom")]
use nom_supreme::error::{ErrorTree, GenericErrorTree};

pub use lexer::parse_token;
pub use parser::parse_from_top;

#[cfg(feature = "nom")]
use crate::parser::nom::TokenVerificationError;

pub mod lexer;
pub mod parser;
pub mod token_match;
pub mod token_stream;
pub mod tokenizer;

pub mod error;
pub mod grammar;

#[cfg(feature = "nom")]
pub type TokenizationError<'t> = ErrorTree<&'t str>;
#[cfg(feature = "nom")]
pub type ParseError<'t> = GenericErrorTree<
    token_stream::TokenStream<'t>,
    &'static str,
    &'static str,
    TokenVerificationError,
>;
#[cfg(feature = "nom")]
pub type TokenizationResult<'t, O> = IResult<&'t str, O, TokenizationError<'t>>;
#[cfg(feature = "nom")]
pub type ParseResult<'t, O> = IResult<token_stream::TokenStream<'t>, O, ParseError<'t>>;

type Span<'t> = &'t str;

#[ext]
impl<T> Option<T> {
    #[inline]
    fn map_into<U: From<T>>(self) -> Option<U> {
        self.map(|x| x.into())
    }
}

