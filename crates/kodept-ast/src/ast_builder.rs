use std::borrow::Cow;

#[cfg(feature = "size-of")]
use size_of::SizeOf;

use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use kodept_core::structure::{rlt, Located};

use crate::graph::SyntaxTree;
use crate::graph::{GenericASTNode, NodeId};
use crate::rlt_accessor::{ASTFamily, RLTAccessor, RLTFamily};
use crate::traits::{Identifiable, Linker, PopulateTree};

#[derive(Debug)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct ASTBuilder {
    last_id: NodeId<GenericASTNode>,
}

impl Default for ASTBuilder {
    fn default() -> Self {
        Self {
            last_id: NodeId::new(0),
        }
    }
}

impl ASTBuilder {
    pub fn recursive_build<'n, C: CodeHolder>(
        &mut self,
        from: &'n rlt::File,
        code: &C,
    ) -> (SyntaxTree, RLTAccessor<'n>) {
        let mut links = RLTAccessor::default();
        let mut linker = ASTLinker(self, &mut links, code);
        let mut tree = SyntaxTree::new();
        from.convert(&mut tree, &mut linker);
        (tree.build(), links)
    }
}

struct ASTLinker<'a, 'b, C>(&'a mut ASTBuilder, &'a mut RLTAccessor<'b>, &'a C)
where
    C: CodeHolder;

impl<'a, 'b, C: CodeHolder> Linker<'b> for ASTLinker<'a, 'b, C> {
    fn link_ref<A, B>(&mut self, ast: NodeId<A>, with: B)
    where
        NodeId<A>: Into<ASTFamily>,
        B: Into<RLTFamily<'b>>,
    {
        self.1.save(ast, with);
    }

    fn link_existing<A, B>(&mut self, a: A, b: &B) -> A
    where
        A: Identifiable + 'static,
        B: Identifiable + 'static,
        NodeId<A>: Into<ASTFamily>,
        NodeId<B>: Into<ASTFamily>,
    {
        self.1.save_existing(&a, b);
        a
    }
}

impl<'a, 'b, C: CodeHolder> CodeHolder for ASTLinker<'a, 'b, C> {
    fn get_chunk(&self, at: CodePoint) -> Cow<str> {
        self.2.get_chunk(at)
    }

    fn get_chunk_located<L: Located>(&self, for_item: &L) -> Cow<str> {
        self.2.get_chunk_located(for_item)
    }
}
