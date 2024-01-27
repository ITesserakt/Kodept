use std::fmt::Debug;
use std::rc::Rc;

use kodept_core::ConvertibleTo;
use tracing::warn;

use kodept_core::structure::span::CodeHolder;

use crate::graph::{GenericASTNode, NodeId};
use crate::graph::{SyntaxTree, SyntaxTreeBuilder};
use crate::rlt_accessor::{ASTFamily, RLTFamily};

pub trait IntoASTFamily: Identifiable {
    fn as_member(&self) -> ASTFamily;
}

impl<A> IntoASTFamily for A
where
    A: Identifiable,
    NodeId<A>: Into<ASTFamily>,
{
    fn as_member(&self) -> ASTFamily {
        let id = self.get_id();
        id.into()
    }
}

pub trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
}

impl<T: crate::graph::Identifiable> Identifiable for T {
    fn get_id(&self) -> NodeId<Self> {
        <Self as crate::graph::Identifiable>::get_id(self)
    }
}

pub trait Linker<'x> {
    fn link_ref<A, B>(&mut self, ast: NodeId<A>, with: B)
    where
        NodeId<A>: Into<ASTFamily>,
        B: Into<RLTFamily<'x>>;

    fn link<A, B>(&mut self, ast: A, with: B) -> A
    where
        A: IntoASTFamily,
        B: Into<RLTFamily<'x>>;

    fn link_existing<A: IntoASTFamily>(&mut self, a: A, b: &impl IntoASTFamily) -> A;
}

pub trait Accessor<'a> {
    fn access<B: 'a>(&self, ast: &impl IntoASTFamily) -> Option<&B>
    where
        RLTFamily<'a>: ConvertibleTo<&'a B>;

    fn access_unknown(&self, ast: &impl IntoASTFamily) -> Option<RLTFamily>;

    fn tree(&self) -> Rc<SyntaxTree>;
}

#[derive(Debug)]
pub struct LinkGuard<I>(I);

impl<I: IntoASTFamily> LinkGuard<I> {
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
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output>;
}
