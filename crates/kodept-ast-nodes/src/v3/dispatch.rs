use crate::v3::tags::*;
use crate::v3::types::*;
use crate::Error::{CannotParseFloat, CannotParseInt, NoQuotesInLiteral, WrongLiteralLength};
use bevy_ecs::prelude::Entity;
use bigdecimal::{BigDecimal, Num};
use kodept_ast::arity::{Arity, Plural};
use kodept_ast::experimental::{AstBuilder, Dispatch, DispatchContext, SplitRef};
use kodept_ast::prelude::CodeHolder;
use kodept_ast::properties::{Lexeme, SourceSpan};
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::experimental::{Buffer, SpawnedIn};
use kodept_ast::Str;
use kodept_rlt::exported::{Located, SpanBounds};
use kodept_rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_rlt::prelude::Literal::{Binary, Hex, Octal};
use kodept_rlt::prelude::{BlockLevelNode, Body, Operation, Term, TopLevelNode};
use num_bigint::BigInt;
use std::borrow::Cow;
use std::convert::Infallible;
use std::str::FromStr;

pub(super) struct Dispatcher<'a, T>(&'a T);

impl<'a, T> SplitRef<'a, T> for Dispatcher<'a, T> {
    fn split(self) -> (Self, &'a T) {
        let reference = self.0;
        (self, reference)
    }
}

impl<'a> Dispatch<'a, Module, Declaration, Plural> for Dispatcher<'a, TopLevelNode> {
    type Node = TopLevelNode;
    type Error = crate::Error;

    fn dispatch<B: Buffer>(
        self,
        mut spawner: DispatchContext<B, Module, Declaration, Plural>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        match self.0 {
            TopLevelNode::Enum(node) => spawner.forward::<_, UserType>(node, source),
            TopLevelNode::Struct(node) => spawner.forward::<_, UserType>(node, source),
            TopLevelNode::BodiedFunction(node) => {
                spawner.forward::<_, UserFunction<_>>(node, source)
            }
        }
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, Body>
where
    T: Send + Sync + 'static,
    A: Arity,
    R: HasChild<Block, T, Arity = A>,
    R: HasChild<UserFunction<Option<Unresolved>>, T, Arity = A>,
    R: HasChild<Variable<Option<Unresolved>>, T, Arity = A>,
    R: HasChild<Value<Unresolved>, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Call, T, Arity = A>,
    R: HasChild<AnonFunction<Option<Unresolved>>, T, Arity = A>,
    R: HasChild<If, T, Arity = A>,
{
    type Node = Body;
    type Error = crate::Error;

    fn dispatch<B: Buffer>(
        self,
        mut spawner: DispatchContext<B, R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        match self.0 {
            Body::Block(node) => spawner.forward::<_, Block>(node, source),
            Body::Simplified { expression, .. } => {
                spawner.dispatch::<Dispatcher<_>>(expression, source)
            }
        }
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, BlockLevelNode>
where
    R: HasChild<Block, T, Arity = A>,
    R: HasChild<UserFunction<Option<Unresolved>>, T, Arity = A>,
    R: HasChild<Variable<Option<Unresolved>>, T, Arity = A>,
    R: HasChild<Value<Unresolved>, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Call, T, Arity = A>,
    R: HasChild<AnonFunction<Option<Unresolved>>, T, Arity = A>,
    R: HasChild<If, T, Arity = A>,
    T: Send + Sync + 'static,
    A: Arity,
{
    type Node = BlockLevelNode;
    type Error = crate::Error;

    fn dispatch<B: Buffer>(
        self,
        mut spawner: DispatchContext<B, R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        match self.0 {
            BlockLevelNode::InitVar(node) => spawner.forward::<_, Variable<_>>(node, source),
            BlockLevelNode::Block(node) => spawner.forward::<_, Block>(node, source),
            BlockLevelNode::Function(node) => spawner.forward::<_, UserFunction<_>>(node, source),
            BlockLevelNode::Operation(node) => spawner.dispatch::<Dispatcher<_>>(node, source),
        }
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, Operation>
where
    T: Send + Sync + 'static,
    A: Arity,
    R: HasChild<Block, T, Arity = A>,
    R: HasChild<Value<Unresolved>, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Call, T, Arity = A>,
    R: HasChild<AnonFunction<Option<Unresolved>>, T, Arity = A>,
    R: HasChild<If, T, Arity = A>,
{
    type Node = Operation;
    type Error = crate::Error;

    fn dispatch<B: Buffer>(
        self,
        mut spawner: DispatchContext<B, R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        const CORE_PATH: Path = Path {
            is_global: true,
            segments: Cow::Borrowed(&[Str::Borrowed("Core"), Str::Borrowed("Traits")]),
        };

        match self.0 {
            Operation::Block(node) => spawner.forward::<_, Block>(node, source),
            Operation::Expression(node) => spawner.dispatch::<Dispatcher<_>>(node, source),
            Operation::Application(node) => spawner.forward::<_, Call>(node, source),
            Operation::Unary { operator, expr } => {
                let ident = match operator {
                    UnaryOperationSymbol::Neg(_) => "neg".into(),
                    UnaryOperationSymbol::Not(_) => "not".into(),
                    UnaryOperationSymbol::Inv(_) => "inv".into(),
                    UnaryOperationSymbol::Plus(_) => "pos".into(),
                };

                let mut builder = AstBuilder::new(Call)
                    .with_property(SourceSpan(self.0.bounds()))
                    .with_property(Lexeme::new(self.0))
                    .spawn_in((spawner, self.0));

                builder.with_dispatch_fn::<_, Lhs, _, Infallible>(operator, |node, spawner| {
                    Ok(AstBuilder::new(Value {
                        inner: Unresolved::Named {
                            context: CORE_PATH,
                            ident,
                        },
                    })
                    .with_property(SourceSpan(node.bounds()))
                    .with_property(Lexeme::new(node))
                    .spawn_in((spawner, node))
                    .finish_any())
                })?;
                builder.with_dispatch::<Dispatcher<_>, Rhs, _>(expr.as_ref(), source)?;

                Ok(builder.finish_any())
            }
            Operation::Binary {
                left,
                operation,
                right,
            } => {
                let ident = match operation {
                    BinaryOperationSymbol::Pow(_) => "pow".into(),
                    BinaryOperationSymbol::Mul(_) => "mul".into(),
                    BinaryOperationSymbol::Div(_) => "div".into(),
                    BinaryOperationSymbol::Rem(_) => "rem".into(),
                    BinaryOperationSymbol::Add(_) => "add".into(),
                    BinaryOperationSymbol::Sub(_) => "sub".into(),
                    BinaryOperationSymbol::ComplexComparison(_) => "spaceship".into(),
                    BinaryOperationSymbol::LessEq(_) => "leq".into(),
                    BinaryOperationSymbol::NEq(_) => "neq".into(),
                    BinaryOperationSymbol::Eq(_) => "eq".into(),
                    BinaryOperationSymbol::GreaterEq(_) => "geq".into(),
                    BinaryOperationSymbol::Less(_) => "less".into(),
                    BinaryOperationSymbol::Greater(_) => "greater".into(),
                    BinaryOperationSymbol::Or(_) => "or".into(),
                    BinaryOperationSymbol::And(_) => "and".into(),
                    BinaryOperationSymbol::Xor(_) => "xor".into(),
                    BinaryOperationSymbol::Disjunction(_) => "disj".into(),
                    BinaryOperationSymbol::Conjunction(_) => "conj".into(),
                    BinaryOperationSymbol::Assign(_) => {
                        return Err(crate::Error::Unsupported(operation.bounds()));
                    }
                };

                let mut builder = AstBuilder::new(Call)
                    .with_property(SourceSpan(self.0.bounds()))
                    .with_property(Lexeme::new(self.0))
                    .spawn_in((spawner, self.0));

                builder.with_dispatch_fn::<_, Lhs, _, Infallible>(operation, |node, spawner| {
                    Ok(AstBuilder::new(Value {
                        inner: Unresolved::Named {
                            context: CORE_PATH,
                            ident,
                        },
                    })
                    .with_property(SourceSpan(node.bounds()))
                    .with_property(Lexeme::new(node))
                    .spawn_in((spawner, node))
                    .finish_any())
                })?;
                builder.with_dispatch::<Dispatcher<_>, Rhs, _>(left.as_ref(), source)?;
                builder.with_dispatch::<Dispatcher<_>, Rhs, _>(right.as_ref(), source)?;

                Ok(builder.finish_any())
            }
            Operation::Access { .. } => Err(crate::Error::Unsupported(self.0.bounds())),
        }
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, kodept_rlt::prelude::Expression>
where
    T: Send + Sync + 'static,
    A: Arity,
    R: HasChild<Value<Unresolved>, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<AnonFunction<Option<Unresolved>>, T, Arity = A>,
    R: HasChild<If, T, Arity = A>,
{
    type Node = kodept_rlt::prelude::Expression;
    type Error = crate::Error;

    fn dispatch<B: Buffer>(
        self,
        mut spawner: DispatchContext<B, R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        match self.0 {
            kodept_rlt::prelude::Expression::Term(node) => {
                spawner.dispatch::<Dispatcher<_>>(node, source)
            }
            kodept_rlt::prelude::Expression::Literal(node) => {
                spawner.dispatch::<Dispatcher<_>>(node, source)
            }
            kodept_rlt::prelude::Expression::Lambda(node) => {
                spawner.forward::<_, AnonFunction<_>>(node, source)
            }
            kodept_rlt::prelude::Expression::If(node) => spawner.forward::<_, If>(node, source),
        }
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, kodept_rlt::prelude::Literal>
where
    T: Send + Sync + 'static,
    A: Arity,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
{
    type Node = kodept_rlt::prelude::Literal;
    type Error = crate::Error;

    fn dispatch<B: Buffer>(
        self,
        spawner: DispatchContext<B, R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        let node = self.0;
        let text = source.get_chunk_located(node);
        let value = match node {
            kodept_rlt::prelude::Literal::String(_) => {
                if !text.starts_with('"') || !text.ends_with('"') {
                    return Err(NoQuotesInLiteral(node.location()));
                }
                let quotes_removed = match text {
                    Cow::Borrowed(s) => Cow::Borrowed(&s[1..s.len() - 1]),
                    Cow::Owned(mut s) => {
                        s.remove(s.len() - 1);
                        s.remove(0);
                        Cow::Owned(s)
                    }
                };
                Literal::String(quotes_removed)
            }
            kodept_rlt::prelude::Literal::Char(_) => {
                if !text.starts_with('\'') || !text.ends_with('\'') {
                    return Err(NoQuotesInLiteral(node.location()));
                }
                if text.len() != 3 {
                    return Err(WrongLiteralLength(node.location(), 3));
                }
                Literal::Char(text.chars().nth(1).unwrap())
            }
            kodept_rlt::prelude::Literal::Floating(point) => {
                if text.contains('.') {
                    BigDecimal::from_str(text.as_ref())
                        .map_err(|e| CannotParseFloat(*point, e))
                        .map(Literal::Floating)?
                } else {
                    BigInt::from_str(text.as_ref())
                        .map_err(|e| CannotParseInt(*point, e))
                        .map(Literal::Integer)?
                }
            }
            Binary(point) | Hex(point) | Octal(point) if text.len() < 3 => {
                return Err(WrongLiteralLength(*point, 3))
            }
            Binary(point) => BigInt::from_str_radix(&text[2..], 2)
                .map_err(|e| CannotParseInt(*point, e))
                .map(Literal::Integer)?,
            Hex(point) => BigInt::from_str_radix(&text[2..], 16)
                .map_err(|e| CannotParseInt(*point, e))
                .map(Literal::Integer)?,
            Octal(point) => BigInt::from_str_radix(&text[2..], 8)
                .map_err(|e| CannotParseInt(*point, e))
                .map(Literal::Integer)?,
            kodept_rlt::prelude::Literal::Tuple(items) => {
                return Ok(AstBuilder::new(Tuple)
                    .with_property(SourceSpan(items.left.bounds() + items.right.bounds()))
                    .with_property(Lexeme::new(node))
                    .spawn_in((spawner, node))
                    .with_dispatches::<Dispatcher<_>, _, _>(items.inner.as_ref(), source)?
                    .finish_any())
            }
        };
        Ok(AstBuilder::new(value)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Lexeme::new(node))
            .spawn_in((spawner, node))
            .finish_any())
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, Term>
where
    T: Send + Sync + 'static,
    A: Arity,
    R: HasChild<Value<Unresolved>, T, Arity = A>,
{
    type Node = Term;
    type Error = crate::Error;

    fn dispatch<B: Buffer>(
        self,
        spawner: DispatchContext<B, R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        let value = match self.0 {
            Term::Reference(x) => Value {
                inner: Unresolved::Named {
                    ident: source.get_chunk_located(x),
                    context: Path::empty(false),
                },
            },
            Term::ContextualReference(x) => Value {
                inner: Unresolved::Named {
                    ident: source.get_chunk_located(&x.inner),
                    context: (&x.context, source).into(),
                },
            },
            Term::Constant(x) => Value {
                inner: Unresolved::Named {
                    ident: source.get_chunk_located(x),
                    context: Path::empty(false),
                },
            },
            Term::ContextualConstant(x) => Value {
                inner: Unresolved::Named {
                    ident: source.get_chunk_located(&x.inner),
                    context: (&x.context, source).into(),
                },
            },
        };
        let builder = AstBuilder::new(value)
            .with_property(SourceSpan(self.0.bounds()))
            .with_property(Lexeme::new(self.0))
            .spawn_in((spawner, self.0));

        Ok(builder.finish_any())
    }
}

impl<'a, T> From<&'a T> for Dispatcher<'a, T> {
    fn from(value: &'a T) -> Self {
        Self(value)
    }
}
