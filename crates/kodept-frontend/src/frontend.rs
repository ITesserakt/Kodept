use crate::plugin::{ExitEvent, Plugin};
use bevy_ecs::component::Tick;
use bevy_ecs::event::{EventRegistry, Events};
use bevy_ecs::prelude::{Event, IntoSystemConfigs, Resource, Schedule, Schedules, World};
use bevy_ecs::schedule::ScheduleLabel;
use std::ops::{Deref, DerefMut};
use std::time::Instant;
use tracing::{debug, trace};

#[derive(Debug, ScheduleLabel, Copy, Clone, Eq, PartialEq, Hash)]
struct Main;

#[derive(Debug, ScheduleLabel, Copy, Clone, Eq, PartialEq, Hash)]
struct Freestanding;

#[derive(Debug, ScheduleLabel, Copy, Clone, Eq, PartialEq, Hash)]
struct Shutdown;

#[derive(Debug, ScheduleLabel, Copy, Clone, Eq, PartialEq, Hash)]
struct Startup;

#[derive(Debug, Resource, Default)]
pub struct ActiveSystemsTracker {
    pub count: usize,
}

pub struct Frontend {
    world: World,
}

impl Frontend {
    pub fn new() -> Self {
        let mut world = World::new();
        for label in [
            Main.intern(),
            Freestanding.intern(),
            Startup.intern(),
            Shutdown.intern(),
        ] {
            let schedule = Schedule::new(label);
            world.add_schedule(schedule);
        }

        world.init_resource::<ActiveSystemsTracker>();

        Self { world }
    }

    pub fn add_freestanding_systems<M>(&mut self, systems: impl IntoSystemConfigs<M>) -> &mut Self {
        self.world
            .resource_mut::<Schedules>()
            .add_systems(Freestanding, systems);
        self
    }

    pub fn on_startup<M>(&mut self, systems: impl IntoSystemConfigs<M>) -> &mut Self {
        self.world
            .resource_mut::<Schedules>()
            .add_systems(Startup, systems);
        self
    }

    pub fn add_systems<M>(&mut self, systems: impl IntoSystemConfigs<M>) -> &mut Self {
        self.world
            .resource_mut::<Schedules>()
            .add_systems(Main, systems);
        self
    }

    pub fn on_shutdown<M>(&mut self, systems: impl IntoSystemConfigs<M>) -> &mut Self {
        self.world
            .resource_mut::<Schedules>()
            .add_systems(Shutdown, systems);
        self
    }

    fn run_once(&mut self, last_tick: Tick) -> Tick {
        let ran_systems = self.world.schedule_scope(Main, |w, s| {
            s.run(w);
            let tick_after = w.change_tick();

            s.systems()
                .into_iter()
                .flatten()
                .filter(|(_, system)| system.get_last_run().is_newer_than(last_tick, tick_after))
                .inspect(|it| trace!(system=?it))
                .count()
        });
        self.world.resource_mut::<ActiveSystemsTracker>().count = ran_systems;
        let tick = self.world.change_tick();
        self.world.run_schedule(Freestanding);
        tick
    }

    pub fn run(mut self) {
        self.run_schedule(Startup);
        let mut last_tick = self.world.change_tick();
        loop {
            let tick_duration = Instant::now();
            last_tick = self.run_once(last_tick);
            let elapsed = tick_duration.elapsed().as_secs_f64() * 1000.0;
            debug!("Tick took {elapsed}ms");

            let active = self.world.resource::<ActiveSystemsTracker>().count;
            let exits = self.world.resource_mut::<Events<ExitEvent>>();
            if exits.is_empty() && active > 0 {
                continue;
            } else {
                self.world.trigger(ExitEvent);
                self.run_schedule(Shutdown);
                break;
            }
        }
    }

    pub fn add_event<T: Event>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<Events<T>>() {
            EventRegistry::register_event::<T>(&mut self.world);
        }
        self
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }
}

impl Deref for Frontend {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl DerefMut for Frontend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}
