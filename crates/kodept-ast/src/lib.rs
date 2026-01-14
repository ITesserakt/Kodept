//! Provides structures and abstractions for managing AST (abstract syntax tree).
//! Features ECS as an implementation.

pub mod arity;
mod entity;
pub mod macros;
mod node_id;
pub mod properties;
pub mod relationship;
pub mod resource;
pub mod syntax_tree;
mod traits;
mod utils;

pub mod prelude {
    pub use super::entity::children::TryFromIter;
    pub use super::entity::entity_ref::{AnyNodeRef, AnyNodeRefItem, NodeRef};
    pub use super::entity::traits::{FromEnum, IntoEnum};
    pub use super::node_id::{Erase, NodeId};
    pub use super::traits::{ASTNode, CodeHolder};
    pub use crate::entity::children::{HierarchicalQuery, ChildrenBetween};
    pub use crate::entity::properties::PropertyQuery;
}

#[cfg(feature = "reflection")]
pub fn register_reflection_info(registry: &mut resource::reflection::DebugRegistry) {
    use bevy_ecs::prelude::{ChildOf, Children};
    
    registry.register::<properties::Lexeme>();
    registry.register::<properties::Node>();
    registry.register::<properties::SourceSpan>();
    registry.register::<properties::Name>();
    registry.register::<properties::Root>();
    registry.register::<ChildOf>();
    registry.register::<Children>();
}

pub mod experimental {
    pub use super::syntax_tree::experimental::{AstBuilder, SpawnContext, DispatchContext};
    pub use super::traits::{FromSyntax, Dispatch, SplitRef};
}

#[deprecated]
pub mod external {
    pub use bevy_ecs::prelude::Component;
}

pub type Str = std::borrow::Cow<'static, str>;
