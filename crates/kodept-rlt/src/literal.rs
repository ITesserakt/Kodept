use crate::new_types::Enclosed;
use crate::prelude::Operation;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;

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
