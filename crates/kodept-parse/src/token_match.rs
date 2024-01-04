use derive_more::Constructor;
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use kodept_core::structure::span::Span;

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq, Constructor)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct TokenMatch<'t> {
    pub token: Token<'t>,
    pub span: Span,
}
