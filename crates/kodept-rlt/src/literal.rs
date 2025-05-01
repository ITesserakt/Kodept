use crate::new_types::Enclosed;
use crate::prelude::Operation;
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Literal {
    Binary(CodePoint),
    Octal(CodePoint),
    Hex(CodePoint),
    Floating(CodePoint),
    Char(CodePoint),
    String(CodePoint),
    Tuple(Enclosed<Box<[Operation]>>),
}

impl Located for Literal {
    fn location(&self) -> CodePoint {
        match self {
            Literal::Binary(x) => x.location(),
            Literal::Octal(x) => x.location(),
            Literal::Hex(x) => x.location(),
            Literal::Floating(x) => x.location(),
            Literal::Char(x) => x.location(),
            Literal::String(x) => x.location(),
            Literal::Tuple(x) => x.left.location(),
        }
    }
}

impl SpanBounds for Literal {
    fn bounds(&self) -> Span {
        match self {
            Literal::Binary(x) => Span::from(*x),
            Literal::Octal(x) => Span::from(*x),
            Literal::Hex(x) => Span::from(*x),
            Literal::Floating(x) => Span::from(*x),
            Literal::Char(x) => Span::from(*x),
            Literal::String(x) => Span::from(*x),
            Literal::Tuple(x) => x.left.0 + x.right.0,
        }
    }
}
