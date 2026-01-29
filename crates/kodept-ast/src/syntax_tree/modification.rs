use crate::node_id::Erase;
use crate::prelude::{ASTNode, NodeId};
use crate::syntax_tree::buffer::Buffer;
use crate::syntax_tree::builder_v4::{Constructed, ConstructingNode, RelatedNodeSpawner};
use crate::syntax_tree::children::{Family, HasChild};
use bevy_ecs::error::CommandWithEntity;
use bevy_ecs::prelude::{
    Bundle, ChildOf, EntityCommand, EntityWorldMut, RelationshipTarget, World,
};
use bevy_ecs::relationship::{OrderedRelationshipSourceCollection, Relationship};
use derive_more::{Display, Error};
use std::marker::PhantomData;

pub struct NodeModification<T, B: Buffer> {
    buffer: B,
    id: NodeId<T>,
    removed: Vec<std::sync::mpsc::Receiver<NodeId>>,
}

pub struct ChainedNodeModification<Parent, Child, Tag, B> {
    buffer: B,
    parent_id: NodeId<Parent>,
    child_id: NodeId<Child>,
    _phantom: PhantomData<Tag>,
}

#[derive(Debug, Clone)]
struct Slot(Option<NodeId>, std::sync::mpsc::SyncSender<NodeId>);

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
    parent: NodeId,
    _phantom: PhantomData<R>,
}

#[derive(Debug, Display, Error)]
#[display(
    "While inserting node to AST: node is already connected to the tree ({} -> {})",
    parent,
    this
)]
struct NodeConnectedError {
    parent: NodeId,
    this: NodeId,
}

impl<R: Relationship> EntityCommand<Result<(), NodeConnectedError>> for AddRelatedCommand<R> {
    fn apply(self, mut entity: EntityWorldMut) -> Result<(), NodeConnectedError> {
        if let Some(&ChildOf(parent)) = entity.get::<ChildOf>() {
            return Err(NodeConnectedError {
                parent: parent.into(),
                this: entity.id().into(),
            });
        }
        entity.insert::<R>(Relationship::from(self.parent.entity()));
        Ok(())
    }
}

impl Slot {
    fn new(value: NodeId) -> (Self, std::sync::mpsc::Receiver<NodeId>) {
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

impl<B: Buffer> NodeModification<(), B> {
    pub fn new_unchecked<T: ASTNode>(buffer: B, entity: impl Erase) -> NodeModification<T, B> {
        NodeModification {
            buffer,
            id: entity.erase().cast(),
            removed: vec![],
        }
    }
}

impl<Node, B> NodeModification<Node, B>
where
    Node: ASTNode,
    B: Buffer,
{
    pub fn new(buffer: B, id: NodeId<Node>) -> Self {
        Self {
            buffer,
            id,
            removed: vec![],
        }
    }

    #[track_caller]
    pub fn spawn_child<Child, Properties, Clones, Tag>(
        &mut self,
        builder: ConstructingNode<Child, Properties, Clones>,
    ) -> ChainedNodeModification<Node, Child, Tag, B::Reborrowed<'_>>
    where
        Node: HasChild<Child, Tag>,
        Child: ASTNode,
        Properties: Bundle,
        Clones: Bundle,
    {
        let node = builder.spawn(RelatedNodeSpawner::new(self.buffer.reborrow(), self.id));

        ChainedNodeModification {
            buffer: self.buffer.reborrow(),
            parent_id: self.id,
            child_id: node.id(),
            _phantom: PhantomData,
        }
    }

    pub fn remove_child<Child, Tag>(&mut self, id: NodeId<Child>) -> RemovedNode<Child>
    where
        Node: HasChild<Child, Tag>,
        Tag: Send + Sync + 'static,
        Child: ASTNode,
    {
        let child_id = id.entity();
        (&mut self.buffer).queue(move |w: &mut World| {
            w.entity_mut(child_id).remove::<Node::Relationship>();
        });
        let (slot, rx) = Slot::new(id.cast());
        self.removed.push(rx);
        RemovedNode {
            slot,
            _phantom: PhantomData,
        }
    }

    pub fn remove_child_unchecked<Tag>(&mut self, id: impl Erase) -> RemovedFamilyNode<Node, Tag>
    where
        Node: Family<Tag>,
        Tag: Send + Sync + 'static,
    {
        let id = id.erase();
        (&mut self.buffer).queue(move |w: &mut World| {
            w.entity_mut(id.entity()).remove::<Node::Relationship>();
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
    ) -> ChainedNodeModification<Node, Child, Tag, B::Reborrowed<'_>>
    where
        Node: HasChild<Child, Tag>,
        Child: ASTNode,
        Tag: Send + Sync + 'static,
    {
        let child_id = id.entity();
        (&mut self.buffer).queue(
            AddRelatedCommand {
                parent: self.id.cast(),
                _phantom: PhantomData::<Node::Relationship>,
            }
            .with_entity(child_id),
        );
        ChainedNodeModification {
            child_id: id,
            parent_id: self.id,
            buffer: self.buffer.reborrow(),
            _phantom: PhantomData,
        }
    }
}

impl<T, B> Drop for NodeModification<T, B>
where
    B: Buffer,
{
    fn drop(&mut self) {
        let to_despawn = std::mem::take(&mut self.removed);
        (&mut self.buffer).queue(move |w: &mut World| {
            to_despawn
                .into_iter()
                .filter_map(|it| it.try_recv().ok())
                .for_each(|it| {
                    w.despawn(it.entity());
                })
        });
    }
}

impl<Parent, Child, Tag, B> ChainedNodeModification<Parent, Child, Tag, B>
where
    B: Buffer,
{
    pub fn spawn_child<GrandChild, Properties, Clones, ChildTag>(
        &mut self,
        builder: ConstructingNode<GrandChild, Properties, Clones>,
    ) -> ChainedNodeModification<Child, GrandChild, ChildTag, B::Reborrowed<'_>>
    where
        Child: HasChild<GrandChild, ChildTag>,
        GrandChild: ASTNode,
        Properties: Bundle,
        Clones: Bundle,
    {
        let node = builder.spawn(RelatedNodeSpawner::new(
            self.buffer.reborrow(),
            self.child_id,
        ));

        ChainedNodeModification {
            buffer: self.buffer.reborrow(),
            parent_id: self.child_id,
            child_id: node.id(),
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
        (&mut self.buffer).queue(move |w: &mut World| {
            let component =
                w.get_mut::<<Parent::Relationship as Relationship>::RelationshipTarget>(parent_id);
            if let Some(mut component) = component {
                let collection = component.collection_mut_risky();
                collection.place(child_id, index);
            }
        });
        self
    }

    pub fn add_child_unchecked<T>(&mut self, id: impl Erase) -> &mut Self
    where
        Child: Family<T>,
    {
        (&mut self.buffer).queue(
            AddRelatedCommand {
                parent: self.child_id.cast(),
                _phantom: PhantomData::<Child::Relationship>,
            }
            .with_entity(id.erase().entity()),
        );
        self
    }
}

impl<T, Tag> Erase for RemovedFamilyNode<T, Tag> {
    #[inline]
    fn erase(mut self) -> NodeId {
        self.slot.0.take().unwrap()
    }
}

impl<T> Erase for RemovedNode<T> {
    #[inline]
    fn erase(mut self) -> NodeId {
        self.slot.0.take().unwrap()
    }
}
