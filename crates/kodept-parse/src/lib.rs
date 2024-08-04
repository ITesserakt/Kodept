pub mod lexer;
pub mod parser;

pub mod token_match;
pub mod token_stream;
pub mod tokenizer;

pub mod error;

#[cfg(feature = "peg")]
mod peg;
#[cfg(feature = "pest")]
mod pest;
#[cfg(feature = "nom")]
mod nom;

pub mod common;

pub(crate) const TRACING_OPTION: bool = if cfg!(feature = "trace") { true } else { false };