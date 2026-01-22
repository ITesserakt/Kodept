use crate::{
    node_id::{Erase, NodeId},
    properties::{HasProperty, Lexeme, Node, NodeProperty},
    relationship::NodeRelationship,
    resource::rlt::LexemeId,
    syntax_tree::children::HasChild,
    traits::{ASTNode, CodeHolder, Dispatch, FromSyntax},
    utils::IntoCommonIter,
};
#[cfg(feature = "parallel")]
use bevy_ecs::prelude::ParallelCommands;
use bevy_ecs::prelude::{ChildOf, Commands, World};
use bevy_ecs::{bundle::Bundle, entity::Entity, relationship::Relationship};
use bevy_utils::prelude::DebugName;
use derive_more::{Deref, DerefMut};
use kodept_rlt::traversal::{ErasedNodePtr, SyntaxNode};
use std::marker::PhantomData;

pub struct PropsState<R, P, C> {
    root: R,
    properties: P,
    clones: PhantomData<C>,
}

pub(crate) struct ChildState<R, B: Buffer, Inner = ()> {
    spawner: GenericSpawnContext<Inner, B>,
    _phantom: PhantomData<R>,
}

pub trait SpawnedIn<Root>: Sized {
    type Buffer: Buffer;

    fn with_child<T, U, Tag>(
        &mut self,
        node: &T,
        source: impl CodeHolder,
    ) -> Result<&mut Self, U::Error>
    where
        Root: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        T: SyntaxNode,
    {
        self.with_children([node], source)
    }

    fn with_children<'i, T, U, Tag>(
        &mut self,
        nodes: impl IntoCommonIter<Item = &'i T>,
        source: impl CodeHolder,
    ) -> Result<&mut Self, U::Error>
    where
        Root: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        T: SyntaxNode;

    fn with_dispatch<'a, T, Tag, Arity>(
        &mut self,
        node: impl Into<T>,
        source: impl CodeHolder,
    ) -> Result<&mut Self, T::Error>
    where
        T: Dispatch<'a, Root, Tag, Arity, Node: SyntaxNode> + Send,
        Arity: crate::arity::Arity,
        Tag: Send + Sync + 'static,
    {
        self.with_dispatches::<T, Tag, Arity>([node.into()], source)
    }

    fn with_dispatch_fn<'a, T: SyntaxNode, Tag, Arity, E>(
        &mut self,
        node: &T,
        f: impl FnOnce(
            &T,
            DispatchContext<<Self::Buffer as Buffer>::Reborrowed<'_>, Root, Tag, Arity>,
        ) -> Result<Entity, E>,
    ) -> Result<&mut Self, E>
    where
        Tag: Send + Sync + 'static,
        Arity: crate::arity::Arity;

    fn with_dispatches<'a, T, Tag, Arity>(
        &mut self,
        nodes: impl IntoCommonIter<Item: Into<T>>,
        source: impl CodeHolder,
    ) -> Result<&mut Self, T::Error>
    where
        T: Dispatch<'a, Root, Tag, Arity, Node: SyntaxNode>,
        Arity: crate::arity::Arity,
        Tag: Send + Sync + 'static;

    fn with_manual<Tag, Arity, Result>(
        &mut self,
        f: impl FnOnce(
            DispatchContext<<Self::Buffer as Buffer>::Reborrowed<'_>, Root, Tag, Arity>,
        ) -> Result,
    ) -> Result
    where
        Tag: Send + Sync + 'static,
        Arity: crate::arity::Arity;

    fn finish(&self) -> NodeId<Root>;

    fn finish_any(&self) -> Entity {
        self.finish().entity()
    }
}

trait Context<U> {
    fn spawn_builder<P, C>(
        self,
        builder: AstBuilder<PropsState<U, P, C>>,
    ) -> AstBuilder<impl SpawnedIn<U>>
    where
        P: Bundle,
        C: Bundle,
        U: ASTNode;
}

pub trait Buffer {
    type Reborrowed<'a>: Buffer
    where
        Self: 'a;

    fn reborrow(&mut self) -> Self::Reborrowed<'_>;

    fn spawn(self, bundle: impl Bundle) -> (Entity, Self);
    fn insert(self, entity: Entity, bundle: impl Bundle) -> Self;
    fn clone_specific<B: Bundle>(self, from: Entity, to: Entity) -> Self;
}

pub trait RefBuffer: Buffer {
    fn reborrow_ref(&self) -> Self::Reborrowed<'_>;
}

impl<'w, 's> Buffer for Commands<'w, 's> {
    type Reborrowed<'a>
        = Commands<'w, 'a>
    where
        Self: 'a;

    #[inline]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        Commands::reborrow(self)
    }

    #[inline]
    fn spawn(mut self, bundle: impl Bundle) -> (Entity, Self) {
        let id = Commands::spawn(&mut self, bundle).id();
        (id, self)
    }

    #[inline]
    fn insert(mut self, entity: Entity, bundle: impl Bundle) -> Self {
        Commands::entity(&mut self, entity).insert(bundle);
        self
    }

    fn clone_specific<B: Bundle>(mut self, from: Entity, to: Entity) -> Self {
        self.entity(from).clone_with_opt_in(to, |builder| {
            builder.allow_if_new::<B>();
        });
        self
    }
}

impl<'a> Buffer for &'a mut World {
    type Reborrowed<'b>
        = &'b mut World
    where
        'a: 'b;

    #[inline]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        self as &mut World
    }

    #[inline]
    fn spawn(self, bundle: impl Bundle) -> (Entity, Self) {
        let id = World::spawn(self, bundle).id();
        (id, self)
    }

    #[inline]
    fn insert(self, entity: Entity, bundle: impl Bundle) -> Self {
        World::entity_mut(self, entity).insert(bundle);
        self
    }

    #[inline]
    fn clone_specific<B: Bundle>(self, from: Entity, to: Entity) -> Self {
        self.entity_mut(from).clone_with_opt_in(to, |builder| {
            builder.allow_if_new::<B>();
        });
        self
    }
}

#[cfg(feature = "parallel")]
impl<'w, 's, 'a> Buffer for &'a ParallelCommands<'w, 's> {
    type Reborrowed<'b>
        = &'b ParallelCommands<'w, 's>
    where
        'a: 'b;

    #[inline]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        self as &ParallelCommands
    }

    #[inline]
    fn spawn(self, bundle: impl Bundle) -> (Entity, Self) {
        let id = self.command_scope(|mut c| Commands::spawn(&mut c, bundle).id());
        (id, self)
    }

    #[inline]
    fn insert(self, entity: Entity, bundle: impl Bundle) -> Self {
        self.command_scope(|mut c| {
            c.entity(entity).insert(bundle);
        });
        self
    }

    #[inline]
    fn clone_specific<B: Bundle>(self, from: Entity, to: Entity) -> Self {
        self.command_scope(|mut c| {
            c.entity(from).clone_with_opt_in(to, |builder| {
                builder.allow_if_new::<B>();
            });
        });
        self
    }
}

#[cfg(feature = "parallel")]
impl<'a, 'w, 's> RefBuffer for &'a ParallelCommands<'w, 's> {
    #[inline]
    fn reborrow_ref(&self) -> Self::Reborrowed<'_> {
        self as &ParallelCommands
    }
}

#[cfg(feature = "parallel")]
impl<'w, 's> Buffer for ParallelCommands<'w, 's> {
    type Reborrowed<'a>
        = &'a ParallelCommands<'w, 'a>
    where
        Self: 'a;

    #[inline]
    fn reborrow(&mut self) -> Self::Reborrowed<'_> {
        self as &ParallelCommands
    }

    #[inline]
    fn spawn(self, bundle: impl Bundle) -> (Entity, Self) {
        let id = self.command_scope(|mut c| Commands::spawn(&mut c, bundle).id());
        (id, self)
    }

    #[inline]
    fn insert(self, entity: Entity, bundle: impl Bundle) -> Self {
        self.command_scope(|mut c| {
            c.entity(entity).insert(bundle);
        });
        self
    }

    #[inline]
    fn clone_specific<B: Bundle>(self, from: Entity, to: Entity) -> Self {
        (&self).clone_specific::<B>(from, to);
        self
    }
}

#[cfg(feature = "parallel")]
impl<'w, 's> RefBuffer for ParallelCommands<'w, 's> {
    #[inline]
    fn reborrow_ref(&self) -> Self::Reborrowed<'_> {
        self
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct AstBuilder<State> {
    state: State,
}

pub enum GenericSpawnContext<R, B: Buffer> {
    Empty(B),
    Related {
        buffer: B,
        related_id: Entity,
        _phantom: PhantomData<R>,
    },
}

pub type SpawnContext<'w, 's, R> = GenericSpawnContext<R, Commands<'w, 's>>;

impl<R, B: Buffer> GenericSpawnContext<R, B> {
    #[inline]
    fn cast_relationship<Next: Relationship, O>(
        self,
        f: impl FnOnce(GenericSpawnContext<Next, B>) -> O,
    ) -> O {
        match self {
            GenericSpawnContext::Empty(x) => f(GenericSpawnContext::Empty(x)),
            GenericSpawnContext::Related {
                buffer,
                related_id,
                _phantom,
            } => f(GenericSpawnContext::Related {
                buffer,
                related_id,
                _phantom: PhantomData,
            }),
        }
    }

    #[inline]
    fn spawn<Next, Clones>(self, bundle: impl Bundle) -> GenericSpawnContext<Next, B>
    where
        R: Relationship,
        Clones: Bundle,
    {
        match self {
            GenericSpawnContext::Empty(buffer) => {
                let (related_id, c) = buffer.spawn(bundle);
                GenericSpawnContext::Related {
                    buffer: c,
                    related_id,
                    _phantom: PhantomData,
                }
            }
            GenericSpawnContext::Related {
                buffer,
                related_id: parent,
                ..
            } => {
                let (related_id, buffer) = buffer.spawn((R::from(parent), bundle));
                let buffer = buffer.clone_specific::<Clones>(parent, related_id);
                GenericSpawnContext::Related {
                    buffer,
                    related_id,
                    _phantom: PhantomData,
                }
            }
        }
    }

    #[inline]
    pub fn reborrow(&mut self) -> GenericSpawnContext<R, B::Reborrowed<'_>> {
        match self {
            GenericSpawnContext::Empty(c) => GenericSpawnContext::Empty(c.reborrow()),
            GenericSpawnContext::Related {
                buffer: commands,
                related_id,
                ..
            } => GenericSpawnContext::Related {
                buffer: commands.reborrow(),
                related_id: *related_id,
                _phantom: PhantomData,
            },
        }
    }

    pub fn reborrow_ref(&self) -> GenericSpawnContext<R, B::Reborrowed<'_>>
    where
        B: RefBuffer,
    {
        match self {
            GenericSpawnContext::Empty(c) => GenericSpawnContext::Empty(c.reborrow_ref()),
            GenericSpawnContext::Related {
                buffer, related_id, ..
            } => GenericSpawnContext::Related {
                buffer: buffer.reborrow_ref(),
                related_id: *related_id,
                _phantom: PhantomData,
            },
        }
    }

    #[inline]
    fn link_with_lexeme(&mut self, spawned: impl Erase<Entity>, node: &impl SyntaxNode) {
        let ptr = ErasedNodePtr::new(node);
        match self {
            GenericSpawnContext::Empty(buffer) | GenericSpawnContext::Related { buffer, .. } => {
                let buffer = buffer.reborrow();
                buffer.insert(spawned.erase(), Lexeme(LexemeId::from(ptr)));
            }
        }
    }
}

impl<B: Buffer> GenericSpawnContext<(), B> {
    #[inline]
    pub fn new(buffer: B) -> GenericSpawnContext<ChildOf, B> {
        GenericSpawnContext::Empty(buffer)
    }
}

impl<B: Buffer> GenericSpawnContext<(), B> {
    #[inline]
    pub fn top_level<T: SyntaxNode, U: FromSyntax<T>>(
        node: &T,
        buffer: B,
        source: impl CodeHolder,
    ) -> Result<NodeId<U>, U::Error> {
        let mut spawner = GenericSpawnContext::<ChildOf, _>::Empty(buffer);
        let id = U::from_syntax(node, spawner.reborrow(), source)?;
        spawner.link_with_lexeme(id, node);
        Ok(id)
    }
}

pub struct DispatchContext<B, Root, Tag, Arity>
where
    Tag: Send + Sync + 'static,
    Arity: crate::arity::Arity,
    B: Buffer,
{
    inner: GenericSpawnContext<crate::relationship::ContainedBy<Tag, Arity>, B>,
    _phantom: PhantomData<(Root, Tag, Arity)>,
}

impl<B, Root, Tag, Arity> DispatchContext<B, Root, Tag, Arity>
where
    Tag: Send + Sync + 'static,
    Arity: crate::arity::Arity,
    B: Buffer,
{
    #[inline]
    fn new(value: GenericSpawnContext<crate::relationship::ContainedBy<Tag, Arity>, B>) -> Self {
        Self {
            inner: value,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn reborrow(&mut self) -> DispatchContext<B::Reborrowed<'_>, Root, Tag, Arity> {
        DispatchContext {
            inner: self.inner.reborrow(),
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn forward<T, U>(&mut self, node: &T, source: impl CodeHolder) -> Result<Entity, U::Error>
    where
        Root: HasChild<U, Tag, Arity = Arity>,
        U: ASTNode + FromSyntax<T>,
    {
        U::from_syntax(node, self.inner.reborrow(), source).map(|it| it.entity())
    }

    #[inline]
    pub fn dispatch<'a, T>(
        &mut self,
        node: impl Into<T>,
        source: impl CodeHolder,
    ) -> Result<Entity, T::Error>
    where
        T: Dispatch<'a, Root, Tag, Arity>,
    {
        node.into().dispatch(self.reborrow(), source)
    }
}

impl<B, Tag, Arity> DispatchContext<B, (), Tag, Arity>
where
    Tag: Send + Sync + 'static,
    Arity: crate::arity::Arity,
    B: Buffer,
{
    pub fn from_buffer<Root>(
        buffer: B,
        parent: impl Into<NodeId<Root>>,
    ) -> DispatchContext<B, Root, Tag, Arity> {
        DispatchContext {
            inner: GenericSpawnContext::Related {
                buffer,
                related_id: parent.into().entity(),
                _phantom: PhantomData,
            },
            _phantom: PhantomData,
        }
    }
}

impl AstBuilder<()> {
    #[inline]
    pub fn new<Root>(root: Root) -> AstBuilder<PropsState<Root, (), ()>> {
        AstBuilder {
            state: PropsState {
                root,
                properties: (),
                clones: PhantomData,
            },
        }
    }
}

impl<B: Buffer, Rel: Relationship, Root> Context<Root> for GenericSpawnContext<Rel, B> {
    #[inline]
    fn spawn_builder<P, C>(
        self,
        builder: AstBuilder<PropsState<Root, P, C>>,
    ) -> AstBuilder<impl SpawnedIn<Root>>
    where
        P: Bundle,
        Root: ASTNode,
        C: Bundle,
    {
        let spawner = self.spawn::<Rel, C>(builder.state.into_bundle());
        AstBuilder {
            state: ChildState {
                spawner,
                _phantom: PhantomData,
            },
        }
    }
}

impl<B, R, T, A, U, Node: SyntaxNode> Context<U> for (DispatchContext<B, R, T, A>, &Node)
where
    R: HasChild<U, T, Arity = A>,
    U: ASTNode,
    T: Send + Sync + 'static,
    A: crate::arity::Arity,
    B: Buffer,
{
    #[inline]
    fn spawn_builder<P, C>(
        self,
        builder: AstBuilder<PropsState<U, P, C>>,
    ) -> AstBuilder<impl SpawnedIn<U>>
    where
        P: Bundle,
        C: Bundle,
    {
        let spawner = self
            .0
            .inner
            .spawn::<<R as NodeRelationship<U, T>>::Relationship, C>(builder.state.into_bundle());
        let mut builder = AstBuilder {
            state: ChildState {
                spawner,
                _phantom: PhantomData,
            },
        };
        let id = builder.finish();
        builder.spawner.link_with_lexeme(id, self.1);

        builder
    }
}

impl<B, R, T, A, U> Context<U> for DispatchContext<B, R, T, A>
where
    B: Buffer,
    R: HasChild<U, T, Arity = A>,
    U: ASTNode,
    T: Send + Sync + 'static,
    A: crate::arity::Arity,
{
    #[inline]
    fn spawn_builder<P, C>(
        self,
        builder: AstBuilder<PropsState<U, P, C>>,
    ) -> AstBuilder<impl SpawnedIn<U>>
    where
        P: Bundle,
        C: Bundle,
        U: ASTNode,
    {
        let spawner = self
            .inner
            .spawn::<<R as NodeRelationship<U, T>>::Relationship, (C, Lexeme)>(
                builder.state.into_bundle(),
            );
        AstBuilder {
            state: ChildState {
                spawner,
                _phantom: PhantomData,
            },
        }
    }
}

impl<R, P, C> PropsState<R, P, C> {
    #[inline]
    fn into_bundle(self) -> impl Bundle
    where
        R: ASTNode,
        P: Bundle,
    {
        (
            self.root,
            Node {
                kind: DebugName::type_name::<R>(),
            },
            self.properties,
        )
    }
}

impl<R, P, C> AstBuilder<PropsState<R, P, C>> {
    #[inline]
    pub fn with_property<Prop>(self, property: Prop) -> AstBuilder<PropsState<R, (P, Prop), C>>
    where
        Prop: NodeProperty,
        R: HasProperty<Prop>,
    {
        AstBuilder {
            state: PropsState {
                root: self.state.root,
                properties: (self.state.properties, property),
                clones: PhantomData,
            },
        }
    }

    pub fn clone_property<Prop>(self) -> AstBuilder<PropsState<R, P, (C, Prop)>>
    where
        Prop: NodeProperty,
        R: HasProperty<Prop>,
    {
        AstBuilder {
            state: PropsState {
                root: self.state.root,
                properties: self.state.properties,
                clones: PhantomData,
            },
        }
    }

    #[inline]
    #[allow(private_bounds)]
    pub fn spawn_in(self, spawner: impl Context<R>) -> AstBuilder<impl SpawnedIn<R>>
    where
        P: Bundle,
        C: Bundle,
        R: ASTNode,
    {
        spawner.spawn_builder(self)
    }
}

impl<R, B, Rel> SpawnedIn<R> for ChildState<R, B, Rel>
where
    Rel: Relationship,
    B: Buffer,
{
    type Buffer = B;

    #[inline]
    fn with_child<T, U, Tag>(
        &mut self,
        node: &T,
        source: impl CodeHolder,
    ) -> Result<&mut Self, U::Error>
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        T: SyntaxNode,
    {
        let spawner = &mut self.spawner;

        let id = spawner.reborrow().cast_relationship(|spawner| {
            U::from_syntax::<_, <R as NodeRelationship<U, Tag>>::Relationship>(
                node, spawner, source,
            )
        })?;
        spawner.link_with_lexeme(id, node);
        Ok(self)
    }

    #[inline]
    fn with_children<'i, T, U, Tag>(
        &mut self,
        nodes: impl IntoCommonIter<Item = &'i T>,
        source: impl CodeHolder,
    ) -> Result<&mut Self, U::Error>
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        T: SyntaxNode,
    {
        let spawner = &mut self.spawner;
        for node in nodes.into_iter() {
            let id = spawner.reborrow().cast_relationship(|spawner| {
                U::from_syntax::<_, <R as NodeRelationship<U, Tag>>::Relationship>(
                    node, spawner, source,
                )
            })?;
            spawner.link_with_lexeme(id, node);
        }

        Ok(self)
    }

    #[inline]
    fn with_dispatch<'a, T, Tag, Arity>(
        &mut self,
        node: impl Into<T>,
        source: impl CodeHolder,
    ) -> Result<&mut Self, T::Error>
    where
        T: Dispatch<'a, R, Tag, Arity, Node: SyntaxNode> + Send,
        Arity: crate::arity::Arity,
        Tag: Send + Sync + 'static,
    {
        let spawner = &mut self.spawner;
        let (dispatcher, node) = node.into().split();
        let entity = spawner.reborrow().cast_relationship(|spawner| {
            dispatcher.dispatch(DispatchContext::new(spawner), source)
        })?;
        spawner.link_with_lexeme(entity, node);

        Ok(self)
    }

    #[inline]
    fn with_dispatch_fn<'a, T: SyntaxNode, Tag, Arity, E>(
        &mut self,
        node: &T,
        f: impl FnOnce(&T, DispatchContext<B::Reborrowed<'_>, R, Tag, Arity>) -> Result<Entity, E>,
    ) -> Result<&mut Self, E>
    where
        Tag: Send + Sync + 'static,
        Arity: crate::arity::Arity,
    {
        let spawner = &mut self.spawner;
        let entity = spawner
            .reborrow()
            .cast_relationship(|spawner| f(node, DispatchContext::new(spawner)))?;
        spawner.link_with_lexeme(entity, node);
        Ok(self)
    }

    #[inline]
    fn with_dispatches<'a, T, Tag, Arity>(
        &mut self,
        nodes: impl IntoCommonIter<Item: Into<T>>,
        source: impl CodeHolder,
    ) -> Result<&mut Self, T::Error>
    where
        T: Dispatch<'a, R, Tag, Arity, Node: SyntaxNode>,
        Arity: crate::arity::Arity,
        Tag: Send + Sync + 'static,
    {
        let spawner = &mut self.spawner;
        for node in nodes.into_iter() {
            let (dispatcher, node) = node.into().split();
            let entity = spawner.reborrow().cast_relationship(|spawner| {
                dispatcher.dispatch(DispatchContext::new(spawner), source)
            })?;
            spawner.link_with_lexeme(entity, node);
        }

        Ok(self)
    }

    fn with_manual<Tag, Arity, Result>(
        &mut self,
        f: impl FnOnce(DispatchContext<B::Reborrowed<'_>, R, Tag, Arity>) -> Result,
    ) -> Result
    where
        Tag: Send + Sync + 'static,
        Arity: crate::arity::Arity,
    {
        let spawner = &mut self.spawner;
        spawner
            .reborrow()
            .cast_relationship(|spawner| f(DispatchContext::new(spawner)))
    }

    #[inline]
    fn finish(&self) -> NodeId<R> {
        match self.spawner {
            GenericSpawnContext::Related { related_id, .. } => NodeId::from(related_id),
            GenericSpawnContext::Empty(_) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::arity::Plural;
    use crate::experimental::AstBuilder;
    use crate::prelude::ASTNode;
    use crate::properties::{Lexeme, SourceSpan};
    use crate::resource::rlt::LexemeId;
    use crate::syntax_tree::builder_v3::GenericSpawnContext;
    use crate::syntax_tree::builder_v3::SpawnedIn;
    use crate::syntax_tree::children::HasChild;
    use bevy_ecs::prelude::{ChildOf, Component, Entity, World};
    use kodept_core::code_point::{CodePoint, Span};
    use kodept_rlt::prelude::Literal;
    use kodept_rlt::traversal::ErasedNodePtr;
    use std::convert::Infallible;

    #[derive(Component)]
    struct A;
    #[derive(Component)]
    struct B;

    impl ASTNode for A {}
    impl ASTNode for B {}
    impl HasChild<B, ()> for A {
        type Arity = Plural;
    }

    static NODE: &'static Literal = &Literal::String(CodePoint::new(3, 1));
    static LEXEME: Lexeme = Lexeme(LexemeId::from(ErasedNodePtr::new(NODE)));

    #[test]
    fn test_lexeme_propagation() {
        let mut world = World::new();
        let mut child_id = Entity::PLACEHOLDER;
        let context = GenericSpawnContext::<ChildOf, _>::Empty(&mut world);

        let mut builder = AstBuilder::new(A)
            .with_property(SourceSpan(Span::default()))
            .with_property(Lexeme(LexemeId::PLACEHOLDER))
            .spawn_in(context);

        _ = builder.with_dispatch_fn(NODE, |_, spawner| {
            let id = AstBuilder::new(B)
                .clone_property::<SourceSpan>()
                .spawn_in(spawner)
                .finish_any();
            child_id = id;
            Ok::<_, Infallible>(id)
        });
        drop(builder);

        let lexeme = world.entity(child_id).get::<Lexeme>();
        assert_eq!(lexeme.map(|it| it.0), Some(LEXEME.0));
    }
}
