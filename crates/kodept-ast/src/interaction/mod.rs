use bevy_ecs::prelude::{IntoScheduleConfigs, IntoSystem, Schedule, Schedules, World};
use bevy_ecs::schedule::ScheduleLabel;
use bevy_ecs::system::{RunSystemOnce, ScheduleSystem, SystemInput};

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
        I: SystemInput<Inner<'a> = Input>
    {
        self.world
            .run_system_once_with(system, input)
            .expect("Could not run system")
    }

    pub fn immediate_exclusive<T>(&mut self, f: impl FnOnce(&mut World) -> T) -> T {
        f(self.world)
    }

    pub fn register<M>(&mut self, system: impl IntoScheduleConfigs<ScheduleSystem, M>) {
        self.schedule.add_systems(system);
    }

    pub fn launch(&mut self) {
        self.schedule.run(self.world);
        self.schedule.apply_deferred(self.world);
    }
}
