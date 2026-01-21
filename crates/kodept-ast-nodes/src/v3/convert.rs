use crate::v2::term::ReferenceContext;
use crate::v3::dispatch::Dispatcher;
use crate::v3::tags::*;
use crate::v3::types::*;
use bevy_ecs::prelude::Name;
use bevy_ecs::relationship::Relationship;
use kodept_ast::experimental::{AstBuilder, FromSyntax};
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::SourceSpan;
use kodept_ast::syntax_tree::experimental::{Buffer, GenericSpawnContext, SpawnedIn};
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

impl FromSyntax<kodept_rlt::prelude::Module> for super::types::Module {
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

        builder.with_dispatches::<Dispatcher<_>, _, _>(rest.as_ref(), source)?;

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

impl FromSyntax<InitializedVariable> for super::types::Variable<Option<Unresolved>> {
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

        let mut builder = AstBuilder::new(super::types::Variable {
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
