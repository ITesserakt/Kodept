use bevy_ecs::entity::Entity;
use bevy_ecs::query::{QueryEntityError, With};
use bevy_ecs::system::{Query, SystemParam};
use bevy_utils::prelude::DebugName;
use crate::node_id::Erase;
use crate::prelude::{ASTNode, NodeRef};
use crate::properties::{HasProperty, Name, Node, NodeProperty, RequireProperty, SourceSpan};

impl<'a, T> NodeRef<'a, &'a T> {
    pub fn name(&self) -> &Name
    where
        T: RequireProperty<Name>,
    {
        self.property()
    }

    pub fn kind(&self) -> &DebugName
    where
        T: ASTNode,
    {
        &self.property::<Node>().kind
    }

    pub fn span(&self) -> SourceSpan
    where
        T: ASTNode,
    {
        *self.property::<SourceSpan>()
    }
}

#[derive(Debug, SystemParam)]
pub struct PropertyQuery<'w, 's, T, P>
where
    T: ASTNode,
    P: NodeProperty,
{
    query: Query<'w, 's, Option<&'static P>, With<T>>,
}

impl<'w, 's, T, P> PropertyQuery<'w, 's, T, P>
where
    T: ASTNode,
    P: NodeProperty,
    T: HasProperty<P>,
{
    pub fn iter(&self) -> impl Iterator<Item = Option<&P>> {
        self.query.iter()
    }

    pub fn get(&self, id: impl Erase<Entity>) -> Result<Option<&P>, QueryEntityError> {
        self.query.get(id.erase())
    }
}

impl<'w, 's, T, P> PropertyQuery<'w, 's, T, P>
where
    T: ASTNode,
    P: NodeProperty,
    T: RequireProperty<P>,
{
    pub fn iter_as_required(&self) -> impl Iterator<Item = &P> {
        self.query.iter().filter_map(|it| it)
    }

    pub fn get_required(&self, id: impl Erase<Entity>) -> Result<&P, QueryEntityError> {
        Ok(self
            .query
            .get(id.erase())?
            .expect("Cannot get reqiured property"))
    }
}
