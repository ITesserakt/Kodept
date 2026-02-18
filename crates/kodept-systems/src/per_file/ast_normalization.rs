use crate::source::collection::Reporter;
use crate::utils::{LogSystemEx, ReportSystemEx};
use kodept_ast::prelude::{ASTNode, ChildrenFetch, HierarchicalQuery, NodeId};
use kodept_ast::properties::{Lexeme, Node, SourceSpan};
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::experimental::{NodeBuilder, NodeModification};
use kodept_ast_nodes::{
    AnonFunction, Block, Expression, Link, Literal, Module, NormalizedBlock, Param, ResolvedType,
    ResolvedTypeAnnotation, Statement, Tuple, TypeAnnotation, UnresolvedType, UserFunction, Value,
    ValueCtor,
};
use kodept_core::code_point::Span;
use kodept_core::structure::SpanBounds;
use kodept_ecs::archetype::Archetype;
use kodept_ecs::component::{Component, ComponentIdFor};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::hierarchy::ChildOf;
use kodept_ecs::lifecycle::{Add, Insert};
use kodept_ecs::query::{Has, With};
use kodept_ecs::schedule::IntoScheduleConfigs;
use kodept_ecs::system::{Commands, On, Query, Res, SystemParam};
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report_macros::Report;
use kodept_rlt::traversal::SyntaxNode;

define_phase! {
    pub phase AstNormalizationPhase[AstNormalizationPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        build(engine)
    }
}

fn build(engine: &mut PhaseEngine<AstNormalizationPhase>) {
    engine.add_systems(
        (
            normalize_blocks.trace_completion(),
            ensure_no_non_normalized_blocks,
        )
            .chain(),
    );

    engine.add_systems(
        ensure_named_params_are_at_the_end(
            |it: &ValueCtor<UnresolvedType>| &*it.params,
            |_: &kodept_rlt::prelude::Struct| None,
        )
        .trace_completion()
        .extract_reports(),
    );
    engine.add_systems(
        ensure_named_params_are_at_the_end(
            |it: &ValueCtor<ResolvedType>| &*it.params,
            |_: &kodept_rlt::prelude::Struct| None,
        )
        .trace_completion()
        .extract_reports(),
    );
    engine.add_systems(
        ensure_named_params_are_at_the_end(
            |it: &UserFunction<TypeAnnotation>| &*it.params,
            |it: &kodept_rlt::prelude::BodiedFunction| {
                it.params
                    .as_ref()
                    .map(|it| it.left.bounds() + it.right.bounds())
                    .or(Some(it.id.bounds()))
            },
        )
        .trace_completion()
        .extract_reports(),
    );
    engine.add_systems(
        ensure_named_params_are_at_the_end(
            |it: &UserFunction<ResolvedTypeAnnotation>| &*it.params,
            |it: &kodept_rlt::prelude::BodiedFunction| {
                it.params
                    .as_ref()
                    .map(|it| it.left.bounds() + it.right.bounds())
                    .or(Some(it.id.bounds()))
            },
        )
        .trace_completion()
        .extract_reports(),
    );

    engine.add_observer(propagate_module_info);

    #[cfg(feature = "reflection")]
    engine.add_systems(register_reflection_info);
}

#[cfg(feature = "reflection")]
fn register_reflection_info(
    mut debug_registry: bevy_ecs::prelude::If<
        bevy_ecs::prelude::ResMut<kodept_ast::resource::reflection::DebugRegistry>,
    >,
) {
    debug_registry.register::<InModule>();
}

#[derive(Debug, Report)]
#[severity("error")]
#[message("Expression in this position is unexpected")]
#[note("Remove, assign to a variable, call or link this expression")]
struct DanglingExpression {
    #[primary_label("expected statement")]
    span: Span,
}

#[derive(Debug, Report)]
#[severity("bug")]
#[message("Unexpected non-normalized block: {}", self.id)]
struct UnexpectedNonNormalizedBlock {
    id: NodeId<Block>,
    #[primary_label("this block should be normalized")]
    span: Span,
}

#[derive(Debug, Report)]
#[severity("error")]
#[message("Named parameters should appear last")]
struct NamedParamsShouldBeLast {
    #[primary_label]
    span: Span,
}

#[derive(SystemParam)]
struct StatementComponentIds<'s> {
    anon_function: ComponentIdFor<'s, AnonFunction<TypeAnnotation>>,
    literal: ComponentIdFor<'s, Literal>,
    tuple: ComponentIdFor<'s, Tuple>,
    value: ComponentIdFor<'s, Value>,
    user_function: ComponentIdFor<'s, UserFunction<TypeAnnotation>>,
    link: ComponentIdFor<'s, Link>,
}

#[derive(Debug, Component, Clone)]
#[component(immutable)]
pub struct InModule(pub NodeId<Module>);

impl StatementComponentIds<'_> {
    fn is_non_normalized(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.anon_function.get())
            || archetype.contains(self.literal.get())
            || archetype.contains(self.tuple.get())
            || archetype.contains(self.value.get())
    }
}

fn propagate_module_info(
    parent_changed: On<Insert, ChildOf>,
    nodes: Query<(&ChildOf, Option<&InModule>, Has<Module>), With<Node>>,
    mut commands: Commands,
) {
    let this = parent_changed.entity;
    let mut current = this;
    loop {
        match nodes.get(current) {
            Ok((_, _, true)) => {
                commands.entity(this).insert(InModule(current.into()));
                return;
            }
            Ok((_, Some(value), false)) if current != this => {
                commands.entity(this).insert(value.clone());
                return;
            }
            Ok((ChildOf(parent), _, _)) => {
                current = *parent;
                continue;
            }
            Err(_) => return,
        }
    }
}

fn ensure_no_non_normalized_blocks(
    blocks: Query<(NodeId<Block>, &SourceSpan)>,
    mut reporter: Reporter,
    mut commands: Commands,
) {
    blocks
        .iter()
        .for_each(|(id, span)| reporter.report(UnexpectedNonNormalizedBlock { id, span: span.0 }));
    commands.add_observer(
        |block_added: On<Add, Block>, spans: Query<&SourceSpan>, mut reporter: Reporter| {
            let span = spans.get(block_added.entity).unwrap();
            reporter.report(UnexpectedNonNormalizedBlock {
                id: block_added.entity.into(),
                span: span.0,
            })
        },
    );
}

fn normalize_block(
    block_id: NodeId<Block>,
    statements: ChildrenFetch<(&Archetype, &SourceSpan, &Lexeme), Block, Statement>,
    mut modification: NodeModification<Block, Commands>,
    statement_component_ids: &StatementComponentIds,
) -> Option<DanglingExpression> {
    let mut linked = false;

    for (statement_id, (archetype, span, lexeme)) in statements.into_iter().rev() {
        if statement_component_ids.is_non_normalized(archetype) && linked {
            return Some(DanglingExpression { span: span.0 });
        }

        if linked {
            continue;
        }

        if archetype.contains(statement_component_ids.user_function.get()) {
        } else if statement_component_ids.is_non_normalized(archetype) {
            linked = true;
            let statement = modification.remove_child_unchecked::<Statement>(statement_id);

            modification
                .spawn_child(
                    NodeBuilder::new(Link)
                        .with_property(*span)
                        .with_property(*lexeme),
                )
                .add_child_unchecked::<Expression>(statement);
        } else if archetype.contains(statement_component_ids.link.get()) {
            linked = true;
        } else {
            linked = true;
            modification
                .spawn_child(
                    NodeBuilder::new(Link)
                        .with_property(*span)
                        .with_property(*lexeme),
                )
                .spawn_child(
                    NodeBuilder::new(Tuple)
                        .with_property(*span)
                        .with_property(*lexeme),
                );
        }
    }

    modification.transmute(NormalizedBlock {});
    None
}

#[cfg(not(feature = "parallel"))]
fn normalize_blocks(
    mut blocks: HierarchicalQuery<Block, Statement, (), (&Archetype, &SourceSpan, &Lexeme)>,
    statement_component_ids: StatementComponentIds,
    mut commands: Commands,
    mut reporter: Reporter,
) {
    for (id, _, statements) in blocks.iter_by_layers() {
        if let Some(dangling) = normalize_block(
            id,
            statements,
            NodeModification::new(commands.reborrow(), id),
            &statement_component_ids,
        ) {
            reporter.report(dangling);
        }
    }
}

#[cfg(feature = "parallel")]
fn normalize_blocks(
    mut blocks: HierarchicalQuery<Block, Statement, (), (&Archetype, &SourceSpan, &Lexeme)>,
    statement_component_ids: StatementComponentIds,
    commands: kodept_ecs::system::ParallelCommands,
    reporter: crate::source::collection::ParallelReporter,
) {
    blocks.par_iter_by_layers(|id, _, statements| {
        commands.command_scope(|c| {
            if let Some(dangling) = normalize_block(
                id,
                statements,
                NodeModification::new(c, id),
                &statement_component_ids,
            ) {
                reporter.report(dangling);
            }
        });
    })
}

fn ensure_named_params_are_at_the_end<T: ASTNode, U, L: SyntaxNode>(
    mut get_params: impl FnMut(&T) -> &[Param<U>],
    mut get_span: impl FnMut(&L) -> Option<Span>,
) -> impl FnMut(
    Query<(&T, &Lexeme, &SourceSpan)>,
    Res<SyntaxResolver>,
) -> Result<(), NamedParamsShouldBeLast> {
    move |query, syntax| {
        for (item, lexeme, span) in query {
            let span = syntax
                .try_get::<L>(lexeme.0)
                .ok()
                .and_then(|it| get_span(it))
                .unwrap_or(span.0);

            let params = get_params(item);
            let mut is_named_params = false;
            for param in params {
                match (is_named_params, param) {
                    (false, Param::Positional { .. }) => continue,
                    (false, Param::Named { .. }) => is_named_params = true,
                    (true, Param::Named { .. }) => continue,
                    (true, Param::Positional { .. }) => {
                        return Err(NamedParamsShouldBeLast { span });
                    }
                }
            }
        }
        Ok(())
    }
}
