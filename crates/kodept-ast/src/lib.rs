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

#[cfg(feature = "interning")]
pub type Str = kodept_core::shared_str::SharedStr;

#[cfg(not(feature = "interning"))]
pub type Str = String;
