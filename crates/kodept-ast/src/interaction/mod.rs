use bevy_ecs::event::{Event, EventRegistry, Events};
use bevy_ecs::prelude::{IntoScheduleConfigs, IntoSystem, Schedule, Schedules, World};
use bevy_ecs::resource::Resource;
use bevy_ecs::schedule::{InternedSystemSet, ScheduleLabel};
use bevy_ecs::system::{RunSystemOnce, ScheduleSystem, SystemInput};
use bevy_ecs::world::FromWorld;

pub struct Interaction<'w> {
    world: &'w mut World,
    schedule: Schedule,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, ScheduleLabel)]
struct Main;

impl<'w> Interaction<'w> {
    pub(crate) fn new(world: &'w mut World) -> Self {
        if !world.contains_resource::<Schedules>() {
            world.init_resource::<Schedules>();
        }
        if !world.contains_resource::<EventRegistry>() {
            world.init_resource::<EventRegistry>();
        }

        Self {
            world,
            schedule: Schedule::new(Main),
        }
    }

    pub fn immediate<O, M>(&mut self, system: impl IntoSystem<(), O, M>) -> O {
        self.immediate_with((), system)
    }

    pub fn immediate_with<'a, I, Input, O, M>(
        &mut self,
        input: Input,
        system: impl IntoSystem<I, O, M>,
    ) -> O
    where
        I: SystemInput<Inner<'a> = Input>,
    {
        self.world
            .run_system_once_with(system, input)
            .expect("Could not run system")
    }

    pub fn immediate_exclusive<T>(&mut self, f: impl FnOnce(&mut World) -> T) -> T {
        f(self.world)
    }

    pub fn register<M>(
        &mut self,
        system: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.schedule.add_systems(system);
        self
    }

    pub fn configure_sets<M>(
        &mut self,
        sets: impl IntoScheduleConfigs<InternedSystemSet, M>,
    ) -> &mut Self {
        self.schedule.configure_sets(sets);
        self
    }

    pub fn init_resource<R: Resource + FromWorld>(&mut self) {
        self.world.init_resource::<R>();
    }

    pub fn register_event<E: Event>(&mut self) {
        if !self.world.contains_resource::<Events<E>>() {
            EventRegistry::register_event::<E>(self.world);
        }
    }

    /// Runs all registered systems once
    pub fn launch(&mut self) {
        self.schedule.run(self.world);
        self.schedule.apply_deferred(self.world);
        self.world.clear_trackers();
    }
}
