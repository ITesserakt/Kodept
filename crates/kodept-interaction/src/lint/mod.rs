use crate::{done, Ctx, Interaction, InteractionWrapper, Result};
use bevy_ecs::prelude::{Changed, Component, IntoSystem, Query, World, IntoScheduleConfigs, Populated};
use bevy_ecs::system::SystemId;
use std::borrow::Cow;
use std::convert::Infallible;
use std::sync::OnceLock;
use tracing::info;
use kodept_report::traits::IntoSpannedReportMessage;

mod module;
mod rlt_linking;
mod debug;

pub use module::SingleModuleWithBrackets;
pub use rlt_linking::RLTLinkLint;
pub use debug::*;

#[derive(Debug, Component)]
pub struct LintDescriptor {
    pub enabled: bool,
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
    
    pub fn disabled_by_default(self) -> Self {
        Self {
            enabled: false,
            ..self
        }
    }

    pub fn set_system(&mut self, id: SystemId) {
        _ = self.system_id.set(id);
    }
    
    pub fn name(&self) -> &str {
        &self.name
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

pub struct ShowLints;

impl Interaction for ShowLints {
    type Error = Infallible;

    fn interaction() -> InteractionWrapper<Self::Error> {
        InteractionWrapper::wrap(|lints: Populated<&LintDescriptor, Changed<LintDescriptor>>| {
            let lint_names = lints
                .into_iter()
                .filter(|it| it.enabled)
                .map(|it| format!("{}, ", it.name))
                .collect::<String>();

            info!("Enabled lints: [{lint_names}]");
            done()
        })
    }
}
