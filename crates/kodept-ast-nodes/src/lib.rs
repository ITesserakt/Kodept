//! This crate contains actual AST nodes used in Kodept with appropriate
//! conversion implementation from RLT nodes.

extern crate core;

use bigdecimal::ParseBigDecimalError;
use kodept_rlt::exported::CodePoint;
use num_bigint::ParseBigIntError;
use std::convert::Infallible;

mod v3;
pub use v3::*;

#[derive(Debug)]
pub enum Error {
    NoQuotesInLiteral(CodePoint),
    WrongLiteralLength(CodePoint, usize),
    CannotParseFloat(CodePoint, ParseBigDecimalError),
    CannotParseInt(CodePoint, ParseBigIntError),
    Unsupported(kodept_rlt::exported::Span),
    UnexpectedStatement(kodept_rlt::exported::Span),
    UnexpectedExpression(kodept_rlt::exported::Span),
    UnicodeLiteral(CodePoint),
}

#[cfg(feature = "reflection")]
pub fn register_reflection_info(registry: &mut kodept_ast::resource::reflection::DebugRegistry) {
    v3::register_reflection_info(registry);
}

impl From<Infallible> for Error {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}
