use crate::per_file::utils::{IntoParNodeSystem, IterableSystemParam, ParIterableSystem};
use crate::source::collection::Reporter;
use crate::utils::TryReport;
use kodept_ast::prelude::{HierarchicalQuery, NodeId, Property};
use kodept_ast::properties::{Lexeme, Node, NodeProperty, RequireProperty, SourceSpan};
use kodept_ast::syntax_tree::experimental::{Buffer, NodeBuilder, NodeModification};
use kodept_ast_nodes::{
    AnonFunction, Block, Expression, ForeignFunction, Link, Literal, Module, NormalizedBlock,
    Param, Statement, Tuple, UserFunction, Value, Variable,
};
use kodept_core::code_point::Span;
use kodept_ecs::archetype::Archetype;
use kodept_ecs::component::{Component, ComponentIdFor};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::hierarchy::ChildOf;
use kodept_ecs::lifecycle::{Add, Insert};
use kodept_ecs::query::{Has, With};
use kodept_ecs::schedule::IntoScheduleConfigs;
use kodept_ecs::system::{Commands, On, Query, SystemParam};
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report_macros::Report;

define_phase! {
    pub phase AstNormalizationPhase[AstNormalizationPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        build(engine)
    }
}

fn build(engine: &mut PhaseEngine<AstNormalizationPhase>) {
    engine.add_systems(
        (
            NormalizeBlock::par_system(),
            ensure_no_non_normalized_blocks,
        )
            .chain(),
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

#[derive(Debug, Component, Clone)]
#[component(immutable)]
pub struct InModule(pub NodeId<Module>);
impl NodeProperty for InModule {}
impl RequireProperty<InModule> for Value {}
impl RequireProperty<InModule> for Variable {}
impl RequireProperty<InModule> for UserFunction {}
impl RequireProperty<InModule> for AnonFunction {}
impl RequireProperty<InModule> for ForeignFunction {}
impl RequireProperty<InModule> for Param {}

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

#[derive(SystemParam)]
struct NormalizeBlock<'s> {
    anon_function: ComponentIdFor<'s, AnonFunction>,
    literal: ComponentIdFor<'s, Literal>,
    tuple: ComponentIdFor<'s, Tuple>,
    value: ComponentIdFor<'s, Value>,
    user_function: ComponentIdFor<'s, UserFunction>,
    link: ComponentIdFor<'s, Link>,
    block: ComponentIdFor<'s, Block>,
}

impl NormalizeBlock<'_> {
    fn is_non_normalized(&self, archetype: &Archetype) -> bool {
        archetype.contains(self.anon_function.get())
            || archetype.contains(self.literal.get())
            || archetype.contains(self.tuple.get())
            || archetype.contains(self.value.get())
            || archetype.contains(self.block.get())
    }
}

impl ParIterableSystem for NormalizeBlock<'_> {
    type Iterable = HierarchicalQuery<
        'static,
        'static,
        Block,
        Statement,
        (),
        (&'static Archetype, Property<SourceSpan>, Property<Lexeme>),
    >;

    fn for_each<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let ((), statements) = params;
        let mut linked = false;

        for (statement_id, (archetype, span, lexeme)) in statements.iter().rev() {
            if self.is_non_normalized(archetype) && linked {
                return Err(DanglingExpression { span: span.0 });
            }

            if linked {
                continue;
            }

            if archetype.contains(self.user_function.get()) {
            } else if self.is_non_normalized(archetype) {
                linked = true;
                let statement = modification.remove_child_unchecked::<Statement>(statement_id);

                modification
                    .spawn_child(
                        NodeBuilder::new(Link)
                            .with_property(*span)
                            .with_property(*lexeme),
                    )
                    .add_child_unchecked::<Expression>(statement);
            } else if archetype.contains(self.link.get()) {
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
        Ok(())
    }
}
