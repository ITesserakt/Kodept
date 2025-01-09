use crate::code_flow::IfExpr;
use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::properties::Expr;
use crate::term::Ref;
use crate::Unit;
use kodept_ast::{derive_node, Str};
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder};
use kodept_ast::properties::tags::Tagged;
use kodept_ast::syntax_tree::children::{ChildrenDisjoint, HasChild};
use kodept_ast::syntax_tree::prelude::ASTBuilder;
use kodept_core::structure::rlt;

#[derive(Debug, PartialEq, Component)]
pub enum Literal {
    Binary(Str),
    Octal(Str),
    Hex(Str),
    Floating(Str),
    Char(Str),
    String(Str),
}

#[derive(Debug, PartialEq, Component)]
pub struct Tuple;

derive_node!(Literal);
derive_node!(Tuple {
    relations = [
        children Exprs where tag = Expr,
        children App where tag = Expr,
        children Lambda where tag = Expr,
        children IfExpr where tag = Expr,
        children BinExpr where tag = Expr,
        children UnExpr where tag = Expr,
        children Ref where tag = Expr,
        children Literal where tag = Expr,
        children Tuple where tag = Expr,
    ],
    properties = []
});

impl<R, Tag> Choose<rlt::Literal, R, Tag> for Unit
where 
    Tag: Tagged,
    R: HasChild<Tuple, Tag>,
    R: HasChild<Literal, Tag>
{
    fn branch<Source: CodeHolder>(node: &rlt::Literal) -> ChildrenDisjoint<R, Source, Tag> {
        match node {
            rlt::Literal::Tuple(_) => ChildrenDisjoint::ad_hoc(node, |node, source, pool| {
                let rlt::Literal::Tuple(node) = node else { unreachable!() };
                ASTBuilder::new(pool, Tuple).with_children(source, pool, |scope| {
                    scope.choose(Unit, node.inner.as_ref());
                })
            }),
            _ => ChildrenDisjoint::ad_hoc(node, |node, source: Source, pool| {
                let value = match node {
                    rlt::Literal::Binary(span) => Literal::Binary(source.get_chunk_located(span)),
                    rlt::Literal::Octal(span) => Literal::Octal(source.get_chunk_located(span)),
                    rlt::Literal::Hex(span) => Literal::Hex(source.get_chunk_located(span)),
                    rlt::Literal::Floating(span) => Literal::Floating(source.get_chunk_located(span)),
                    rlt::Literal::Char(span) => Literal::Char(source.get_chunk_located(span)),
                    rlt::Literal::String(span) => Literal::String(source.get_chunk_located(span)),
                    _ => unreachable!()
                };
                ASTBuilder::new(pool, value)
            })
        }
    }
}
