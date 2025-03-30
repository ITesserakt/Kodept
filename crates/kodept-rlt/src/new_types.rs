use derive_more::{From, Into};
use kodept_core::structure::span::Span;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;

macro_rules! make_wrappers {
    ($($name:ident,)*) => {
        $(
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq, From, Into)]
        pub struct $name(pub Span);

        impl kodept_core::structure::Located for $name {
            #[inline(always)]
            fn location(&self) -> CodePoint {
                self.0.point
            }
        }
        
        impl $name {
            #[inline(always)]
            pub fn from_located<L: kodept_core::structure::Located>(value: L) -> Self {
                let span = kodept_core::structure::span::Span::new(value.location());
                $name(span)
            }
        }
        )*
    };
}

make_wrappers!(Keyword, Symbol, TypeName, Identifier,);

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperationSymbol {
    Neg(Symbol),
    Not(Symbol),
    Inv(Symbol),
    Plus(Symbol),
}

#[derive(Debug, Clone, PartialEq)]
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
    /// =
    Assign(Symbol)
}

#[derive(Debug, Clone, PartialEq, From)]
pub struct Enclosed<T> {
    pub left: Symbol,
    pub inner: T,
    pub right: Symbol,
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
            BinaryOperationSymbol::Assign(x) => x.location()
        }
    }
}
