mod rlt_consistency;

use crate::lint::rlt_consistency::RLTConsistencyLint;
use bevy_ecs::prelude::{
    Component, IntoScheduleConfigs, IntoSystem, Local, Query, ReadOnlySystem, SystemInput,
};
use bevy_ecs::schedule::ScheduleLabel;
use kodept_frontend::engine::{Engine, Plugin};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

pub trait IntoReadonlySystem<In, Out, Marker>
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
pub enum RunMode {
    Once,
    Unlimited,
}

#[derive(Component)]
pub struct LintDescriptor {
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
    pub const fn new(name: &'static str) -> LintDescriptor {
        LintDescriptor {
            enabled: true,
            name: Cow::Borrowed(name),
            run_mode: RunMode::Once,
        }
    }

    pub fn disabled_by_default(self) -> LintDescriptor {
        Self {
            enabled: false,
            ..self
        }
    }

    pub const fn name(&self) -> &str {
        match &self.name {
            Cow::Borrowed(x) => x,
            Cow::Owned(x) => x.as_str(),
        }
    }
}

#[derive(Debug, ScheduleLabel, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Linting;

pub trait Lint {
    fn descriptor() -> LintDescriptor;
    fn lint() -> impl IntoReadonlySystem<(), (), ()>;
}

pub trait LintExt {
    fn install_lint<L: Lint>(&mut self);
}

impl LintExt for Engine {
    fn install_lint<L: Lint>(&mut self) {
        let id = self.spawn_entity(L::descriptor()).id();
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
        self.add_systems(Linting, config);
    }
}

pub struct DefaultLintsPlugin;

impl Plugin for DefaultLintsPlugin {
    fn build(self, engine: &mut Engine) {
        #[cfg(feature = "parallel")]
        engine.set_schedule_executor_kind(Linting, bevy_ecs::schedule::ExecutorKind::MultiThreaded);
        engine.install_lint::<RLTConsistencyLint>();
    }
}
