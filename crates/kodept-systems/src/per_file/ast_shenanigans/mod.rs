use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use std::num::NonZeroUsize;
use tracing::error_span;
use crate::lint::Linting;

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
pub struct PassesOrder {
    passes: Vec<(InternedScheduleLabel, PassRepeatingMode)>,
}

impl PassesOrder {
    /// Register a pass that runs a fixed number of times
    pub fn register_fixed(&mut self, schedule_label: impl ScheduleLabel, count: NonZeroUsize) {
        self.passes
            .push((schedule_label.intern(), PassRepeatingMode::Fixed(count)));
    }

    /// Register a pass that runs once
    pub fn register_once(&mut self, schedule_label: impl ScheduleLabel) {
        self.passes.push((
            schedule_label.intern(),
            PassRepeatingMode::Fixed(NonZeroUsize::MIN),
        ));
    }

    /// Clear all registered passes
    pub fn clear(&mut self) {
        self.passes.clear();
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
        engine.add_systems(system);
    }
);

// Stop-the-world system
fn system(world: &mut World) {
    let order = world.resource::<PassesOrder>();
    for (label, mode) in order.passes.clone().into_iter() {
        mode.execute(label, world);
    }
}
