use kodept_core::structure::span::CodeHolder;

pub use super::graph::Identifiable;
use crate::graph::{AnyNode, SubSyntaxTree};
use crate::interning::SharedStr;

#[allow(clippy::wrong_self_convention)]
pub trait AsEnum {
    type Enum;

    fn as_enum(self) -> Self::Enum;
}

pub trait PopulateTree<'a> {
    type Root: Into<AnyNode>;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root>;
}
