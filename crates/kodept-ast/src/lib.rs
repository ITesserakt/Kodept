//! Provides structures and abstractions for managing AST (abstract syntax tree).
//! Features ECS as an implementation.

use kodept_core::static_assert_size;

pub mod arity;
mod entity;
pub mod interaction;
pub mod macros;
mod node_id;
pub mod properties;
pub mod query;
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
    pub use super::traits::{ASTNode, CodeHolder, FromSyntax};
}

#[deprecated]
pub mod external {
    pub use bevy_ecs::prelude::Component;
}

// TODO: optimize size further by using readonly string
pub type Str = std::borrow::Cow<'static, str>;
static_assert_size!(Str, 24);
