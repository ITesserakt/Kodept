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

impl<A, B> Either<A, B> {
    #[inline(always)]
    pub(crate) fn v21(value: A) -> Self {
        Self::Left(value)
    }

    #[inline(always)]
    pub(crate) fn v22(value: B) -> Self {
        Self::Right(value)
    }
}

impl<A, B, C> Either<Either<A, B>, C> {
    #[inline(always)]
    pub(crate) fn v31(value: A) -> Self {
        Either::Left(Either::Left(value))
    }

    #[inline(always)]
    pub(crate) fn v32(value: B) -> Self {
        Either::Left(Either::Right(value))
    }

    #[inline(always)]
    pub(crate) fn v33(value: C) -> Self {
        Either::Right(value)
    }
}

impl<A, B, C, D> Either<Either<A, B>, Either<C, D>> {
    #[inline(always)]
    pub(crate) fn v41(value: A) -> Self {
        Either::Left(Either::Left(value))
    }

    #[inline(always)]
    pub(crate) fn v42(value: B) -> Self {
        Either::Left(Either::Right(value))
    }

    #[inline(always)]
    pub(crate) fn v43(value: C) -> Self {
        Either::Right(Either::Left(value))
    }

    #[inline(always)]
    pub(crate) fn v44(value: D) -> Self {
        Either::Right(Either::Right(value))
    }
}

#[allow(dead_code)]
impl<A, B, C, D, E> Either<Either<Either<A, B>, Either<C, D>>, E> {
    #[inline(always)]
    pub(crate) fn v51(value: A) -> Self {
        Either::Left(Either::Left(Either::Left(value)))
    }

    #[inline(always)]
    pub(crate) fn v52(value: B) -> Self {
        Either::Left(Either::Left(Either::Right(value)))
    }

    #[inline(always)]
    pub(crate) fn v53(value: C) -> Self {
        Either::Left(Either::Right(Either::Left(value)))
    }

    #[inline(always)]
    pub(crate) fn v54(value: D) -> Self {
        Either::Left(Either::Right(Either::Right(value)))
    }

    #[inline(always)]
    pub(crate) fn v55(value: E) -> Self {
        Either::Right(value)
    }
}

impl<A, B, C, D, E, F> Either<Either<Either<A, B>, Either<C, D>>, Either<E, F>> {
    #[inline(always)]
    pub(crate) fn v61(value: A) -> Self {
        Either::Left(Either::Left(Either::Left(value)))
    }

    #[inline(always)]
    pub(crate) fn v62(value: B) -> Self {
        Either::Left(Either::Left(Either::Right(value)))
    }

    #[inline(always)]
    pub(crate) fn v63(value: C) -> Self {
        Either::Left(Either::Right(Either::Left(value)))
    }

    #[inline(always)]
    pub(crate) fn v64(value: D) -> Self {
        Either::Left(Either::Right(Either::Right(value)))
    }

    #[inline(always)]
    pub(crate) fn v65(value: E) -> Self {
        Either::Right(Either::Left(value))
    }

    #[inline(always)]
    pub(crate) fn v66(value: F) -> Self {
        Either::Right(Either::Right(value))
    }
}
