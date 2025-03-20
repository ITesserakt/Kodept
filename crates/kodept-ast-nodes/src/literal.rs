use crate::code_flow::IfExpr;
use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::term::Ref;
use crate::Unit;
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder};
use kodept_ast::syntax_tree::children::{ChildrenDisjoint, HasChild};
use kodept_ast::syntax_tree::prelude::ASTBuilder;
use kodept_ast::{derive_node, relation, Str};
use kodept_ast::arity::Arity;
use kodept_rlt::prelude as rlt;

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

derive_node!(Tuple);
relation!(Tuple => children Exprs);
relation!(Tuple => children App);
relation!(Tuple => children Lambda);
relation!(Tuple => children IfExpr);
relation!(Tuple => children BinExpr);
relation!(Tuple => children UnExpr);
relation!(Tuple => children Ref);
relation!(Tuple => children Literal);
relation!(Tuple => children Tuple);

impl<R, Tag, A> Choose<rlt::Literal, R, Tag> for Unit
where
    Tag: Send + Sync + 'static,
    A: Arity,
    R: HasChild<Tuple, Tag, Arity = A>,
    R: HasChild<Literal, Tag, Arity = A>,
{
    type Arity = A;

    fn branch<Source: CodeHolder>(node: &rlt::Literal) -> ChildrenDisjoint<R, Source, A, Tag> {
        match node {
            rlt::Literal::Tuple(_) => ChildrenDisjoint::ad_hoc(node, |node, source, pool| {
                let rlt::Literal::Tuple(node) = node else {
                    unreachable!()
                };
                ASTBuilder::new(pool, Tuple).with_children(source, pool, |scope| {
                    scope.choose(Unit, node.inner.as_ref());
                })
            }),
            _ => ChildrenDisjoint::ad_hoc(node, |node, source: Source, pool| {
                let value = match node {
                    rlt::Literal::Binary(span) => Literal::Binary(source.get_chunk_located(span)),
                    rlt::Literal::Octal(span) => Literal::Octal(source.get_chunk_located(span)),
                    rlt::Literal::Hex(span) => Literal::Hex(source.get_chunk_located(span)),
                    rlt::Literal::Floating(span) => {
                        Literal::Floating(source.get_chunk_located(span))
                    }
                    rlt::Literal::Char(span) => Literal::Char(source.get_chunk_located(span)),
                    rlt::Literal::String(span) => Literal::String(source.get_chunk_located(span)),
                    _ => unreachable!(),
                };
                ASTBuilder::new(pool, value)
            }),
        }
    }
}
