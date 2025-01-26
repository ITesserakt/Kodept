use crate::expression::Exprs;
use crate::types::{Params, TyParam, TyParams};
use crate::Unit;
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::tags::Tagged;
use kodept_ast::resource::rlt::SyntaxVariant;
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::prelude::{ASTBuilder, ChildrenScope};
use kodept_rlt::prelude::{Body, Parameter, TypedParameter};

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

pub(crate) fn wrap_params<'p, R, S, Tag>(
    parent_node: &'p R::Syntax,
    params: &'p impl AsRef<[Parameter]>,
    scope: &mut ChildrenScope<'p, '_, R, S>,
) where
    R: HasChild<Params, Tag> + FromSyntax,
    Tag: Tagged,
    S: CodeHolder,
    &'p R::Syntax: Into<SyntaxVariant<'p>>,
{
    let fake = ASTBuilder::new(scope.pool(), Params).with_children(
        scope.source(),
        scope.pool(),
        |inner_scope| inner_scope.choose(Unit, params.as_ref()),
    );
    scope.from_builder(parent_node, fake);
}

pub(crate) fn wrap_ty_params<'p, R, S, Tag>(
    parent_node: &'p R::Syntax,
    params: &'p impl AsRef<[TypedParameter]>,
    scope: &mut ChildrenScope<'p, '_, R, S>,
) where
    R: HasChild<TyParams, Tag> + FromSyntax,
    Tag: Tagged,
    S: CodeHolder,
    &'p R::Syntax: Into<SyntaxVariant<'p>>,
{
    let fake = ASTBuilder::new(scope.pool(), TyParams).with_children(
        scope.source(),
        scope.pool(),
        |inner_scope| inner_scope.many::<TyParam, _>(params.as_ref()),
    );
    scope.from_builder(parent_node, fake);
}
