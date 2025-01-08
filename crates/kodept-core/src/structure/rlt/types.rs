use derive_more::From;

use crate::code_point::CodePoint;
use crate::structure::Located;
use crate::structure::rlt::new_types::*;

#[derive(Debug, Clone, PartialEq, From)]
pub enum Type {
    Reference(TypeName),
    Tuple(Tuple)
}

#[derive(Debug, Clone, PartialEq, From)]
pub struct Tuple(pub Enclosed<Box<[Type]>>);

#[derive(Debug, Clone, PartialEq)]
pub struct TypedParameter {
    pub id: Identifier,
    pub parameter_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UntypedParameter {
    pub id: Identifier,
}

#[derive(Debug, Clone, PartialEq, From)]
pub enum Parameter {
    Typed(TypedParameter),
    Untyped(UntypedParameter),
}

impl Located for Type {
    fn location(&self) -> CodePoint {
        match self {
            Type::Reference(x) => x.location(),
            Type::Tuple(x) => x.location()
        }
    }
}

impl Located for TypedParameter {
    fn location(&self) -> CodePoint {
        self.id.location()
    }
}

impl Located for UntypedParameter {
    fn location(&self) -> CodePoint {
        self.id.location()
    }
}

impl Located for Parameter {
    fn location(&self) -> CodePoint {
        match self {
            Parameter::Typed(x) => x.location(),
            Parameter::Untyped(x) => x.location(),
        }
    }
}

impl Located for Tuple {
    fn location(&self) -> CodePoint {
        self.0.left.location()
    }
}
