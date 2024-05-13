use std::fmt::Debug;
use std::rc::Weak;

use kodept_core::ConvertibleToRef;
use kodept_core::structure::span::CodeHolder;

use crate::graph::{GenericASTNode, NodeId};
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
        A: Identifiable + Into<GenericASTNode>,
        B: Into<RLTFamily> + Clone;

    fn link_existing<A, B>(&mut self, a: A, b: &B) -> A
    where 
        A: Identifiable + Into<GenericASTNode>,
        B: Identifiable + Into<GenericASTNode>;
}

pub trait Accessor {
    fn access<A, B>(&self, ast: &A) -> Option<&B>
    where
        A: Identifiable + Into<GenericASTNode>,
        RLTFamily: ConvertibleToRef<B>;

    fn access_unknown<A>(&self, ast: &A) -> Option<RLTFamily>
    where 
        A: Identifiable + Into<GenericASTNode>;

    fn tree(&self) -> Weak<SyntaxTree>;
}

#[derive(Debug)]
#[repr(transparent)]
pub struct LinkGuard<I>(I);

pub(crate) trait PopulateTree {
    type Output: Into<GenericASTNode>;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output>;
}
