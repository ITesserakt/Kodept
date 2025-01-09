use crate::expression::Exprs;
use crate::Unit;
use kodept_ast::prelude::CodeHolder;
use kodept_ast::properties::tags::Tagged;
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::prelude::{ASTBuilder, ChildrenScope};
use kodept_rlt::prelude::Body;

pub(crate) fn unwrap_body<'p, R, S, Tag>(node: &'p Body, scope: &mut ChildrenScope<'p, '_, R, S>)
where
    R: HasChild<Exprs, Tag>,
    Tag: Tagged,
    S: CodeHolder,
{
    match node {
        Body::Block(x) => scope.many([x]),
        Body::Simplified { expression, .. } => {
            let fake = ASTBuilder::new(scope.pool(), Exprs).with_children(
                scope.source(),
                scope.pool(),
                |scope| scope.choose(Unit, [expression]),
            );
            scope.from_builder(node, fake)
        }
    };
}
