mod lexer;
mod error;
mod compatibility;
mod parser;

pub(crate) use lexer::Lexer;
pub(crate) use parser::Parser;

mod macros {
    macro_rules! tok {
        ($pat:pat) => {$crate::token_match::TokenMatch { token: $pat, .. }};
    }

    pub(crate) use tok;
}