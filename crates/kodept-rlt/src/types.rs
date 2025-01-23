use crate::new_types::{Enclosed, Identifier, TypeName};
use derive_more::From;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Type {
    Reference(TypeName),
    Tuple(Tuple),
}

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tuple(pub Enclosed<Box<[Type]>>);

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypedParameter {
    pub id: Identifier,
    pub parameter_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UntypedParameter {
    pub id: Identifier,
}

#[derive(Debug, Clone, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Parameter {
    Typed(TypedParameter),
    Untyped(UntypedParameter),
}

impl Located for Type {
    fn location(&self) -> CodePoint {
        match self {
            Type::Reference(x) => x.location(),
            Type::Tuple(x) => x.location(),
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
