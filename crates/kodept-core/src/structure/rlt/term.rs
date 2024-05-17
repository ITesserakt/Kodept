use derive_more::From;

use crate::code_point::CodePoint;
use crate::structure::Located;
use crate::structure::rlt::Context;
use crate::structure::rlt::new_types::{Identifier, TypeName};

#[derive(Debug, Clone, PartialEq, From)]
pub enum Term {
    Reference(Reference),
    Contextual(ContextualReference)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Reference {
    Type(TypeName),
    Identifier(Identifier),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextualReference {
    pub context: Context,
    pub inner: Reference
}

impl Located for Term {
    fn location(&self) -> CodePoint {
        match self {
            Term::Reference(x) => x.location(),
            Term::Contextual(x) => x.location()
        }
    }
}

impl Located for ContextualReference {
    fn location(&self) -> CodePoint {
        let (_, unfolded) = self.context.clone().unfold();
        let first = unfolded.first().unwrap_or(&self.inner);
        let last = &self.inner;
        let length = last.location().offset + last.location().length - first.location().offset;
        CodePoint::new(length, first.location().offset)
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
