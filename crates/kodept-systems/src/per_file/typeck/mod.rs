mod first_part;

use std::collections::HashSet;
use crate::per_file::typeck::first_part::{
    CollectMonomorphicContext, FillParamTyStubs, MonomorphicContext, PartiallyTypechecked,
    TypeckBlock, TypeckCall, TypeckFunction, TypeckIf, TypeckLink, TypeckLiteral, TypeckTuple,
    TypeckValue, TypeckVariable,
};
use crate::per_file::utils::{IntoNodeSystem, IntoParNodeSystem};
use kodept_ast::prelude::NodeId;
use kodept_ast::properties::Name;
use kodept_ast_nodes::{AnonFunction, UserFunction};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::query::{Added, With};
use kodept_ecs::schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel};
use kodept_ecs::system::{Commands, Query};
use kodept_ecs::world::World;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_frontend::engine::reporter::{CompilationFailed, Reporter};
use kodept_report::message::Diagnostic;
use kodept_report::prelude::Severity;
use std::convert::Infallible;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, ScheduleLabel)]
struct PartialsPropagationSchedule;

define_phase! {
    pub phase TypeCheckPhase[TypeCheckPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        build(engine);
    }
}

fn build(engine: &mut PhaseEngine<TypeCheckPhase>) {
    engine.add_systems(TypeckLiteral::par_system().before(partial_propagation_system));
    engine.add_systems(TypeckValue::par_system().before(partial_propagation_system));
    engine.add_systems(FillParamTyStubs::par_system().before(partial_propagation_system));

    engine.add_systems(
        (
            CollectMonomorphicContext::par_system(),
            partial_propagation_system,
            cleanup,
            log_partials,
        )
            .chain(),
    );
}

fn partial_propagation_system(world: &mut World) {
    let mut schedule = Schedule::new(PartialsPropagationSchedule);
    schedule.add_systems((
        TypeckIf::system(),
        TypeckTuple::system(),
        TypeckCall::system(),
        TypeckBlock::par_system(),
        TypeckLink::par_system(),
        TypeckFunction::<AnonFunction>::system(),
        TypeckFunction::<UserFunction>::system(),
        TypeckVariable::par_system(),
    ));
    world.add_schedule(schedule);
    let mut any_partial_added_state = world.query_filtered::<(), Added<PartiallyTypechecked>>();

    loop {
        world.run_schedule(PartialsPropagationSchedule);

        if world.contains_resource::<CompilationFailed>() {
            return;
        }
        if any_partial_added_state.query(world).is_empty() {
            return;
        }
        world.clear_trackers();
    }
}

fn cleanup(
    partials: Query<(NodeId, &PartiallyTypechecked)>,
    monomorphic_contexts: Query<NodeId, With<MonomorphicContext>>,
    mut commands: Commands,
) {
    for (id, partial) in partials {
        if partial.is_empty() {
            commands
                .entity(id.entity())
                .remove::<PartiallyTypechecked>();
        }
    }
    for id in monomorphic_contexts {
        commands.entity(id.entity()).remove::<MonomorphicContext>();
    }
}

fn log_partials(
    partials: Query<(Option<&Name>, &mut PartiallyTypechecked)>,
    mut reporter: Reporter,
) {
    for (name, mut partial) in partials {
        let partial = partial.take();
        let result = partial.resolve(|_| Ok::<_, Infallible>(None));
        match result {
            Ok((s, t)) => reporter.report_ad_hoc(|| {
                let gen_t = t.0.generalize(&HashSet::new());
                Diagnostic::new(Severity::Note)
                    .with_message(format!("{} :: {gen_t}", name.unwrap_or(&Name::new("UNKNOWN"))))
                    .with_note(format!("{s}"))
            }),
            Err(e) => reporter
                .report_ad_hoc(|| Diagnostic::new(Severity::Error).with_message(format!("{e:?}"))),
        }
    }
}
