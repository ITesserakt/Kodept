use std::{
    convert::Infallible,
    ops::{Deref, DerefMut},
};

use bevy_ecs::resource::Resource;

use crate::Interaction;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum Phase {
    #[default]
    None,
    ScopeBuilding,
    ReferenceResolving,
    Done,
}

#[derive(Debug, Resource, Default, PartialEq, Eq)]
pub(crate) struct CurrentPhase(pub(crate) Phase);

#[derive(Debug)]
pub struct Phases;

impl Interaction for Phases {
    type Error = Infallible;

    fn install(ctx: &mut crate::Ctx) {
        ctx.immediate_exclusive(|w| w.init_resource::<CurrentPhase>());
    }
}

impl Deref for CurrentPhase {
    type Target = Phase;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CurrentPhase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
