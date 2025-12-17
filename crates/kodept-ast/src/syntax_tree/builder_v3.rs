use crate::{
    node_id::{Erase, NodeId},
    properties::{HasProperty, Lexeme, Node, NodeProperty, SourceSpan},
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
use derive_more::{Deref, DerefMut};
use kodept_rlt::traversal::{ErasedNodePtr, SyntaxNode};
use std::marker::PhantomData;

pub struct PropsState<R, P> {
    root: R,
    properties: P,
}

pub(crate) struct ChildState<'w, 's, R, Inner = ()> {
    spawner: SpawnContext<'w, 's, Inner>,
    _phantom: PhantomData<R>,
}

pub trait SpawnedIn<Root>: Sized {
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
        f: impl FnOnce(&T, DispatchContext<Root, Tag, Arity>) -> Result<Entity, E>,
    ) -> Result<&mut Self, E>;

    fn with_dispatches<'a, T, Tag, Arity>(
        &mut self,
        nodes: impl IntoCommonIter<Item: Into<T>>,
        source: impl CodeHolder,
    ) -> Result<&mut Self, T::Error>
    where
        T: Dispatch<'a, Root, Tag, Arity, Node: SyntaxNode>,
        Arity: crate::arity::Arity,
        Tag: Send + Sync + 'static;

    fn finish(&self) -> NodeId<Root>;

    fn finish_any(&self) -> Entity {
        self.finish().entity()
    }
}

trait Context<U> {
    fn spawn_builder<P, I>(
        self,
        builder: AstBuilder<PropsState<U, P>>,
    ) -> AstBuilder<impl SpawnedIn<U>>
    where
        P: Bundle + Contains<SourceSpan, I>,
        U: ASTNode;
}

pub trait Buffer {
    type Ref<'a>
    where
        Self: 'a;

    fn spawn(this: Self::Ref<'_>, bundle: impl Bundle) -> Entity;
    fn insert(this: Self::Ref<'_>, entity: Entity, bundle: impl Bundle);
}

impl<'w, 's> Buffer for Commands<'w, 's> {
    type Ref<'a>
        = &'a mut Commands<'w, 's>
    where
        Self: 'a;

    fn spawn(this: Self::Ref<'_>, bundle: impl Bundle) -> Entity {
        this.spawn(bundle).id()
    }

    fn insert(this: Self::Ref<'_>, entity: Entity, bundle: impl Bundle) {
        this.entity(entity).insert(bundle);
    }
}

impl Buffer for World {
    type Ref<'a> = &'a mut World;

    fn spawn(this: Self::Ref<'_>, bundle: impl Bundle) -> Entity {
        this.spawn(bundle).id()
    }

    fn insert(this: Self::Ref<'_>, entity: Entity, bundle: impl Bundle) {
        this.entity_mut(entity).insert(bundle);
    }
}

#[cfg(feature = "parallel")]
impl<'w, 's> Buffer for ParallelCommands<'w, 's> {
    type Ref<'a>
        = &'a ParallelCommands<'w, 's>
    where
        Self: 'a;

    fn spawn(this: Self::Ref<'_>, bundle: impl Bundle) -> Entity {
        this.command_scope(move |mut commands| commands.spawn(bundle).id())
    }

    fn insert(this: Self::Ref<'_>, entity: Entity, bundle: impl Bundle) {
        this.command_scope(move |mut commands| {
            commands.entity(entity).insert(bundle);
        })
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
    fn cast_relationship<Next: Relationship>(self) -> GenericSpawnContext<Next, B> {
        match self {
            GenericSpawnContext::Empty(x) => GenericSpawnContext::Empty(x),
            GenericSpawnContext::Related {
                buffer,
                related_id,
                _phantom,
            } => GenericSpawnContext::Related {
                buffer,
                related_id,
                _phantom: PhantomData,
            },
        }
    }
}

impl<'w, 's, R> SpawnContext<'w, 's, R> {
    pub fn reborrow<'a>(&'a mut self) -> SpawnContext<'w, 'a, R> {
        match self {
            SpawnContext::Empty(c) => SpawnContext::Empty(c.reborrow()),
            SpawnContext::Related {
                buffer: commands,
                related_id,
                ..
            } => SpawnContext::Related {
                buffer: commands.reborrow(),
                related_id: *related_id,
                _phantom: PhantomData,
            },
        }
    }

    #[inline]
    pub(crate) fn spawn<Next>(self, bundle: impl Bundle) -> SpawnContext<'w, 's, Next>
    where
        R: Relationship,
    {
        match self {
            SpawnContext::Empty(mut c) => {
                let entity = c.spawn(bundle);
                let related_id = entity.id();
                SpawnContext::Related {
                    buffer: c,
                    related_id,
                    _phantom: PhantomData,
                }
            }
            SpawnContext::Related {
                buffer: mut commands,
                related_id,
                ..
            } => {
                let entity = commands.spawn((R::from(related_id), bundle));
                let related_id = entity.id();
                SpawnContext::Related {
                    buffer: commands,
                    related_id,
                    _phantom: PhantomData,
                }
            }
        }
    }

    fn link_with_lexeme(&mut self, spawned: impl Erase<Entity>, node: &impl SyntaxNode) {
        let ptr = ErasedNodePtr::new(node);
        match self {
            SpawnContext::Empty(commands)
            | SpawnContext::Related {
                buffer: commands, ..
            } => {
                commands
                    .entity(spawned.erase())
                    .insert(Lexeme(LexemeId::from(ptr)));
            }
        }
    }
}

impl<'w, 's> SpawnContext<'w, 's, ()> {
    pub fn top_level<T: SyntaxNode, R: FromSyntax<T>>(
        node: &T,
        commands: Commands,
        source: impl CodeHolder,
    ) -> Result<NodeId<R>, R::Error> {
        let mut spawner = SpawnContext::<ChildOf>::Empty(commands);
        let id = R::from_syntax(node, spawner.reborrow(), source)?;
        spawner.link_with_lexeme(id, node);
        Ok(id)
    }
}

pub struct DispatchContext<'w, 's, Root, Tag, Arity> {
    inner: SpawnContext<'w, 's, ChildOf>,
    _phantom: PhantomData<(Root, Tag, Arity)>,
}

impl<'w, 's, Root, Tag, Arity> DispatchContext<'w, 's, Root, Tag, Arity> {
    fn new(value: SpawnContext<'w, 's, ChildOf>) -> Self {
        Self {
            inner: value,
            _phantom: PhantomData,
        }
    }

    pub fn reborrow<'a>(&'a mut self) -> DispatchContext<'w, 'a, Root, Tag, Arity> {
        DispatchContext {
            inner: self.inner.reborrow(),
            _phantom: PhantomData,
        }
    }

    pub fn forward<T, U>(&mut self, node: &T, source: impl CodeHolder) -> Result<Entity, U::Error>
    where
        Root: HasChild<U, Tag, Arity = Arity>,
        U: ASTNode + FromSyntax<T>,
    {
        U::from_syntax(node, self.inner.reborrow(), source).map(|it| it.entity())
    }

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

impl AstBuilder<()> {
    pub fn new<Root>(root: Root) -> AstBuilder<PropsState<Root, ()>> {
        AstBuilder {
            state: PropsState {
                root,
                properties: (),
            },
        }
    }
}

trait Contains<T, I> {}
pub struct Here;
pub struct There<I>(PhantomData<I>);

impl<T, Tail> Contains<T, Here> for (Tail, T) {}
impl<Head, Tail, FromTail, TailIndex> Contains<FromTail, There<TailIndex>> for (Tail, Head) where
    Tail: Contains<FromTail, TailIndex>
{
}

impl<'w, 's, Rel: Relationship, Root> Context<Root> for SpawnContext<'w, 's, Rel> {
    #[inline]
    fn spawn_builder<P, I>(
        self,
        builder: AstBuilder<PropsState<Root, P>>,
    ) -> AstBuilder<impl SpawnedIn<Root>>
    where
        P: Bundle + Contains<SourceSpan, I>,
        Root: ASTNode,
    {
        let spawner = self.spawn::<Rel>(builder.state.into_bundle());
        AstBuilder {
            state: ChildState {
                spawner,
                _phantom: PhantomData,
            },
        }
    }
}

impl<'w, 's, R, T, A, U, Node: SyntaxNode> Context<U> for (DispatchContext<'w, 's, R, T, A>, &Node)
where
    R: HasChild<U, T, Arity = A>,
    U: ASTNode,
{
    #[inline]
    fn spawn_builder<P, I>(
        self,
        builder: AstBuilder<PropsState<U, P>>,
    ) -> AstBuilder<impl SpawnedIn<U>>
    where
        P: Bundle + Contains<SourceSpan, I>,
    {
        let spawner = self.0.inner.spawn::<ChildOf>(builder.state.into_bundle());
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

impl<R, P> PropsState<R, P> {
    fn into_bundle(self) -> impl Bundle
    where
        R: ASTNode,
        P: Bundle,
    {
        (
            self.root,
            Node {
                kind: std::any::type_name::<R>(),
            },
            Lexeme(LexemeId::PLACEHOLDER),
            self.properties,
        )
    }
}

impl<R, P> AstBuilder<PropsState<R, P>> {
    pub fn with_property<Prop>(self, property: Prop) -> AstBuilder<PropsState<R, (P, Prop)>>
    where
        Prop: NodeProperty,
        R: HasProperty<Prop>,
    {
        AstBuilder {
            state: PropsState {
                root: self.state.root,
                properties: (self.state.properties, property),
            },
        }
    }

    #[allow(private_bounds)]
    pub fn spawn_in<I>(self, spawner: impl Context<R>) -> AstBuilder<impl SpawnedIn<R>>
    where
        P: Bundle + Contains<SourceSpan, I>,
        R: ASTNode,
    {
        spawner.spawn_builder(self)
    }
}

impl<R, Rel: Relationship> SpawnedIn<R> for ChildState<'_, '_, R, Rel> {
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
        let id = U::from_syntax(
            node,
            spawner
                .reborrow()
                .cast_relationship::<<R as NodeRelationship<U, Tag>>::Relationship>(),
            source,
        )?;
        spawner.link_with_lexeme(id, node);
        Ok(self)
    }

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
            let id = U::from_syntax(
                node,
                spawner
                    .reborrow()
                    .cast_relationship::<<R as NodeRelationship<U, Tag>>::Relationship>(),
                source,
            )?;
            spawner.link_with_lexeme(id, node);
        }

        Ok(self)
    }

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
        let entity = dispatcher.dispatch(
            DispatchContext::new(spawner.reborrow().cast_relationship()),
            source,
        )?;
        spawner.link_with_lexeme(entity, node);

        Ok(self)
    }

    fn with_dispatch_fn<'a, T: SyntaxNode, Tag, Arity, E>(
        &mut self,
        node: &T,
        f: impl FnOnce(&T, DispatchContext<R, Tag, Arity>) -> Result<Entity, E>,
    ) -> Result<&mut Self, E> {
        let spawner = &mut self.spawner;
        let entity = f(
            node,
            DispatchContext::new(spawner.reborrow().cast_relationship()),
        )?;
        spawner.link_with_lexeme(entity, node);
        Ok(self)
    }

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
            let entity = dispatcher.dispatch(
                DispatchContext::new(spawner.reborrow().cast_relationship()),
                source,
            )?;
            spawner.link_with_lexeme(entity, node);
        }

        Ok(self)
    }

    fn finish(&self) -> NodeId<R> {
        match self.spawner {
            SpawnContext::Related { related_id, .. } => NodeId::from(related_id),
            SpawnContext::Empty(_) => unreachable!(),
        }
    }
}
