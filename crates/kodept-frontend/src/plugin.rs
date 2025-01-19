use crate::frontend::Frontend;
use bevy_ecs::prelude::{Event, World};

pub trait Plugin {
    fn build(self, world: &mut Frontend);
}

impl<F: FnOnce(&mut World)> Plugin for F {
    fn build(self, world: &mut Frontend) {
        self(world)
    }
}

#[derive(Debug, Event)]
pub struct ExitEvent;
