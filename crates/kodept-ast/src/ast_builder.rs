use crate::node_id::NodeId;
use crate::rlt_accessor::{ASTFamily, RLTAccessor, RLTFamily};
use crate::traits::{IdProducer, Identifiable, IntoAst, Linker};
use crate::FileDeclaration;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use kodept_core::structure::{rlt, Located};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use std::borrow::Cow;

#[derive(Debug)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
pub struct ASTBuilder {
    last_id: NodeId<()>,
}

impl Default for ASTBuilder {
    fn default() -> Self {
        Self {
            last_id: NodeId::new(0),
        }
    }
}

impl ASTBuilder {
    pub fn build_from<'n, B, C>(
        &mut self,
        with: &'n B,
        links: &mut RLTAccessor<'n>,
        code: &C,
    ) -> B::Output
    where
        B: IntoAst,
        B::Output: Identifiable,
        NodeId<B::Output>: Into<ASTFamily> + 'static,
        &'n B: Into<RLTFamily<'n>> + 'n,
        C: CodeHolder,
    {
        let ast_node = with.construct(&mut ASTLinker(self, links, code));
        links.save(&ast_node, with);
        ast_node
    }

    pub fn recursive_build<'n, C: CodeHolder>(
        &mut self,
        from: &'n rlt::File,
        code: &C,
    ) -> (FileDeclaration, RLTAccessor<'n>) {
        let mut links = RLTAccessor::default();
        let node = self.build_from(from, &mut links, code);
        (node, links)
    }
}

struct ASTLinker<'a, 'b, C>(&'a mut ASTBuilder, &'a mut RLTAccessor<'b>, &'a C)
where
    C: CodeHolder;

impl<'a, 'b, C: CodeHolder> Linker<'b> for ASTLinker<'a, 'b, C> {
    fn link<A, B>(&mut self, ast: A, with: B) -> A
    where
        A: Identifiable + 'static,
        NodeId<A>: Into<ASTFamily>,
        B: Into<RLTFamily<'b>>,
    {
        self.1.save(&ast, with);
        ast
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

impl<'a, 'b, C: CodeHolder> IdProducer for ASTLinker<'a, 'b, C> {
    fn next_id<T>(&mut self) -> NodeId<T> {
        self.0.next_id()
    }
}

impl IdProducer for ASTBuilder {
    fn next_id<T>(&mut self) -> NodeId<T> {
        let next_id = self.last_id.next();
        let id = std::mem::replace(&mut self.last_id, next_id);
        id.cast()
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
