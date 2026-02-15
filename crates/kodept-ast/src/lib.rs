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

pub mod prelude {
    pub use super::node_id::{Erase, NodeId};
    pub use super::traits::{ASTNode, CodeHolder};
    pub use crate::entity::children::{
        ChildrenFetch, HierarchicalQuery, NarrowHierarchicalQuery, NodeQueryData, TryFromIter,
    };
}

#[cfg(feature = "reflection")]
pub fn register_reflection_info(registry: &mut resource::reflection::DebugRegistry) {
    use kodept_ecs::hierarchy::{ChildOf, Children};

    registry.register::<properties::Node>();
    registry.register::<properties::SourceSpan>();
    registry.register::<properties::Name>();
    registry.register::<properties::Root>();
    registry.register::<ChildOf>();
    registry.register::<Children>();
}

pub mod experimental {
    pub use super::traits::{Dispatch, FromSyntax, TransmuteInto};
}

pub type Str = std::borrow::Cow<'static, str>;
