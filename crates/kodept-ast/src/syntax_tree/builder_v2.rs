use super::children::HasChild;
use crate::arity::Arity;
use crate::properties::Node;
use crate::properties::NodeProperty;
use crate::properties::{HasProperty, Lexeme, SourceSpan};
use crate::relationship::{ContainedBy, Contains, NodeRelationship};
use crate::resource::rlt::{LexemeId, SyntaxResolver, SyntaxVariant};
use crate::traits::ASTNode;
use crate::traits::CodeHolder;
use crate::traits::FromSyntax;
use crate::utils::IntoCommonIter;
use bevy_ecs::bundle::{BundleEffect, DynamicBundle};
use bevy_ecs::component::{
    ComponentId, Components, ComponentsRegistrator, RequiredComponents, StorageType,
};
use bevy_ecs::prelude::*;
use bevy_ecs::ptr::OwningPtr;
use bevy_ecs::relationship::{RelatedSpawner, Relationship};
use bevy_ecs::spawn::SpawnRelatedBundle;
use bevy_ecs::spawn::{SpawnIter, SpawnOneRelated, SpawnWith};
use smallvec::SmallVec;
use std::convert::identity;
use std::marker::PhantomData;

const SMALLVEC_CAPACITY: usize = 1;
#[cfg(not(feature = "parallel"))]
type ChildrenCollection<B, const CAP: usize = SMALLVEC_CAPACITY> = SmallVec<[B; CAP]>;
#[cfg(feature = "parallel")]
type ChildrenCollection<B, const CAP: usize = SMALLVEC_CAPACITY> = Vec<B>;
type Relation<R, C, T> = <R as NodeRelationship<C, T>>::Relationship;
type RelationTgt<R, C, T> = <Relation<R, C, T> as Relationship>::RelationshipTarget;
type ChildSpawn<N, R, C, T> =
    SpawnOneRelated<Relation<R, C, T>, NodeBundle<<C as FromSyntax<N>>::Bundle>>;
type DynSpawnFn<R> = Box<dyn FnOnce(&mut RelatedSpawner<R>) + Send + Sync + 'static>;
type ChildrenSpawn<N, R, C, T, const CAP: usize = SMALLVEC_CAPACITY> = SpawnRelatedBundle<
    Relation<R, C, T>,
    SpawnIter<
        <ChildrenCollection<NodeBundle<<C as FromSyntax<N>>::Bundle>, CAP> as IntoIterator>::IntoIter,
    >,
>;
type DynChildrenSpawn<T, A> =
    SpawnRelatedBundle<ContainedBy<T, A>, SpawnWith<DynSpawnFn<ContainedBy<T, A>>>>;
type DefaultBundle<R, P, C> = (R, Node, Lexeme, P, C);

pub trait BundleUnion: Send + Sync + 'static {
    fn spawn_with<R: Relationship>(self, spawner: &mut RelatedSpawner<R>);
}

impl<B> BundleUnion for B
where
    B: Bundle,
{
    fn spawn_with<R: Relationship>(self, spawner: &mut RelatedSpawner<R>) {
        spawner.spawn(self);
    }
}

pub struct ASTBuilder<Root, Properties, Children> {
    root: Root,
    properties: Properties,
    children: Children,
}

#[derive(Debug)]
pub struct NodeBundle<T>(T, SyntaxVariant<'static>);

pub struct NodeLinkEffect<E> {
    other_effect: E,
    link_ptr: SyntaxVariant<'static>,
}

pub struct NodeSpawner<R, T, A> {
    _phantom: PhantomData<(R, T, A)>,
}

impl<R, Tag, A> NodeSpawner<R, Tag, A>
where
    A: Arity,
{
    #[allow(unsafe_code, private_bounds)]
    #[inline(always)]
    pub fn spawn_raw<'s, U, P, C, V>(
        &mut self,
        builder: ASTBuilder<U, P, C>,
        rlt_link: impl Into<SyntaxVariant<'s>>,
        wrap: impl FnOnce(NodeBundle<DefaultBundle<U, P, C>>) -> V,
    ) -> V
    where
        R: HasChild<U, Tag, Arity = A>,
        U: ASTNode,
        P: Bundle + ContainsProperty<SourceSpan>,
        C: Bundle,
        V: BundleUnion,
    {
        wrap(NodeBundle(builder.build(), unsafe {
            std::mem::transmute(rlt_link.into())
        }))
    }

    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn spawn<'a, T, U, V>(
        &mut self,
        node: &'a T,
        source: impl CodeHolder,
        wrap: impl FnOnce(NodeBundle<U::Bundle>) -> V,
    ) -> Result<V, U::Error>
    where
        R: HasChild<U, Tag, Arity = A>,
        U: ASTNode + FromSyntax<T>,
        V: BundleUnion,
        &'a T: Into<SyntaxVariant<'a>>,
        T: 'static,
    {
        let variant = node.into();
        Ok(wrap(NodeBundle(U::from_syntax(node, source)?, unsafe {
            std::mem::transmute(variant)
        })))
    }

    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<R> ASTBuilder<R, (), ()> {
    pub fn new(root: R) -> Self {
        Self {
            children: (),
            root,
            properties: (),
        }
    }
}

impl<R, P, C> ASTBuilder<R, P, C> {
    const fn spawner<Tag, A: Arity>() -> NodeSpawner<R, Tag, A> {
        NodeSpawner::new()
    }

    pub fn with_property<Prop>(self, property: Prop) -> ASTBuilder<R, (P, Prop), C>
    where
        Prop: NodeProperty,
        R: HasProperty<Prop>,
    {
        ASTBuilder {
            root: self.root,
            properties: (self.properties, property),
            children: self.children,
        }
    }

    pub fn with_child<'a, T, U, Tag>(
        self,
        node: &'a T,
        source: impl CodeHolder,
    ) -> Result<ASTBuilder<R, P, (C, ChildSpawn<T, R, U, Tag>)>, U::Error>
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        &'a T: Into<SyntaxVariant<'a>>,
        T: 'static,
    {
        let child = Self::spawner().spawn(node, source, identity)?;
        let bundle = RelationTgt::<R, U, Tag>::spawn_one(child);
        Ok(ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        })
    }

    pub fn with_dyn_child<T, Tag, A, U, S, E>(
        self,
        node: T,
        source: S,
        conversion: impl FnOnce(T, &mut NodeSpawner<R, Tag, A>, S) -> Result<U, E>,
    ) -> Result<ASTBuilder<R, P, (C, DynChildrenSpawn<Tag, A>)>, E>
    where
        A: Arity,
        U: BundleUnion,
        Tag: Send + Sync + 'static,
    {
        let child = conversion(node, &mut Self::spawner(), source)?;
        let closure: DynSpawnFn<ContainedBy<Tag, A>> = Box::new(|spawner| {
            child.spawn_with(spawner);
        });
        let bundle = Contains::<Tag, A>::spawn(SpawnWith(closure));
        Ok(ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        })
    }

    #[allow(unsafe_code)]
    pub fn with_children<'a, T, U, Tag>(
        self,
        iter: impl IntoCommonIter<Item = &'a T>,
        source: impl CodeHolder,
    ) -> Result<ASTBuilder<R, P, (C, ChildrenSpawn<T, R, U, Tag>)>, U::Error>
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        &'a T: Into<SyntaxVariant<'a>>,
        T: 'static,
    {
        #[cfg(feature = "parallel")]
        use rayon::prelude::*;

        #[cfg(not(feature = "parallel"))]
        let children: ChildrenCollection<NodeBundle<U::Bundle>> = iter
            .into_iter()
            .map(|it| Self::spawner().spawn(it, source, identity))
            .collect::<Result<_, _>>()?;
        #[cfg(feature = "parallel")]
        let children: ChildrenCollection<NodeBundle<U::Bundle>> = iter
            .into_par_iter()
            .map(|it| Self::spawner().spawn(it, source, identity))
            .collect::<Result<_, _>>()?;
        let bundle = RelationTgt::<R, U, Tag>::spawn(SpawnIter(IntoIterator::into_iter(children)));

        Ok(ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        })
    }

    pub fn with_dyn_children<'a, I, E, Tag, A, U>(
        self,
        iter: I,
        conversion: impl Fn(I::Item, &mut NodeSpawner<R, Tag, A>) -> Result<U, E> + Send + Sync,
    ) -> Result<ASTBuilder<R, P, (C, DynChildrenSpawn<Tag, A>)>, E>
    where
        I: IntoCommonIter,
        A: Arity,
        Tag: Send + Sync + 'static,
        U: BundleUnion,
        E: Send,
    {
        #[cfg(feature = "parallel")]
        use rayon::prelude::*;

        #[cfg(not(feature = "parallel"))]
        let bundles: ChildrenCollection<U> = iter
            .into_iter()
            .map(|it| conversion(it, &mut Self::spawner()))
            .collect::<Result<_, _>>()?;
        #[cfg(feature = "parallel")]
        let bundles: ChildrenCollection<U> = iter
            .into_par_iter()
            .map(|it| conversion(it, &mut Self::spawner()))
            .collect::<Result<_, _>>()?;
        let closure: DynSpawnFn<ContainedBy<Tag, A>> = Box::new(move |spawner| {
            for bundle in bundles {
                bundle.spawn_with(spawner);
            }
        });
        let bundle = Contains::<Tag, A>::spawn(SpawnWith(closure));
        Ok(ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        })
    }

    pub fn with_opt_child<'a, T, U, Tag>(
        self,
        node: Option<&'a T>,
        source: impl CodeHolder,
    ) -> Result<
        ASTBuilder<
            R,
            P,
            (
                C,
                SpawnRelatedBundle<
                    Relation<R, U, Tag>,
                    SpawnIter<smallvec::IntoIter<[NodeBundle<U::Bundle>; 1]>>,
                >,
            ),
        >,
        U::Error,
    >
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        &'a T: Into<SyntaxVariant<'a>>,
        T: 'static,
    {
        let bundles: SmallVec<[_; 1]> = IntoIterator::into_iter(node)
            .map(|it| Self::spawner().spawn(it, source, identity))
            .collect::<Result<_, _>>()?;
        let bundle = RelationTgt::<R, U, Tag>::spawn(SpawnIter(IntoIterator::into_iter(bundles)));
        Ok(ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        })
    }

    pub fn with_opt_dyn_child<T, Tag, A, U, S>(
        self,
        node: Option<T>,
        source: S,
        conversion: impl FnOnce(T, &mut NodeSpawner<R, Tag, A>, S) -> U,
    ) -> ASTBuilder<R, P, (C, DynChildrenSpawn<Tag, A>)>
    where
        A: Arity,
        U: BundleUnion,
        Tag: Send + Sync + 'static,
    {
        let child = node.map(|it| conversion(it, &mut Self::spawner(), source));
        let closure: DynSpawnFn<ContainedBy<Tag, A>> = Box::new(|spawner| {
            child.map(|it| it.spawn_with(spawner));
        });
        let bundle = Contains::<Tag, A>::spawn(SpawnWith(closure));
        ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        }
    }

    pub fn with_opt_dyn_children<'a, I, Tag, A, U, E>(
        self,
        iter: Option<I>,
        conversion: impl Fn(I::Item, &mut NodeSpawner<R, Tag, A>) -> Result<U, E> + Send + Sync,
    ) -> Result<ASTBuilder<R, P, (C, DynChildrenSpawn<Tag, A>)>, E>
    where
        I: IntoCommonIter,
        A: Arity,
        U: BundleUnion,
        Tag: Send + Sync + 'static,
        E: Send,
    {
        #[cfg(feature = "parallel")]
        use rayon::prelude::*;

        #[cfg(not(feature = "parallel"))]
        let bundles: Option<ChildrenCollection<U>> = match iter {
            Some(it) => Some(
                it.into_iter()
                    .map(|it| conversion(it, &mut Self::spawner()))
                    .collect::<Result<_, _>>()?,
            ),
            None => None,
        };
        #[cfg(feature = "parallel")]
        let bundles: Option<ChildrenCollection<U>> = match iter {
            Some(it) => Some(
                it.into_par_iter()
                    .map(|it| conversion(it, &mut Self::spawner()))
                    .collect::<Result<_, _>>()?,
            ),
            None => None,
        };
        let closure: DynSpawnFn<ContainedBy<Tag, A>> = Box::new(move |spawner| {
            for bundle in IntoIterator::into_iter(bundles).flatten() {
                bundle.spawn_with(spawner);
            }
        });
        let bundle = Contains::<Tag, A>::spawn(SpawnWith(closure));
        Ok(ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        })
    }

    pub fn with_opt_children<'a, T, U, Tag>(
        self,
        iter: Option<impl IntoCommonIter<Item = &'a T>>,
        source: impl CodeHolder,
    ) -> Result<
        ASTBuilder<
            R,
            P,
            (
                C,
                SpawnRelatedBundle<
                    Relation<R, U, Tag>,
                    SpawnIter<
                        std::iter::Flatten<
                            std::option::IntoIter<
                                ChildrenCollection<
                                    NodeBundle<<U as FromSyntax<T>>::Bundle>,
                                    SMALLVEC_CAPACITY,
                                >,
                            >,
                        >,
                    >,
                >,
            ),
        >,
        U::Error,
    >
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        &'a T: Into<SyntaxVariant<'a>>,
        T: 'static,
    {
        #[cfg(feature = "parallel")]
        use rayon::prelude::*;

        #[cfg(not(feature = "parallel"))]
        let children: Option<ChildrenCollection<NodeBundle<U::Bundle>>> = match iter {
            Some(it) => Some(
                it.into_iter()
                    .map(|it| Self::spawner().spawn(it, source, identity))
                    .collect::<Result<_, _>>()?,
            ),
            None => None,
        };
        #[cfg(feature = "parallel")]
        let children: Option<ChildrenCollection<NodeBundle<U::Bundle>>> = match iter {
            Some(it) => Some(
                it.into_par_iter()
                    .map(|it| Self::spawner().spawn(it, source, identity))
                    .collect::<Result<_, _>>()?,
            ),
            None => None,
        };
        let bundle =
            RelationTgt::<R, U, Tag>::spawn(SpawnIter(IntoIterator::into_iter(children).flatten()));

        Ok(ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        })
    }
}

impl<E: BundleEffect> BundleEffect for NodeLinkEffect<E> {
    #[allow(unsafe_code)]
    fn apply(self, entity: &mut EntityWorldMut) {
        self.other_effect.apply(entity);
        let mut rlt = entity.resource_mut::<SyntaxResolver>();
        // SAFETY: link_ptr always belongs to the tree
        let lexeme = unsafe { rlt.link(self.link_ptr) };
        entity.insert(Lexeme(lexeme));
    }
}

impl<T: DynamicBundle> DynamicBundle for NodeBundle<T> {
    type Effect = NodeLinkEffect<T::Effect>;

    fn get_components(self, func: &mut impl FnMut(StorageType, OwningPtr<'_>)) -> Self::Effect {
        let other_effect = self.0.get_components(func);

        NodeLinkEffect {
            other_effect,
            link_ptr: self.1,
        }
    }
}

// SAFETY:
// - `Bundle::component_ids` calls `ids` for each component type in the
// bundle, in the exact order that `DynamicBundle::get_components` is called.
// - `Bundle::from_components` calls `func` exactly once for each `ComponentId` returned by `Bundle::component_ids`.
// - `Bundle::get_components` is called exactly once for each member. Relies on the above implementation to pass the correct
//   `StorageType` into the callback.
#[allow(unsafe_code)]
unsafe impl<T: Bundle> Bundle for NodeBundle<T> {
    #[inline]
    fn component_ids(components: &mut ComponentsRegistrator, ids: &mut impl FnMut(ComponentId)) {
        T::component_ids(components, ids);
    }

    #[inline]
    fn get_component_ids(components: &Components, ids: &mut impl FnMut(Option<ComponentId>)) {
        T::get_component_ids(components, ids);
    }

    #[inline]
    fn register_required_components(
        _components: &mut ComponentsRegistrator,
        _required_components: &mut RequiredComponents,
    ) {
        T::register_required_components(_components, _required_components);
    }
}

trait ContainsProperty<P: NodeProperty> {}

impl<P: NodeProperty> ContainsProperty<P> for P {}
impl<P: NodeProperty> ContainsProperty<P> for (P,) {}
impl<B, C: ContainsProperty<P>, P: NodeProperty> ContainsProperty<P> for (B, C) {}

#[allow(private_bounds)]
impl<R, P, C> ASTBuilder<R, P, C>
where
    R: ASTNode,
    P: Bundle + ContainsProperty<SourceSpan>,
    C: Bundle,
{
    #[allow(unsafe_code)]
    pub fn build(self) -> DefaultBundle<R, P, C> {
        (
            self.root,
            Node {
                kind: std::any::type_name::<R>(),
            },
            Lexeme(LexemeId::PLACEHOLDER),
            self.properties,
            self.children,
        )
    }
}
