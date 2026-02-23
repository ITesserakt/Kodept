mod first_part;

use crate::per_file::typeck::first_part::{
    PartiallyTypechecked, TypeckAnonFunction, TypeckBlock, TypeckCall, TypeckIf, TypeckLink,
    TypeckLiteral, TypeckTuple, TypeckUserFunction, TypeckValue,
};
use crate::per_file::utils::IntoNodeSystem;
use crate::utils::LogSystemEx;
use kodept_ast::prelude::NodeId;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::query::Added;
use kodept_ecs::schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel};
use kodept_ecs::system::{Commands, Query};
use kodept_ecs::world::World;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_frontend::engine::reporter::CompilationFailed;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, ScheduleLabel)]
struct PartialsPropagationSchedule;

define_phase! {
    pub phase TypeCheckPhase[TypeCheckPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        build(engine);
    }
}

fn build(engine: &mut PhaseEngine<TypeCheckPhase>) {
    engine.add_systems(TypeckLiteral::system().before(partial_propagation_system));
    engine.add_systems(TypeckValue::system().before(partial_propagation_system));

    engine.add_systems((partial_propagation_system, remove_empty_partials).chain());
}

fn partial_propagation_system(world: &mut World) {
    let mut schedule = Schedule::new(PartialsPropagationSchedule);
    schedule.add_systems((
        TypeckIf::system(),
        TypeckTuple::system(),
        TypeckCall::system(),
        TypeckBlock::system(),
        TypeckLink::system(),
        TypeckAnonFunction::system(),
        TypeckUserFunction::system(),
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

fn remove_empty_partials(partials: Query<(NodeId, &PartiallyTypechecked)>, mut commands: Commands) {
    for (id, partial) in partials {
        if partial.is_empty() {
            commands
                .entity(id.entity())
                .remove::<PartiallyTypechecked>();
        }
    }
}
