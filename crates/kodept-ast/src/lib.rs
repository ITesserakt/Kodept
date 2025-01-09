mod traits;
pub mod syntax_tree;
pub mod resource;
mod node_id;
pub mod properties;
pub mod macros;
mod utils;

pub mod prelude {
    pub use super::traits::{CodeHolder, ASTNode, FromSyntax, Choose};
    pub use super::node_id::{NodeId, AnyNodeId};
}

pub mod external {
    pub use bevy_ecs::prelude::Component;
}

#[cfg(feature = "interning")]
pub type Str = kodept_core::shared_str::SharedStr;

#[cfg(not(feature = "interning"))]
pub type Str = String;
