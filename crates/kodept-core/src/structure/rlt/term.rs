use derive_more::From;

use crate::code_point::CodePoint;
use crate::structure::Located;
use crate::structure::rlt::new_types::{Identifier, TypeName};

#[derive(Debug, Clone, PartialEq, From)]
pub enum Term {
    Reference(Reference),
}

#[derive(Debug, Clone, PartialEq)]
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
