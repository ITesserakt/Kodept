//! Provides structures and abstractions for managing AST (abstract syntax tree).
//! Features ECS as an implementation.

pub mod arity;
mod entity;
pub mod macros;
mod node_id;
pub mod properties;
mod relationship;
pub mod resource;
pub mod syntax_tree;
mod traits;
mod utils;

pub mod prelude {
    pub use super::entity::children::TryFromIter;
    pub use super::entity::entity_ref::{AnyNodeRef, AnyNodeRefItem, NodeRef};
    pub use super::entity::traits::{FromEnum, IntoEnum};
    pub use super::node_id::{Erase, NodeId};
    pub use super::relationship::{ChildOf, Children};
    pub use super::traits::{ASTNode, CodeHolder, FromSyntax};
    pub use crate::entity::children::{HierarchicalQuery, ChildrenBetween};
    pub use crate::entity::properties::PropertyQuery;
}

#[deprecated]
pub mod external {
    pub use bevy_ecs::prelude::Component;
}

pub type Str = std::borrow::Cow<'static, str>;