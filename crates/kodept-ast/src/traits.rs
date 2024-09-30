use kodept_core::structure::span::CodeHolder;

use crate::graph::{AnyNode, NodeId, SubSyntaxTree};

pub use super::graph::Identifiable;

#[allow(clippy::wrong_self_convention)]
pub trait AsEnum {
    type Enum;

    fn as_enum(self) -> Self::Enum;
}

pub trait PopulateTree<'a> {
    type Root: Into<AnyNode>;

    fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root>;
}
