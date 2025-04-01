use kodept_core::structure::span::CodeHolder as BasicCodeHolder;

pub use super::graph::Identifiable;
use crate::graph::{AnyNode, SubSyntaxTree};
use crate::Str;

pub trait CodeHolder: BasicCodeHolder<Str = Str> {}
impl<C: BasicCodeHolder<Str = Str>> CodeHolder for C {}

#[allow(clippy::wrong_self_convention)]
pub trait AsEnum {
    type Enum;

    fn as_enum(self) -> Self::Enum;
}

pub trait PopulateTree<'a> {
    type Root: Into<AnyNode>;

    fn convert(self, context: impl CodeHolder) -> SubSyntaxTree<'a, Self::Root>;
}
