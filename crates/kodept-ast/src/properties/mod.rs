use crate::prelude::ASTNode;
use crate::resource::rlt::LexemeId;
use bevy_ecs::prelude::Component;
use derive_more::{Display, From, Into};
use kodept_core::code_point::Span;
use kodept_core::file_name::FileDescriptor;

pub use bevy_ecs::name::Name;

pub trait NodeProperty: Component {}

#[derive(Debug, Component, Display)]
#[component(immutable)]
pub struct Node {
    pub kind: &'static str,
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
#[component(immutable)]
pub struct Root {
    pub associated_file: FileDescriptor,
}

#[derive(Debug, Component, From, Into, Copy, Clone)]
#[component(immutable)]
pub struct Lexeme(pub LexemeId);

#[derive(Debug, Component, Copy, Clone, From, Into, Display)]
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
