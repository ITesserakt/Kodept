use crate::prelude::ASTNode;
use crate::resource::rlt::LexemeId;
use bevy_ecs::prelude::Component;
use derive_more::{Display, From, Into};
use kodept_core::code_point::Span;
use kodept_core::file_name::FileDescriptor;
use std::fmt::{Debug, Display, Formatter};

pub use bevy_ecs::name::Name;
use bevy_utils::prelude::DebugName;
use kodept_rlt::traversal::{ErasedNodePtr, SyntaxNode};

pub trait NodeProperty: Component {}

#[derive(Component, Display)]
#[component(immutable)]
pub struct Node {
    pub kind: DebugName,
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
#[component(immutable)]
pub struct Root {
    pub associated_file: FileDescriptor,
}

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
    pub fn of<T>() -> Self {
        Self {
            kind: DebugName::type_name::<T>(),
        }
    }
}

impl Lexeme {
    #[inline]
    pub fn new(value: &impl SyntaxNode) -> Self {
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
