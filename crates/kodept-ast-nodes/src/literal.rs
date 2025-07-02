use std::borrow::Cow;

use crate::code_flow::IfExpr;
use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::term::Ref;
use bevy_ecs::prelude::Component;
use kodept_ast::{derive_node, relation, Str};
use kodept_rlt::prelude as rlt;

#[derive(Debug, PartialEq, Component)]
pub enum Literal {
    Integer(i128),
    Floating(f64),
    Char(u8),
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

impl Literal {
    pub(crate) fn from_str(node: &rlt::Literal, value: Str) -> Self {
        match node {
            rlt::Literal::Binary(_) => {
                let digits = &value[2..];
                Self::Integer(i128::from_str_radix(digits, 2).unwrap())
            }
            rlt::Literal::Octal(_) => {
                let digits = &value[2..];
                Self::Integer(i128::from_str_radix(digits, 8).unwrap())
            }
            rlt::Literal::Hex(_) => {
                let digits = &value[2..];
                Self::Integer(i128::from_str_radix(digits, 16).unwrap())
            }
            rlt::Literal::Floating(_) => {
                if value.contains('.') {
                    Self::Floating(
                        value
                            .parse()
                            .expect("Cannot have literals more that f64 can hold"),
                    )
                } else {
                    Self::Integer(
                        value
                            .parse()
                            .expect("Cannot have literals more that i128 can hold"),
                    )
                }
            }
            rlt::Literal::Char(_) => {
                assert!(
                    value.starts_with('\''),
                    "Cannot have literals that does not start with quote"
                );
                assert!(
                    value.ends_with('\''),
                    "Cannot have literals that does not end with quote"
                );
                Self::Char(value.as_bytes()[1])
            }
            rlt::Literal::String(_) => {
                assert!(
                    value.starts_with('"'),
                    "Cannot have literals that does not start with quote"
                );
                assert!(
                    value.ends_with('"'),
                    "Cannot have literals that does not end with quote"
                );
                let quotes_removed = match value {
                    Cow::Borrowed(s) => Cow::Borrowed(&s[1..s.len() - 1]),
                    Cow::Owned(mut s) => {
                        s.remove(s.len() - 1);
                        s.remove(0);
                        Cow::Owned(s)
                    }
                };
                Self::String(quotes_removed)
            }
            rlt::Literal::Tuple(_) => unreachable!(),
        }
    }
}
