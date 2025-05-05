//! This crate contains actual AST nodes used in Kodept with appropriate
//! conversion implementation from RLT nodes.

#![feature(impl_trait_in_assoc_type)]

use bevy_ecs::relationship::{RelatedSpawner, Relationship};
use kodept_ast::syntax_tree::experimental::BundleUnion;

pub mod block_level;
pub mod code_flow;
pub mod enums;
pub mod expression;
pub mod file;
pub mod function;
pub mod literal;
pub mod properties;
pub mod term;
pub mod top_level;
pub mod types;
mod utils;

enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> BundleUnion for Either<A, B>
where
    A: BundleUnion,
    B: BundleUnion,
{
    fn spawn_with<R: Relationship>(self, spawner: &mut RelatedSpawner<R>) {
        match self {
            Either::Left(x) => x.spawn_with(spawner),
            Either::Right(x) => x.spawn_with(spawner),
        }
    }
}
