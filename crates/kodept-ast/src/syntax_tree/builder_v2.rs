use super::children::HasChild;
use crate::arity::Arity;
use crate::properties::HasProperty;
use crate::properties::Node;
use crate::properties::NodeProperty;
use crate::relationship::{ContainedBy, Contains, NodeRelationship};
use crate::resource::rlt::{SyntaxResolver, SyntaxVariant};
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
    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn spawn_raw<'s, U, P, C, V>(
        &mut self,
        builder: ASTBuilder<U, P, C>,
        rlt_link: impl Into<SyntaxVariant<'s>>,
        wrap: impl FnOnce(NodeBundle<(U, Node, P, C)>) -> V,
    ) -> V
    where
        R: HasChild<U, Tag, Arity = A>,
        U: ASTNode,
        P: Bundle,
        C: Bundle,
        V: BundleUnion,
    {
        <R as NodeRelationship<U, Tag>>::register();
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
    ) -> V
    where
        R: HasChild<U, Tag, Arity = A>,
        U: ASTNode + FromSyntax<T>,
        V: BundleUnion,
        &'a T: Into<SyntaxVariant<'a>>,
        T: 'static,
    {
        <R as NodeRelationship<U, Tag>>::register();
        let variant = node.into();
        wrap(NodeBundle(U::from_syntax(node, source), unsafe {
            std::mem::transmute(variant)
        }))
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

    #[allow(unsafe_code)]
    pub fn with_child<'a, T, U, Tag>(
        self,
        node: &'a T,
        source: impl CodeHolder,
    ) -> ASTBuilder<R, P, (C, ChildSpawn<T, R, U, Tag>)>
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        &'a T: Into<SyntaxVariant<'a>>,
        T: 'static,
    {
        let child = Self::spawner().spawn(node, source, identity);
        let bundle = RelationTgt::<R, U, Tag>::spawn_one(child);
        ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        }
    }

    pub fn with_dyn_child<T, Tag, A, U, S>(
        self,
        node: T,
        source: S,
        conversion: impl FnOnce(T, &mut NodeSpawner<R, Tag, A>, S) -> U,
    ) -> ASTBuilder<R, P, (C, DynChildrenSpawn<Tag, A>)>
    where
        A: Arity,
        U: BundleUnion,
        Tag: Send + Sync + 'static,
    {
        let child = conversion(node, &mut Self::spawner(), source);
        let closure: DynSpawnFn<ContainedBy<Tag, A>> = Box::new(|spawner| {
            child.spawn_with(spawner);
        });
        let bundle = Contains::<Tag, A>::spawn(SpawnWith(closure));
        ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        }
    }

    #[allow(unsafe_code)]
    pub fn with_children<'a, T, U, Tag>(
        self,
        iter: impl IntoCommonIter<Item = &'a T>,
        source: impl CodeHolder,
    ) -> ASTBuilder<R, P, (C, ChildrenSpawn<T, R, U, Tag>)>
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        &'a T: Into<SyntaxVariant<'a>>,
        T: 'static,
    {
        #[cfg(feature = "parallel")]
        use rayon::prelude::*;

        <R as NodeRelationship<U, Tag>>::register();
        #[cfg(not(feature = "parallel"))]
        let children: ChildrenCollection<NodeBundle<U::Bundle>> = iter
            .into_iter()
            .map(|it| Self::spawner().spawn(it, source, identity))
            .collect();
        #[cfg(feature = "parallel")]
        let children: ChildrenCollection<NodeBundle<U::Bundle>> = iter
            .into_par_iter()
            .map(|it| Self::spawner().spawn(it, source, identity))
            .collect();
        let bundle = RelationTgt::<R, U, Tag>::spawn(SpawnIter(IntoIterator::into_iter(children)));

        ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        }
    }

    #[allow(unsafe_code)]
    pub fn with_dyn_children<'a, I, Tag, A, U>(
        self,
        iter: I,
        conversion: impl Fn(I::Item, &mut NodeSpawner<R, Tag, A>) -> U + Send + Sync,
    ) -> ASTBuilder<R, P, (C, DynChildrenSpawn<Tag, A>)>
    where
        I: IntoCommonIter,
        A: Arity,
        Tag: Send + Sync + 'static,
        U: BundleUnion,
    {
        #[cfg(feature = "parallel")]
        use rayon::prelude::*;

        #[cfg(not(feature = "parallel"))]
        let bundles: ChildrenCollection<U> = iter
            .into_iter()
            .map(|it| conversion(it, &mut Self::spawner()))
            .collect();
        #[cfg(feature = "parallel")]
        let bundles: ChildrenCollection<U> = iter
            .into_par_iter()
            .map(|it| conversion(it, &mut Self::spawner()))
            .collect();
        let closure: DynSpawnFn<ContainedBy<Tag, A>> = Box::new(move |spawner| {
            for bundle in bundles {
                bundle.spawn_with(spawner);
            }
        });
        let bundle = Contains::<Tag, A>::spawn(SpawnWith(closure));
        ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        }
    }

    pub fn with_opt_child<'a, T, U, Tag>(
        self,
        node: Option<&'a T>,
        source: impl CodeHolder,
    ) -> ASTBuilder<
        R,
        P,
        (
            C,
            SpawnRelatedBundle<
                Relation<R, U, Tag>,
                SpawnIter<smallvec::IntoIter<[NodeBundle<U::Bundle>; 1]>>,
            >,
        ),
    >
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        &'a T: Into<SyntaxVariant<'a>>,
        T: 'static,
    {
        let bundles: SmallVec<[_; 1]> = IntoIterator::into_iter(node)
            .map(|it| Self::spawner().spawn(it, source, identity))
            .collect();
        let bundle = RelationTgt::<R, U, Tag>::spawn(SpawnIter(IntoIterator::into_iter(bundles)));
        ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        }
    }

    pub fn with_opt_dyn_children<'a, I, Tag, A, U>(
        self,
        iter: Option<I>,
        conversion: impl Fn(I::Item, &mut NodeSpawner<R, Tag, A>) -> U + Send + Sync,
    ) -> ASTBuilder<R, P, (C, DynChildrenSpawn<Tag, A>)>
    where
        I: IntoCommonIter,
        A: Arity,
        U: BundleUnion,
        Tag: Send + Sync + 'static,
    {
        #[cfg(feature = "parallel")]
        use rayon::prelude::*;

        #[cfg(not(feature = "parallel"))]
        let bundles: Option<ChildrenCollection<U>> = iter.map(|it| {
            it.into_iter()
                .map(|it| conversion(it, &mut Self::spawner()))
                .collect()
        });
        #[cfg(feature = "parallel")]
        let bundles: Option<ChildrenCollection<U>> = iter.map(|it| {
            it.into_par_iter()
                .map(|it| conversion(it, &mut Self::spawner()))
                .collect()
        });
        let closure: DynSpawnFn<ContainedBy<Tag, A>> = Box::new(move |spawner| {
            for bundle in IntoIterator::into_iter(bundles).flatten() {
                bundle.spawn_with(spawner);
            }
        });
        let bundle = Contains::<Tag, A>::spawn(SpawnWith(closure));
        ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        }
    }

    pub fn with_opt_children<'a, T, U, Tag>(
        self,
        iter: Option<impl IntoCommonIter<Item = &'a T>>,
        source: impl CodeHolder,
    ) -> ASTBuilder<
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
    >
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        &'a T: Into<SyntaxVariant<'a>>,
        T: 'static,
    {
        #[cfg(feature = "parallel")]
        use rayon::prelude::*;

        <R as NodeRelationship<U, Tag>>::register();
        #[cfg(not(feature = "parallel"))]
        let children: Option<ChildrenCollection<NodeBundle<U::Bundle>>> = iter.map(|it| {
            it.into_iter()
                .map(|it| Self::spawner().spawn(it, source, identity))
                .collect()
        });
        #[cfg(feature = "parallel")]
        let children: Option<ChildrenCollection<NodeBundle<U::Bundle>>> = iter.map(|it| {
            it.into_par_iter()
                .map(|it| Self::spawner().spawn(it, source, identity))
                .collect()
        });
        let bundle =
            RelationTgt::<R, U, Tag>::spawn(SpawnIter(IntoIterator::into_iter(children).flatten()));

        ASTBuilder {
            root: self.root,
            properties: self.properties,
            children: (self.children, bundle),
        }
    }
}

impl<E: BundleEffect> BundleEffect for NodeLinkEffect<E> {
    #[allow(unsafe_code)]
    fn apply(self, entity: &mut EntityWorldMut) {
        self.other_effect.apply(entity);
        let id = entity.id();
        let rlt = entity.resource_mut::<SyntaxResolver>();
        // SAFETY: link_ptr always belongs to the tree
        unsafe {
            rlt.insert(id, self.link_ptr);
        }
    }
}

impl<T: DynamicBundle> DynamicBundle for NodeBundle<T> {
    type Effect = NodeLinkEffect<T::Effect>;

    fn get_components(self, func: &mut impl FnMut(StorageType, OwningPtr<'_>)) -> Self::Effect {
        NodeLinkEffect {
            other_effect: self.0.get_components(func),
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
    fn component_ids(components: &mut ComponentsRegistrator, ids: &mut impl FnMut(ComponentId)) {
        T::component_ids(components, ids);
    }

    fn get_component_ids(components: &Components, ids: &mut impl FnMut(Option<ComponentId>)) {
        T::get_component_ids(components, ids);
    }

    fn register_required_components(
        _components: &mut ComponentsRegistrator,
        _required_components: &mut RequiredComponents,
    ) {
        T::register_required_components(_components, _required_components)
    }
}

impl<R, P, C> ASTBuilder<R, P, C>
where
    R: ASTNode,
    P: Bundle,
    C: Bundle,
{
    #[allow(unsafe_code)]
    pub fn build(self) -> (R, Node, P, C) {
        (
            self.root,
            Node {
                kind: std::any::type_name::<R>(),
            },
            self.properties,
            self.children,
        )
    }
}
