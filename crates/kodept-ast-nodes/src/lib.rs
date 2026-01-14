//! This crate contains actual AST nodes used in Kodept with appropriate
//! conversion implementation from RLT nodes.

use kodept_ast::experimental::SplitRef;
use kodept_rlt::exported::CodePoint;
use std::convert::Infallible;
use std::num::{ParseFloatError, ParseIntError};

pub mod block_level;
pub mod code_flow;
pub mod consts;
pub mod expression;
pub mod file;
pub mod function;
pub mod literal;
pub mod properties;
pub mod term;
pub mod top_level;
pub mod types;

#[derive(Debug)]
pub enum Error {
    NoQuotesInLiteral(CodePoint),
    WrongLiteralLength(CodePoint, usize),
    CannotParseFloat(CodePoint, ParseFloatError),
    CannotParseInt(CodePoint, ParseIntError),
}

#[cfg(feature = "reflection")]
pub fn register_reflection_info(registry: &mut kodept_ast::resource::reflection::DebugRegistry) {
    registry.register::<file::FileDecl>();
    registry.register::<file::ModDecl>();
    registry.register::<term::Ref>();
    registry.register::<term::ReferenceContext>();
    registry.register::<types::Ty>();
    registry.register::<types::NonTyParam>();
    registry.register::<types::TyParam>();
    registry.register::<types::ProdTy>();
    registry.register::<literal::Literal>();
    registry.register::<literal::Tuple>();
    registry.register::<expression::BinExpr>();
    registry.register::<expression::Exprs>();
    registry.register::<expression::Lambda>();
    registry.register::<expression::App>();
    registry.register::<block_level::VarDecl>();
    registry.register::<block_level::InitVar>();
    registry.register::<code_flow::IfExpr>();
    registry.register::<code_flow::ElseExpr>();
    registry.register::<code_flow::ElifExpr>();
    registry.register::<consts::Const>();
    registry.register::<function::FuncDecl>();
    registry.register::<top_level::EnumConst>();
    registry.register::<top_level::EnumDecl>();
    registry.register::<top_level::StructDecl>();
}

impl From<Infallible> for Error {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug)]
struct Dispatcher<'a, T>(pub &'a T);

impl<'a, T> From<&'a T> for Dispatcher<'a, T> {
    fn from(value: &'a T) -> Self {
        Self(value)
    }
}

impl<'a, T> SplitRef<'a, T> for Dispatcher<'a, T> {
    fn split(self) -> (Self, &'a T) {
        (Self(self.0), self.0)
    }
}
