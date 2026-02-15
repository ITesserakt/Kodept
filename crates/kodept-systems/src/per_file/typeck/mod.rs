use kodept_ecs::exported::bevy_ecs;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;

define_phase! {
    pub phase TypeCheckPhase[TypeCheckPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {

    }
}
