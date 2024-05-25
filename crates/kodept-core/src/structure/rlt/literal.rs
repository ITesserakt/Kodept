use crate::code_point::CodePoint;
use crate::structure::Located;
use crate::structure::rlt::new_types::Enclosed;
use crate::structure::rlt::Operation;
use crate::structure::span::Span;

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
