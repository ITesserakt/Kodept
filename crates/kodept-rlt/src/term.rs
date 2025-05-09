use crate::new_types::{Identifier, TypeName};
use crate::prelude::Context;
use derive_more::From;
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum Term {
    Reference(Reference),
    Contextual(ContextualReference),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum Reference {
    Type(TypeName),
    Identifier(Identifier),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct ContextualReference {
    pub context: Context,
    pub inner: Reference,
}

impl Located for Term {
    fn location(&self) -> CodePoint {
        match self {
            Term::Reference(x) => x.location(),
            Term::Contextual(x) => x.location(),
        }
    }
}

impl Located for ContextualReference {
    fn location(&self) -> CodePoint {
        let (is_global, unfolded) = self.context.unfold();
        let first = unfolded.first().copied().unwrap_or(&self.inner);
        let last = &self.inner;
        let length = (first.bounds() + last.bounds()).length;
        if is_global.is_some() {
            // Shift to the left by 2 symbols for '::'
            CodePoint::new(length + 2, first.location().offset - 2)
        } else {
            CodePoint::new(length, first.location().offset)
        }
    }
}

impl Located for Reference {
    fn location(&self) -> CodePoint {
        match self {
            Reference::Type(x) => x.location(),
            Reference::Identifier(x) => x.location(),
        }
    }
}

impl SpanBounds for Term {
    fn bounds(&self) -> Span {
        match self {
            Term::Reference(x) => x.bounds(),
            Term::Contextual(x) => x.location().into()
        }
    }
}

impl SpanBounds for Reference {
    fn bounds(&self) -> Span {
        self.location().into()
    }
}
