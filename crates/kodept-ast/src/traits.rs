use crate::syntax_tree::prelude::{ASTBuilder, Pool};
use crate::Str;
use bevy_ecs::prelude::Component;
use kodept_core::structure::span::CodeHolder as BasicCodeHolder;
use crate::properties::tags::Tagged;
use crate::syntax_tree::children::ChildrenDisjoint;

pub trait CodeHolder: BasicCodeHolder<Str = Str> {}
impl<T: BasicCodeHolder<Str = Str>> CodeHolder for T {}

pub trait FromSyntax: Sized {
    type Syntax;

    fn from_syntax(
        node: &Self::Syntax,
        source: impl CodeHolder,
        pool: &Pool,
    ) -> ASTBuilder<Self>;
}

pub trait ASTNode: Component {}

pub trait Choose<T, Root, Tag: Tagged>: Sized {
    fn branch<Source: CodeHolder>(node: &T) -> ChildrenDisjoint<Root, Source, Tag>;
}
