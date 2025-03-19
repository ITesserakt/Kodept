use crate::constants::Const;
use crate::expression::Exprs;
use crate::types::{Params, TyParam, TyParams};
use crate::Unit;
use kodept_ast::prelude::{ASTNode, CodeHolder, FromSyntax};
use kodept_ast::properties::tags::{NoTag, Tagged};
use kodept_ast::properties::Name;
use kodept_ast::resource::rlt::SyntaxVariant;
use kodept_ast::syntax_tree::children::{ChildrenDisjoint, HasChild};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, ChildrenScope};
use kodept_ast::Str;
use kodept_rlt::prelude::{Body, Parameter, TypedParameter};
use std::fmt::Debug;

pub(crate) fn unwrap_body<'scope, 'w, R, S, Tag>(node: &'w Body, scope: &mut ChildrenScope<'scope, 'w, R, S>)
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

pub(crate) fn wrap_params<'scope, 'w, R, S, Tag>(
    parent_node: &'w R::Syntax,
    params: &impl AsRef<[Parameter]>,
    scope: &mut ChildrenScope<'scope, 'w, R, S>,
) where
    R: HasChild<Params, Tag> + FromSyntax,
    Tag: Tagged,
    S: CodeHolder,
    &'w R::Syntax: Into<SyntaxVariant<'w>>,
{
    let fake = ASTBuilder::new(scope.pool(), Params).with_children(
        scope.source(),
        scope.pool(),
        |inner_scope| inner_scope.choose(Unit, params.as_ref()),
    );
    scope.from_builder(parent_node, fake);
}

pub(crate) fn wrap_ty_params<'scope, 'w, R, S, Tag>(
    parent_node: &'w R::Syntax,
    params: &impl AsRef<[TypedParameter]>,
    scope: &mut ChildrenScope<'scope, 'w, R, S>,
) where
    R: HasChild<TyParams, Tag> + FromSyntax,
    Tag: Tagged,
    S: CodeHolder,
    &'w R::Syntax: Into<SyntaxVariant<'w>>,
{
    let fake = ASTBuilder::new(scope.pool(), TyParams).with_children(
        scope.source(),
        scope.pool(),
        |inner_scope| inner_scope.many::<TyParam, _>(params.as_ref()),
    );
    scope.from_builder(parent_node, fake);
}

pub(crate) fn const_disjoint<'p, U, R, S, Tag>(
    node: &'p U::Syntax,
    name_fn: impl FnOnce(&U::Syntax, S) -> Str + 'static,
) -> ChildrenDisjoint<'p, R, S, Tag>
where
    &'p U::Syntax: TryFrom<SyntaxVariant<'p>, Error: Debug> + Into<SyntaxVariant<'p>>,
    U: FromSyntax<Syntax: Sync> + ASTNode,
    Const: HasChild<U>,
    S: CodeHolder,
    R: HasChild<Const, Tag>,
    Tag: Tagged,
{
    ChildrenDisjoint::ad_hoc(node, move |node, source, pool| {
        ASTBuilder::new(pool, Const)
            .with_property(Name(name_fn(node, source)))
            .with_children(source, pool, |scope| scope.many::<U, NoTag>([node]))
    })
}
