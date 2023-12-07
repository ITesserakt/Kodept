use crate::code_point::CodePoint;
use crate::structure::rlt::new_types::Enclosed;
use crate::structure::span::Span;
use crate::structure::Located;
#[cfg(feature = "size-of")]
use size_of::SizeOf;

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Binary(Span),
    Octal(Span),
    Hex(Span),
    Floating(Span),
    Char(Span),
    String(Span),
    Tuple(Enclosed<Box<[Literal]>>),
}

#[cfg(feature = "size-of")]
impl SizeOf for Literal {
    fn size_of_children(&self, context: &mut size_of::Context) {
        match self {
            Literal::Binary(x) => x.size_of_children(context),
            Literal::Octal(x) => x.size_of_children(context),
            Literal::Hex(x) => x.size_of_children(context),
            Literal::Floating(x) => x.size_of_children(context),
            Literal::Char(x) => x.size_of_children(context),
            Literal::String(x) => x.size_of_children(context),
            Literal::Tuple(x) => x.size_of_children(context),
        }
    }
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
