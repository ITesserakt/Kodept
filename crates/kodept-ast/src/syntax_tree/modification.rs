use crate::experimental::AstBuilder;
use crate::prelude::{ASTNode, NodeId};
use crate::syntax_tree::builder_v3::Constructed;
use crate::syntax_tree::builder_v3::SpawnedIn;
use crate::syntax_tree::children::{Family, HasChild};
use crate::traits::DispatchContext;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{
    ChildOf, Commands, EntityCommand, EntityWorldMut, RelationshipTarget, World,
};
use bevy_ecs::relationship::{OrderedRelationshipSourceCollection, Relationship};
use derive_more::{Display, Error};
use std::marker::PhantomData;

pub struct NodeModification<'w, 's, T> {
    commands: Commands<'w, 's>,
    id: NodeId<T>,
    removed: Vec<std::sync::mpsc::Receiver<Entity>>,
}

pub struct ChainedNodeModification<'w, 's, Parent, Child, Tag> {
    commands: Commands<'w, 's>,
    parent_id: NodeId<Parent>,
    child_id: NodeId<Child>,
    _phantom: PhantomData<Tag>,
}

#[derive(Debug, Clone)]
struct Slot(Option<Entity>, std::sync::mpsc::SyncSender<Entity>);

#[derive(Debug, Clone)]
pub struct RemovedNode<T> {
    slot: Slot,
    _phantom: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct RemovedFamilyNode<F, Tag> {
    slot: Slot,
    _phantom: PhantomData<(F, Tag)>,
}

struct AddRelatedCommand<R: Relationship> {
    parent: Entity,
    _phantom: PhantomData<R>,
}

#[derive(Debug, Display, Error)]
#[display(
    "While inserting node to AST: node is already connected to the tree ({} -> {})",
    parent,
    this
)]
struct NodeConnectedError {
    parent: Entity,
    this: Entity,
}

impl<R: Relationship> EntityCommand<Result<(), NodeConnectedError>> for AddRelatedCommand<R> {
    fn apply(self, mut entity: EntityWorldMut) -> Result<(), NodeConnectedError> {
        if let Some(&ChildOf(parent)) = entity.get::<ChildOf>() {
            return Err(NodeConnectedError {
                parent,
                this: entity.id(),
            });
        }
        entity.insert::<R>(Relationship::from(self.parent));
        Ok(())
    }
}

impl Slot {
    fn new(value: Entity) -> (Self, std::sync::mpsc::Receiver<Entity>) {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        (Self(Some(value), tx), rx)
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        if let Some(id) = self.0 {
            _ = self.1.try_send(id);
        }
    }
}

impl<'w, 's, T> NodeModification<'w, 's, T>
where
    T: ASTNode,
{
    pub fn new_unchecked(commands: &'s mut Commands<'w, '_>, entity: Entity) -> Self {
        Self {
            commands: commands.reborrow(),
            id: entity.into(),
            removed: vec![],
        }
    }

    #[track_caller]
    pub fn spawn_child<C, Tag>(
        &mut self,
        builder: AstBuilder<C>,
    ) -> ChainedNodeModification<'w, '_, T, C::Root, Tag>
    where
        C: Constructed,
        T: HasChild<C::Root, Tag>,
        Tag: Send + Sync + 'static,
    {
        let buffer = self.commands.reborrow();
        let builder = builder.spawn_in(DispatchContext::from_buffer(buffer, self.id));
        let child_id = builder.finish();
        drop(builder);

        ChainedNodeModification {
            commands: self.commands.reborrow(),
            parent_id: self.id,
            child_id,
            _phantom: PhantomData,
        }
    }

    pub fn remove_child<Child, Tag>(&mut self, id: NodeId<Child>) -> RemovedNode<Child>
    where
        T: HasChild<Child, Tag>,
        Tag: Send + Sync + 'static,
        Child: ASTNode,
    {
        let child_id = id.entity();
        self.commands.queue(move |w: &mut World| {
            w.entity_mut(child_id).remove::<T::Relationship>();
        });
        let (slot, rx) = Slot::new(child_id);
        self.removed.push(rx);
        RemovedNode {
            slot,
            _phantom: PhantomData,
        }
    }

    pub fn remove_child_unchecked<Tag>(&mut self, id: Entity) -> RemovedFamilyNode<T, Tag>
    where
        T: Family<Tag>,
        Tag: Send + Sync + 'static,
    {
        self.commands.queue(move |w: &mut World| {
            w.entity_mut(id).remove::<T::Relationship>();
        });
        let (slot, rx) = Slot::new(id);
        self.removed.push(rx);
        RemovedFamilyNode {
            slot,
            _phantom: PhantomData,
        }
    }

    pub fn add_child<Child, Tag>(
        &mut self,
        id: NodeId<Child>,
    ) -> ChainedNodeModification<'w, '_, T, Child, Tag>
    where
        T: HasChild<Child, Tag>,
        Child: ASTNode,
        Tag: Send + Sync + 'static,
    {
        let child_id = id.entity();
        let parent_id = self.id.entity();
        self.commands.entity(child_id).queue(AddRelatedCommand {
            parent: parent_id,
            _phantom: PhantomData::<T::Relationship>,
        });
        ChainedNodeModification {
            child_id: id,
            parent_id: self.id,
            commands: self.commands.reborrow(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Drop for NodeModification<'_, '_, T> {
    fn drop(&mut self) {
        let to_despawn = std::mem::take(&mut self.removed);
        self.commands.queue(move |w: &mut World| {
            to_despawn
                .into_iter()
                .filter_map(|it| it.try_recv().ok())
                .for_each(|it| {
                    w.despawn(it);
                })
        });
    }
}

impl<'w, 's, Parent, Child, Tag> ChainedNodeModification<'w, 's, Parent, Child, Tag> {
    pub fn spawn_child<C, T>(
        &mut self,
        builder: AstBuilder<C>,
    ) -> ChainedNodeModification<'w, '_, Child, C::Root, T>
    where
        C: Constructed,
        Child: HasChild<C::Root, T>,
        T: Send + Sync + 'static,
    {
        let buffer = self.commands.reborrow();
        let builder = builder.spawn_in(DispatchContext::from_buffer(buffer, self.child_id));
        let child_id = builder.finish();
        drop(builder);

        ChainedNodeModification {
            commands: self.commands.reborrow(),
            parent_id: self.child_id,
            child_id,
            _phantom: PhantomData,
        }
    }

    pub fn place_at(&mut self, index: usize) -> &mut Self
    where
        Parent: HasChild<Child, Tag>,
        Child: ASTNode,
        <<Parent::Relationship as Relationship>::RelationshipTarget as RelationshipTarget>::Collection: OrderedRelationshipSourceCollection,
    {
        let parent_id = self.parent_id.entity();
        let child_id = self.child_id.entity();
        self.commands.queue(move |w: &mut World| {
            let component =
                w.get_mut::<<Parent::Relationship as Relationship>::RelationshipTarget>(parent_id);
            if let Some(mut component) = component {
                let collection = component.collection_mut_risky();
                collection.place(child_id, index);
            }
        });
        self
    }

    pub fn add_child_unchecked<T>(&mut self, id: impl Into<Entity>) -> &mut Self
    where
        Child: Family<T>,
    {
        let parent_id = self.child_id.entity();
        let child_id = id.into();
        self.commands.entity(child_id).queue(AddRelatedCommand {
            parent: parent_id,
            _phantom: PhantomData::<Child::Relationship>,
        });
        self
    }
}

impl<T, Tag> From<RemovedFamilyNode<T, Tag>> for Entity {
    fn from(mut value: RemovedFamilyNode<T, Tag>) -> Self {
        value.slot.0.take().unwrap()
    }
}

impl<T> From<RemovedNode<T>> for NodeId<T> {
    fn from(mut value: RemovedNode<T>) -> Self {
        value.slot.0.take().unwrap().into()
    }
}
