use crate::structure::span::Span;
use derive_more::{From, Into};

use crate::code_point::CodePoint;
use crate::structure::Located;
#[cfg(feature = "size-of")]
use size_of::Context;
#[cfg(feature = "size-of")]
use size_of::SizeOf;

macro_rules! make_wrappers {
    ($($name:ident,)*) => {
        $(
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq, From, Into)]
        #[cfg_attr(feature = "size-of", derive(SizeOf))]
        pub struct $name(pub Span);

        impl $crate::structure::Located for $name {
            fn location(&self) -> CodePoint {
                self.0.point
            }
        }
        )*
    };
}

make_wrappers!(Keyword, Symbol, TypeName, Identifier,);

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum UnaryOperationSymbol {
    Neg(Symbol),
    Not(Symbol),
    Inv(Symbol),
    Plus(Symbol),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum BinaryOperationSymbol {
    /// **
    Pow(Symbol),
    /// * / %
    Mul(Symbol),
    /// + -
    Add(Symbol),
    /// <=>
    ComplexComparison(Symbol),
    /// <= != == >=
    CompoundComparison(Symbol),
    /// < >
    Comparison(Symbol),
    /// | & ^
    Bit(Symbol),
    /// || &&
    Logic(Symbol),
}

#[derive(Debug, Clone, PartialEq, From)]
pub struct Enclosed<T> {
    pub left: Symbol,
    pub inner: T,
    pub right: Symbol,
}

#[cfg(feature = "size-of")]
impl<T: SizeOf> SizeOf for Enclosed<Box<[T]>> {
    fn size_of_children(&self, context: &mut Context) {
        self.left.size_of_children(context);
        self.inner.size_of_children(context);
        self.right.size_of_children(context);
    }
}

impl Located for UnaryOperationSymbol {
    fn location(&self) -> CodePoint {
        match self {
            UnaryOperationSymbol::Neg(x) => x.location(),
            UnaryOperationSymbol::Not(x) => x.location(),
            UnaryOperationSymbol::Inv(x) => x.location(),
            UnaryOperationSymbol::Plus(x) => x.location(),
        }
    }
}

impl Located for BinaryOperationSymbol {
    fn location(&self) -> CodePoint {
        match self {
            BinaryOperationSymbol::Pow(x) => x.location(),
            BinaryOperationSymbol::Mul(x) => x.location(),
            BinaryOperationSymbol::Add(x) => x.location(),
            BinaryOperationSymbol::ComplexComparison(x) => x.location(),
            BinaryOperationSymbol::CompoundComparison(x) => x.location(),
            BinaryOperationSymbol::Comparison(x) => x.location(),
            BinaryOperationSymbol::Bit(x) => x.location(),
            BinaryOperationSymbol::Logic(x) => x.location(),
        }
    }
}
