use extend::ext;
#[cfg(feature = "nom")]
use nom::IResult;
#[cfg(feature = "nom")]
use nom_supreme::error::{ErrorTree, GenericErrorTree};
use kodept_core::structure::rlt::RLT;
use crate::error::ParseErrors;
use crate::lexer::Token;
#[cfg(feature = "nom")]
use crate::parser::nom::TokenVerificationError;
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

#[inline(always)]
#[allow(unreachable_code)]
pub fn parse_from_top(stream: TokenStream) -> Result<RLT, ParseErrors<Token>> {
    #[cfg(feature = "peg")]
    return grammar::parser::kodept(&stream).map_err(|it| (it, stream).into());
    #[cfg(feature = "nom")]
    return match nom_supreme::final_parser::final_parser(parser::file::grammar)(stream) {
        Ok(x) => Ok(RLT(x)),
        Err(e) => {
            let e: ParseError = e;
            Err(ParseErrors::from((e, stream)))
        }
    };
    #[cfg(not(any(feature = "nom", feature = "peg")))]
    compile_error!("Either feature `peg` or `nom` must be enabled for this crate")
}
