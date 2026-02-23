mod first_part;

use crate::per_file::typeck::first_part::{
    PartiallyTypechecked, TypeckIf, TypeckLiteral, TypeckTuple, TypeckValue,
};
use crate::per_file::utils::IntoNodeSystem;
use kodept_ast::prelude::NodeId;
use kodept_ecs::exported::bevy_ecs;
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
    schedule.add_systems((TypeckIf::system(), TypeckTuple::system()));
    world.add_schedule(schedule);

    for _ in 0..10 {
        if world.contains_resource::<CompilationFailed>() {
            return;
        }

        world.run_schedule(PartialsPropagationSchedule);
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
