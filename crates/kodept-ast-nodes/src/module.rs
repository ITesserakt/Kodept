use crate::term::ReferenceContext;
use crate::Error::{CannotParseFloat, CannotParseInt, NoQuotesInLiteral, WrongLiteralLength};
use bevy_ecs::prelude::{Component, Entity};
use bevy_ecs::relationship::Relationship;
use bigdecimal::{BigDecimal, Num};
use kodept_ast::arity::{Arity, Optional, Plural, Singular};
use kodept_ast::experimental::{AstBuilder, Dispatch, DispatchContext, FromSyntax, SplitRef};
use kodept_ast::prelude::{ASTNode, CodeHolder, NodeId};
use kodept_ast::properties::Node;
use kodept_ast::properties::{HasProperty, Name, RequireProperty, SourceSpan};
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::experimental::SpawnedIn;
use kodept_ast::syntax_tree::experimental::{Buffer, GenericSpawnContext};
use kodept_ast::Str;
use kodept_rlt::exported::Located;
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::new_types::{BinaryOperationSymbol, UnaryOperationSymbol};
use kodept_rlt::prelude::Literal::{Binary, Hex, Octal};
use kodept_rlt::prelude::{
    Application, BlockLevelNode, BodiedFunction, Body, Context, Enum, ExpressionBlock, IfExpr,
    InitializedVariable, Lambda, Operation, Parameter, Struct, Term, TopLevelNode, Type,
    TypedParameter, UntypedParameter,
};
use kodept_rlt::traversal::SyntaxNode;
use num_bigint::BigInt;
pub use refs::*;
use std::borrow::Cow;
use std::convert::Infallible;
use std::str::FromStr;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Modules;

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
pub struct Module;

pub trait IsDeclaration: ASTNode {}
pub struct Declaration;
pub trait IsStatement: ASTNode {}
pub struct Statement;
pub trait IsExpression: ASTNode {}
pub struct Expression;
pub struct Lhs;
pub struct Rhs;
pub struct Condition;

#[derive(Debug, PartialEq)]
pub enum CtorName {
    Inline,
    Explicit(Str),
}

mod refs {
    use crate::term::ReferenceContext;
    use bevy_ecs::prelude::Entity;
    use kodept_ast::Str;

    pub(super) trait NameRef: Send + Sync + 'static {}
    pub(super) trait TypeRef<const REQUIRED: bool>: Send + Sync + 'static {}

    #[derive(Debug, PartialEq)]
    pub enum Unresolved {
        Named {
            context: ReferenceContext,
            ident: Str,
        },
        Tuple(Vec<Unresolved>),
    }

    #[derive(Debug, PartialEq)]
    pub struct Resolved(pub Entity);

    impl NameRef for Unresolved {}
    impl NameRef for Resolved {}

    impl TypeRef<true> for Unresolved {}
    impl<const REQUIRED: bool> TypeRef<REQUIRED> for Resolved {}

    impl TypeRef<false> for Option<Unresolved> {}
}

#[derive(Debug, PartialEq)]
pub enum Param<T> {
    Positional {
        name: Option<Str>,
        ty_id: T,
    },
    Named {
        name: Str,
        ty_id: T,
        default_expr_id: Option<Entity>,
    },
}

#[derive(Debug, PartialEq, Component)]
pub struct TypeCtor<T> {
    pub name: CtorName,
    pub params: Vec<Param<T>>,
}

#[derive(Debug, PartialEq, Component)]
pub struct UserType;

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
pub struct PrimType;

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
pub struct UserFunction<T> {
    params: Vec<Param<T>>,
    return_type: T,
}

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
pub struct ForeignFunction<T> {
    params: Vec<T>,
    return_type: T,
}

#[derive(Debug, PartialEq, Component)]
pub struct AnonFunction<T> {
    params: Vec<Param<T>>,
    return_type: T,
}

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
pub struct Variable<T> {
    mutable: bool,
    annotation: T,
}

#[derive(Debug, PartialEq, Component)]
pub struct Block;

#[derive(Debug, PartialEq, Component)]
pub struct Value<T> {
    inner: T,
}

#[derive(Debug, PartialEq, Component)]
pub enum Literal {
    Integer(BigInt),
    Floating(BigDecimal),
    Char(char),
    String(Str),
}

#[derive(Debug, PartialEq, Component)]
pub struct Tuple;

#[derive(Debug, PartialEq, Component)]
pub struct Call;

#[derive(Debug, PartialEq, Component)]
pub struct If;

#[derive(Debug, PartialEq, Component)]
pub struct Branch;

#[derive(Debug, PartialEq, Component)]
pub struct Otherwise;

impl ASTNode for Module {}
impl RequireProperty<Name> for Module {}
impl<T: IsDeclaration> HasChild<T, Declaration> for Module {
    type Arity = Plural;
}

impl ASTNode for UserType {}
impl IsDeclaration for UserType {}
impl HasProperty<Name> for UserType {}
impl<T: TypeRef<true>> HasChild<TypeCtor<T>, ()> for UserType {
    type Arity = Plural;
}
impl<T: TypeRef<false>> HasChild<UserFunction<T>, Declaration> for UserType {
    type Arity = Plural;
}

impl<T: TypeRef<true>> ASTNode for TypeCtor<T> {}

impl ASTNode for PrimType {}
impl IsDeclaration for PrimType {}

impl<T: TypeRef<false>> ASTNode for UserFunction<T> {}
impl<T: TypeRef<false>> IsDeclaration for UserFunction<T> {}
impl<T: TypeRef<false>> IsStatement for UserFunction<T> {}
impl<T: TypeRef<false>> RequireProperty<Name> for UserFunction<T> {}
impl<U: TypeRef<false>> HasChild<Block, ()> for UserFunction<U> {
    type Arity = Singular;
}

impl<T: TypeRef<true>> ASTNode for ForeignFunction<T> {}
impl<T: TypeRef<true>> IsDeclaration for ForeignFunction<T> {}
impl<T: TypeRef<true>> RequireProperty<Name> for ForeignFunction<T> {}

impl<T: TypeRef<false>> ASTNode for AnonFunction<T> {}
impl<T: TypeRef<false>> IsExpression for AnonFunction<T> {}
impl<T: TypeRef<false>> IsStatement for AnonFunction<T> {}
impl<T: TypeRef<false>> HasChild<Block, ()> for AnonFunction<T> {
    type Arity = Singular;
}

impl<T: TypeRef<false>> ASTNode for Variable<T> {}
impl<T: TypeRef<false>> IsStatement for Variable<T> {}
impl<T: TypeRef<false>> RequireProperty<Name> for Variable<T> {}
impl<T: IsExpression, U: TypeRef<false>> HasChild<T, Expression> for Variable<U> {
    type Arity = Singular;
}

impl ASTNode for Block {}
impl IsStatement for Block {}
impl IsExpression for Block {}
impl<T: IsStatement> HasChild<T, Statement> for Block {
    type Arity = Plural;
}

impl<T: NameRef> ASTNode for Value<T> {}
impl<T: NameRef> IsExpression for Value<T> {}
impl<T: NameRef> IsStatement for Value<T> {}

impl ASTNode for Literal {}
impl IsExpression for Literal {}
impl IsStatement for Literal {}

impl ASTNode for Tuple {}
impl IsExpression for Tuple {}
impl IsStatement for Tuple {}
impl<T: IsExpression> HasChild<T, Expression> for Tuple {
    type Arity = Plural;
}

impl ASTNode for Call {}
impl IsStatement for Call {}
impl IsExpression for Call {}
impl<T: IsExpression> HasChild<T, Lhs> for Call {
    type Arity = Singular;
}
impl<T: IsExpression> HasChild<T, Rhs> for Call {
    type Arity = Plural;
}

impl ASTNode for If {}
impl IsStatement for If {}
impl IsExpression for If {}
impl HasChild<Branch, ()> for If {
    type Arity = Plural;
}
impl HasChild<Otherwise, ()> for If {
    type Arity = Optional;
}

impl ASTNode for Branch {}
impl<T: IsExpression> HasChild<T, Condition> for Branch {
    type Arity = Singular;
}
impl<T: IsStatement> HasChild<T, Statement> for Branch {
    type Arity = Singular;
}

impl ASTNode for Otherwise {}
impl<T: IsStatement> HasChild<T, Statement> for Otherwise {
    type Arity = Singular;
}

fn build_from_top_level(
    value: &TopLevelNode,
    source: impl CodeHolder,
    parent_builder: &mut AstBuilder<impl SpawnedIn<Module>>,
) -> Result<(), crate::Error> {
    match value {
        TopLevelNode::Enum(node) => parent_builder.with_child::<_, UserType, _>(node, source)?,
        TopLevelNode::Struct(node) => parent_builder.with_child::<_, UserType, _>(node, source)?,
        TopLevelNode::BodiedFunction(node) => {
            parent_builder.with_child::<_, UserFunction<_>, _>(node, source)?
        }
    };

    Ok(())
}

fn type_to_unresolved_type(value: &Type, source: impl CodeHolder) -> Unresolved {
    match value {
        Type::ContextualReference(ctx, ident) => Unresolved::Named {
            context: (ctx, source).into(),
            ident: source.get_chunk_located(ident),
        },
        Type::Reference(ident) => Unresolved::Named {
            context: ReferenceContext::empty(false),
            ident: source.get_chunk_located(ident),
        },
        Type::Tuple(items) => Unresolved::Tuple(
            items
                .0
                .inner
                .iter()
                .map(|it| type_to_unresolved_type(it, source))
                .collect(),
        ),
    }
}

impl<T: CodeHolder> From<(&Context, T)> for ReferenceContext {
    fn from((value, source): (&Context, T)) -> Self {
        let (is_global, items) = value.unfold();
        if is_global.is_some() {
            ReferenceContext::global(items.into_iter().map(|it| source.get_chunk_located(it)))
        } else {
            ReferenceContext::local(items.into_iter().map(|it| source.get_chunk_located(it)))
        }
    }
}

impl FromSyntax<kodept_rlt::prelude::Module> for Module {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &kodept_rlt::prelude::Module,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let (module_name, rest) = match node {
            kodept_rlt::prelude::Module::Global { id, rest, .. } => {
                (source.get_chunk_located(id), rest)
            }
            kodept_rlt::prelude::Module::Ordinary { id, rest, .. } => {
                (source.get_chunk_located(id), rest)
            }
        };
        let mut builder = AstBuilder::new(Module)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Name::new(module_name))
            .spawn_in(spawner);

        for top_level in rest {
            build_from_top_level(top_level, source, &mut builder)?;
        }

        Ok(builder.finish())
    }
}

impl FromSyntax<Enum> for UserType {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &Enum,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let (name, inner) = match node {
            Enum::Stack { id, contents, .. } => (source.get_chunk_located(id), contents),
            Enum::Heap { .. } => return Err(crate::Error::Unsupported(node.bounds())),
        };

        let mut builder = AstBuilder::new(UserType)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Name::new(name))
            .spawn_in(spawner);

        for variant in inner.into_iter().flat_map(|it| it.inner.as_ref()) {
            let variant_name = source.get_chunk_located(variant);
            builder.with_dispatch_fn(variant, |node, spawner| {
                Ok::<_, Infallible>(
                    AstBuilder::new(TypeCtor::<Resolved> {
                        name: CtorName::Explicit(variant_name),
                        params: vec![],
                    })
                    .with_property(SourceSpan(node.bounds()))
                    .spawn_in((spawner, node))
                    .finish_any(),
                )
            })?;
        }

        Ok(builder.finish())
    }
}

impl FromSyntax<Struct> for UserType {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &Struct,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(&node.id);

        let mut builder = AstBuilder::new(UserType)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Name::new(name))
            .spawn_in(spawner);
        let params = node
            .parameters
            .iter()
            .flat_map(|it| it.inner.as_ref())
            .map(|it| Param::Positional {
                name: Some(source.get_chunk_located(&it.id)),
                ty_id: type_to_unresolved_type(&it.parameter_type, source),
            });

        builder.with_dispatch_fn(node, |node, spawner| {
            let bounds = match &node.parameters {
                Some(x) => x.left.bounds() + x.right.bounds(),
                None => node.id.bounds(),
            };

            Ok::<_, Infallible>(
                AstBuilder::new(TypeCtor {
                    name: CtorName::Inline,
                    params: params.collect(),
                })
                .with_property(SourceSpan(bounds))
                .spawn_in((spawner, node))
                .finish_any(),
            )
        })?;

        if let Some(body) = &node.body {
            builder.with_children::<_, UserFunction<_>, _>(body.inner.as_ref(), source)?;
        }

        Ok(builder.finish())
    }
}

impl FromSyntax<BodiedFunction> for UserFunction<Option<Unresolved>> {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &BodiedFunction,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(&node.id);
        let return_type = node
            .return_type
            .as_ref()
            .map(|it| type_to_unresolved_type(&it.1, source));

        let mut builder = AstBuilder::new(UserFunction {
            params: node
                .params
                .iter()
                .flat_map(|it| it.inner.as_ref())
                .map(|it| match it {
                    Parameter::Typed(TypedParameter { id, parameter_type }) => Param::Positional {
                        name: Some(source.get_chunk_located(id)),
                        ty_id: Some(type_to_unresolved_type(parameter_type, source)),
                    },
                    Parameter::Untyped(UntypedParameter { id }) => Param::Positional {
                        name: Some(source.get_chunk_located(id)),
                        ty_id: None,
                    },
                })
                .collect(),
            return_type,
        })
        .with_property(SourceSpan(node.bounds()))
        .with_property(Name::new(name))
        .spawn_in(spawner);

        match &*node.body {
            Body::Simplified { expression, .. } => {
                builder.with_dispatch_fn(expression, |node, spawner| {
                    Ok::<_, crate::Error>(
                        AstBuilder::new(Block)
                            .with_property(SourceSpan(node.bounds()))
                            .spawn_in((spawner, node))
                            .with_dispatch::<Dispatcher<_>, _, _>(node, source)?
                            .finish_any(),
                    )
                })?;
            }
            Body::Block(list) => {
                builder.with_child::<_, Block, _>(list, source)?;
            }
        };

        Ok(builder.finish())
    }
}

impl FromSyntax<ExpressionBlock> for Block {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &ExpressionBlock,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let mut builder = AstBuilder::new(Block)
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner);

        builder.with_dispatches::<Dispatcher<_>, _, _>(node.expression.as_ref(), source)?;

        Ok(builder.finish())
    }
}

impl FromSyntax<InitializedVariable> for Variable<Option<Unresolved>> {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &InitializedVariable,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let (name, mutable, annotation) = match &node.variable {
            kodept_rlt::prelude::Variable::Immutable {
                id, assigned_type, ..
            } => (source.get_chunk_located(id), false, assigned_type),
            kodept_rlt::prelude::Variable::Mutable {
                id, assigned_type, ..
            } => (source.get_chunk_located(id), true, assigned_type),
        };

        let mut builder = AstBuilder::new(Variable {
            mutable,
            annotation: annotation
                .as_ref()
                .map(|it| type_to_unresolved_type(&it.1, source)),
        })
        .with_property(SourceSpan(node.bounds()))
        .with_property(Name::new(name))
        .spawn_in(spawner);

        builder.with_dispatch::<Dispatcher<_>, _, _>(&node.expression, source)?;

        Ok(builder.finish())
    }
}

impl FromSyntax<Application> for Call {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &Application,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let mut builder = AstBuilder::new(Call)
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner);
        builder.with_dispatch::<Dispatcher<_>, Lhs, _>(&node.expr, source)?;
        if let Some(params) = &node.params {
            builder.with_dispatches::<Dispatcher<_>, Rhs, _>(params.inner.as_ref(), source)?;
        }

        Ok(builder.finish())
    }
}

impl FromSyntax<Lambda> for AnonFunction<Option<Unresolved>> {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &Lambda,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let mut builder = AstBuilder::new(AnonFunction {
            return_type: None,
            params: node
                .binds
                .inner
                .iter()
                .map(|it| match it {
                    Parameter::Typed(TypedParameter { id, parameter_type }) => Param::Positional {
                        name: Some(source.get_chunk_located(id)),
                        ty_id: Some(type_to_unresolved_type(parameter_type, source)),
                    },
                    Parameter::Untyped(UntypedParameter { id }) => Param::Positional {
                        name: Some(source.get_chunk_located(id)),
                        ty_id: None,
                    },
                })
                .collect(),
        })
        .with_property(SourceSpan(node.bounds()))
        .spawn_in(spawner);

        builder.with_dispatch_fn(&*node.expr, |node, spawner| {
            Ok::<_, crate::Error>(
                AstBuilder::new(Block)
                    .with_property(SourceSpan(node.bounds()))
                    .spawn_in((spawner, node))
                    .with_dispatch::<Dispatcher<_>, _, _>(node, source)?
                    .finish_any(),
            )
        })?;

        Ok(builder.finish())
    }
}

impl FromSyntax<IfExpr> for If {
    type Error = crate::Error;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &IfExpr,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let mut builder = AstBuilder::new(If)
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner);

        fn make_branch<T>(
            builder: &mut AstBuilder<impl SpawnedIn<If>>,
            node: &T,
            source: impl CodeHolder,
            condition: fn(&T) -> &Operation,
            body: fn(&T) -> &Body,
        ) -> Result<(), crate::Error>
        where
            T: SyntaxNode,
        {
            builder.with_dispatch_fn(node, move |node, spawner| {
                Ok::<_, crate::Error>(
                    AstBuilder::new(Branch)
                        .with_property(SourceSpan(condition(node).bounds() + body(node).bounds()))
                        .spawn_in((spawner, node))
                        .with_dispatch::<Dispatcher<_>, Condition, _>(condition(node), source)?
                        .with_dispatch::<Dispatcher<_>, Statement, _>(body(node), source)?
                        .finish_any(),
                )
            })?;
            Ok(())
        }

        make_branch(
            &mut builder,
            node,
            source,
            |it| &it.condition,
            |it| &it.body,
        )?;

        for else_ifs in node.elif.iter() {
            make_branch(
                &mut builder,
                else_ifs,
                source,
                |it| &it.condition,
                |it| &it.body,
            )?;
        }

        if let Some(el) = &node.el {
            builder.with_dispatch_fn(el, |node, spawner| {
                Ok::<_, crate::Error>(
                    AstBuilder::new(Otherwise)
                        .with_property(SourceSpan(node.bounds()))
                        .spawn_in((spawner, node))
                        .with_dispatch::<Dispatcher<_>, Statement, _>(&node.body, source)?
                        .finish_any(),
                )
            })?;
        }

        Ok(builder.finish())
    }
}

struct Dispatcher<'a, T>(&'a T);

impl<'a, T> SplitRef<'a, T> for Dispatcher<'a, T> {
    fn split(self) -> (Self, &'a T) {
        let reference = self.0;
        (self, reference)
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
        match self.0 {
            Operation::Block(node) => spawner.forward::<_, Block>(node, source),
            Operation::Expression(node) => spawner.dispatch::<Dispatcher<_>>(node, source),
            Operation::Application(node) => spawner.forward::<_, Call>(node, source),
            Operation::Unary { operator, expr } => {
                let context = ReferenceContext::global(["Core", "Traits"]);
                let ident = match operator {
                    UnaryOperationSymbol::Neg(_) => "neg".into(),
                    UnaryOperationSymbol::Not(_) => "not".into(),
                    UnaryOperationSymbol::Inv(_) => "inv".into(),
                    UnaryOperationSymbol::Plus(_) => "pos".into(),
                };

                let mut builder = AstBuilder::new(Call)
                    .with_property(SourceSpan(self.0.bounds()))
                    .spawn_in((spawner, self.0));

                builder.with_dispatch_fn::<_, Lhs, _, Infallible>(operator, |node, spawner| {
                    Ok(AstBuilder::new(Value {
                        inner: Unresolved::Named { context, ident },
                    })
                    .with_property(SourceSpan(node.bounds()))
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
                let context = ReferenceContext::global(["Core", "Traits"]);
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
                    .spawn_in((spawner, self.0));

                builder.with_dispatch_fn::<_, Lhs, _, Infallible>(operation, |node, spawner| {
                    Ok(AstBuilder::new(Value {
                        inner: Unresolved::Named { context, ident },
                    })
                    .with_property(SourceSpan(node.bounds()))
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
                    .spawn_in((spawner, node))
                    .with_dispatches::<Dispatcher<_>, _, _>(items.inner.as_ref(), source)?
                    .finish_any())
            }
        };
        Ok(AstBuilder::new(value)
            .with_property(SourceSpan(node.bounds()))
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
                    context: ReferenceContext::empty(false),
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
                    context: ReferenceContext::empty(false),
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
            .spawn_in((spawner, self.0));

        Ok(builder.finish_any())
    }
}

impl<'a, T> From<&'a T> for Dispatcher<'a, T> {
    fn from(value: &'a T) -> Self {
        Self(value)
    }
}
