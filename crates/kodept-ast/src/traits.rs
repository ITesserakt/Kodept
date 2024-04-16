use std::fmt::Debug;
use std::rc::Weak;

use tracing::warn;

use kodept_core::ConvertibleToRef;
use kodept_core::structure::span::CodeHolder;

use crate::generic_ast_node_map;
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

impl IntoASTFamily for GenericASTNode {
    fn as_member(&self) -> ASTFamily {
        generic_ast_node_map!(self, |x| x.as_member())
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

pub trait Linker {
    fn link_ref<A, B>(&mut self, ast: NodeId<A>, with: &B)
    where
        NodeId<A>: Into<ASTFamily>,
        B: Into<RLTFamily> + Clone;

    fn link<A, B>(&mut self, ast: A, with: &B) -> A
    where
        A: IntoASTFamily,
        B: Into<RLTFamily> + Clone;

    fn link_existing<A: IntoASTFamily>(&mut self, a: A, b: &impl IntoASTFamily) -> A;
}

pub trait Accessor {
    fn access<B>(&self, ast: &impl IntoASTFamily) -> Option<&B>
    where
        RLTFamily: ConvertibleToRef<B>;

    fn access_unknown(&self, ast: &impl IntoASTFamily) -> Option<RLTFamily>;

    fn tree(&self) -> Weak<SyntaxTree>;
}

#[derive(Debug)]
#[repr(transparent)]
pub struct LinkGuard<I>(I);

impl<I: IntoASTFamily> LinkGuard<I> {
    pub fn new(item: I) -> Self {
        Self(item)
    }

    pub fn link<L: Linker>(self, ctx: &mut L, with: &RLTFamily) -> I {
        ctx.link(self.0, with)
    }

    pub fn unlink(self) -> I {
        warn!("Possible missed link to rlt");
        self.0
    }

    pub fn link_with_existing<L: Linker, B>(self, ctx: &mut L, with: &B) -> I
    where
        B: Identifiable + 'static,
        NodeId<B>: Into<ASTFamily>,
    {
        ctx.link_existing(self.0, with)
    }
}

pub(crate) trait PopulateTree {
    type Output: Into<GenericASTNode>;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output>;
}
