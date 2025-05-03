mod builder_v2;
pub mod children;
mod export;
mod query;
mod storage;

pub mod prelude {
    pub use super::builder_v2::{ASTBuilder, BundleUnion, NodeSpawner};
    pub use super::query::{ASTQuery, QueryError};
    pub use super::storage::{AST, SourceCode};
}

#[deprecated]
pub mod experimental {
    pub use super::builder_v2::{ASTBuilder, BundleUnion, NodeSpawner};
}
