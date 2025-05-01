use crate::code_flow::IfExpr;
use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::term::Ref;
use crate::utils::unwrap_operation;
use bevy_ecs::prelude::{Bundle, Component};
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::syntax_tree::experimental::ASTBuilder;
use kodept_ast::{derive_node, relation, Str};
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
