use std::fmt::Debug;
use std::rc::Rc;

use kodept_core::structure::span::CodeHolder;
use tracing::warn;

use crate::graph::SyntaxTree;
use crate::graph::{GenericASTNode, NodeId};
use crate::rlt_accessor::{ASTFamily, RLTFamily};

pub trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
}

impl<T: crate::graph::Identifiable> Identifiable for T {
    fn get_id(&self) -> NodeId<Self> {
        <Self as crate::graph::Identifiable>::get_id(self)
    }
}

pub trait IdProducer {
    fn next_id<T>(&mut self) -> NodeId<T>;
}

pub trait Linker<'x> {
    fn link_ref<A, B>(&mut self, ast: NodeId<A>, with: B)
    where
        NodeId<A>: Into<ASTFamily>,
        B: Into<RLTFamily<'x>>;

    fn link<A, B>(&mut self, ast: A, with: B) -> A
    where
        A: Identifiable + 'static,
        NodeId<A>: Into<ASTFamily>,
        B: Into<RLTFamily<'x>>,
    {
        self.link_ref(ast.get_id(), with);
        ast
    }

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

    fn tree(&self) -> Rc<SyntaxTree>;
}

#[derive(Debug)]
pub struct LinkGuard<I>(I);

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

pub trait PopulateTree {
    type Output: Into<GenericASTNode>;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output>;
}
