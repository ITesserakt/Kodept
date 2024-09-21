#![feature(impl_trait_in_assoc_type)]

pub mod lexer;
pub mod parser;

pub mod token_match;
pub mod token_stream;
pub mod tokenizer;

pub mod error;

mod peg;
mod pest;
mod nom;

pub mod common;

pub const TRACING_OPTION: bool = cfg!(feature = "trace");