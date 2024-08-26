use kodept_core::structure::span::CodeHolder;

use crate::graph::{AnyNode, NodeId, SubSyntaxTree};

pub trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
}

impl<T: crate::graph::Identifiable> Identifiable for T {
    fn get_id(&self) -> NodeId<Self> {
        <Self as crate::graph::Identifiable>::get_id(self)
    }
}

#[allow(clippy::wrong_self_convention)]
pub trait AsEnum {
    type Enum;

    fn as_enum(self) -> Self::Enum;
}

pub trait PopulateTree {
    type Root: Into<AnyNode>;

    fn convert(&self, context: &mut impl CodeHolder) -> SubSyntaxTree<Self::Root>;
}
