use derive_more::{From, Into};
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

macro_rules! make_wrappers {
    ($($name:ident,)*) => {
        $(
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq, From, Into, Copy)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
        pub struct $name(pub kodept_core::code_point::CodePoint);

        impl kodept_core::structure::Located for $name {
            #[inline(always)]
            fn location(&self) -> kodept_core::code_point::CodePoint {
                self.0
            }
        }

        impl kodept_core::structure::SpanBounds for $name {
            #[inline(always)]
            fn bounds(&self) -> kodept_core::code_point::Span {
                self.0.into()
            }
        }

        impl $name {
            #[inline(always)]
            pub fn from_located<L: kodept_core::structure::Located>(value: L) -> Self {
                $name(value.location())
            }
        }
        )*
    };
}

make_wrappers!(Keyword, Symbol, TypeName, Identifier,);

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum UnaryOperationSymbol {
    Neg(Symbol),
    Not(Symbol),
    Inv(Symbol),
    Plus(Symbol),
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum BinaryOperationSymbol {
    /// **
    Pow(Symbol),
    /// * / %
    Mul(Symbol),
    Div(Symbol),
    Rem(Symbol),
    /// + -
    Add(Symbol),
    Sub(Symbol),
    /// <=>
    ComplexComparison(Symbol),
    /// <= != == >=
    LessEq(Symbol),
    NEq(Symbol),
    Eq(Symbol),
    GreaterEq(Symbol),
    /// < >
    Less(Symbol),
    Greater(Symbol),
    /// | & ^
    Or(Symbol),
    And(Symbol),
    Xor(Symbol),
    /// || &&
    Disjunction(Symbol),
    Conjunction(Symbol),
    /// =
    Assign(Symbol),
}

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct Enclosed<T> {
    pub left: Symbol,
    pub inner: T,
    pub right: Symbol,
}

impl Located for UnaryOperationSymbol {
    #[inline]
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
    #[inline]
    fn location(&self) -> CodePoint {
        match self {
            BinaryOperationSymbol::Pow(x) => x.location(),
            BinaryOperationSymbol::Mul(x) => x.location(),
            BinaryOperationSymbol::Div(x) => x.location(),
            BinaryOperationSymbol::Rem(x) => x.location(),
            BinaryOperationSymbol::Add(x) => x.location(),
            BinaryOperationSymbol::Sub(x) => x.location(),
            BinaryOperationSymbol::ComplexComparison(x) => x.location(),
            BinaryOperationSymbol::LessEq(x) => x.location(),
            BinaryOperationSymbol::NEq(x) => x.location(),
            BinaryOperationSymbol::Eq(x) => x.location(),
            BinaryOperationSymbol::GreaterEq(x) => x.location(),
            BinaryOperationSymbol::Less(x) => x.location(),
            BinaryOperationSymbol::Greater(x) => x.location(),
            BinaryOperationSymbol::Or(x) => x.location(),
            BinaryOperationSymbol::And(x) => x.location(),
            BinaryOperationSymbol::Xor(x) => x.location(),
            BinaryOperationSymbol::Disjunction(x) => x.location(),
            BinaryOperationSymbol::Conjunction(x) => x.location(),
            BinaryOperationSymbol::Assign(x) => x.location(),
        }
    }
}

impl SpanBounds for BinaryOperationSymbol {
    #[inline]
    fn bounds(&self) -> Span {
        self.location().into()
    }
}

impl SpanBounds for UnaryOperationSymbol {
    #[inline]
    fn bounds(&self) -> Span {
        self.location().into()
    }
}

impl<T> From<(Symbol, Vec<T>, Symbol)> for Enclosed<Box<[T]>> {
    fn from(value: (Symbol, Vec<T>, Symbol)) -> Self {
        Self {
            left: value.0,
            right: value.2,
            inner: value.1.into_boxed_slice(),
        }
    }
}

impl<'a, T> IntoIterator for &'a Enclosed<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
