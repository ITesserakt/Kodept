use std::rc::Weak;

use kodept_core::ConvertibleToRef;
use kodept_core::structure::span::CodeHolder;

use crate::graph::{AnyNode, NodeId};
use crate::graph::{SyntaxTree, SyntaxTreeBuilder};
use crate::rlt_accessor::{RLTFamily};

pub trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
}

impl<T: crate::graph::Identifiable> Identifiable for T {
    fn get_id(&self) -> NodeId<Self> {
        <Self as crate::graph::Identifiable>::get_id(self)
    }
}

pub trait Linker {
    fn link<A, B>(&mut self, ast: &A, with: &B)
    where 
        A: Identifiable + Into<AnyNode>,
        B: Into<RLTFamily> + Clone;

    fn link_existing<A, B>(&mut self, a: A, b: &B) -> A
    where 
        A: Identifiable + Into<AnyNode>,
        B: Identifiable + Into<AnyNode>;
}

pub trait Accessor {
    fn access<A, B>(&self, ast: &A) -> Option<&B>
    where
        A: Identifiable + Into<AnyNode>,
        RLTFamily: ConvertibleToRef<B>;

    fn access_unknown<A>(&self, ast: &A) -> Option<RLTFamily>
    where 
        A: Identifiable + Into<AnyNode>;

    fn tree(&self) -> Weak<SyntaxTree>;
}

pub(crate) trait PopulateTree {
    type Output: Into<AnyNode>;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output>;
}
