mod rlt_consistency;
mod single_module;

use crate::per_file::ast_shenanigans::lint::rlt_consistency::RLTConsistencyLint;
use crate::per_file::ast_shenanigans::lint::single_module::SingleModuleWithBracketsLint;
use bevy_ecs::prelude::{
    Component, IntoScheduleConfigs, IntoSystem, Local, Populated, Query, ReadOnlySystem, Schedule,
    SystemInput, World,
};
use bevy_ecs::query::Changed;
use bevy_ecs::schedule::{ScheduleConfigs, ScheduleLabel};
use bevy_ecs::system::ScheduleSystem;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use tracing::info;

pub(super) trait IntoReadonlySystem<In, Out, Marker>
where
    Self: IntoSystem<In, Out, Marker, System: ReadOnlySystem<In = In, Out = Out>>,
    In: SystemInput,
{
}

impl<In, Out, Marker, T> IntoReadonlySystem<In, Out, Marker> for T
where
    T: IntoSystem<In, Out, Marker, System: ReadOnlySystem<In = In, Out = Out>>,
    In: SystemInput,
{
}

#[derive(Debug, Copy, Clone)]
enum RunMode {
    Once,
    Unlimited,
}

#[derive(Component)]
pub(super) struct LintDescriptor {
    pub enabled: bool,
    run_mode: RunMode,
    name: Cow<'static, str>,
}

impl Debug for LintDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LintDescriptor")
            .field("enabled", &self.enabled)
            .field("name", &self.name)
            .field("run_mode", &self.run_mode)
            .finish_non_exhaustive()
    }
}

impl LintDescriptor {
    const fn new(name: &'static str) -> LintDescriptor {
        LintDescriptor {
            enabled: true,
            name: Cow::Borrowed(name),
            run_mode: RunMode::Once,
        }
    }

    fn disabled_by_default(self) -> LintDescriptor {
        Self {
            enabled: false,
            ..self
        }
    }

    const fn name(&self) -> &str {
        match &self.name {
            Cow::Borrowed(x) => x,
            Cow::Owned(x) => x.as_str(),
        }
    }
}

#[derive(Debug, ScheduleLabel, Copy, Clone, PartialEq, Eq, Hash)]
pub(super) struct Linting;

impl Linting {
    pub(super) fn configure(world: &mut World, schedule: &mut Schedule) {
        #[cfg(feature = "parallel")]
        schedule.set_executor_kind(bevy_ecs::schedule::ExecutorKind::MultiThreaded);

        schedule.add_systems((
            world.install_lint::<RLTConsistencyLint>(),
            world.install_lint::<SingleModuleWithBracketsLint>(),
            show_enabled_lints,
        ));
    }
}

fn show_enabled_lints(lints: Populated<&LintDescriptor, Changed<LintDescriptor>>) {
    let mut lint_names = String::new();
    let mut iter = lints.iter();

    if let Some(first) = iter.next() {
        if first.enabled {
            lint_names.push_str(first.name());
        }
    }
    for lint in iter {
        if lint.enabled {
            lint_names.push_str(", ");
            lint_names.push_str(lint.name())
        }
    }

    info!("Enabled lints: [{lint_names}]");
}

pub(super) trait Lint {
    fn descriptor() -> LintDescriptor;
    fn lint() -> impl IntoReadonlySystem<(), (), ()>;
}

trait LintExt {
    fn install_lint<L: Lint>(&mut self) -> ScheduleConfigs<ScheduleSystem>;
}

impl LintExt for World {
    fn install_lint<L: Lint>(&mut self) -> ScheduleConfigs<ScheduleSystem> {
        let id = self.spawn(L::descriptor()).id();
        let config = L::lint().run_if(
            move |lints: Query<&LintDescriptor>, mut has_run: Local<bool>| {
                let Ok(lint) = lints.get(id) else {
                    return false;
                };

                match (lint.enabled, lint.run_mode, *has_run) {
                    (false, _, _) => false,
                    (true, RunMode::Unlimited, _) => true,
                    (true, RunMode::Once, false) => {
                        *has_run = true;
                        true
                    }
                    (true, RunMode::Once, true) => false,
                }
            },
        );
        config
    }
}
