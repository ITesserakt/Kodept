use super::children::HasChild;
use crate::arity::Arity;
use crate::properties::Node;
use crate::properties::NodeProperty;
use crate::properties::{HasProperty, Lexeme, SourceSpan};
use crate::relationship::{ContainedBy, Contains, NodeRelationship};
use crate::resource::rlt::{LexemeId, SyntaxResolver};
use crate::traits::ASTNode;
use crate::traits::CodeHolder;
use crate::traits::FromSyntax;
use crate::utils::IntoCommonIter;
use bevy_ecs::bundle::DynamicBundle;
use bevy_ecs::component::{ComponentId, Components, ComponentsRegistrator, StorageType};
use bevy_ecs::prelude::*;
use bevy_ecs::ptr::{MovingPtr, OwningPtr};
use bevy_ecs::relationship::{RelatedSpawner, Relationship};
use bevy_ecs::spawn::SpawnRelatedBundle;
use bevy_ecs::spawn::{SpawnIter, SpawnOneRelated, SpawnWith};
use kodept_rlt::traversal::{ErasedNodePtr, SyntaxNode};
use smallvec::SmallVec;
use std::convert::identity;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

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
pub struct NodeBundle<T>(T, ErasedNodePtr);

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
        rlt_link: &impl SyntaxNode,
        wrap: impl FnOnce(NodeBundle<DefaultBundle<U, P, C>>) -> V,
    ) -> V
    where
        R: HasChild<U, Tag, Arity = A>,
        U: ASTNode,
        P: Bundle + ContainsProperty<SourceSpan>,
        C: Bundle,
        V: BundleUnion,
    {
        wrap(NodeBundle(builder.build(), ErasedNodePtr::new(rlt_link)))
    }

    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn spawn<T, U, V>(
        &mut self,
        node: &T,
        source: impl CodeHolder,
        wrap: impl FnOnce(NodeBundle<U::Bundle>) -> V,
    ) -> Result<V, U::Error>
    where
        R: HasChild<U, Tag, Arity = A>,
        U: ASTNode + FromSyntax<T>,
        V: BundleUnion,
        T: SyntaxNode,
    {
        Ok(wrap(NodeBundle(
            U::from_syntax(node, source)?,
            ErasedNodePtr::new(node),
        )))
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

    pub fn with_child<T, U, Tag>(
        self,
        node: &T,
        source: impl CodeHolder,
    ) -> Result<ASTBuilder<R, P, (C, ChildSpawn<T, R, U, Tag>)>, U::Error>
    where
        R: HasChild<U, Tag>,
        U: ASTNode + FromSyntax<T>,
        T: SyntaxNode,
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
        T: SyntaxNode,
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

    pub fn with_opt_child<T, U, Tag>(
        self,
        node: Option<&T>,
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
        T: SyntaxNode,
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
        T: SyntaxNode,
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

#[allow(unsafe_code)]
impl<T: Bundle> DynamicBundle for NodeBundle<T> {
    type Effect = ();

    unsafe fn get_components(
        ptr: MovingPtr<'_, Self>,
        func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    ) {
        ptr.partial_move(|it| unsafe {
            bevy_ecs::ptr::deconstruct_moving_ptr!({
                let NodeBundle {
                    0: bundle,
                    1: syntax,
                } = it;
            });
            <T as DynamicBundle>::get_components(bundle, func);
            std::mem::forget(syntax);
        });
    }

    unsafe fn apply_effect(ptr: MovingPtr<'_, MaybeUninit<Self>>, entity: &mut EntityWorldMut) {
        bevy_ecs::ptr::deconstruct_moving_ptr!({
            let MaybeUninit::<NodeBundle> {
                0: bundle,
                1: syntax,
            } = ptr;
        });
        unsafe {
            <T as DynamicBundle>::apply_effect(bundle, entity);
        }
        let syntax = unsafe { syntax.assume_init() }.read();
        let resolver = entity.resource::<SyntaxResolver>();
        entity.insert(Lexeme(resolver.get_id_for_ptr(&syntax)));
    }
}

#[allow(unsafe_code)]
unsafe impl<T: Bundle> Bundle for NodeBundle<T> {
    fn component_ids(components: &mut ComponentsRegistrator, ids: &mut impl FnMut(ComponentId)) {
        <T as Bundle>::component_ids(components, ids);
    }

    fn get_component_ids(components: &Components, ids: &mut impl FnMut(Option<ComponentId>)) {
        <T as Bundle>::get_component_ids(components, ids);
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
