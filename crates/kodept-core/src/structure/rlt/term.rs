use crate::code_point::CodePoint;
use crate::structure::rlt::new_types::{Identifier, TypeName};
use crate::structure::Located;
use derive_more::From;
#[cfg(feature = "size-of")]
use size_of::SizeOf;

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum Term {
    Reference(Reference),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub enum Reference {
    Type(TypeName),
    Identifier(Identifier),
}

impl Located for Term {
    fn location(&self) -> CodePoint {
        match self {
            Term::Reference(x) => x.location(),
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
