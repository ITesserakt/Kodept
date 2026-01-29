use crate::Str;
use crate::node_id::NodeId;
use crate::syntax_tree::experimental::{AnonSpawner, Spawner};
use bevy_ecs::prelude::Component;
use kodept_core::structure::span::CodeHolder as BasicCodeHolder;

pub trait CodeHolder: BasicCodeHolder<Str = Str> {}
impl<T: BasicCodeHolder<Str = Str>> CodeHolder for T {}

pub trait FromSyntax<Syntax, Buffer>: Sized {
    type Error;

    fn from_syntax(
        node: &Syntax,
        spawner: impl Spawner<Self, Buffer = Buffer>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error>;
}

pub trait Dispatch<Parent, Tag, Buffer> {
    type Syntax;
    type Error;

    fn dispatch(
        node: &Self::Syntax,
        spawner: impl AnonSpawner<Parent, Tag, Buffer = Buffer>,
        source: impl CodeHolder,
    ) -> Result<NodeId, Self::Error>;
}

pub trait ASTNode: Component {}
