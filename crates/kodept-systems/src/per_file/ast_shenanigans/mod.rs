mod lint;
mod symbols;

use self::lint::Linting;
use crate::per_file::ast_shenanigans::symbols::SymbolResolution;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use std::num::NonZeroUsize;
use tracing::error_span;

#[derive(Debug, Copy, Clone)]
enum PassRepeatingMode {
    Fixed(NonZeroUsize),
}

impl Default for PassRepeatingMode {
    fn default() -> Self {
        Self::Fixed(NonZeroUsize::MIN)
    }
}

#[derive(Debug, Resource, Default)]
struct PassesOrder {
    passes: Vec<(InternedScheduleLabel, PassRepeatingMode)>,
}

impl PassesOrder {
    /// Register a pass that runs a fixed number of times
    fn register_fixed(&mut self, schedule_label: impl ScheduleLabel, count: NonZeroUsize) {
        self.passes
            .push((schedule_label.intern(), PassRepeatingMode::Fixed(count)));
    }
}

impl PassRepeatingMode {
    fn execute(&self, schedule: InternedScheduleLabel, world: &mut World) {
        match self {
            PassRepeatingMode::Fixed(n) => {
                for i in 0..n.get() {
                    error_span!("", frame = i);
                    world.run_schedule(Linting);
                    world.run_schedule(schedule);
                }
            }
        }
    }
}

define_phase!(
    pub phase AstPassesPhase[AstPassesPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        let mut order = PassesOrder::default();
        order.register_fixed(SymbolResolution, const { NonZeroUsize::new(4).unwrap() });
        engine.insert_resource(order);

        engine.add_systems(system);
    }
);

// Stop-the-world system
fn system(world: &mut World) {
    world.resource_scope::<Schedules, _>(|w, mut s| {
        Linting::configure(w, s.entry(Linting));
        SymbolResolution::configure(w, s.entry(SymbolResolution));
    });

    let order = world.resource::<PassesOrder>();
    for (label, mode) in order.passes.clone().into_iter() {
        mode.execute(label, world);
    }
}
