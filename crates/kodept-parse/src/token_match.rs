use crate::lexer::Token;
use derive_more::Constructor;
use kodept_core::structure::span::Span;

#[cfg(feature = "size-of")]
use size_of::SizeOf;

#[derive(Debug, Clone, PartialEq, Constructor)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct TokenMatch<'t> {
    pub token: Token<'t>,
    pub span: Span,
}
