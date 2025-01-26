use crate::{Ctx, Interaction, InteractionWrapper, Result};
use bevy_ecs::prelude::{Component, IntoSystem, IntoSystemConfigs, Query, World};
use bevy_ecs::system::SystemId;
use kodept_report::error::report::IntoSpannedReportMessage;
use std::borrow::Cow;
use std::sync::OnceLock;

pub(crate) mod module;
pub(crate) mod rlt_linking;

#[derive(Debug, Component)]
pub struct LintDescriptor {
    enabled: bool,
    name: Cow<'static, str>,
    system_id: OnceLock<SystemId>,
}

impl LintDescriptor {
    pub const fn new(name: &'static str) -> Self {
        Self {
            enabled: true,
            name: Cow::Borrowed(name),
            system_id: OnceLock::new(),
        }
    }

    pub fn set_system(&mut self, id: SystemId) {
        _ = self.system_id.set(id);
    }
}

pub trait Lint {
    type Error: IntoSpannedReportMessage + 'static;

    fn descriptor() -> LintDescriptor;

    fn lint() -> impl IntoSystem<(), Result<Self::Error>, ()>;
}

impl<L: Lint> Interaction for L {
    type Error = L::Error;

    fn interaction() -> InteractionWrapper<Self::Error> {
        InteractionWrapper::wrap(L::lint())
    }

    fn install(ctx: &mut Ctx) {
        let id = ctx.immediate(|world: &mut World| world.spawn(L::descriptor()).id());
        let config = Self::interaction()
            .unwrap()
            .run_if(move |lints: Query<&LintDescriptor>| lints.get(id).is_ok_and(|it| it.enabled));
        ctx.register(config);
    }
}
