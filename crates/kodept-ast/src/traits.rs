use crate::node_id::NodeId;
use crate::rlt_accessor::{ASTFamily, RLTFamily};
use kodept_core::structure::span::CodeHolder;
use std::arch::asm;
use std::fmt::Debug;
use tracing::warn;

pub trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
}

pub trait IdProducer {
    fn next_id<T>(&mut self) -> NodeId<T>;
}

pub trait Linker<'x> {
    fn link<A, B>(&mut self, ast: A, with: B) -> A
    where
        A: Identifiable + 'static,
        NodeId<A>: Into<ASTFamily>,
        B: Into<RLTFamily<'x>>;

    fn link_existing<A, B>(&mut self, a: A, b: &B) -> A
    where
        A: Identifiable + 'static,
        B: Identifiable + 'static,
        NodeId<A>: Into<ASTFamily>,
        NodeId<B>: Into<ASTFamily>;
}

pub trait Accessor<'a> {
    fn access<A, B>(&self, ast: &A) -> Option<&'a B>
    where
        A: Identifiable + 'static,
        NodeId<A>: Into<ASTFamily>,
        &'a B: TryFrom<RLTFamily<'a>> + 'a;

    fn access_unknown<A>(&self, ast: &A) -> Option<RLTFamily>
    where
        A: Identifiable + 'static,
        NodeId<A>: Into<ASTFamily>;
}

pub trait IntoAst: Sized {
    type Output;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output;
}

pub trait Instantiable {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self;
}

#[derive(Debug)]
pub struct LinkGuard<I>(I);

pub trait NewInstance: Sized {
    type Constructor;

    fn instantiate<P: IdProducer>(init: Self::Constructor, ctx: &mut P) -> LinkGuard<Self>;

    fn guard(self) -> LinkGuard<Self> {
        LinkGuard(self)
    }
}

impl<I> LinkGuard<I>
where
    I: Identifiable + 'static,
    NodeId<I>: Into<ASTFamily>,
{
    pub fn link<'x, L: Linker<'x>>(self, ctx: &mut L, with: &'x RLTFamily<'x>) -> I {
        ctx.link(self.0, with)
    }

    pub fn unlink(self) -> I {
        warn!("Possible missed link to rlt");
        self.0
    }

    pub fn link_with_existing<'x, L: Linker<'x>, B>(self, ctx: &mut L, with: &B) -> I
    where
        B: Identifiable + 'static,
        NodeId<B>: Into<ASTFamily>,
    {
        ctx.link_existing(self.0, with)
    }
}

pub(crate) mod macros {
    #[macro_export]
    macro_rules! impl_identifiable {
        ($($t:ty$(,)*)*) => {
            $(impl $crate::traits::Identifiable for $t {
                fn get_id(&self) -> $crate::node_id::NodeId<Self> {
                    self.id.clone()
                }
            })*
        };
    }
}
