use crate::source::collection::Reporter;
use crate::utils::LogSystemEx;
use bevy_ecs::archetype::Archetype;
use bevy_ecs::component::ComponentIdFor;
use bevy_ecs::prelude::{Add, ChildOf, Commands, Has, Insert, On, Query, With};
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::SystemParam;
use kodept_ast::export::Component;
use kodept_ast::prelude::{HierarchicalQuery, NodeId};
use kodept_ast::properties::{Lexeme, Node, SourceSpan};
use kodept_ast::syntax_tree::experimental::{NodeBuilder, NodeModification};
use kodept_ast_nodes::{
    AnonFunction, Block, Expression, Link, Literal, Module, NormalizedBlock, Statement, Tuple,
    TypeAnnotation, UnresolvedName, UserFunction, Value,
};
use kodept_core::code_point::Span;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report_macros::Report;

define_phase! {
    pub phase AstNormalizationPhase[AstNormalizationPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems((
            normalize_blocks.trace_completion(),
            ensure_no_non_normalized_blocks.trace_completion()
        ).chain());

        engine.add_observer(propagate_module_info);
    }
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

#[derive(SystemParam)]
struct StatementComponentIds<'s> {
    anon_function: ComponentIdFor<'s, AnonFunction<TypeAnnotation>>,
    literal: ComponentIdFor<'s, Literal>,
    tuple: ComponentIdFor<'s, Tuple>,
    value: ComponentIdFor<'s, Value<UnresolvedName>>,
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

fn normalize_blocks(
    mut blocks_: HierarchicalQuery<Block, Statement, (), (&Archetype, &SourceSpan, &Lexeme)>,
    statement_component_ids: StatementComponentIds,
    mut commands: Commands,
    mut reporter: Reporter,
) {
    for (id, _, statements) in blocks_.iter_by_layers() {
        let mut modification = NodeModification::new(commands.reborrow(), id);

        let mut linked = false;
        let mut dangling = false;
        for (statement_id, (archetype, span, lexeme)) in statements.into_iter().rev() {
            if statement_component_ids.is_non_normalized(archetype) && linked {
                reporter.report(DanglingExpression { span: span.0 });
                dangling = true;
                continue;
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

        if !dangling {
            modification.transmute(NormalizedBlock {});
        }
    }
}
