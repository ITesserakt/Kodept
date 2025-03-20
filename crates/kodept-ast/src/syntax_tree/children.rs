use crate::prelude::{ASTNode, CodeHolder, FromSyntax};
use crate::resource::rlt::SyntaxVariant;
use crate::syntax_tree::children::arity::Arity;
use crate::syntax_tree::prelude::{ASTBuilder, Pool};
use bevy_ecs::component::{
    ComponentId, ComponentsRegistrator, Mutable, RequiredComponents, StorageType,
};
use bevy_ecs::prelude::{Component, Entity, RelationshipTarget};
use bevy_ecs::relationship::{Relationship, RelationshipSourceCollection};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub mod arity {
    use bevy_ecs::entity::{VisitEntities, VisitEntitiesMut};
    use bevy_ecs::prelude::Entity;
    use bevy_ecs::relationship::RelationshipSourceCollection;
    use private::Sealed;

    mod private {
        pub trait Sealed {}
    }

    pub trait Arity: Sealed + 'static + Send + Sync {
        type Collection: RelationshipSourceCollection
            + Sync
            + Send
            + VisitEntities
            + VisitEntitiesMut;
    }

    #[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct Option(Entity);

    /// Describes that parent must have single child of that type
    /// One-to-one relationship
    pub struct Singular;

    /// Describes that parent may not have any child of that type
    /// Zero or one-to-one relationship
    pub struct Optional;

    /// Describes that parent may have multiple children of that type
    /// Many-to-one relationship
    pub struct Plural;

    impl Sealed for Singular {}
    impl Arity for Singular {
        type Collection = Entity;
    }
    impl Sealed for Optional {}
    impl Arity for Optional {
        type Collection = Option;
    }
    impl Sealed for Plural {}
    impl Arity for Plural {
        type Collection = Vec<Entity>;
    }

    impl RelationshipSourceCollection for Option {
        type SourceIter<'a> = std::option::IntoIter<Entity>;

        fn with_capacity(_: usize) -> Self {
            Self(Entity::PLACEHOLDER)
        }

        fn add(&mut self, entity: Entity) -> bool {
            if self.0 == Entity::PLACEHOLDER {
                *self = Option(entity);
                true
            } else {
                false
            }
        }

        fn remove(&mut self, entity: Entity) -> bool {
            if self.0 == entity {
                *self = Option(Entity::PLACEHOLDER);
                true
            } else {
                false
            }
        }

        fn iter(&self) -> Self::SourceIter<'_> {
            if self.0 == Entity::PLACEHOLDER {
                Some(self.0).into_iter()
            } else {
                None.into_iter()
            }
        }

        fn len(&self) -> usize {
            if self.0 == Entity::PLACEHOLDER {
                0
            } else {
                1
            }
        }

        fn clear(&mut self) {
            *self = Option(Entity::PLACEHOLDER);
        }
    }

    impl Option {
        pub fn into_inner(self) -> std::option::Option<Entity> {
            if self.0 == Entity::PLACEHOLDER {
                None
            } else {
                Some(self.0)
            }
        }
    }

    impl From<Option> for std::option::Option<Entity> {
        fn from(value: Option) -> Self {
            value.into_inner()
        }
    }

    impl VisitEntities for Option {
        fn visit_entities<F: FnMut(Entity)>(&self, mut f: F) {
            f(self.0)
        }
    }

    impl VisitEntitiesMut for Option {
        fn visit_entities_mut<F: FnMut(&mut Entity)>(&mut self, mut f: F) {
            f(&mut self.0)
        }
    }
}

/// Describes the entity that acts like a parent node for this entity
///
/// This is the source of truth for the relationship,
/// and can be modified directly to change the target
///
/// Type Parameters:
/// * [T] - Associated with this relationship tag
/// * [A] - Degree of relationship
#[derive(Debug)]
pub struct ContainedBy<T, A>(Entity, PhantomData<(T, A)>);

/// Describes all entities that is children for this entity
///
/// Type parameters:
/// * [T] - Associated with this relationship tag
/// * [A] - Degree of relationship
#[derive(Debug)]
pub struct Contains<T, A: Arity>(A::Collection, PhantomData<(T, A)>);

pub trait HasChild<Child, Tag>: NodeRelationship<Child, Tag>
where
    Self: ASTNode,
    Child: ASTNode,
{
    type Arity: Arity;
}

pub trait NodeRelationship<Child, Tag> {
    type Relationship: Relationship<Mutability = Mutable>;
    type RelationshipTarget: RelationshipTarget;
}

impl<T, U, Tag> NodeRelationship<U, Tag> for T
where
    T: HasChild<U, Tag> + ASTNode,
    U: ASTNode,
    Tag: 'static + Send + Sync,
{
    type Relationship = ContainedBy<Tag, T::Arity>;
    type RelationshipTarget = <Self::Relationship as Relationship>::RelationshipTarget;
}

type DynFromSyntax<'w, Source> = dyn FnOnce(SyntaxVariant<'w>, Source, Pool<'w>) -> ASTBuilder<()>;

pub struct ChildrenDisjoint<'p, Root, Source, Arity, Tag = ()>
where
    Source: CodeHolder,
{
    inner: SyntaxVariant<'p>,
    conversion: Box<DynFromSyntax<'p, Source>>,
    _phantom: PhantomData<(Tag, Root, Arity)>,
}

impl<T, A> From<Entity> for ContainedBy<T, A> {
    fn from(value: Entity) -> Self {
        ContainedBy(value, PhantomData)
    }
}

impl<T, A> Deref for ContainedBy<T, A> {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, A> DerefMut for ContainedBy<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, A: Arity, C> AsRef<C> for Contains<T, A>
where
    A::Collection: AsRef<C>,
{
    fn as_ref(&self) -> &C {
        self.0.as_ref()
    }
}

impl<'a, T, A> IntoIterator for &'a Contains<T, A>
where
    A: Arity,
{
    type Item = Entity;
    type IntoIter = <A::Collection as RelationshipSourceCollection>::SourceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T, A> Component for Contains<T, A>
where
    T: Send + Sync + 'static,
    A: Arity,
{
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;
    fn on_replace() -> Option<bevy_ecs::component::ComponentHook> {
        Some(<Self as RelationshipTarget>::on_replace)
    }
    fn register_required_components(
        _requiree: ComponentId,
        components: &mut ComponentsRegistrator,
        _required_components: &mut RequiredComponents,
        _inheritance_depth: u16,
        recursion_check_stack: &mut Vec<ComponentId>,
    ) {
        bevy_ecs::component::enforce_no_required_components_recursion(
            components,
            recursion_check_stack,
        );
        let self_id = components.register_component::<Self>();
        recursion_check_stack.push(self_id);
        recursion_check_stack.pop();
    }
    fn clone_behavior() -> bevy_ecs::component::ComponentCloneBehavior {
        bevy_ecs::component::ComponentCloneBehavior::Custom(
            bevy_ecs::relationship::clone_relationship_target::<Self>,
        )
    }
    fn visit_entities(this: &Self, mut func: impl FnMut(Entity)) {
        use bevy_ecs::entity::VisitEntities;
        this.0.visit_entities(&mut func);
    }
    fn visit_entities_mut(this: &mut Self, mut func: impl FnMut(&mut Entity)) {
        use bevy_ecs::entity::VisitEntitiesMut;
        this.0.visit_entities_mut(&mut func);
    }
}

impl<T, A> Component for ContainedBy<T, A>
where
    T: Send + Sync + 'static,
    A: Arity,
{
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;

    fn on_insert() -> Option<bevy_ecs::component::ComponentHook> {
        Some(<Self as Relationship>::on_insert)
    }
    fn on_replace() -> Option<bevy_ecs::component::ComponentHook> {
        Some(<Self as Relationship>::on_replace)
    }
    fn register_required_components(
        _requiree: ComponentId,
        components: &mut ComponentsRegistrator,
        _required_components: &mut RequiredComponents,
        _inheritance_depth: u16,
        recursion_check_stack: &mut Vec<ComponentId>,
    ) {
        bevy_ecs::component::enforce_no_required_components_recursion(
            components,
            recursion_check_stack,
        );
        let self_id = components.register_component::<Self>();
        recursion_check_stack.push(self_id);
        recursion_check_stack.pop();
    }
    fn clone_behavior() -> bevy_ecs::component::ComponentCloneBehavior {
        use bevy_ecs::component::DefaultCloneBehaviorBase;
        (&&&bevy_ecs::component::DefaultCloneBehaviorSpecialization::<Self>::default())
            .default_clone_behavior()
    }
    fn visit_entities(this: &Self, mut func: impl FnMut(Entity)) {
        use bevy_ecs::entity::VisitEntities;
        this.0.visit_entities(&mut func);
    }
    fn visit_entities_mut(this: &mut Self, mut func: impl FnMut(&mut Entity)) {
        use bevy_ecs::entity::VisitEntitiesMut;
        this.0.visit_entities_mut(&mut func);
    }
}

impl<T, A> Relationship for ContainedBy<T, A>
where
    T: 'static + Send + Sync,
    A: Arity,
{
    type RelationshipTarget = Contains<T, A>;

    fn get(&self) -> Entity {
        self.0
    }

    fn from(entity: Entity) -> Self {
        Self(entity, PhantomData)
    }
}

impl<T, A> RelationshipTarget for Contains<T, A>
where
    T: 'static + Send + Sync,
    A: Arity,
{
    const LINKED_SPAWN: bool = true;
    type Relationship = ContainedBy<T, A>;
    type Collection = A::Collection;

    fn collection(&self) -> &Self::Collection {
        &self.0
    }

    fn collection_mut_risky(&mut self) -> &mut Self::Collection {
        &mut self.0
    }

    fn from_collection_risky(collection: Self::Collection) -> Self {
        Self(collection, PhantomData)
    }
}

impl<'w, Root, Source, Arity, Tag> ChildrenDisjoint<'w, Root, Source, Arity, Tag>
where
    Source: CodeHolder,
{
    #[inline(always)]
    pub fn new<U>(node: &'w U::Syntax) -> Self
    where
        &'w U::Syntax: Into<SyntaxVariant<'w>>,
        SyntaxVariant<'w>: TryInto<&'w U::Syntax, Error: Debug>,
        Root: HasChild<U, Tag, Arity = Arity>,
        U: FromSyntax + ASTNode,
    {
        Self::ad_hoc(node, |node, source, pool| {
            U::from_syntax(node, source, pool)
        })
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    pub(crate) fn call(self, source: Source, pool: Pool<'w>) -> ASTBuilder<()> {
        let part = (self.conversion)(self.inner, source, pool);
        unsafe {
            pool.link_syntax(part.id(), self.inner);
        }
        part
    }

    #[inline(always)]
    pub fn ad_hoc<'a, T, U>(
        node: &'a T,
        f: impl FnOnce(&'w T, Source, Pool<'w>) -> ASTBuilder<U> + 'static,
    ) -> Self
    where
        &'a T: Into<SyntaxVariant<'w>>,
        SyntaxVariant<'w>: TryInto<&'w T, Error: Debug>,
        Root: HasChild<U, Tag, Arity = Arity>,
        U: ASTNode,
        T: 'w,
    {
        Self {
            inner: node.into(),
            conversion: Box::new(move |node, source, pool| {
                let node = node.try_into().unwrap();
                f(node, source, pool).erase()
            }),
            _phantom: PhantomData,
        }
    }
}
