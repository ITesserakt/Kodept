#[cfg(feature = "pest")]
mod pest;

#[cfg(feature = "peg")]
mod peg;

#[cfg(all(feature = "pest", feature = "peg"))]
pub type KodeptParser<'t> = peg::Tokenizer<'t>;

#[cfg(all(feature = "pest", feature = "peg"))]
pub type PestKodeptParser<'t> = pest::Tokenizer<'t>;

#[cfg(all(feature = "peg", not(feature = "pest")))]
pub type KodeptParser<'t> = peg::Tokenizer<'t>;

#[cfg(all(feature = "pest", not(feature = "peg")))]
pub type KodeptParser<'t> = pest::Tokenizer<'t>;