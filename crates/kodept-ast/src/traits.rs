use crate::node_id::NodeId;
pub(crate) use crate::syntax_tree::experimental::{DispatchContext, SpawnContext};
use crate::Str;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::relationship::Relationship;
use kodept_core::structure::span::CodeHolder as BasicCodeHolder;

pub trait CodeHolder: BasicCodeHolder<Str = Str> {}
impl<T: BasicCodeHolder<Str = Str>> CodeHolder for T {}

pub trait FromSyntax<Syntax>: Sized {
    type Error: Send + 'static;

    fn from_syntax<R: Relationship>(
        node: &Syntax,
        spawner: SpawnContext<R>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error>;
}

pub trait SplitRef<'a, T>: Sized {
    fn split(self) -> (Self, &'a T);
}

pub trait Dispatch<'a, Root, Tag, Arity>: SplitRef<'a, Self::Node>
where
    Tag: Send + Sync + 'static,
    Arity: crate::arity::Arity
{
    type Node;
    type Error: Send + 'static;

    fn dispatch(
        self,
        spawner: DispatchContext<Root, Tag, Arity>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error>;
}

pub trait ASTNode: Component {}
