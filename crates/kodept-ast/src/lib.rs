//! Provides structures and abstractions for managing AST (abstract syntax tree).
//! Features ECS as an implementation.

use kodept_core::static_assert_size;

pub mod macros;
mod node_id;
pub mod properties;
pub mod resource;
pub mod syntax_tree;
mod traits;
mod utils;

pub mod prelude {
    pub use super::node_id::{AnyNodeId, NodeId};
    pub use super::traits::{ASTNode, Choose, CodeHolder, FromSyntax};
}

pub mod external {
    pub use bevy_ecs::prelude::Component;
}

// TODO: optimize size further by using readonly string
pub type Str = std::borrow::Cow<'static, str>;
static_assert_size!(Str, 24);
