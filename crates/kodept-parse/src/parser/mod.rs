use cfg_if::cfg_if;

use kodept_core::structure::rlt::RLT;

use crate::error::ParseErrors;
use crate::lexer::Token;
use crate::token_stream::TokenStream;

#[cfg(feature = "nom")]
pub(crate) mod block_level;
#[cfg(feature = "nom")]
pub(crate) mod code_flow;
#[cfg(feature = "nom")]
pub(crate) mod expression;
#[cfg(feature = "nom")]
pub mod file;
#[cfg(feature = "nom")]
pub(crate) mod function;
#[cfg(feature = "nom")]
pub(crate) mod literal;
#[cfg(feature = "nom")]
pub(crate) mod nom;
#[cfg(feature = "nom")]
pub(crate) mod operator;
#[cfg(feature = "nom")]
pub(crate) mod parameter;
#[cfg(feature = "nom")]
pub(crate) mod term;
#[cfg(feature = "nom")]
pub(crate) mod top_level;
#[cfg(feature = "nom")]
pub(crate) mod r#type;

pub(crate) mod common;

pub mod error {
    #[cfg(feature = "nom")]
    pub use crate::parser::nom::TokenVerificationError;
}

#[inline(always)]
#[allow(unreachable_code)]
pub fn parse_from_top(stream: TokenStream) -> Result<RLT, ParseErrors<Token>> {
    cfg_if! {
        if #[cfg(feature = "peg")] {
            crate::grammar::parser::kodept(&stream).map_err(|it| (it, stream).into())
        } else if #[cfg(feature = "nom")] {
            match nom_supreme::final_parser::final_parser(file::grammar)(stream) {
                Ok(x) => Ok(RLT(x)),
                Err(e) => Err((e, stream).into())
            }
        } else {
            compile_error!("Either feature `peg` or `nom` must be enabled for this crate")
        }
    }
}

pub(crate) mod macros {
    #[macro_export]
    macro_rules! function {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            name.strip_suffix("::f").unwrap()
        }};
    }
}
