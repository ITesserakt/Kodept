use std::borrow::Cow;

#[cfg(feature = "size-of")]
use size_of::SizeOf;

use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use kodept_core::structure::{rlt, Located};

use crate::graph::NodeId;
use crate::graph::{SyntaxTree, SyntaxTreeBuilder};
use crate::rlt_accessor::{ASTFamily, RLTAccessor, RLTFamily};
use crate::traits::{IntoASTFamily, Linker, PopulateTree};

#[derive(Debug, Default)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct ASTBuilder;

impl ASTBuilder {
    pub fn recursive_build<'n, C: CodeHolder>(
        &mut self,
        from: &'n rlt::File,
        code: &C,
    ) -> (SyntaxTreeBuilder, RLTAccessor<'n>) {
        let mut links = RLTAccessor::default();
        let mut linker = ASTLinker {
            access: &mut links,
            code,
        };
        let mut tree = SyntaxTree::new();
        from.convert(&mut tree, &mut linker);
        (tree, links)
    }
}

struct ASTLinker<'a, 'b, C>
where
    C: CodeHolder,
{
    access: &'a mut RLTAccessor<'b>,
    code: &'a C,
}

impl<'a, 'b, C: CodeHolder> Linker<'b> for ASTLinker<'a, 'b, C> {
    fn link_ref<A, B>(&mut self, ast: NodeId<A>, with: B)
    where
        NodeId<A>: Into<ASTFamily>,
        B: Into<RLTFamily<'b>>,
    {
        self.access.save(ast, with);
    }

    fn link<A, B>(&mut self, ast: A, with: B) -> A
    where
        A: IntoASTFamily,
        B: Into<RLTFamily<'b>>,
    {
        self.access.save(ast.as_member(), with);
        ast
    }

    fn link_existing<A: IntoASTFamily>(&mut self, a: A, b: &impl IntoASTFamily) -> A {
        self.access.save_existing(&a, b);
        a
    }
}

impl<'a, 'b, C: CodeHolder> CodeHolder for ASTLinker<'a, 'b, C> {
    fn get_chunk(&self, at: CodePoint) -> Cow<str> {
        self.code.get_chunk(at)
    }

    fn get_chunk_located<L: Located>(&self, for_item: &L) -> Cow<str> {
        self.code.get_chunk_located(for_item)
    }
}
