use crate::v3::dispatch::Dispatcher;
use crate::v3::types;
use crate::v3::types::*;
use crate::{Condition, Lhs, Rhs};
use kodept_ast::experimental::{Dispatch, FromSyntax};
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::{Lexeme, Name, SourceSpan};
use kodept_ast::syntax_tree::experimental::{
    Buffer, Constructed, NodeBuilder, Spawner, SpawnerNode,
};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::prelude::*;
use kodept_rlt::traversal::SyntaxNode;
use std::convert::Infallible;

fn type_to_unresolved_type(value: &Type, source: impl CodeHolder) -> Unresolved {
    match value {
        Type::ContextualReference(ctx, ident) => Unresolved::Named {
            context: (ctx, source).into(),
            ident: source.get_chunk_located(ident),
        },
        Type::Reference(ident) => Unresolved::Named {
            context: Path::empty(false),
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

impl<B> FromSyntax<kodept_rlt::prelude::Module, B> for types::Module
where
    B: Buffer,
{
    type Error = crate::Error;

    fn from_syntax(
        node: &kodept_rlt::prelude::Module,
        spawner: impl Spawner<Self, Buffer = B>,
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
        let mut builder = NodeBuilder::new(Module)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Name::new(module_name))
            .with_property(Lexeme::new(node))
            .spawn_in(spawner);

        for item in rest.iter() {
            Dispatcher::dispatch(item, builder.spawner(), source)?;
        }

        Ok(builder.id())
    }
}

impl<B: Buffer> FromSyntax<Enum, B> for UserType {
    type Error = crate::Error;

    fn from_syntax(
        node: &Enum,
        spawner: impl Spawner<Self, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let (name, inner) = match node {
            Enum::Stack { id, contents, .. } => (source.get_chunk_located(id), contents),
            Enum::Heap { .. } => return Err(crate::Error::Unsupported(node.bounds())),
        };

        let mut builder = NodeBuilder::new(UserType)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Name::new(name))
            .with_property(Lexeme::new(node))
            .spawn_in(spawner);

        for variant in inner.into_iter().flat_map(|it| it.inner.as_ref()) {
            let variant_name = source.get_chunk_located(variant);
            NodeBuilder::new(ValueCtor::<Resolved> {
                name: CtorName::Explicit(variant_name),
                params: vec![],
            })
            .with_property(SourceSpan(variant.bounds()))
            .with_property(Lexeme::new(variant))
            .spawn_in(&mut builder.spawner());
        }

        Ok(builder.id())
    }
}

impl<B: Buffer> FromSyntax<Struct, B> for UserType {
    type Error = crate::Error;

    fn from_syntax(
        node: &Struct,
        spawner: impl Spawner<Self, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(&node.id);

        let mut builder = NodeBuilder::new(UserType)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Name::new(name))
            .with_property(Lexeme::new(node))
            .spawn_in(spawner);

        let params = node
            .parameters
            .iter()
            .flat_map(|it| it.inner.as_ref())
            .map(|it| Param::Positional {
                name: Some(source.get_chunk_located(&it.id)),
                ty_id: type_to_unresolved_type(&it.parameter_type, source),
            });

        NodeBuilder::new(ValueCtor {
            name: CtorName::Inline,
            params: params.collect(),
        })
        .with_property(SourceSpan(node.parameters.as_ref().map_or_else(
            || node.id.bounds(),
            |it| it.left.bounds() + it.right.bounds(),
        )))
        .with_property(Lexeme::new(&node.id))
        .spawn_in(&mut builder.spawner());

        for func in node.body.iter().flat_map(|it| it.inner.as_ref()) {
            UserFunction::from_syntax(func, builder.spawner(), source)?;
        }

        Ok(builder.id())
    }
}

impl<B: Buffer> FromSyntax<BodiedFunction, B> for UserFunction<Option<Unresolved>> {
    type Error = crate::Error;

    fn from_syntax(
        node: &BodiedFunction,
        spawner: impl Spawner<Self, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(&node.id);
        let return_type = node
            .return_type
            .as_ref()
            .map(|it| type_to_unresolved_type(&it.1, source));

        let mut builder = NodeBuilder::new(UserFunction {
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
        .with_property(Lexeme::new(node))
        .with_property(Name::new(name))
        .spawn_in(spawner);

        match &*node.body {
            Body::Simplified {
                expression: BlockLevelNode::Function(node),
                ..
            } => {
                return Err(crate::Error::UnexpectedStatement(node.bounds()));
            }
            Body::Simplified {
                expression: BlockLevelNode::InitVar(node),
                ..
            } => {
                return Err(crate::Error::UnexpectedStatement(node.bounds()));
            }
            Body::Simplified {
                expression: BlockLevelNode::Block(node),
                ..
            } => {
                Block::from_syntax(node, builder.spawner(), source)?;
            }
            Body::Simplified {
                expression: BlockLevelNode::Operation(node),
                ..
            } => {
                let mut builder = NodeBuilder::new(Block::<false> {})
                    .with_property(SourceSpan(node.bounds()))
                    .with_property(Lexeme::new(node))
                    .spawn_in(builder.spawner());

                let mut builder = NodeBuilder::new(Link)
                    .clone_property::<SourceSpan>()
                    .clone_property::<Lexeme>()
                    .spawn_in(builder.spawner());

                Dispatcher::<Operation>::dispatch(node, builder.spawner(), source)?;
            }
            Body::Block(list) => {
                Block::from_syntax(list, builder.spawner(), source)?;
            }
        };
        Ok(builder.id())
    }
}

impl<B: Buffer> FromSyntax<ExpressionBlock, B> for Block {
    type Error = crate::Error;

    fn from_syntax(
        node: &ExpressionBlock,
        spawner: impl Spawner<Self, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let mut builder = NodeBuilder::new(Block)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Lexeme::new(node))
            .spawn_in(spawner);

        for statement in node.expression.iter() {
            Dispatcher::<BlockLevelNode>::dispatch(statement, builder.spawner(), source)?;
        }

        Ok(builder.id())
    }
}

impl<B: Buffer> FromSyntax<Term, B> for Value<Unresolved> {
    type Error = Infallible;

    fn from_syntax(
        node: &Term,
        spawner: impl Spawner<Self, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let value = match node {
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

        let builder = NodeBuilder::new(value)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Lexeme::new(node))
            .spawn_in(spawner);

        Ok(builder.id())
    }
}

impl<B: Buffer> FromSyntax<InitializedVariable, B> for super::types::Variable<Option<Unresolved>> {
    type Error = crate::Error;

    fn from_syntax(
        node: &InitializedVariable,
        spawner: impl Spawner<Self, Buffer = B>,
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

        let mut builder = NodeBuilder::new(super::types::Variable {
            mutable,
            annotation: annotation
                .as_ref()
                .map(|it| type_to_unresolved_type(&it.1, source)),
            name: match name.as_ref() {
                "_" => VariableName::Empty,
                _ => VariableName::Name(name),
            },
        })
        .with_property(SourceSpan(node.bounds()))
        .with_property(Lexeme::new(node))
        .spawn_in(spawner);

        Dispatcher::<Operation>::dispatch(&node.expression, builder.spawner(), source)?;

        Ok(builder.id())
    }
}

impl<B: Buffer> FromSyntax<Application, B> for Call {
    type Error = crate::Error;

    fn from_syntax(
        node: &Application,
        spawner: impl Spawner<Self, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let mut builder = NodeBuilder::new(Call)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Lexeme::new(node))
            .spawn_in(spawner);

        Dispatcher::<Operation>::dispatch(&node.expr, builder.spawner::<Lhs>(), source)?;
        for param in node.params.iter().flat_map(|it| it.inner.as_ref()) {
            Dispatcher::<Operation>::dispatch(param, builder.spawner::<Rhs>(), source)?;
        }

        Ok(builder.id())
    }
}

impl<B: Buffer> FromSyntax<Lambda, B> for AnonFunction<Option<Unresolved>> {
    type Error = crate::Error;

    fn from_syntax(
        node: &Lambda,
        spawner: impl Spawner<Self, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let mut builder = NodeBuilder::new(AnonFunction {
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
        .with_property(Lexeme::new(node))
        .with_property(SourceSpan(node.bounds()))
        .spawn_in(spawner);

        let mut block_builder = NodeBuilder::new(Block)
            .with_property(SourceSpan(node.expr.bounds()))
            .with_property(Lexeme::new(&*node.expr))
            .spawn_in(builder.spawner());

        Dispatcher::<Operation>::dispatch(&*node.expr, block_builder.spawner(), source)?;
        block_builder.finish();

        Ok(builder.id())
    }
}

impl<B: Buffer> FromSyntax<IfExpr, B> for If {
    type Error = crate::Error;

    fn from_syntax(
        node: &IfExpr,
        spawner: impl Spawner<Self, Buffer = B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let mut builder = NodeBuilder::new(If)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Lexeme::new(node))
            .spawn_in(spawner);

        fn make_branch<T, B: Buffer>(
            builder: &mut SpawnerNode<If, B>,
            node: &T,
            source: impl CodeHolder,
            condition: fn(&T) -> &Operation,
            body: fn(&T) -> &Body,
        ) -> Result<(), crate::Error>
        where
            T: SyntaxNode,
        {
            let condition = condition(node);
            let body = body(node);
            let mut builder = NodeBuilder::new(Branch)
                .with_property(SourceSpan(condition.bounds() + body.bounds()))
                .with_property(Lexeme::new(node))
                .spawn_in(builder.spawner());
            Dispatcher::<Operation>::dispatch(condition, builder.spawner::<Condition>(), source)?;
            Dispatcher::<Body>::dispatch(body, builder.spawner(), source)?;
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
            let mut builder = NodeBuilder::new(Otherwise)
                .with_property(SourceSpan(node.bounds()))
                .with_property(Lexeme::new(node))
                .spawn_in(builder.spawner());
            Dispatcher::<Body>::dispatch(&el.body, builder.spawner(), source)?;
        }

        Ok(builder.id())
    }
}
