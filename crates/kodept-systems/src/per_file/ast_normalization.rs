use crate::source::collection::Reporter;
use crate::utils::LogSystemEx;
use bevy_ecs::archetype::Archetype;
use bevy_ecs::component::ComponentIdFor;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use kodept_ast::experimental::AstBuilder;
use kodept_ast::properties::{Lexeme, Node, SourceSpan};
use kodept_ast::relationship::Nodes;
use kodept_ast::syntax_tree::experimental::NodeModification;
use kodept_ast_nodes::{
    AnonFunction, Block, Link, Literal, Statement, Tuple, Unresolved, UserFunction, Value, Variable,
};
use kodept_core::code_point::Span;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report_macros::Report;
use tracing::error;

define_phase! {
    pub phase AstNormalizationPhase[AstNormalizationPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(normalize_blocks.trace_completion());
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

#[derive(SystemParam)]
struct StatementComponentIds<'s> {
    anon_function: ComponentIdFor<'s, AnonFunction<Option<Unresolved>>>,
    literal: ComponentIdFor<'s, Literal>,
    tuple: ComponentIdFor<'s, Tuple>,
    value: ComponentIdFor<'s, Value<Unresolved>>,
    user_function: ComponentIdFor<'s, UserFunction<Option<Unresolved>>>,
    variable: ComponentIdFor<'s, Variable<Option<Unresolved>>>,
}

impl StatementComponentIds<'_> {
    fn is_non_normalized(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.anon_function.get())
            || archetype.contains(self.literal.get())
            || archetype.contains(self.tuple.get())
            || archetype.contains(self.value.get())
    }

    fn is_pure(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.user_function.get()) || archetype.contains(self.variable.get())
    }
}

fn normalize_blocks(
    blocks: Query<(Entity, &Nodes<Statement>), With<Block>>,
    statements: Query<(&Archetype, &SourceSpan, &Lexeme), With<Node>>,
    statement_component_ids: StatementComponentIds,
    mut commands: Commands,
    mut reporter: Reporter,
) {
    for (id, statement_ids) in blocks {
        // if block contains no statements or all of them are pure
        // then just add a link to unit
        if statements
            .iter_many(statement_ids)
            .all(|it| statement_component_ids.is_pure(&it.0))
        {
            NodeModification::<Block>::new_unchecked(&mut commands, id)
                .spawn_child(
                    AstBuilder::new(Link)
                        .clone_property::<SourceSpan>()
                        .clone_property::<Lexeme>(),
                )
                .spawn_child(
                    AstBuilder::new(Tuple)
                        .clone_property::<SourceSpan>()
                        .clone_property::<Lexeme>(),
                );
            continue;
        }

        let mut linked = false;
        let mut dangling = false;
        let mut iter = statement_ids.iter().enumerate();

        loop {
            let Some((index, statement_id)) = iter.next_back() else {
                break;
            };
            let Ok((archetype, span, lexeme)) = statements.get(statement_id) else {
                error!("AST Node does not have span or archetype");
                break;
            };

            if archetype.contains(statement_component_ids.user_function.get()) {
                continue;
            }

            if archetype.contains(statement_component_ids.variable.get()) && !linked {
                linked = true;
                NodeModification::<Block>::new_unchecked(&mut commands, id)
                    .spawn_child(
                        AstBuilder::new(Link)
                            .with_property(*span)
                            .with_property(*lexeme),
                    )
                    .place_at(index)
                    .spawn_child(
                        AstBuilder::new(Tuple)
                            .with_property(*span)
                            .with_property(*lexeme),
                    );
            } else if statement_component_ids.is_non_normalized(archetype) && !linked {
                let mut modification = NodeModification::<Block>::new_unchecked(&mut commands, id);
                let statement = modification.remove_child_unchecked(statement_id);

                modification
                    .spawn_child(
                        AstBuilder::new(Link).clone_property::<SourceSpan>(), // .clone_property::<Lexeme>(),
                    )
                    .place_at(index)
                    .add_child_unchecked(statement);
            } else if statement_component_ids.is_non_normalized(archetype) && linked {
                reporter.report(DanglingExpression { span: span.0 });
                dangling = true;
            }

            if !statement_component_ids.is_pure(archetype) {
                linked = true;
            }
        }

        if !dangling {
            commands.entity(id).remove::<Block>().insert(Block::<true>);
        }
    }
}
