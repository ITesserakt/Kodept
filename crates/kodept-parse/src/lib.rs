//! This crate contains a tokenizer along with parser for Kodept.
//! Specifically, there are 3 different implementations for lexing:
//! - based on nom,
//! - based on peg,
//! - based on pest.
//!
//! And 2 different implementations for parsing:
//! - based on nom,
//! - based on peg.
//!
//! It also provides some common structures like Token, TokenStream,
//! leveraging an abstraction around error handling etc.

pub mod lexer;
pub mod parser;

pub mod token_match;
pub mod token_stream;
pub mod tokenizer;

pub mod error;

#[cfg(feature = "nom")]
mod nom;
mod peg;

pub mod common;

pub const TRACING_OPTION: bool = cfg!(feature = "trace");
