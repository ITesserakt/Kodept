use crate::node_id::{Erase, NodeId};
use crate::prelude::ASTNode;
use crate::properties::{HasProperty, Lexeme, Node, NodeProperty, SourceSpan};
use crate::syntax_tree::buffer::{Buffer, RefBuffer};
use crate::syntax_tree::children::HasChild;
use bevy_ecs::change_detection::MaybeLocation;
use bevy_ecs::component::Component;
use bevy_ecs::error::{CommandWithEntity, HandleError};
use bevy_ecs::prelude::{Bundle, Command, EntityCommand, EntityWorldMut};
use bevy_ecs::relationship::Relationship;
use bevy_ecs::system::entity_command::EntityCommandError;
use bevy_ecs::world::error::EntityMutableFetchError;
use bevy_utils::prelude::DebugName;
use derive_more::{Deref, DerefMut, Display, Error};
use std::marker::PhantomData;

#[derive(Debug, Deref, DerefMut)]
pub struct NodeBuilder<State>(State);

pub trait Spawner<Node> {
    type Buffer: Buffer;

    fn spawn_builder<C: Constructed<Node = Node>>(
        self,
        builder: C,
    ) -> SpawnerNode<Node, Self::Buffer>;
}

pub trait AnonSpawner<Parent, Tag> {
    type Buffer: Buffer;
    type Spawner<Node>: Spawner<Node, Buffer = Self::Buffer>
    where
        Parent: HasChild<Node, Tag>,
        Node: ASTNode;

    fn into_concrete<Node>(self) -> Self::Spawner<Node>
    where
        Parent: HasChild<Node, Tag>,
        Node: ASTNode;
}

pub trait Constructed: Sized {
    type Node: ASTNode;
    type Properties: Bundle;
    type Clones: Bundle;

    fn spawn_in<S>(
        self,
        spawner: S,
    ) -> SpawnedNode<Self::Node, RelatedNodeSpawner<Self::Node, (), S::Buffer>>
    where
        S: Spawner<Self::Node>;

    #[inline]
    #[track_caller]
    fn spawn(self, spawner: impl Spawner<Self::Node>) -> SpawnedNode<Self::Node, ()> {
        self.spawn_in(spawner).finish()
    }

    fn into_bundle(self) -> impl Bundle;
}

pub struct Constructing<Node, Properties, Clones> {
    node: Node,
    properties: Properties,
    clones: PhantomData<Clones>,
}

pub struct Spawned<Node, Spawner> {
    spawner: Spawner,
    id: NodeId<Node>,
}

pub struct NodeSpawner<Buffer> {
    buffer: Buffer,
}

pub struct RelatedNodeSpawner<Parent, Tag, Buffer> {
    buffer: Buffer,
    parent_id: NodeId<Parent>,
    tag: PhantomData<Tag>,
}

struct Propagate<Clones> {
    to: NodeId,
    clones: PhantomData<Clones>,
}

struct EnsureRequiredProperties<Node> {
    on: PhantomData<Node>,
}

#[derive(Debug, Display, Error)]
#[display(
    "Expected {}[{}] to have required property {}",
    on,
    spawned_by,
    property_name
)]
struct MissingRequiredProperty {
    on: NodeId,
    spawned_by: MaybeLocation,
    property_name: DebugName,
}

pub type ConstructingNode<Node, Properties, Clones> =
    NodeBuilder<Constructing<Node, Properties, Clones>>;
pub type SpawnedNode<Node, Spawner = ()> = NodeBuilder<Spawned<Node, Spawner>>;
pub type SpawnerNode<Node, Buffer> = SpawnedNode<Node, RelatedNodeSpawner<Node, (), Buffer>>;

impl<N, P, C> Constructed for ConstructingNode<N, P, C>
where
    N: ASTNode,
    P: Bundle,
    C: Bundle,
{
    type Node = N;
    type Properties = P;
    type Clones = C;

    #[inline]
    #[track_caller]
    fn spawn_in<S>(self, spawner: S) -> SpawnerNode<Self::Node, S::Buffer>
    where
        S: Spawner<Self::Node>,
    {
        spawner.spawn_builder(self)
    }

    #[inline]
    fn into_bundle(self) -> impl Bundle {
        ConstructingNode::into_bundle(self)
    }
}

impl<Node> ConstructingNode<Node, (), ()> {
    #[inline]
    pub const fn new(node: Node) -> Self {
        Self(Constructing {
            node,
            properties: (),
            clones: PhantomData,
        })
    }
}

impl<N, P, C> ConstructingNode<N, P, C> {
    #[inline]
    pub fn with_property<Property>(self, value: Property) -> ConstructingNode<N, (P, Property), C>
    where
        N: HasProperty<Property>,
        Property: NodeProperty,
    {
        NodeBuilder(Constructing {
            node: self.0.node,
            properties: (self.0.properties, value),
            clones: PhantomData,
        })
    }

    #[inline]
    pub fn clone_property<Property>(self) -> ConstructingNode<N, P, (C, Property)>
    where
        N: HasProperty<Property>,
        Property: NodeProperty,
    {
        NodeBuilder(Constructing {
            node: self.0.node,
            properties: self.0.properties,
            clones: PhantomData,
        })
    }

    #[inline]
    fn into_bundle(self) -> impl Bundle
    where
        N: ASTNode,
        P: Bundle,
    {
        (self.0.node, self.0.properties, Node::of::<N>())
    }
}

impl<N, Spawner> SpawnedNode<N, Spawner> {
    #[inline]
    pub fn id(&self) -> NodeId<N> {
        self.id
    }

    #[inline]
    pub fn finish(self) -> SpawnedNode<N> {
        NodeBuilder(Spawned {
            id: self.id,
            spawner: (),
        })
    }
}

impl<N, B> SpawnerNode<N, B>
where
    B: Buffer,
{
    #[inline]
    pub fn spawner<Tag>(&mut self) -> RelatedNodeSpawner<N, Tag, B::Reborrowed<'_>> {
        RelatedNodeSpawner {
            buffer: self.0.spawner.buffer.reborrow(),
            parent_id: self.0.spawner.parent_id,
            tag: PhantomData,
        }
    }
}

impl<Clones: Bundle> EntityCommand for Propagate<Clones> {
    fn apply(self, mut entity: EntityWorldMut) -> () {
        entity.clone_with_opt_in(self.to.entity(), |b| {
            b.allow_if_new::<Clones>();
        });
    }
}

impl<N: ASTNode> EntityCommand<Result<(), MissingRequiredProperty>>
    for EnsureRequiredProperties<N>
{
    fn apply(self, entity: EntityWorldMut) -> Result<(), MissingRequiredProperty> {
        fn require<T: Component>(entity: &EntityWorldMut) -> Result<(), MissingRequiredProperty> {
            if !entity.contains::<T>() {
                return Err(MissingRequiredProperty {
                    on: entity.id().into(),
                    spawned_by: entity.spawned_by(),
                    property_name: DebugName::type_name::<T>(),
                });
            }
            Ok(())
        }

        require::<Node>(&entity)
            .and(require::<N>(&entity))
            .and(require::<SourceSpan>(&entity))
            .and(require::<Lexeme>(&entity))
    }
}

impl<Clones> Propagate<Clones> {
    #[inline]
    fn to(to: impl Erase) -> Self {
        Self {
            to: to.erase(),
            clones: PhantomData,
        }
    }

    #[inline]
    fn from(
        self,
        from: impl Erase,
    ) -> impl Command<Result<(), EntityMutableFetchError>>
    + HandleError<Result<(), EntityMutableFetchError>>
    where
        Self: EntityCommand,
    {
        self.with_entity(from.erase().entity())
    }
}

impl<Node> EnsureRequiredProperties<Node> {
    #[inline]
    fn on(
        id: impl Into<NodeId<Node>>,
    ) -> impl Command<Result<(), EntityCommandError<MissingRequiredProperty>>>
    + HandleError<Result<(), EntityCommandError<MissingRequiredProperty>>>
    where
        Node: ASTNode,
    {
        let command = Self { on: PhantomData };
        command.with_entity(id.into().entity())
    }
}

impl<B> NodeSpawner<B> {
    #[inline]
    pub fn new(buffer: B) -> Self {
        Self { buffer }
    }
}

impl<P, T, B> RelatedNodeSpawner<P, T, B> {
    #[inline]
    pub fn new(buffer: B, parent_id: impl Into<NodeId<P>>) -> Self {
        Self {
            buffer,
            parent_id: parent_id.into(),
            tag: PhantomData,
        }
    }
}

impl<'a, N, B> Spawner<N> for &'a mut NodeSpawner<B>
where
    N: ASTNode,
    B: Buffer,
{
    type Buffer = B::Reborrowed<'a>;

    #[inline]
    #[track_caller]
    fn spawn_builder<C: Constructed<Node = N>>(self, builder: C) -> SpawnerNode<N, Self::Buffer> {
        let bundle = builder.into_bundle();
        let (id, buffer) = (&mut self.buffer).spawn(bundle);
        buffer.queue(EnsureRequiredProperties::<N>::on(id));
        let id = id.into();

        NodeBuilder(Spawned {
            id,
            spawner: RelatedNodeSpawner::new(self.buffer.reborrow(), id),
        })
    }
}

impl<'a, N, B> Spawner<N> for &'a NodeSpawner<B>
where
    N: ASTNode,
    B: RefBuffer,
{
    type Buffer = B::Reborrowed<'a>;

    #[inline]
    #[track_caller]
    fn spawn_builder<C: Constructed<Node = N>>(self, builder: C) -> SpawnerNode<N, Self::Buffer> {
        let bundle = builder.into_bundle();
        let (id, buffer) = (&self.buffer).spawn(bundle);
        buffer.queue(EnsureRequiredProperties::<N>::on(id));
        let id = id.into();

        NodeBuilder(Spawned {
            id,
            spawner: RelatedNodeSpawner::new(self.buffer.borrow(), id),
        })
    }
}

impl<N, B> Spawner<N> for NodeSpawner<B>
where
    B: Buffer,
    N: ASTNode,
{
    type Buffer = B;

    #[inline]
    #[track_caller]
    fn spawn_builder<C: Constructed<Node = N>>(
        mut self,
        builder: C,
    ) -> SpawnerNode<N, Self::Buffer> {
        let node = builder.spawn(&mut self);
        NodeBuilder(Spawned {
            id: node.id,
            spawner: RelatedNodeSpawner::new(self.buffer, node.id),
        })
    }
}

impl<'a, N, B, P, T> Spawner<N> for &'a mut RelatedNodeSpawner<P, T, B>
where
    N: ASTNode,
    B: Buffer,
    P: HasChild<N, T>,
{
    type Buffer = B::Reborrowed<'a>;

    #[inline]
    #[track_caller]
    fn spawn_builder<C: Constructed<Node = N>>(self, builder: C) -> SpawnerNode<N, Self::Buffer> {
        let bundle = builder.into_bundle();
        let relationship: P::Relationship = Relationship::from(self.parent_id.entity());
        let (id, buffer) = (&mut self.buffer).spawn((bundle, relationship));
        buffer
            .queue(Propagate::<C::Clones>::to(id).from(self.parent_id))
            .queue(EnsureRequiredProperties::<N>::on(id));
        let id = id.into();

        NodeBuilder(Spawned {
            id,
            spawner: RelatedNodeSpawner::new(self.buffer.reborrow(), id),
        })
    }
}

impl<'a, N, B, P, T> Spawner<N> for &'a RelatedNodeSpawner<P, T, B>
where
    N: ASTNode,
    B: RefBuffer,
    P: HasChild<N, T>,
{
    type Buffer = B::Reborrowed<'a>;

    #[inline]
    #[track_caller]
    fn spawn_builder<C: Constructed<Node = N>>(self, builder: C) -> SpawnerNode<N, Self::Buffer> {
        let bundle = builder.into_bundle();
        let relationship: P::Relationship = Relationship::from(self.parent_id.entity());
        let (id, buffer) = (&self.buffer).spawn((bundle, relationship));
        buffer
            .queue(Propagate::<C::Clones>::to(id).from(self.parent_id))
            .queue(EnsureRequiredProperties::<N>::on(id));
        let id = id.into();

        NodeBuilder(Spawned {
            id,
            spawner: RelatedNodeSpawner::new(self.buffer.borrow(), id),
        })
    }
}

impl<N, B, P, T> Spawner<N> for RelatedNodeSpawner<P, T, B>
where
    N: ASTNode,
    B: Buffer,
    P: HasChild<N, T>,
{
    type Buffer = B;

    #[inline]
    #[track_caller]
    fn spawn_builder<C: Constructed<Node = N>>(
        mut self,
        builder: C,
    ) -> SpawnerNode<N, Self::Buffer> {
        let node = builder.spawn(&mut self);
        NodeBuilder(Spawned {
            id: node.id,
            spawner: RelatedNodeSpawner::new(self.buffer, node.id),
        })
    }
}

impl<'a, P, T, B> AnonSpawner<P, T> for &'a mut RelatedNodeSpawner<P, T, B>
where
    B: Buffer,
{
    type Buffer = B::Reborrowed<'a>;
    type Spawner<Node>
        = Self
    where
        P: HasChild<Node, T>,
        Node: ASTNode;

    #[inline]
    fn into_concrete<Node>(self) -> Self::Spawner<Node>
    where
        P: HasChild<Node, T>,
        Node: ASTNode,
    {
        self
    }
}

impl<'a, P, T, B> AnonSpawner<P, T> for &'a RelatedNodeSpawner<P, T, B>
where
    B: RefBuffer,
{
    type Buffer = B::Reborrowed<'a>;
    type Spawner<Node>
        = Self
    where
        P: HasChild<Node, T>,
        Node: ASTNode;

    #[inline]
    fn into_concrete<Node>(self) -> Self::Spawner<Node>
    where
        P: HasChild<Node, T>,
        Node: ASTNode,
    {
        self
    }
}

impl<P, T, B> AnonSpawner<P, T> for RelatedNodeSpawner<P, T, B>
where
    B: Buffer,
{
    type Buffer = B;
    type Spawner<Node>
        = Self
    where
        P: HasChild<Node, T>,
        Node: ASTNode;

    #[inline]
    fn into_concrete<Node>(self) -> Self::Spawner<Node>
    where
        P: HasChild<Node, T>,
        Node: ASTNode,
    {
        self
    }
}
