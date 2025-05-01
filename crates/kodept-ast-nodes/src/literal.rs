use crate::code_flow::IfExpr;
use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::term::Ref;
use bevy_ecs::prelude::Component;
use kodept_ast::{derive_node, relation, Str};

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
