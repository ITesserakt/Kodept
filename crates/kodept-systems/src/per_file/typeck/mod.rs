mod first_part;

use crate::per_file::typeck::first_part::{typeck_literals, typeck_value_ctors};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::schedule::IntoScheduleConfigs;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;

define_phase! {
    pub phase TypeCheckPhase[TypeCheckPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        build(engine);
    }
}

fn build(engine: &mut PhaseEngine<TypeCheckPhase>) {
    engine.add_systems(typeck_literals);
    engine.add_systems((typeck_value_ctors,).chain());
}
