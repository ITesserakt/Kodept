use crate::source::collection::Reporter;
use crate::utils::LogSystemEx;
use bevy_ecs::archetype::Archetype;
use bevy_ecs::component::ComponentIdFor;
use bevy_ecs::prelude::Commands;
use bevy_ecs::system::SystemParam;
use kodept_ast::prelude::HierarchicalQuery;
use kodept_ast::properties::{Lexeme, SourceSpan};
use kodept_ast::syntax_tree::experimental::{NodeBuilder, NodeModification};
use kodept_ast_nodes::{
    AnonFunction, Block, Link, Literal, Statement, Tuple, Unresolved, UserFunction, Value, Variable,
};
use kodept_core::code_point::Span;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report_macros::Report;

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
    blocks_: HierarchicalQuery<Block, Statement, (), (&Archetype, &SourceSpan, &Lexeme)>,
    statement_component_ids: StatementComponentIds,
    mut commands: Commands,
    mut reporter: Reporter,
) {
    for (id, _, statements) in blocks_.iter_by_layers() {
        let mut modification = NodeModification::new(commands.reborrow(), id);

        if statements
            .iter()
            .all(|(_, (archetype, _, _))| statement_component_ids.is_pure(archetype))
        {
            modification
                .spawn_child(
                    NodeBuilder::new(Link)
                        .clone_property::<SourceSpan>()
                        .clone_property::<Lexeme>(),
                )
                .spawn_child(
                    NodeBuilder::new(Tuple)
                        .clone_property::<SourceSpan>()
                        .clone_property::<Lexeme>(),
                );
            continue;
        }

        let mut linked = false;
        let mut dangling = false;
        for (statement_id, (archetype, span, lexeme)) in statements.into_iter().rev() {
            if archetype.contains(statement_component_ids.user_function.get()) {
                continue;
            }

            if statement_component_ids.is_non_normalized(archetype) && linked {
                reporter.report(DanglingExpression { span: span.0 });
                dangling = true;
                continue;
            }

            if linked {
                continue;
            }

            linked = true;
            if archetype.contains(statement_component_ids.variable.get()) {
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
            } else if statement_component_ids.is_non_normalized(archetype) {
                let statement = modification.remove_child_unchecked(statement_id);

                modification
                    .spawn_child(
                        NodeBuilder::new(Link)
                            .with_property(*span)
                            .with_property(*lexeme),
                    )
                    .add_child_unchecked(statement);
            }
        }

        drop(modification);
        if !dangling {
            commands
                .entity(id.entity())
                .remove::<Block>()
                .insert(Block::<true>);
        }
    }
}
