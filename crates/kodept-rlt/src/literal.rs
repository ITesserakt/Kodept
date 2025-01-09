use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use kodept_core::structure::span::Span;
use crate::new_types::Enclosed;
use crate::Operation;

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Binary(Span),
    Octal(Span),
    Hex(Span),
    Floating(Span),
    Char(Span),
    String(Span),
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
