use crate::code_flow::IfExpr;
use crate::expression::{App, BinExpr, Exprs, Lambda};
use crate::term::Ref;
use crate::types::Ty;
use crate::Dispatcher;
use crate::Error::{CannotParseFloat, CannotParseInt, NoQuotesInLiteral, WrongLiteralLength};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use kodept_ast::experimental::{AstBuilder, Dispatch, DispatchContext};
use kodept_ast::prelude::CodeHolder;
use kodept_ast::properties::SourceSpan;
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::experimental::SpawnedIn;
use kodept_ast::{derive_node, relation, Str};
use kodept_rlt::exported::Located;
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::prelude as rlt;
use std::borrow::Cow;

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
relation!(Tuple => children Ref);
relation!(Tuple => children Ty);
relation!(Tuple => children Literal);
relation!(Tuple => children Tuple);

impl Literal {
    pub(crate) fn from_str(node: &rlt::Literal, value: Str) -> Result<Self, crate::Error> {
        match node {
            rlt::Literal::Binary(_) => {
                if value.len() <= 2 {
                    return Err(WrongLiteralLength(node.location(), 3));
                }
                let digits = &value[2..];
                i128::from_str_radix(digits, 2)
                    .map_err(|e| CannotParseInt(node.location(), e))
                    .map(Self::Integer)
            }
            rlt::Literal::Octal(_) => {
                if value.len() <= 2 {
                    return Err(WrongLiteralLength(node.location(), 3));
                }
                let digits = &value[2..];
                i128::from_str_radix(digits, 8)
                    .map_err(|e| CannotParseInt(node.location(), e))
                    .map(Self::Integer)
            }
            rlt::Literal::Hex(_) => {
                if value.len() <= 2 {
                    return Err(WrongLiteralLength(node.location(), 3));
                }
                let digits = &value[2..];
                i128::from_str_radix(digits, 16)
                    .map_err(|e| CannotParseInt(node.location(), e))
                    .map(Self::Integer)
            }
            rlt::Literal::Floating(_) => {
                if value.contains('.') {
                    value
                        .parse()
                        .map_err(|e| CannotParseFloat(node.location(), e))
                        .map(Self::Floating)
                } else {
                    value
                        .parse()
                        .map_err(|e| CannotParseInt(node.location(), e))
                        .map(Self::Integer)
                }
            }
            rlt::Literal::Char(_) => {
                if !value.starts_with('\'') || !value.ends_with('\'') {
                    return Err(NoQuotesInLiteral(node.location()));
                }
                if value.len() != 3 {
                    return Err(WrongLiteralLength(node.location(), 3));
                }
                Ok(Self::Char(value.as_bytes()[1]))
            }
            rlt::Literal::String(_) => {
                if !value.starts_with('"') || !value.ends_with('"') {
                    return Err(NoQuotesInLiteral(node.location()));
                }
                let quotes_removed = match value {
                    Cow::Borrowed(s) => Cow::Borrowed(&s[1..s.len() - 1]),
                    Cow::Owned(mut s) => {
                        s.remove(s.len() - 1);
                        s.remove(0);
                        Cow::Owned(s)
                    }
                };
                Ok(Self::String(quotes_removed))
            }
            rlt::Literal::Tuple(_) => unreachable!("This method called on tuple literal"),
        }
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, rlt::Literal>
where
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    T: Send + Sync + 'static,
    A: kodept_ast::arity::Arity
{
    type Node = rlt::Literal;
    type Error = crate::Error;

    fn dispatch(
        self,
        spawner: DispatchContext<R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        match self.0 {
            rlt::Literal::Tuple(x) => Ok(AstBuilder::new(Tuple)
                .with_property(SourceSpan(x.left.0 + x.right.0))
                .spawn_in((spawner, self.0))
                .with_dispatches::<Dispatcher<_>, _, _>(x.inner.as_ref(), source)?
                .finish_any()),
            _ => {
                let text = source.get_chunk_located(self.0);
                let value = Literal::from_str(self.0, text)?;
                Ok(AstBuilder::new(value)
                    .with_property(SourceSpan(self.0.bounds()))
                    .spawn_in((spawner, self.0))
                    .finish_any())
            }
        }
    }
}
