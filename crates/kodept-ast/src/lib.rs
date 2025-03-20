//! Provides structures and abstractions for managing AST (abstract syntax tree).
//! Features ECS as an implementation.

use kodept_core::static_assert_size;

mod entity;
pub mod interaction;
pub mod macros;
pub mod properties;
pub mod resource;
pub mod syntax_tree;
mod traits;
mod utils;
mod node_id;

pub mod prelude {
    pub use super::entity::children::TryFromIter;
    pub use super::entity::entity_ref::{AnyNodeRef, AnyNodeRefItem, NodeRef};
    pub use super::entity::traits::{FromEnum, IntoEnum};
    pub use super::traits::{ASTNode, Choose, CodeHolder, FromSyntax};
    pub use super::node_id::{NodeId, Erase};
}

#[deprecated]
pub mod external {
    pub use bevy_ecs::prelude::Component;
}

// TODO: optimize size further by using readonly string
pub type Str = std::borrow::Cow<'static, str>;
static_assert_size!(Str, 24);
