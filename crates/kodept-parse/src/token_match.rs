use derive_more::Constructor;

use kodept_core::structure::span::Span;

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct TokenMatch<'t> {
    pub token: Token<'t>,
    pub span: Span,
}
