use std::error::Error;
use crate::graph::{AnyNode, AnyNodeD, SyntaxTree};
use derive_more::Display;
use std::fmt::Formatter;

pub trait SubEnum {
    const VARIANTS: &'static [AnyNodeD];

    #[inline]
    fn contains(node: &AnyNode) -> bool {
        Self::VARIANTS.contains(&node.describe())
    }
}

#[derive(Debug)]
pub struct ConversionError {
    pub expected_types: &'static [AnyNodeD],
    pub actual_type: AnyNodeD,
}

pub trait Node: SubEnum {
    fn erase(self) -> AnyNode;
    fn describe(&self) -> AnyNodeD;

    fn try_from_ref(value: &AnyNode) -> Result<&Self, ConversionError>;
    fn try_from_mut(value: &mut AnyNode) -> Result<&mut Self, ConversionError>;
}

pub trait HasParent: Node {
    type Parent: Node;
    
    fn parent<'a>(&self, ast: &'a SyntaxTree) -> Option<&'a Self::Parent>;
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.expected_types.is_empty() {
            write!(f, "Expected node not like {}", self.actual_type)?;
            return Ok(());
        }
        
        write!(f, "Expected nodes like {}", self.expected_types[0])?;
        for item in &self.expected_types[1..] {
            write!(f, ", {}", item)?;
        }
        write!(f, ", but found node like {}", self.actual_type)?;

        Ok(())
    }
}

impl Error for ConversionError {}
