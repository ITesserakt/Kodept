use crate::prelude::ASTNode;
use crate::resource::rlt::LexemeId;
use bevy_ecs::prelude::Component;
use derive_more::{Display, From, Into};
use kodept_core::code_point::Span;
use std::any::TypeId;
use std::fmt::{Debug, Display, Formatter};

pub use bevy_ecs::name::Name;
use bevy_utils::prelude::DebugName;
use kodept_rlt::traversal::{ErasedNodePtr, SyntaxNode};

pub trait NodeProperty: Component {}

#[derive(Component, Display)]
#[component(immutable)]
#[display("{name}")]
pub struct Node {
    pub name: DebugName,
    pub kind: TypeId,
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
#[component(immutable)]
pub struct Root;

#[derive(Component, From, Into, Copy, Clone)]
#[component(immutable)]
pub struct Lexeme(pub LexemeId);

#[derive(Component, Copy, Clone, From, Into, Display)]
#[component(immutable)]
pub struct SourceSpan(pub Span);

pub trait HasProperty<Property: NodeProperty>: Sized {}

pub trait RequireProperty<Property: NodeProperty> {}

impl NodeProperty for Node {}
impl NodeProperty for Root {}
impl NodeProperty for Name {}
impl NodeProperty for Lexeme {}
impl NodeProperty for SourceSpan {}

impl<T: ASTNode> RequireProperty<Node> for T {}
impl<T: ASTNode> RequireProperty<Lexeme> for T {}
impl<T: ASTNode> RequireProperty<SourceSpan> for T {}

impl<P: NodeProperty, T: RequireProperty<P>> HasProperty<P> for T {}

impl Node {
    pub fn of<T: 'static>() -> Self {
        Self {
            name: DebugName::type_name::<T>(),
            kind: TypeId::of::<T>(),
        }
    }

    pub fn is<T: ASTNode>(&self) -> bool {
        self.kind == TypeId::of::<T>()
    }
}

impl Lexeme {
    #[inline]
    pub const fn new(value: &impl SyntaxNode) -> Self {
        Self(LexemeId::from(ErasedNodePtr::new(value)))
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl Debug for SourceSpan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}
