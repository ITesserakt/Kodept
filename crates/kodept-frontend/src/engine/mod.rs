use crate::engine::reporter::Settings;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{ExecutorKind, ScheduleLabel};
use bevy_ecs::system::ScheduleSystem;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub mod macros;
pub mod reporter;
pub mod utils;

#[derive(ScheduleLabel, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Startup;

#[derive(Debug)]
pub struct Engine {
    engine_world: World,
}

#[derive(Debug, Component)]
pub struct SubEngine {
    inner: Engine,
}

pub trait Phase: Sized {
    type Set: IntoSystemSet<()> + Default;

    fn build(self, engine: &mut PhaseEngine<Self>);
}

pub trait Plugin {
    fn build(self, engine: &mut Engine);
}

pub struct Chaining<'a, P> {
    engine: &'a mut Engine,
    _phantom: PhantomData<fn() -> P>,
}

impl<P> Chaining<'_, P> {
    pub fn install<Q>(&mut self, phase: Q) -> Chaining<'_, Q>
    where
        P: Phase,
        Q: Phase,
    {
        self.engine.with_schedule(Startup, |schedule| {
            let label1 = P::Set::default();
            let label2 = Q::Set::default();
            schedule.configure_sets((label1.into_system_set(), label2.into_system_set()).chain());
        });
        self.engine.install(phase)
    }
}

pub struct PhaseEngine<'a, P> {
    engine: &'a mut Engine,
    _phantom: PhantomData<fn() -> P>,
}

impl<P> PhaseEngine<'_, P>
where
    P: Phase,
{
    pub fn add_systems<M>(&mut self, config: impl IntoScheduleConfigs<ScheduleSystem, M>) {
        let label = P::Set::default();
        self.engine
            .add_systems(config.in_set(label.into_system_set()))
    }
}

impl Engine {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut schedules = Schedules::new();
        let startup = schedules.entry(Startup);

        #[cfg(feature = "parallel")]
        startup.set_executor_kind(ExecutorKind::MultiThreaded);
        #[cfg(not(feature = "parallel"))]
        startup.set_executor_kind(ExecutorKind::SingleThreaded);

        world.insert_resource(schedules);
        world.init_resource::<Settings>();

        Self {
            engine_world: world,
        }
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn install<P: Phase>(&mut self, phase: P) -> Chaining<'_, P> {
        phase.build(&mut PhaseEngine {
            engine: self,
            _phantom: PhantomData,
        });
        Chaining {
            engine: self,
            _phantom: PhantomData,
        }
    }

    pub fn run(&mut self) {
        self.engine_world.run_schedule(Startup);
    }

    fn with_schedule(&mut self, label: impl ScheduleLabel, callback: impl FnOnce(&mut Schedule)) {
        let mut schedules = self.engine_world.resource_mut::<Schedules>();
        callback(schedules.entry(label))
    }

    fn add_systems<M>(&mut self, config: impl IntoScheduleConfigs<ScheduleSystem, M>) {
        self.with_schedule(Startup, |schedule| {
            schedule.add_systems(config);
        })
    }

    pub fn insert_resource(&mut self, value: impl Resource) {
        self.engine_world.insert_resource(value);
    }

    pub fn init_resource<T: Resource + FromWorld>(&mut self) {
        self.engine_world.init_resource::<T>();
    }
}

impl SubEngine {
    pub fn new() -> Self {
        Self {
            inner: Engine::new(),
        }
    }
}

impl Deref for SubEngine {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SubEngine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<P> Deref for PhaseEngine<'_, P> {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.engine
    }
}

impl<P> DerefMut for PhaseEngine<'_, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.engine
    }
}
