#[cfg(feature = "pest")]
mod pest;

#[cfg(feature = "peg")]
pub(crate) mod peg;

#[cfg(feature = "peg")]
mod compatibility;
#[cfg(feature = "peg")]
pub(crate) mod parser;

#[cfg(feature = "peg")]
pub type KodeptParser<'t> = peg::Tokenizer<'t, false>;

#[cfg(all(feature = "pest", not(feature = "peg")))]
pub type KodeptParser<'t> = pest::Tokenizer<'t>;

#[cfg(all(feature = "pest", feature = "peg"))]
pub type PestKodeptParser<'t> = pest::Tokenizer<'t>;

#[cfg(feature = "peg")]
mod macros {
    macro_rules! tok {
        ($pat:pat) => {$crate::token_match::TokenMatch { token: $pat, .. }};
    }
    
    pub(crate) use tok;
}
