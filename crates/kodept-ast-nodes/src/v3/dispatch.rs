use crate::Error::{CannotParseFloat, CannotParseInt, NoQuotesInLiteral, WrongLiteralLength};
use crate::{
    AnonFunction, Block, Call, Declaration, If, Lhs, Literal, Module, Path, Rhs, Tuple, Unresolved,
    UserFunction, UserType, Value, Variable,
};
use bigdecimal::{BigDecimal, Num};
use kodept_ast::Str;
use kodept_ast::experimental::{Dispatch, FromSyntax};
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::{Lexeme, SourceSpan};
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::experimental::{AnonSpawner, Buffer, Constructed, NodeBuilder};
use kodept_rlt::exported::{Located, SpanBounds};
use kodept_rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_rlt::prelude::Literal::{Binary, Hex, Octal};
use kodept_rlt::prelude::{BlockLevelNode, Body, Expression, Operation, TopLevelNode};
use num_bigint::BigInt;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::str::FromStr;

pub(super) struct Dispatcher<T>(PhantomData<T>);

impl<B: Buffer> Dispatch<Module, Declaration, B> for Dispatcher<TopLevelNode> {
    type Syntax = TopLevelNode;
    type Error = crate::Error;

    #[inline]
    fn dispatch(
        node: &Self::Syntax,
        spawner: impl AnonSpawner<Module, Declaration, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId, Self::Error> {
        match node {
            TopLevelNode::Enum(node) => {
                Ok(UserType::from_syntax(node, spawner.into_concrete(), source)?.cast())
            }
            TopLevelNode::Struct(node) => {
                Ok(UserType::from_syntax(node, spawner.into_concrete(), source)?.cast())
            }
            TopLevelNode::BodiedFunction(node) => {
                Ok(UserFunction::from_syntax(node, spawner.into_concrete(), source)?.cast())
            }
        }
    }
}

impl<P, T, B: Buffer> Dispatch<P, T, B> for Dispatcher<Body>
where
    P: HasChild<Variable<Option<Unresolved>>, T>,
    P: HasChild<Block, T>,
    P: HasChild<UserFunction<Option<Unresolved>>, T>,
    P: HasChild<Block, T>,
    P: HasChild<AnonFunction<Option<Unresolved>>, T>,
    P: HasChild<If, T>,
    P: HasChild<Literal, T>,
    P: HasChild<Tuple, T>,
    P: HasChild<Value<Unresolved>, T>,
    P: HasChild<Call, T>,
{
    type Syntax = Body;
    type Error = crate::Error;

    #[inline]
    fn dispatch(
        node: &Self::Syntax,
        spawner: impl AnonSpawner<P, T, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId, Self::Error> {
        match node {
            Body::Block(node) => {
                Ok(Block::from_syntax(node, spawner.into_concrete(), source)?.cast())
            }
            Body::Simplified { expression, .. } => {
                Dispatcher::<BlockLevelNode>::dispatch(expression, spawner, source)
            }
        }
    }
}

impl<P, T, B: Buffer> Dispatch<P, T, B> for Dispatcher<BlockLevelNode>
where
    P: HasChild<Variable<Option<Unresolved>>, T>,
    P: HasChild<Block, T>,
    P: HasChild<UserFunction<Option<Unresolved>>, T>,
    P: HasChild<Block, T>,
    P: HasChild<AnonFunction<Option<Unresolved>>, T>,
    P: HasChild<If, T>,
    P: HasChild<Literal, T>,
    P: HasChild<Tuple, T>,
    P: HasChild<Value<Unresolved>, T>,
    P: HasChild<Call, T>,
{
    type Syntax = BlockLevelNode;
    type Error = crate::Error;

    #[inline]
    fn dispatch(
        node: &Self::Syntax,
        spawner: impl AnonSpawner<P, T, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId, Self::Error> {
        match node {
            BlockLevelNode::InitVar(node) => {
                Ok(Variable::from_syntax(node, spawner.into_concrete(), source)?.cast())
            }
            BlockLevelNode::Block(node) => {
                Ok(Block::from_syntax(node, spawner.into_concrete(), source)?.cast())
            }
            BlockLevelNode::Function(node) => {
                Ok(UserFunction::from_syntax(node, spawner.into_concrete(), source)?.cast())
            }
            BlockLevelNode::Operation(node) => {
                Dispatcher::<Operation>::dispatch(node, spawner, source)
            }
        }
    }
}

impl<P, T, B: Buffer> Dispatch<P, T, B> for Dispatcher<Operation>
where
    P: HasChild<Block, T>,
    P: HasChild<AnonFunction<Option<Unresolved>>, T>,
    P: HasChild<If, T>,
    P: HasChild<Literal, T>,
    P: HasChild<Tuple, T>,
    P: HasChild<Value<Unresolved>, T>,
    P: HasChild<Call, T>,
{
    type Syntax = Operation;
    type Error = crate::Error;

    fn dispatch(
        node: &Self::Syntax,
        spawner: impl AnonSpawner<P, T, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId, Self::Error> {
        const CORE_PATH: Path = Path {
            is_global: true,
            segments: Cow::Borrowed(&[Str::Borrowed("Core"), Str::Borrowed("Traits")]),
        };

        match node {
            Operation::Block(node) => {
                Ok(Block::from_syntax(node, spawner.into_concrete(), source)?.cast())
            }
            Operation::Expression(node) => {
                Dispatcher::<kodept_rlt::prelude::Expression>::dispatch(node, spawner, source)
            }
            Operation::Application(node) => {
                Ok(Call::from_syntax(&*node, spawner.into_concrete(), source)?.cast())
            }
            Operation::Unary { operator, expr } => {
                let ident = match operator {
                    UnaryOperationSymbol::Neg(_) => "neg".into(),
                    UnaryOperationSymbol::Not(_) => "not".into(),
                    UnaryOperationSymbol::Inv(_) => "inv".into(),
                    UnaryOperationSymbol::Plus(_) => "pos".into(),
                };

                let mut builder = NodeBuilder::new(Call)
                    .with_property(SourceSpan(node.bounds()))
                    .with_property(Lexeme::new(node))
                    .spawn_in(spawner.into_concrete());

                NodeBuilder::new(Value {
                    inner: Unresolved::Named {
                        context: CORE_PATH,
                        ident,
                    },
                })
                .with_property(SourceSpan(operator.bounds()))
                .with_property(Lexeme::new(operator))
                .spawn_in(builder.spawner::<Lhs>());

                Self::dispatch(&*expr, builder.spawner::<Rhs>(), source)?;

                Ok(builder.id().cast())
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

                let mut builder = NodeBuilder::new(Call)
                    .with_property(SourceSpan(node.bounds()))
                    .with_property(Lexeme::new(node))
                    .spawn_in(spawner.into_concrete());

                NodeBuilder::new(Value {
                    inner: Unresolved::Named {
                        context: CORE_PATH,
                        ident,
                    },
                })
                .with_property(SourceSpan(operation.bounds()))
                .with_property(Lexeme::new(operation))
                .spawn_in(builder.spawner::<Lhs>());

                Self::dispatch(&*left, builder.spawner::<Rhs>(), source)?;
                Self::dispatch(&*right, builder.spawner::<Rhs>(), source)?;

                Ok(builder.id().cast())
            }
            Operation::Access { .. } => Err(crate::Error::Unsupported(node.bounds())),
        }
    }
}

impl<P, T, B: Buffer> Dispatch<P, T, B> for Dispatcher<kodept_rlt::prelude::Expression>
where
    P: HasChild<Literal, T>,
    P: HasChild<Tuple, T>,
    P: HasChild<Value<Unresolved>, T>,
    P: HasChild<AnonFunction<Option<Unresolved>>, T>,
    P: HasChild<If, T>,
{
    type Syntax = kodept_rlt::prelude::Expression;
    type Error = crate::Error;

    #[inline]
    fn dispatch(
        node: &Self::Syntax,
        spawner: impl AnonSpawner<P, T, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId, Self::Error> {
        match node {
            Expression::Lambda(node) => {
                Ok(AnonFunction::from_syntax(node, spawner.into_concrete(), source)?.cast())
            }
            Expression::Term(node) => {
                Ok(Value::from_syntax(node, spawner.into_concrete(), source)?.cast())
            }
            Expression::Literal(node) => {
                Dispatcher::<kodept_rlt::prelude::Literal>::dispatch(node, spawner, source)
            }
            Expression::If(node) => {
                Ok(If::from_syntax(&*node, spawner.into_concrete(), source)?.cast())
            }
        }
    }
}

impl<P, T, B> Dispatch<P, T, B> for Dispatcher<kodept_rlt::prelude::Literal>
where
    P: HasChild<Tuple, T>,
    P: HasChild<Literal, T>,
    B: Buffer,
{
    type Syntax = kodept_rlt::prelude::Literal;
    type Error = crate::Error;

    fn dispatch(
        node: &Self::Syntax,
        spawner: impl AnonSpawner<P, T, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId, Self::Error> {
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
                return Err(WrongLiteralLength(*point, 3));
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
                return Ok(NodeBuilder::new(Tuple)
                    .with_property(SourceSpan(items.left.bounds() + items.right.bounds()))
                    .with_property(Lexeme::new(node))
                    .spawn_in(spawner.into_concrete())
                    .id()
                    .cast());
            }
        };
        Ok(NodeBuilder::new(value)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Lexeme::new(node))
            .spawn_in(spawner.into_concrete())
            .id()
            .cast())
    }
}
