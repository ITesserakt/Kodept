use crate::new_types::{Identifier, TypeName};
use crate::prelude::Context;
use derive_more::From;
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum Term {
    Reference(Identifier),
    ContextualReference(Contextual<Identifier>),
    Constant(TypeName),
    ContextualConstant(Contextual<TypeName>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct Contextual<T> {
    pub context: Context,
    pub inner: T,
}

impl Located for Term {
    fn location(&self) -> CodePoint {
        match self {
            Term::Reference(x) => x.location(),
            Term::ContextualReference(x) => x.location(),
            Term::Constant(x) => x.location(),
            Term::ContextualConstant(x) => x.location(),
        }
    }
}

impl<T: Located> Located for Contextual<T> {
    fn location(&self) -> CodePoint {
        let (is_global, unfolded) = self.context.unfold();
        let first = unfolded
            .first()
            .map_or(self.inner.location(), |it| it.location());
        let last = self.inner.location();
        let length = (first + last).length;
        if is_global.is_some() {
            // Shift to the left by 2 symbols for '::'
            CodePoint::new(length + 2, first.offset.saturating_sub(2))
        } else {
            CodePoint::new(length, first.offset)
        }
    }
}

impl<T: Located> SpanBounds for Contextual<T> {
    #[inline]
    fn bounds(&self) -> Span {
        self.location().into()
    }
}

impl SpanBounds for Term {
    fn bounds(&self) -> Span {
        match self {
            Term::Reference(x) => x.bounds(),
            Term::ContextualReference(x) => x.bounds(),
            Term::Constant(x) => x.bounds(),
            Term::ContextualConstant(x) => x.bounds(),
        }
    }
}
